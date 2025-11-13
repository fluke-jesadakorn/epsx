// Projection Infrastructure
// Read model projections from domain events

use crate::prelude::*;
use diesel::prelude::*;
use diesel_async::{AsyncPgConnection, AsyncConnection, RunQueryDsl, pooled_connection::deadpool::Pool};
use async_trait::async_trait;
use serde_json::Value as JsonValue;
use uuid::Uuid;
use chrono::{DateTime, Utc};

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
        conn: &mut AsyncPgConnection,
        event: &ProjectionEvent,
    ) -> AppResult<()>;

    /// Get last processed checkpoint
    async fn get_checkpoint(&self, pool: &Pool<AsyncPgConnection>) -> AppResult<Option<ProjectionCheckpoint>>;

    /// Save checkpoint after successful projection
    async fn save_checkpoint(
        &self,
        conn: &mut AsyncPgConnection,
        checkpoint: &ProjectionCheckpoint,
    ) -> AppResult<()>;

    /// Rebuild entire projection from event store (dangerous!)
    async fn rebuild(&self, _pool: &Pool<AsyncPgConnection>) -> AppResult<()> {
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
    pool: Arc<&'static Pool<AsyncPgConnection>>,
    projections: Vec<Arc<dyn Projection>>,
    redis_client: Option<redis::Client>,
    redis_stream_name: String,
    is_running: Arc<tokio::sync::RwLock<bool>>,
}

impl ProjectionManager {
    pub fn new(
        pool: Arc<&'static Pool<AsyncPgConnection>>,
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

    /// Register a projection
    pub fn register(mut self, projection: Arc<dyn Projection>) -> Self {
        self.projections.push(projection);
        self
    }

    /// Start projection manager
    pub async fn start(self: Arc<Self>) -> AppResult<()> {
        let mut is_running = self.is_running.write().await;
        if *is_running {
            tracing::warn!("ProjectionManager already running");
            return Ok(());
        }

        *is_running = true;
        drop(is_running);

        tracing::info!(
            "ProjectionManager starting with {} projections",
            self.projections.len()
        );

        // Spawn background task for each projection
        for projection in &self.projections {
            let manager = Arc::clone(&self);
            let proj = Arc::clone(projection);

            tokio::spawn(async move {
                manager.run_projection_loop(proj).await;
            });
        }

        Ok(())
    }

    /// Stop projection manager
    pub async fn stop(&self) {
        let mut is_running = self.is_running.write().await;
        *is_running = false;
        tracing::info!("ProjectionManager stopped");
    }

    /// Main loop for a single projection
    async fn run_projection_loop(&self, projection: Arc<dyn Projection>) {
        let projection_name = projection.projection_name();
        tracing::info!("Starting projection loop for: {}", projection_name);

        loop {
            // Check if we should stop
            {
                let is_running = self.is_running.read().await;
                if !*is_running {
                    break;
                }
            }

            // Process batch of events
            match self.process_projection_batch(&projection).await {
                Ok(processed_count) => {
                    if processed_count > 0 {
                        tracing::debug!(
                            "Projection {} processed {} events",
                            projection_name,
                            processed_count
                        );
                    }
                }
                Err(e) => {
                    tracing::error!("Error in projection {}: {}", projection_name, e);

                    // Mark projection as unhealthy
                    if let Err(e) = self.mark_projection_unhealthy(&projection).await {
                        tracing::error!("Failed to mark projection as unhealthy: {}", e);
                    }
                }
            }

            // Sleep before next iteration
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
        }

        tracing::info!("Projection loop terminated for: {}", projection_name);
    }

    /// Process a batch of events for a projection
    async fn process_projection_batch(&self, projection: &Arc<dyn Projection>) -> AppResult<usize> {
        // Get current checkpoint
        let checkpoint = projection
            .get_checkpoint(self.pool.as_ref())
            .await?
            .unwrap_or_else(|| ProjectionCheckpoint::initial(projection.projection_name().to_string()));

        // Fetch events from Redis Streams (or fallback to outbox polling)
        let events = if self.redis_client.is_some() {
            self.fetch_events_from_redis(&checkpoint).await?
        } else {
            // Fallback: poll outbox_events directly
            self.fetch_events_from_outbox(&checkpoint, projection.handles_event_types()).await?
        };

        if events.is_empty() {
            return Ok(0);
        }

        let event_count = events.len();
        let mut updated_checkpoint = checkpoint;

        // Process each event
        for event in events {
            // Check if projection handles this event type
            if !projection.handles_event_types().contains(&event.event_type.as_str()) {
                continue;
            }

            // Get connection for transaction
            let mut conn = self.pool.get().await.map_err(|e| {
                AppError::database_error(format!("Failed to get connection: {}", e))
            })?;

            // Clone checkpoint before moving into transaction
            let mut checkpoint_clone = updated_checkpoint.clone();

            // Use Diesel transaction
            updated_checkpoint = conn.transaction::<_, diesel::result::Error, _>(|conn| {
                Box::pin(async move {
                    // Project event
                    projection.project_event(conn, &event).await
                        .map_err(|e| diesel::result::Error::RollbackTransaction)?;

                    // Update checkpoint
                    checkpoint_clone.advance(&event);

                    // Save checkpoint
                    projection.save_checkpoint(conn, &checkpoint_clone).await
                        .map_err(|e| diesel::result::Error::RollbackTransaction)?;

                    Ok(checkpoint_clone)
                })
            })
            .await
            .map_err(|e| {
                AppError::database_error(format!("Failed to commit projection: {:?}", e))
            })?;
        }

        Ok(event_count)
    }

    /// Fetch events from Redis Streams
    async fn fetch_events_from_redis(&self, checkpoint: &ProjectionCheckpoint) -> AppResult<Vec<ProjectionEvent>> {
        if let Some(redis_client) = &self.redis_client {
            let mut con = redis_client.get_async_connection().await.map_err(|e| {
                AppError::internal_error(format!("Failed to get Redis connection: {}", e))
            })?;

            // Use last sequence as Redis Stream ID or start from beginning
            let last_id = if checkpoint.last_processed_sequence > 0 {
                format!("{}-0", checkpoint.last_processed_sequence)
            } else {
                "0-0".to_string()
            };

            // XREAD from Redis Stream
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
                            // Parse stream_id to get sequence number
                            let seq_str = stream_id.split('-').next().unwrap_or("0");
                            let sequence_number = seq_str.parse::<i64>().unwrap_or(0);

                            // Parse fields into ProjectionEvent
                            if let Some(event) = self.parse_redis_event(&fields, sequence_number) {
                                events.push(event);
                            }
                        }
                    }

                    Ok(events)
                }
                Err(e) => {
                    tracing::warn!("Failed to read from Redis Stream: {}", e);
                    Ok(Vec::new())
                }
            }
        } else {
            Ok(Vec::new())
        }
    }

    /// Parse Redis Stream fields into ProjectionEvent
    fn parse_redis_event(&self, fields: &[(String, String)], sequence_number: i64) -> Option<ProjectionEvent> {
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
                "created_at" => occurred_at = DateTime::parse_from_rfc3339(value).ok().map(|dt| dt.with_timezone(&Utc)),
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

    /// Fallback: fetch events from outbox_events table
    async fn fetch_events_from_outbox(
        &self,
        checkpoint: &ProjectionCheckpoint,
        event_types: Vec<&'static str>,
    ) -> AppResult<Vec<ProjectionEvent>> {
        let mut conn = self.pool.get().await
            .map_err(|e| AppError::database_error(format!("Failed to get database connection: {}", e)))?;

        let event_type_strs: Vec<String> = event_types.iter().map(|s| s.to_string()).collect();
        let event_types_literal = event_type_strs.iter()
            .map(|s| format!("'{}'", s.replace("'", "''")))
            .collect::<Vec<_>>()
            .join(",");

        let query_str = format!(
            r#"
            SELECT
                id as sequence_number,
                event_id,
                aggregate_id,
                aggregate_type,
                event_type,
                event_payload,
                created_at as occurred_at
            FROM outbox_events
            WHERE id > {}
            AND event_type IN ({})
            AND processed = true
            ORDER BY id ASC
            LIMIT 100
            "#,
            checkpoint.last_processed_sequence,
            event_types_literal
        );

        #[derive(QueryableByName)]
        struct OutboxEventRow {
            #[diesel(sql_type = diesel::sql_types::BigInt)]
            sequence_number: i64,
            #[diesel(sql_type = diesel::sql_types::Uuid)]
            event_id: Uuid,
            #[diesel(sql_type = diesel::sql_types::Text)]
            aggregate_id: String,
            #[diesel(sql_type = diesel::sql_types::Text)]
            aggregate_type: String,
            #[diesel(sql_type = diesel::sql_types::Text)]
            event_type: String,
            #[diesel(sql_type = diesel::sql_types::Jsonb)]
            event_payload: JsonValue,
            #[diesel(sql_type = diesel::sql_types::Timestamptz)]
            occurred_at: DateTime<Utc>,
        }

        let results = diesel::sql_query(&query_str)
            .load::<OutboxEventRow>(&mut conn)
            .await
            .map_err(|e| {
                AppError::database_error(format!("Failed to fetch events from outbox: {}", e))
            })?;

        let events = results
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
            .collect();

        Ok(events)
    }

    /// Mark projection as unhealthy
    async fn mark_projection_unhealthy(&self, projection: &Arc<dyn Projection>) -> AppResult<()> {
        let mut conn = self.pool.get().await
            .map_err(|e| AppError::database_error(format!("Failed to get database connection: {}", e)))?;

        diesel::sql_query(
            r#"
            UPDATE read_model.projection_checkpoints
            SET is_healthy = false, processed_at = NOW()
            WHERE projection_name = $1
            "#
        )
        .bind::<diesel::sql_types::Text, _>(projection.projection_name())
        .execute(&mut conn)
        .await
        .map_err(|e| {
            AppError::database_error(format!("Failed to mark projection unhealthy: {}", e))
        })?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    // Integration tests will be added when test database is available
}
