// Transactional Outbox Pattern
// Ensures atomic persistence of aggregates and events
//
// The outbox pattern guarantees that:
// 1. Aggregate state changes are persisted
// 2. Events are persisted to event store
// 3. Events are queued for async publishing
// All within a single database transaction (ACID)

use crate::prelude::*;
use sqlx::{PgPool, Postgres, Transaction};
use tracing::{info, error, debug, warn};
use uuid::Uuid;

use super::event_store::EventStore;

/// Callback type for saving aggregate state
/// This allows repositories to provide their own save logic
pub type AggregateSaveFn = Box<dyn for<'a> Fn(&'a mut Transaction<'a, Postgres>) -> std::pin::Pin<Box<dyn std::future::Future<Output = AppResult<()>> + Send + 'a>> + Send + Sync>;

/// Transactional Outbox - Atomic Event Persistence
///
/// This is the heart of the event sourcing system. It ensures that:
/// - Aggregate state is saved to write tables
/// - Events are saved to event_store (immutable log)
/// - Events are saved to outbox_events (for async publishing)
/// - Everything happens atomically in one transaction
pub struct TransactionalOutbox {
    pool: Arc<PgPool>,
    event_store: Arc<dyn EventStore>,
}

impl TransactionalOutbox {
    pub fn new(pool: Arc<PgPool>, event_store: Arc<dyn EventStore>) -> Self {
        Self { pool, event_store }
    }

    /// Simplified API: Append events for an already-saved aggregate
    ///
    /// This is a simpler alternative to `save_with_events()` that avoids Rust async lifetime issues.
    /// Use this when you've already saved the aggregate via repository and just need to persist events.
    ///
    /// # Arguments
    /// * `aggregate_id` - Unique identifier for the aggregate
    /// * `aggregate_type` - Type of aggregate (WalletUser, Subscription, etc.)
    /// * `events` - Domain events to persist
    /// * `causation_id` - Optional command ID that caused these events
    /// * `correlation_id` - Optional trace ID for distributed tracing
    /// * `user_id` - Optional user who triggered these events
    ///
    /// # Returns
    /// Result indicating success or failure
    ///
    /// # Note
    /// This does NOT provide atomic guarantees with aggregate save.
    /// For true atomicity, the repository should use `save_with_events()`.
    /// This is a pragmatic workaround for the callback lifetime issue.
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

        // Start transaction
        let mut tx = self.pool.begin().await.map_err(|e| {
            error!("Failed to start transaction: {}", e);
            AppError::database_error(format!("Transaction start failed: {}", e))
        })?;

        info!(
            "Appending {} events for aggregate {} (type: {})",
            events.len(),
            aggregate_id,
            aggregate_type
        );

        // 1. Save events to event store (immutable log)
        self.event_store
            .append_events(&mut tx, &events, causation_id, correlation_id, user_id.clone())
            .await
            .map_err(|e| {
                error!("Failed to append events to event store: {}", e);
                e
            })?;

        debug!("Events appended to event store");

        // 2. Save events to outbox (for async publishing)
        self.save_to_outbox(&mut tx, &events, aggregate_id, aggregate_type)
            .await
            .map_err(|e| {
                error!("Failed to save events to outbox: {}", e);
                e
            })?;

        debug!("Events saved to outbox");

        // 3. Commit transaction
        tx.commit().await.map_err(|e| {
            error!("Failed to commit transaction: {}", e);
            AppError::database_error(format!("Transaction commit failed: {}", e))
        })?;

        info!(
            "✅ Successfully appended {} events for aggregate {}",
            events.len(),
            aggregate_id
        );

        Ok(())
    }

    /// Save aggregate with events atomically
    ///
    /// # Arguments
    /// * `aggregate_id` - Unique identifier for the aggregate
    /// * `aggregate_type` - Type of aggregate (WalletUser, Subscription, etc.)
    /// * `events` - Domain events to persist
    /// * `save_aggregate` - Callback to save aggregate state
    /// * `causation_id` - Optional command ID that caused these events
    /// * `correlation_id` - Optional trace ID for distributed tracing
    /// * `user_id` - Optional user who triggered these events
    ///
    /// # Returns
    /// Result indicating success or failure of the atomic operation
    pub async fn save_with_events<F, Fut>(
        &self,
        aggregate_id: &str,
        aggregate_type: &str,
        events: Vec<Box<dyn DomainEvent>>,
        save_aggregate: F,
        causation_id: Option<Uuid>,
        correlation_id: Option<Uuid>,
        user_id: Option<String>,
    ) -> AppResult<()>
    where
        F: FnOnce(&mut Transaction<'_, Postgres>) -> Fut + Send,
        Fut: std::future::Future<Output = AppResult<()>> + Send,
    {
        if events.is_empty() {
            debug!("No events to save for aggregate {}", aggregate_id);
            return Ok(());
        }

        // Start transaction
        let mut tx = self.pool.begin().await.map_err(|e| {
            error!("Failed to start transaction: {}", e);
            AppError::database_error(format!("Transaction start failed: {}", e))
        })?;

        info!(
            "Starting atomic save: aggregate={}, type={}, events={}",
            aggregate_id,
            aggregate_type,
            events.len()
        );

        // 1. Save aggregate state (using provided callback)
        save_aggregate(&mut tx).await.map_err(|e| {
            error!("Failed to save aggregate state: {}", e);
            e
        })?;

        debug!("Aggregate state saved");

        // 2. Save events to event store (immutable log)
        self.event_store
            .append_events(&mut tx, &events, causation_id, correlation_id, user_id.clone())
            .await
            .map_err(|e| {
                error!("Failed to append events to event store: {}", e);
                e
            })?;

        debug!("Events appended to event store");

        // 3. Save events to outbox (for async publishing)
        self.save_to_outbox(&mut tx, &events, aggregate_id, aggregate_type)
            .await
            .map_err(|e| {
                error!("Failed to save events to outbox: {}", e);
                e
            })?;

        debug!("Events saved to outbox");

        // 4. Commit transaction (atomic!)
        tx.commit().await.map_err(|e| {
            error!("Failed to commit transaction: {}", e);
            AppError::database_error(format!("Transaction commit failed: {}", e))
        })?;

        info!(
            "✅ Atomic save successful: {} events for aggregate {}",
            events.len(),
            aggregate_id
        );

        Ok(())
    }

    /// Save events to outbox for async publishing
    async fn save_to_outbox(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        events: &[Box<dyn DomainEvent>],
        aggregate_id: &str,
        aggregate_type: &str,
    ) -> AppResult<()> {
        for event in events {
            let event_json_str = event.to_json()
                .map_err(|e| AppError::internal_error(format!("Failed to serialize event: {}", e)))?;

            let event_json: serde_json::Value = serde_json::from_str(&event_json_str)
                .map_err(|e| AppError::internal_error(format!("Failed to parse event JSON: {}", e)))?;

            sqlx::query!(
                r#"
                INSERT INTO outbox_events (
                    event_id,
                    aggregate_id,
                    aggregate_type,
                    event_type,
                    event_payload
                ) VALUES ($1, $2, $3, $4, $5::jsonb)
                "#,
                event.event_id(),
                aggregate_id,
                aggregate_type,
                event.event_type(),
                event_json
            )
            .execute(&mut **tx)
            .await
            .map_err(|e| {
                error!("Failed to insert event into outbox: {}", e);
                AppError::database_error(format!("Outbox insert failed: {}", e))
            })?;
        }

        Ok(())
    }

    /// Get unprocessed events from outbox (for EventDispatcher)
    pub async fn get_unprocessed_events(&self, batch_size: i64) -> AppResult<Vec<OutboxEvent>> {
        let events = sqlx::query_as!(
            OutboxEvent,
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
            batch_size
        )
        .fetch_all(self.pool.as_ref())
        .await
        .map_err(|e| {
            error!("Failed to fetch unprocessed events: {}", e);
            AppError::database_error(format!("Outbox query failed: {}", e))
        })?;

        Ok(events)
    }

    /// Mark events as processed in outbox
    pub async fn mark_events_processed(&self, event_ids: &[i64]) -> AppResult<()> {
        if event_ids.is_empty() {
            return Ok(());
        }

        sqlx::query!(
            r#"
            UPDATE outbox_events
            SET processed = true, processed_at = NOW()
            WHERE id = ANY($1)
            "#,
            event_ids
        )
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

        sqlx::query!(
            r#"
            UPDATE outbox_events
            SET
                retry_count = $1,
                last_error = $2,
                next_retry_at = NOW() + ($3 || ' seconds')::interval
            WHERE id = $4
            "#,
            retry_count,
            error_message,
            retry_delay_secs.to_string(),
            event_id
        )
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
        let stats = sqlx::query!(
            r#"
            SELECT
                COUNT(*) FILTER (WHERE processed = false) as pending_count,
                COUNT(*) FILTER (WHERE processed = true) as processed_count,
                COUNT(*) FILTER (WHERE retry_count > 0) as retry_count,
                COUNT(*) FILTER (WHERE retry_count >= 10) as failed_count,
                MAX(sequence_number) as max_sequence
            FROM outbox_events
            "#
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
    use super::*;

    // Integration tests will be added when test database is available
}
