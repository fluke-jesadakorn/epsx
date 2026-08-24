// Projection Infrastructure
// Read model projections from domain events
//
// BIG-BANG: migrated to sqlx (real). All diesel DSL replaced with raw SQL.

use crate::prelude::*;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde_json::Value as JsonValue;
use sqlx::{PgPool, Postgres, Transaction};
use std::sync::Arc;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

// Type alias for complex Redis result
type RedisStreamResult = redis::RedisResult<Vec<(String, Vec<(String, Vec<(String, String)>)>)>>;

/// Projection trait - Implement this to create read model projections
#[async_trait]
pub trait Projection: Send + Sync {
    /// Name of this projection (unique identifier)
    fn projection_name(&self) -> &'static str;

    /// Event types this projection handles
    fn handles_event_types(&self) -> Vec<&'static str>;

    /// Project a single event into the read model
    async fn project_event(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        event: &ProjectionEvent,
    ) -> AppResult<()>;

    /// Get last processed checkpoint
    async fn get_checkpoint(&self, pool: &PgPool) -> AppResult<Option<ProjectionCheckpoint>>;

    /// Save checkpoint after successful projection
    async fn save_checkpoint(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        checkpoint: &ProjectionCheckpoint,
    ) -> AppResult<()>;

    /// Rebuild entire projection from event store (dangerous!)
    async fn rebuild(&self, _pool: &PgPool) -> AppResult<()> {
        Err(AppError::internal_error(
            "Rebuild not implemented for this projection".to_string(),
        ))
    }
}

/// Event data for projection
#[derive(Debug, Clone)]
pub struct ProjectionEvent {
    pub event_id: Uuid,
    pub sequence_number: i64,
    pub aggregate_id: String,
    pub aggregate_type: String,
    pub event_type: String,
    pub event_payload: JsonValue,
    pub occurred_at: DateTime<Utc>,
}

/// Projection checkpoint for resumability
#[derive(Debug, Clone)]
pub struct ProjectionCheckpoint {
    pub projection_name: String,
    pub last_processed_event_id: Option<Uuid>,
    pub last_processed_sequence: i64,
    pub events_processed_count: i64,
    pub processed_at: DateTime<Utc>,
    pub is_healthy: bool,
}

impl ProjectionCheckpoint {
    pub fn initial(projection_name: String) -> Self {
        Self {
            projection_name,
            last_processed_event_id: None,
            last_processed_sequence: 0,
            events_processed_count: 0,
            processed_at: Utc::now(),
            is_healthy: true,
        }
    }

    pub fn advance(&mut self, event: &ProjectionEvent) {
        self.last_processed_event_id = Some(event.event_id);
        self.last_processed_sequence = event.sequence_number;
        self.events_processed_count += 1;
        self.processed_at = Utc::now();
    }
}

/// ProjectionManager - Orchestrates multiple projections
pub struct ProjectionManager {
    pool: Arc<PgPool>,
    projections: Vec<Arc<dyn Projection>>,
    redis_client: Option<redis::Client>,
    redis_stream_name: String,
    is_running: Arc<tokio::sync::RwLock<bool>>,
}

impl ProjectionManager {
    pub fn new(
        pool: Arc<PgPool>,
        redis_url: Option<String>,
        redis_stream_name: String,
    ) -> AppResult<Self> {
        let redis_client = if let Some(url) = redis_url {
            Some(redis::Client::open(url).map_err(|e| {
                AppError::internal_error(format!("Failed to create Redis client: {}", e))
            })?)
        } else {
            None
        };

        Ok(Self {
            pool,
            projections: Vec::new(),
            redis_client,
            redis_stream_name,
            is_running: Arc::new(tokio::sync::RwLock::new(false)),
        })
    }

    pub fn register(mut self, projection: Arc<dyn Projection>) -> Self {
        self.projections.push(projection);
        self
    }

    pub async fn start(self: Arc<Self>) -> AppResult<()> {
        let mut is_running = self.is_running.write().await;
        if *is_running {
            warn!("ProjectionManager already running");
            return Ok(());
        }

        *is_running = true;
        drop(is_running);

        info!(
            "ProjectionManager starting with {} projections",
            self.projections.len()
        );

        for projection in &self.projections {
            let manager = Arc::clone(&self);
            let proj = Arc::clone(projection);
            tokio::spawn(async move {
                manager.run_projection_loop(proj).await;
            });
        }

        Ok(())
    }

    pub async fn stop(&self) {
        let mut is_running = self.is_running.write().await;
        *is_running = false;
        info!("ProjectionManager stopped");
    }

    async fn run_projection_loop(&self, projection: Arc<dyn Projection>) {
        let projection_name = projection.projection_name();
        info!("Starting projection loop for: {}", projection_name);

        loop {
            {
                let is_running = self.is_running.read().await;
                if !*is_running {
                    break;
                }
            }

            match self.process_projection_batch(&projection).await {
                Ok(processed_count) => {
                    if processed_count > 0 {
                        debug!(
                            "Projection {} processed {} events",
                            projection_name,
                            processed_count
                        );
                    }
                }
                Err(e) => {
                    error!("Error in projection {}: {}", projection_name, e);
                    if let Err(e) = self.mark_projection_unhealthy(&projection).await {
                        error!("Failed to mark projection as unhealthy: {}", e);
                    }
                }
            }

            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
        }

        info!("Projection loop terminated for: {}", projection_name);
    }

    async fn process_projection_batch(&self, projection: &Arc<dyn Projection>) -> AppResult<usize> {
        let checkpoint = projection
            .get_checkpoint(self.pool.as_ref())
            .await?
            .unwrap_or_else(|| {
                ProjectionCheckpoint::initial(projection.projection_name().to_string())
            });

        let events = if self.redis_client.is_some() {
            self.fetch_events_from_redis(&checkpoint).await?
        } else {
            self.fetch_events_from_outbox(&checkpoint, projection.handles_event_types())
                .await?
        };

        if events.is_empty() {
            return Ok(0);
        }

        let event_count = events.len();
        let mut updated_checkpoint = checkpoint;

        for event in events {
            if !projection
                .handles_event_types()
                .contains(&event.event_type.as_str())
            {
                continue;
            }

            // Use sqlx transaction
            let mut tx = self.pool.begin().await.map_err(|e| {
                AppError::database_error(format!("Failed to begin transaction: {}", e))
            })?;

            let mut checkpoint_clone = updated_checkpoint.clone();

            projection
                .project_event(&mut tx, &event)
                .await
                .map_err(|e| {
                    error!("Failed to project event: {}", e);
                    AppError::database_error(format!("Projection event failed: {}", e))
                })?;

            checkpoint_clone.advance(&event);
            projection
                .save_checkpoint(&mut tx, &checkpoint_clone)
                .await
                .map_err(|e| {
                    error!("Failed to save checkpoint: {}", e);
                    AppError::database_error(format!("Checkpoint save failed: {}", e))
                })?;

            tx.commit().await.map_err(|e| {
                AppError::database_error(format!("Projection commit failed: {}", e))
            })?;
            updated_checkpoint = checkpoint_clone;
        }

        Ok(event_count)
    }

    async fn fetch_events_from_redis(
        &self,
        checkpoint: &ProjectionCheckpoint,
    ) -> AppResult<Vec<ProjectionEvent>> {
        if let Some(redis_client) = &self.redis_client {
            let mut con = redis_client
                .get_multiplexed_async_connection()
                .await
                .map_err(|e| {
                    AppError::internal_error(format!("Failed to get Redis connection: {}", e))
                })?;

            let last_id = if checkpoint.last_processed_sequence > 0 {
                format!("{}-0", checkpoint.last_processed_sequence)
            } else {
                "0-0".to_string()
            };

            let results: RedisStreamResult = redis::cmd("XREAD")
                .arg("COUNT")
                .arg(100)
                .arg("STREAMS")
                .arg(&self.redis_stream_name)
                .arg(&last_id)
                .query_async(&mut con)
                .await;

            match results {
                Ok(streams) => {
                    let mut events = Vec::new();
                    for (_stream_name, messages) in streams {
                        for (stream_id, fields) in messages {
                            let seq_str = stream_id.split('-').next().unwrap_or("0");
                            let sequence_number = seq_str.parse::<i64>().unwrap_or(0);
                            if let Some(event) = self.parse_redis_event(&fields, sequence_number) {
                                events.push(event);
                            }
                        }
                    }
                    Ok(events)
                }
                Err(e) => {
                    warn!("Failed to read from Redis Stream: {}", e);
                    Ok(Vec::new())
                }
            }
        } else {
            Ok(Vec::new())
        }
    }

    fn parse_redis_event(
        &self,
        fields: &[(String, String)],
        sequence_number: i64,
    ) -> Option<ProjectionEvent> {
        let mut event_id: Option<Uuid> = None;
        let mut aggregate_id: Option<String> = None;
        let mut aggregate_type: Option<String> = None;
        let mut event_type: Option<String> = None;
        let mut event_payload: Option<JsonValue> = None;
        let mut occurred_at: Option<DateTime<Utc>> = None;

        for (key, value) in fields {
            match key.as_str() {
                "event_id" => event_id = Uuid::parse_str(value).ok(),
                "aggregate_id" => aggregate_id = Some(value.clone()),
                "aggregate_type" => aggregate_type = Some(value.clone()),
                "event_type" => event_type = Some(value.clone()),
                "payload" => event_payload = serde_json::from_str(value).ok(),
                "created_at" => {
                    occurred_at = DateTime::parse_from_rfc3339(value)
                        .ok()
                        .map(|dt| dt.with_timezone(&Utc))
                }
                _ => {}
            }
        }

        Some(ProjectionEvent {
            event_id: event_id?,
            sequence_number,
            aggregate_id: aggregate_id?,
            aggregate_type: aggregate_type?,
            event_type: event_type?,
            event_payload: event_payload?,
            occurred_at: occurred_at.unwrap_or_else(Utc::now),
        })
    }

    async fn fetch_events_from_outbox(
        &self,
        checkpoint: &ProjectionCheckpoint,
        event_types: Vec<&'static str>,
    ) -> AppResult<Vec<ProjectionEvent>> {
        #[derive(sqlx::FromRow)]
        struct OutboxEventRow {
            sequence_number: i64,
            event_id: Uuid,
            aggregate_id: String,
            aggregate_type: String,
            event_type: String,
            event_payload: JsonValue,
            occurred_at: DateTime<Utc>,
        }

        // Build IN clause using sqlx::QueryBuilder to safely bind event types
        let mut qb: sqlx::QueryBuilder<sqlx::Postgres> = sqlx::QueryBuilder::new(
            "SELECT id as sequence_number, event_id, aggregate_id, aggregate_type, \
                    event_type, event_payload, created_at as occurred_at \
             FROM outbox_events \
             WHERE id > ",
        );
        qb.push_bind(checkpoint.last_processed_sequence);
        qb.push(" AND event_type IN (");
        let mut sep = qb.separated(", ");
        for et in &event_types {
            sep.push_bind(et.to_string());
        }
        qb.push(") AND processed = true ORDER BY id ASC LIMIT 100");

        let results: Vec<OutboxEventRow> = qb
            .build_query_as()
            .fetch_all(self.pool.as_ref())
            .await
            .map_err(|e| AppError::database_error(format!("Failed to fetch events from outbox: {}", e)))?;

        Ok(results
            .into_iter()
            .map(|row| ProjectionEvent {
                event_id: row.event_id,
                sequence_number: row.sequence_number,
                aggregate_id: row.aggregate_id,
                aggregate_type: row.aggregate_type,
                event_type: row.event_type,
                event_payload: row.event_payload,
                occurred_at: row.occurred_at,
            })
            .collect())
    }

    async fn mark_projection_unhealthy(&self, projection: &Arc<dyn Projection>) -> AppResult<()> {
        sqlx::query(
            r#"
            UPDATE read_model.projection_checkpoints
            SET is_healthy = false, processed_at = NOW()
            WHERE projection_name = $1
            "#,
        )
        .bind(projection.projection_name())
        .execute(self.pool.as_ref())
        .await
        .map_err(|e| AppError::database_error(format!("Failed to mark projection unhealthy: {}", e)))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    // Integration tests will be added when test database is available
}
