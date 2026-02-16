// Transactional Outbox Pattern
// Ensures atomic persistence of aggregates and events
//
// The outbox pattern guarantees that:
// 1. Aggregate state changes are persisted
// 2. Events are persisted to event store
// 3. Events are queued for async publishing
// All within a single database transaction (ACID)

use crate::prelude::*;
use diesel::prelude::*;
use diesel_async::{AsyncConnection, RunQueryDsl};
use tracing::{info, error, debug, warn};
use uuid::Uuid;

use super::event_store::EventStore;

/// Callback type for saving aggregate state with Diesel
/// This allows repositories to provide their own save logic
pub type AggregateSaveFn = Box<dyn for<'a> Fn(&'a mut diesel_async::AsyncPgConnection) -> std::pin::Pin<Box<dyn std::future::Future<Output = AppResult<()>> + Send + 'a>> + Send + Sync>;

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
///
/// This is the heart of the event sourcing system. It ensures that:
/// - Aggregate state is saved to write tables
/// - Events are saved to event_store (immutable log)
/// - Events are saved to outbox_events (for async publishing)
/// - Everything happens atomically in one transaction
pub struct TransactionalOutbox {
    pool: Arc<&'static TlsPool>,
    event_store: Arc<dyn EventStore>,
}

impl TransactionalOutbox {
    pub fn new(pool: Arc<&'static TlsPool>, event_store: Arc<dyn EventStore>) -> Self {
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

        let mut conn = self.pool.get().await.map_err(|e| {
            error!("Failed to get connection: {}", e);
            AppError::database_error(format!("Connection failed: {}", e))
        })?;

        let event_count = events.len();

        info!(
            "Appending {} events for aggregate {} (type: {})",
            event_count,
            aggregate_id,
            aggregate_type
        );

        let event_store = Arc::clone(&self.event_store);
        let agg_id = aggregate_id.to_string();
        let agg_type = aggregate_type.to_string();

        conn.transaction::<_, diesel::result::Error, _>(|conn| {
            Box::pin(async move {
                // 1. Save events to event store (immutable log)
                event_store
                    .append_events(conn, &events, causation_id, correlation_id, user_id.clone())
                    .await
                    .map_err(|e| {
                        error!("Failed to append events to event store: {}", e);
                        diesel::result::Error::RollbackTransaction
                    })?;

                debug!("Events appended to event store");

                // 2. Save events to outbox (for async publishing)
                for event in &events {
                    let event_json_str = event.to_json()
                        .map_err(|_| diesel::result::Error::RollbackTransaction)?;

                    let event_json: serde_json::Value = serde_json::from_str(&event_json_str)
                        .map_err(|_| diesel::result::Error::RollbackTransaction)?;

                    diesel::sql_query(
                        r#"
                        INSERT INTO outbox_events (
                            event_id,
                            aggregate_id,
                            aggregate_type,
                            event_type,
                            event_payload
                        ) VALUES ($1, $2, $3, $4, $5)
                        "#
                    )
                    .bind::<diesel::sql_types::Uuid, _>(event.event_id())
                    .bind::<diesel::sql_types::Text, _>(&agg_id)
                    .bind::<diesel::sql_types::Text, _>(&agg_type)
                    .bind::<diesel::sql_types::Text, _>(event.event_type())
                    .bind::<diesel::sql_types::Jsonb, _>(event_json)
                    .execute(conn)
                    .await
                    .map_err(|e| {
                        error!("Failed to insert event into outbox: {}", e);
                        diesel::result::Error::RollbackTransaction
                    })?;
                }

                debug!("Events saved to outbox");

                Ok(())
            })
        })
        .await
        .map_err(|e| {
            error!("Transaction failed: {:?}", e);
            AppError::database_error(format!("Transaction failed: {:?}", e))
        })?;

        info!(
            "Successfully appended {} events for aggregate {}",
            event_count,
            aggregate_id
        );

        Ok(())
    }

    /// Save aggregate with events atomically
    ///
    /// # Arguments
    /// * `params` - SaveWithEventsParams containing all required fields
    ///
    /// # Returns
    /// Result indicating success or failure of the atomic operation
    pub async fn save_with_events<F, Fut>(&self, params: SaveWithEventsParams<F>) -> AppResult<()>
    where
        F: FnOnce(&mut diesel_async::AsyncPgConnection) -> Fut + Send,
        Fut: std::future::Future<Output = AppResult<()>> + Send,
    {
        if params.events.is_empty() {
            debug!("No events to save for aggregate {}", params.aggregate_id);
            return Ok(());
        }

        let mut conn = self.pool.get().await.map_err(|e| {
            error!("Failed to get connection: {}", e);
            AppError::database_error(format!("Connection failed: {}", e))
        })?;

        info!(
            "Starting atomic save: aggregate={}, type={}, events={}",
            params.aggregate_id,
            params.aggregate_type,
            params.events.len()
        );

        let event_store = Arc::clone(&self.event_store);
        let agg_id = params.aggregate_id.clone();
        let agg_type = params.aggregate_type.clone();
        let events = params.events;
        let event_count = events.len();
        let agg_id_clone = agg_id.clone();
        let causation_id = params.causation_id;
        let correlation_id = params.correlation_id;
        let user_id = params.user_id.clone();

        conn.transaction::<_, diesel::result::Error, _>(|conn| {
            Box::pin(async move {
                // 1. Save aggregate state (using provided callback)
                (params.save_aggregate)(conn).await.map_err(|e| {
                    error!("Failed to save aggregate state: {}", e);
                    diesel::result::Error::RollbackTransaction
                })?;

                debug!("Aggregate state saved");

                // 2. Save events to event store (immutable log)
                event_store
                    .append_events(conn, &events, causation_id, correlation_id, user_id.clone())
                    .await
                    .map_err(|e| {
                        error!("Failed to append events to event store: {}", e);
                        diesel::result::Error::RollbackTransaction
                    })?;

                debug!("Events appended to event store");

                // 3. Save events to outbox (for async publishing)
                for event in &events {
                    let event_json_str = event.to_json()
                        .map_err(|_| diesel::result::Error::RollbackTransaction)?;

                    let event_json: serde_json::Value = serde_json::from_str(&event_json_str)
                        .map_err(|_| diesel::result::Error::RollbackTransaction)?;

                    diesel::sql_query(
                        r#"
                        INSERT INTO outbox_events (
                            event_id,
                            aggregate_id,
                            aggregate_type,
                            event_type,
                            event_payload
                        ) VALUES ($1, $2, $3, $4, $5)
                        "#
                    )
                    .bind::<diesel::sql_types::Uuid, _>(event.event_id())
                    .bind::<diesel::sql_types::Text, _>(&agg_id)
                    .bind::<diesel::sql_types::Text, _>(&agg_type)
                    .bind::<diesel::sql_types::Text, _>(event.event_type())
                    .bind::<diesel::sql_types::Jsonb, _>(event_json)
                    .execute(conn)
                    .await
                    .map_err(|e| {
                        error!("Failed to insert event into outbox: {}", e);
                        diesel::result::Error::RollbackTransaction
                    })?;
                }

                debug!("Events saved to outbox");

                Ok(())
            })
        })
        .await
        .map_err(|e| {
            error!("Transaction failed: {:?}", e);
            AppError::database_error(format!("Transaction failed: {:?}", e))
        })?;

        info!(
            "Atomic save successful: {} events for aggregate {}",
            event_count,
            agg_id_clone
        );

        Ok(())
    }

    /// Get unprocessed events from outbox (for EventDispatcher)
    pub async fn get_unprocessed_events(&self, batch_size: i64) -> AppResult<Vec<OutboxEvent>> {
        let mut conn = self.pool.get().await.map_err(|e| {
            error!("Failed to get connection: {}", e);
            AppError::database_error(format!("Connection failed: {}", e))
        })?;

        #[derive(QueryableByName)]
        struct OutboxEventRow {
            #[diesel(sql_type = diesel::sql_types::BigInt)]
            id: i64,
            #[diesel(sql_type = diesel::sql_types::Uuid)]
            event_id: Uuid,
            #[diesel(sql_type = diesel::sql_types::Text)]
            aggregate_id: String,
            #[diesel(sql_type = diesel::sql_types::Text)]
            aggregate_type: String,
            #[diesel(sql_type = diesel::sql_types::Text)]
            event_type: String,
            #[diesel(sql_type = diesel::sql_types::Jsonb)]
            event_payload: serde_json::Value,
            #[diesel(sql_type = diesel::sql_types::Bool)]
            processed: bool,
            #[diesel(sql_type = diesel::sql_types::Integer)]
            retry_count: i32,
            #[diesel(sql_type = diesel::sql_types::Timestamptz)]
            created_at: chrono::DateTime<chrono::Utc>,
        }

        let events = diesel::sql_query(
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
            "#
        )
        .bind::<diesel::sql_types::BigInt, _>(batch_size)
        .load::<OutboxEventRow>(&mut conn)
        .await
        .map_err(|e| {
            error!("Failed to fetch unprocessed events: {}", e);
            AppError::database_error(format!("Outbox query failed: {}", e))
        })?
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
        .collect();

        Ok(events)
    }

    /// Mark events as processed in outbox
    pub async fn mark_events_processed(&self, event_ids: &[i64]) -> AppResult<()> {
        if event_ids.is_empty() {
            return Ok(());
        }

        let mut conn = self.pool.get().await.map_err(|e| {
            error!("Failed to get connection: {}", e);
            AppError::database_error(format!("Connection failed: {}", e))
        })?;

        diesel::sql_query(
            r#"
            UPDATE outbox_events
            SET processed = true, processed_at = NOW()
            WHERE id = ANY($1)
            "#
        )
        .bind::<diesel::sql_types::Array<diesel::sql_types::BigInt>, _>(event_ids)
        .execute(&mut conn)
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

        let mut conn = self.pool.get().await.map_err(|e| {
            error!("Failed to get connection: {}", e);
            AppError::database_error(format!("Connection failed: {}", e))
        })?;

        diesel::sql_query(
            r#"
            UPDATE outbox_events
            SET
                retry_count = $1,
                last_error = $2,
                next_retry_at = NOW() + ($3 || ' seconds')::interval
            WHERE id = $4
            "#
        )
        .bind::<diesel::sql_types::Integer, _>(retry_count)
        .bind::<diesel::sql_types::Text, _>(error_message)
        .bind::<diesel::sql_types::Text, _>(retry_delay_secs.to_string())
        .bind::<diesel::sql_types::BigInt, _>(event_id)
        .execute(&mut conn)
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
        let mut conn = self.pool.get().await.map_err(|e| {
            error!("Failed to get connection: {}", e);
            AppError::database_error(format!("Connection failed: {}", e))
        })?;

        #[derive(QueryableByName)]
        struct StatsRow {
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::BigInt>)]
            pending_count: Option<i64>,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::BigInt>)]
            processed_count: Option<i64>,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::BigInt>)]
            retry_count: Option<i64>,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::BigInt>)]
            failed_count: Option<i64>,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::BigInt>)]
            max_sequence: Option<i64>,
        }

        let stats = diesel::sql_query(
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
        .get_result::<StatsRow>(&mut conn)
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
