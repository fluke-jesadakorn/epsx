// Transactional Outbox Pattern
// Ensures atomic persistence of aggregates and events
//
// The outbox pattern guarantees that:
// 1. Aggregate state changes are persisted
// 2. Events are persisted to event store
// 3. Events are queued for async publishing
// All within a single database transaction (ACID)
//
// BIG-BANG: migrated to sqlx (real).

use crate::prelude::*;
use sqlx::PgPool;
use std::sync::Arc;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use super::event_store::EventStore;

/// Parameters for save_with_events operation
pub struct SaveWithEventsParams<F> {
    pub aggregate_id: String,
    pub aggregate_type: String,
    pub events: Vec<Box<dyn DomainEvent>>,
    pub save_aggregate: F,
    pub causation_id: Option<Uuid>,
    pub correlation_id: Option<Uuid>,
    pub user_id: Option<String>,
}

/// Transactional Outbox - Atomic Event Persistence
pub struct TransactionalOutbox {
    pool: Arc<PgPool>,
    event_store: Arc<dyn EventStore>,
}

impl TransactionalOutbox {
    pub fn new(pool: Arc<PgPool>, event_store: Arc<dyn EventStore>) -> Self {
        Self { pool, event_store }
    }

    /// Simplified API: Append events for an already-saved aggregate (best-effort, non-atomic).
    pub async fn append_and_publish_events(
        &self,
        aggregate_id: &str,
        aggregate_type: &str,
        events: Vec<Box<dyn DomainEvent>>,
        causation_id: Option<Uuid>,
        correlation_id: Option<Uuid>,
        user_id: Option<String>,
    ) -> AppResult<()> {
        if events.is_empty() {
            debug!("No events to append for aggregate {}", aggregate_id);
            return Ok(());
        }

        let event_count = events.len();

        info!(
            "Appending {} events for aggregate {} (type: {})",
            event_count, aggregate_id, aggregate_type
        );

        // Insert directly via sqlx (no transaction wrapper needed for append-only).
        for event in &events {
            let event_json_str = event.to_json().map_err(|e| {
                AppError::internal_error(format!("Failed to serialize event: {}", e))
            })?;

            let event_json: serde_json::Value = serde_json::from_str(&event_json_str)
                .map_err(|e| {
                    AppError::internal_error(format!("Failed to parse event JSON: {}", e))
                })?;

            // Append to event store first (immutable log)
            let mut tx = self.pool.begin().await.map_err(|e| {
                AppError::database_error(format!("Pool begin error: {}", e))
            })?;
            self.event_store
                .append_events(
                    &mut tx,
                    std::slice::from_ref(event),
                    causation_id,
                    correlation_id,
                    user_id.clone(),
                )
                .await
                .map_err(|e| {
                    error!("Failed to append events to event store: {}", e);
                    AppError::database_error(format!("Event store append failed: {}", e))
                })?;

            sqlx::query(
                r#"
                INSERT INTO outbox_events (
                    event_id,
                    aggregate_id,
                    aggregate_type,
                    event_type,
                    event_payload
                ) VALUES ($1, $2, $3, $4, $5)
                "#,
            )
            .bind(event.event_id())
            .bind(aggregate_id)
            .bind(aggregate_type)
            .bind(event.event_type())
            .bind(&event_json)
            .execute(&mut *tx)
            .await
            .map_err(|e| {
                error!("Failed to insert event into outbox: {}", e);
                AppError::database_error(format!("Outbox insert failed: {}", e))
            })?;

            tx.commit().await.map_err(|e| {
                AppError::database_error(format!("Outbox commit failed: {}", e))
            })?;
        }

        info!(
            "Successfully appended {} events for aggregate {}",
            event_count, aggregate_id
        );

        Ok(())
    }

    /// Save aggregate with events atomically.
    pub async fn save_with_events<F, Fut>(&self, params: SaveWithEventsParams<F>) -> AppResult<()>
    where
        F: FnOnce(&mut sqlx::PgPool) -> Fut + Send,
        Fut: std::future::Future<Output = AppResult<()>> + Send,
    {
        if params.events.is_empty() {
            debug!("No events to save for aggregate {}", params.aggregate_id);
            return Ok(());
        }

        info!(
            "Starting atomic save: aggregate={}, type={}, events={}",
            params.aggregate_id,
            params.aggregate_type,
            params.events.len()
        );

        let event_count = params.events.len();
        let event_store = Arc::clone(&self.event_store);
        let agg_id = params.aggregate_id.clone();
        let agg_type = params.aggregate_type.clone();
        let events = params.events;
        let causation_id = params.causation_id;
        let correlation_id = params.correlation_id;
        let user_id = params.user_id.clone();

        let mut tx = self.pool.begin().await.map_err(|e| {
            AppError::database_error(format!("Transaction begin failed: {}", e))
        })?;

        // 1. Save aggregate state via callback
        (params.save_aggregate)(&mut tx).await.map_err(|e| {
            error!("Failed to save aggregate state: {}", e);
            AppError::database_error(format!("Aggregate save failed: {}", e))
        })?;
        debug!("Aggregate state saved");

        // 2. Append events to event store
        event_store
            .append_events(&mut tx, &events, causation_id, correlation_id, user_id.clone())
            .await
            .map_err(|e| {
                error!("Failed to append events to event store: {}", e);
                AppError::database_error(format!("Event store append failed: {}", e))
            })?;
        debug!("Events appended to event store");

        // 3. Save events to outbox (for async publishing)
        for event in &events {
            let event_json_str = event
                .to_json()
                .map_err(|e| AppError::internal_error(format!("Failed to serialize event: {}", e)))?;
            let event_json: serde_json::Value = serde_json::from_str(&event_json_str)
                .map_err(|e| AppError::internal_error(format!("Failed to parse event JSON: {}", e)))?;

            sqlx::query(
                r#"
                INSERT INTO outbox_events (
                    event_id,
                    aggregate_id,
                    aggregate_type,
                    event_type,
                    event_payload
                ) VALUES ($1, $2, $3, $4, $5)
                "#,
            )
            .bind(event.event_id())
            .bind(&agg_id)
            .bind(&agg_type)
            .bind(event.event_type())
            .bind(&event_json)
            .execute(&mut *tx)
            .await
            .map_err(|e| {
                error!("Failed to insert event into outbox: {}", e);
                AppError::database_error(format!("Outbox insert failed: {}", e))
            })?;
        }
        debug!("Events saved to outbox");

        tx.commit().await.map_err(|e| {
            AppError::database_error(format!("Transaction commit failed: {}", e))
        })?;

        info!(
            "Atomic save successful: {} events for aggregate {}",
            event_count, agg_id
        );

        Ok(())
    }

    /// Get unprocessed events from outbox (for EventDispatcher)
    pub async fn get_unprocessed_events(&self, batch_size: i64) -> AppResult<Vec<OutboxEvent>> {
        #[derive(sqlx::FromRow)]
        struct OutboxEventRow {
            id: i64,
            event_id: Uuid,
            aggregate_id: String,
            aggregate_type: String,
            event_type: String,
            event_payload: serde_json::Value,
            processed: bool,
            retry_count: i32,
            created_at: chrono::DateTime<chrono::Utc>,
        }

        let events: Vec<OutboxEventRow> = sqlx::query_as(
            r#"
            SELECT
                id,
                event_id,
                aggregate_id,
                aggregate_type,
                event_type,
                event_payload,
                processed,
                retry_count,
                created_at
            FROM outbox_events
            WHERE processed = false
            ORDER BY sequence_number ASC
            LIMIT $1
            FOR UPDATE SKIP LOCKED
            "#,
        )
        .bind(batch_size)
        .fetch_all(self.pool.as_ref())
        .await
        .map_err(|e| {
            error!("Failed to fetch unprocessed events: {}", e);
            AppError::database_error(format!("Outbox query failed: {}", e))
        })?;

        Ok(events
            .into_iter()
            .map(|row| OutboxEvent {
                id: row.id,
                event_id: row.event_id,
                aggregate_id: row.aggregate_id,
                aggregate_type: row.aggregate_type,
                event_type: row.event_type,
                event_payload: row.event_payload,
                processed: row.processed,
                retry_count: row.retry_count,
                created_at: row.created_at,
            })
            .collect())
    }

    /// Mark events as processed in outbox
    pub async fn mark_events_processed(&self, event_ids: &[i64]) -> AppResult<()> {
        if event_ids.is_empty() {
            return Ok(());
        }

        sqlx::query(
            r#"
            UPDATE outbox_events
            SET processed = true, processed_at = NOW()
            WHERE id = ANY($1)
            "#,
        )
        .bind(event_ids)
        .execute(self.pool.as_ref())
        .await
        .map_err(|e| {
            error!("Failed to mark events as processed: {}", e);
            AppError::database_error(format!("Outbox update failed: {}", e))
        })?;

        debug!("Marked {} events as processed", event_ids.len());
        Ok(())
    }

    /// Mark event as failed with retry
    pub async fn mark_event_failed(
        &self,
        event_id: i64,
        error_message: &str,
        retry_count: i32,
    ) -> AppResult<()> {
        // Calculate exponential backoff: 2^retry_count seconds
        let retry_delay_secs = 2_i32.pow(retry_count as u32).min(3600); // Max 1 hour

        sqlx::query(
            r#"
            UPDATE outbox_events
            SET
                retry_count = $1,
                last_error = $2,
                next_retry_at = NOW() + ($3 || ' seconds')::interval
            WHERE id = $4
            "#,
        )
        .bind(retry_count)
        .bind(error_message)
        .bind(retry_delay_secs.to_string())
        .bind(event_id)
        .execute(self.pool.as_ref())
        .await
        .map_err(|e| {
            error!("Failed to mark event as failed: {}", e);
            AppError::database_error(format!("Outbox update failed: {}", e))
        })?;

        warn!(
            "Event {} failed (retry {}/10), next retry in {}s",
            event_id, retry_count, retry_delay_secs
        );

        Ok(())
    }

    /// Get outbox statistics for monitoring
    pub async fn get_stats(&self) -> AppResult<OutboxStats> {
        #[derive(sqlx::FromRow)]
        struct StatsRow {
            pending_count: Option<i64>,
            processed_count: Option<i64>,
            retry_count: Option<i64>,
            failed_count: Option<i64>,
            max_sequence: Option<i64>,
        }

        let stats: StatsRow = sqlx::query_as(
            r#"
            SELECT
                COUNT(*) FILTER (WHERE processed = false) as pending_count,
                COUNT(*) FILTER (WHERE processed = true) as processed_count,
                COUNT(*) FILTER (WHERE retry_count > 0) as retry_count,
                COUNT(*) FILTER (WHERE retry_count >= 10) as failed_count,
                MAX(sequence_number) as max_sequence
            FROM outbox_events
            "#,
        )
        .fetch_one(self.pool.as_ref())
        .await
        .map_err(|e| {
            error!("Failed to get outbox stats: {}", e);
            AppError::database_error(format!("Stats query failed: {}", e))
        })?;

        Ok(OutboxStats {
            pending_count: stats.pending_count.unwrap_or(0),
            processed_count: stats.processed_count.unwrap_or(0),
            retry_count: stats.retry_count.unwrap_or(0),
            failed_count: stats.failed_count.unwrap_or(0),
            max_sequence: stats.max_sequence.unwrap_or(0),
        })
    }
}

/// Outbox event from database
#[derive(Debug, Clone)]
pub struct OutboxEvent {
    pub id: i64,
    pub event_id: Uuid,
    pub aggregate_id: String,
    pub aggregate_type: String,
    pub event_type: String,
    pub event_payload: serde_json::Value,
    pub processed: bool,
    pub retry_count: i32,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Outbox statistics for monitoring
#[derive(Debug, Clone)]
pub struct OutboxStats {
    pub pending_count: i64,
    pub processed_count: i64,
    pub retry_count: i64,
    pub failed_count: i64,
    pub max_sequence: i64,
}

#[cfg(test)]
mod tests {
    // Integration tests will be added when test database is available
}
