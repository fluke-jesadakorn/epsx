// Event Store - Immutable Event Log
// Handles persisting domain events to the event_store table

use crate::prelude::*;
use diesel::prelude::*;
use diesel_async::{AsyncPgConnection, RunQueryDsl, pooled_connection::deadpool::Pool};
use tracing::{info, error, debug};
use uuid::Uuid;

/// Configuration for event store
#[derive(Clone, Debug)]
pub struct EventStoreConfig {
    pub enable_snapshots: bool,
    pub snapshot_frequency: u64, // Take snapshot every N events
}

impl Default for EventStoreConfig {
    fn default() -> Self {
        Self {
            enable_snapshots: true,
            snapshot_frequency: 50, // Snapshot every 50 events
        }
    }
}

/// Event Store trait for persisting domain events
#[async_trait]
pub trait EventStore: Send + Sync {
    /// Append events to the event store (using existing connection, can be within transaction)
    async fn append_events(
        &self,
        conn: &mut AsyncPgConnection,
        events: &[Box<dyn DomainEvent>],
        causation_id: Option<Uuid>,
        correlation_id: Option<Uuid>,
        user_id: Option<String>,
    ) -> AppResult<()>;

    /// Load events for an aggregate
    async fn load_events(
        &self,
        aggregate_id: &str,
        from_version: u64,
    ) -> AppResult<Vec<serde_json::Value>>;

    /// Load events for an aggregate up to a specific version
    async fn load_events_until(
        &self,
        aggregate_id: &str,
        until_version: u64,
    ) -> AppResult<Vec<serde_json::Value>>;

    /// Get current version of an aggregate from event store
    async fn get_aggregate_version(&self, aggregate_id: &str) -> AppResult<Option<u64>>;

    /// Save aggregate snapshot for performance
    async fn save_snapshot(
        &self,
        aggregate_id: &str,
        aggregate_type: &str,
        version: u64,
        snapshot_data: serde_json::Value,
    ) -> AppResult<()>;

    /// Load latest snapshot for an aggregate
    async fn load_snapshot(
        &self,
        aggregate_id: &str,
    ) -> AppResult<Option<(u64, serde_json::Value)>>;
}

/// PostgreSQL implementation of EventStore
pub struct PostgresEventStore {
    pool: Arc<&'static Pool<AsyncPgConnection>>,
    config: EventStoreConfig,
}

impl PostgresEventStore {
    pub fn new(pool: Arc<&'static Pool<AsyncPgConnection>>) -> Self {
        Self {
            pool,
            config: EventStoreConfig::default(),
        }
    }

    pub fn with_config(pool: Arc<&'static Pool<AsyncPgConnection>>, config: EventStoreConfig) -> Self {
        Self { pool, config }
    }
}

#[async_trait]
impl EventStore for PostgresEventStore {
    async fn append_events(
        &self,
        conn: &mut AsyncPgConnection,
        events: &[Box<dyn DomainEvent>],
        causation_id: Option<Uuid>,
        correlation_id: Option<Uuid>,
        user_id: Option<String>,
    ) -> AppResult<()> {
        if events.is_empty() {
            return Ok(());
        }

        for event in events {
            let event_json_str = event.to_json()
                .map_err(|e| AppError::internal_error(format!("Failed to serialize event: {}", e)))?;

            let event_json: serde_json::Value = serde_json::from_str(&event_json_str)
                .map_err(|e| AppError::internal_error(format!("Failed to parse event JSON: {}", e)))?;

            let metadata = serde_json::json!({
                "occurred_at": event.occurred_at(),
                "aggregate_version": event.aggregate_version(),
            });

            // Insert into event_store
            diesel::sql_query(
                r#"
                INSERT INTO event_store (
                    event_id,
                    aggregate_id,
                    aggregate_type,
                    aggregate_version,
                    event_type,
                    event_data,
                    metadata,
                    occurred_at,
                    causation_id,
                    correlation_id,
                    user_id
                ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
                ON CONFLICT (aggregate_id, aggregate_version) DO NOTHING
                "#
            )
            .bind::<diesel::sql_types::Uuid, _>(event.event_id())
            .bind::<diesel::sql_types::Text, _>(event.aggregate_id())
            .bind::<diesel::sql_types::Text, _>(event.aggregate_type())
            .bind::<diesel::sql_types::BigInt, _>(event.aggregate_version() as i64)
            .bind::<diesel::sql_types::Text, _>(event.event_type())
            .bind::<diesel::sql_types::Jsonb, _>(&event_json)
            .bind::<diesel::sql_types::Jsonb, _>(&metadata)
            .bind::<diesel::sql_types::Timestamptz, _>(event.occurred_at())
            .bind::<diesel::sql_types::Nullable<diesel::sql_types::Uuid>, _>(causation_id)
            .bind::<diesel::sql_types::Nullable<diesel::sql_types::Uuid>, _>(correlation_id)
            .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(user_id.as_deref())
            .execute(conn)
            .await
            .map_err(|e| {
                error!("Failed to insert event into event_store: {}", e);
                AppError::database_error(format!("Event store insert failed: {}", e))
            })?;

            debug!(
                "Event persisted: {} v{} - {}",
                event.aggregate_id(),
                event.aggregate_version(),
                event.event_type()
            );
        }

        info!("Appended {} events to event store", events.len());
        Ok(())
    }

    async fn load_events(
        &self,
        aggregate_id: &str,
        from_version: u64,
    ) -> AppResult<Vec<serde_json::Value>> {
        let mut conn = self.pool.get().await
            .map_err(|e| AppError::database_error(format!("Pool error: {}", e)))?;

        #[derive(QueryableByName)]
        struct EventData {
            #[diesel(sql_type = diesel::sql_types::Jsonb)]
            event_data: serde_json::Value,
        }

        let events = diesel::sql_query(
            r#"
            SELECT event_data
            FROM event_store
            WHERE aggregate_id = $1 AND aggregate_version >= $2
            ORDER BY aggregate_version ASC
            "#
        )
        .bind::<diesel::sql_types::Text, _>(aggregate_id)
        .bind::<diesel::sql_types::BigInt, _>(from_version as i64)
        .load::<EventData>(&mut conn)
        .await
        .map_err(|e| {
            error!("Failed to load events for aggregate {}: {}", aggregate_id, e);
            AppError::database_error(format!("Event load failed: {}", e))
        })?;

        Ok(events.into_iter().map(|row| row.event_data).collect())
    }

    async fn load_events_until(
        &self,
        aggregate_id: &str,
        until_version: u64,
    ) -> AppResult<Vec<serde_json::Value>> {
        let mut conn = self.pool.get().await
            .map_err(|e| AppError::database_error(format!("Pool error: {}", e)))?;

        #[derive(QueryableByName)]
        struct EventData {
            #[diesel(sql_type = diesel::sql_types::Jsonb)]
            event_data: serde_json::Value,
        }

        let events = diesel::sql_query(
            r#"
            SELECT event_data
            FROM event_store
            WHERE aggregate_id = $1 AND aggregate_version <= $2
            ORDER BY aggregate_version ASC
            "#
        )
        .bind::<diesel::sql_types::Text, _>(aggregate_id)
        .bind::<diesel::sql_types::BigInt, _>(until_version as i64)
        .load::<EventData>(&mut conn)
        .await
        .map_err(|e| {
            error!("Failed to load events for aggregate {} until version {}: {}",
                aggregate_id, until_version, e);
            AppError::database_error(format!("Event load failed: {}", e))
        })?;

        Ok(events.into_iter().map(|row| row.event_data).collect())
    }

    async fn get_aggregate_version(&self, aggregate_id: &str) -> AppResult<Option<u64>> {
        let mut conn = self.pool.get().await
            .map_err(|e| AppError::database_error(format!("Pool error: {}", e)))?;

        #[derive(QueryableByName)]
        struct MaxVersion {
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::BigInt>)]
            max_version: Option<i64>,
        }

        let result = diesel::sql_query(
            r#"
            SELECT MAX(aggregate_version) as max_version
            FROM event_store
            WHERE aggregate_id = $1
            "#
        )
        .bind::<diesel::sql_types::Text, _>(aggregate_id)
        .get_result::<MaxVersion>(&mut conn)
        .await
        .map_err(|e| {
            error!("Failed to get aggregate version for {}: {}", aggregate_id, e);
            AppError::database_error(format!("Version query failed: {}", e))
        })?;

        Ok(result.max_version.map(|v| v as u64))
    }

    async fn save_snapshot(
        &self,
        aggregate_id: &str,
        aggregate_type: &str,
        version: u64,
        snapshot_data: serde_json::Value,
    ) -> AppResult<()> {
        if !self.config.enable_snapshots {
            return Ok(());
        }

        let mut conn = self.pool.get().await
            .map_err(|e| AppError::database_error(format!("Pool error: {}", e)))?;

        // Get event count at this version
        let event_count = version;

        diesel::sql_query(
            r#"
            INSERT INTO aggregate_snapshots (
                aggregate_id,
                aggregate_type,
                aggregate_version,
                snapshot_data,
                event_count_at_snapshot
            ) VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (aggregate_id) DO UPDATE SET
                aggregate_version = $3,
                snapshot_data = $4,
                event_count_at_snapshot = $5,
                created_at = NOW()
            "#
        )
        .bind::<diesel::sql_types::Text, _>(aggregate_id)
        .bind::<diesel::sql_types::Text, _>(aggregate_type)
        .bind::<diesel::sql_types::BigInt, _>(version as i64)
        .bind::<diesel::sql_types::Jsonb, _>(&snapshot_data)
        .bind::<diesel::sql_types::Integer, _>(event_count as i32)
        .execute(&mut conn)
        .await
        .map_err(|e| {
            error!("Failed to save snapshot for aggregate {}: {}", aggregate_id, e);
            AppError::database_error(format!("Snapshot save failed: {}", e))
        })?;

        info!("Snapshot saved for aggregate {} at version {}", aggregate_id, version);
        Ok(())
    }

    async fn load_snapshot(
        &self,
        aggregate_id: &str,
    ) -> AppResult<Option<(u64, serde_json::Value)>> {
        if !self.config.enable_snapshots {
            return Ok(None);
        }

        let mut conn = self.pool.get().await
            .map_err(|e| AppError::database_error(format!("Pool error: {}", e)))?;

        #[derive(QueryableByName)]
        struct SnapshotData {
            #[diesel(sql_type = diesel::sql_types::BigInt)]
            aggregate_version: i64,
            #[diesel(sql_type = diesel::sql_types::Jsonb)]
            snapshot_data: serde_json::Value,
        }

        let result = diesel::sql_query(
            r#"
            SELECT aggregate_version, snapshot_data
            FROM aggregate_snapshots
            WHERE aggregate_id = $1
            "#
        )
        .bind::<diesel::sql_types::Text, _>(aggregate_id)
        .get_result::<SnapshotData>(&mut conn)
        .await
        .optional()
        .map_err(|e| {
            error!("Failed to load snapshot for aggregate {}: {}", aggregate_id, e);
            AppError::database_error(format!("Snapshot load failed: {}", e))
        })?;

        Ok(result.map(|row| (row.aggregate_version as u64, row.snapshot_data)))
    }
}

#[cfg(test)]
mod tests {
    // Tests will be added when we have test database setup
}
