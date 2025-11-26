// EventDispatcher - Background Worker for Event Publishing
// Polls outbox_events and publishes to Redis Streams

use crate::prelude::*;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{info, error, warn, debug};

use super::outbox::TransactionalOutbox;

/// Configuration for EventDispatcher
#[derive(Clone, Debug)]
pub struct EventDispatcherConfig {
    /// How often to poll outbox for new events (milliseconds)
    pub poll_interval_ms: u64,
    /// How many events to process per batch
    pub batch_size: i64,
    /// Maximum retry attempts before marking as failed
    pub max_retries: i32,
    /// Redis stream name for events
    pub redis_stream_name: String,
    /// Whether dispatcher is enabled
    pub enabled: bool,
}

impl Default for EventDispatcherConfig {
    fn default() -> Self {
        Self {
            poll_interval_ms: 5000, // 5 seconds
            batch_size: 100,
            max_retries: 10,
            redis_stream_name: "domain_events".to_string(),
            enabled: true,
        }
    }
}

/// EventDispatcher - Publishes events from outbox to Redis Streams
pub struct EventDispatcher {
    outbox: Arc<TransactionalOutbox>,
    redis_client: Option<redis::Client>,
    config: EventDispatcherConfig,
    is_running: Arc<tokio::sync::RwLock<bool>>,
}

impl EventDispatcher {
    pub fn new(
        outbox: Arc<TransactionalOutbox>,
        redis_url: Option<String>,
        config: EventDispatcherConfig,
    ) -> AppResult<Self> {
        let redis_client = if let Some(url) = redis_url {
            Some(redis::Client::open(url).map_err(|e| {
                AppError::internal_error(format!("Failed to create Redis client: {}", e))
            })?)
        } else {
            None
        };

        Ok(Self {
            outbox,
            redis_client,
            config,
            is_running: Arc::new(tokio::sync::RwLock::new(false)),
        })
    }

    /// Start the event dispatcher in the background
    pub async fn start(self: Arc<Self>) -> AppResult<()> {
        if !self.config.enabled {
            warn!("EventDispatcher is disabled - events will not be published");
            return Ok(());
        }

        let mut is_running = self.is_running.write().await;
        if *is_running {
            warn!("EventDispatcher is already running");
            return Ok(());
        }

        *is_running = true;
        drop(is_running); // Release lock

        info!(
            "EventDispatcher starting (poll_interval={}ms, batch_size={})",
            self.config.poll_interval_ms, self.config.batch_size
        );

        // Spawn background task
        let dispatcher = Arc::clone(&self);
        tokio::spawn(async move {
            dispatcher.run_dispatch_loop().await;
        });

        Ok(())
    }

    /// Stop the event dispatcher
    pub async fn stop(&self) {
        let mut is_running = self.is_running.write().await;
        *is_running = false;
        info!("EventDispatcher stopped");
    }

    /// Main dispatch loop - runs continuously
    async fn run_dispatch_loop(self: Arc<Self>) {
        loop {
            // Check if we should stop
            {
                let is_running = self.is_running.read().await;
                if !*is_running {
                    break;
                }
            }

            // Process batch of events
            match self.process_batch().await {
                Ok(processed_count) => {
                    if processed_count > 0 {
                        debug!("Processed {} events", processed_count);
                    }
                }
                Err(e) => {
                    error!("Error processing event batch: {}", e);
                }
            }

            // Sleep before next poll
            sleep(Duration::from_millis(self.config.poll_interval_ms)).await;
        }

        info!("EventDispatcher loop terminated");
    }

    /// Process a batch of events from outbox
    async fn process_batch(&self) -> AppResult<usize> {
        // Fetch unprocessed events
        let events = self
            .outbox
            .get_unprocessed_events(self.config.batch_size)
            .await?;

        if events.is_empty() {
            return Ok(0);
        }

        let event_count = events.len();
        let mut processed_ids = Vec::new();

        // Process each event
        for event in events {
            match self.publish_event(&event).await {
                Ok(_) => {
                    processed_ids.push(event.id);
                    debug!(
                        "Published event {} (type: {}, aggregate: {})",
                        event.event_id, event.event_type, event.aggregate_type
                    );
                }
                Err(e) => {
                    error!(
                        "Failed to publish event {} (retry {}/{}): {}",
                        event.event_id, event.retry_count, self.config.max_retries, e
                    );

                    // Handle retry logic
                    if event.retry_count >= self.config.max_retries {
                        error!(
                            "Event {} exceeded max retries, marking as permanently failed",
                            event.event_id
                        );
                    } else {
                        // Mark for retry
                        let _ = self
                            .outbox
                            .mark_event_failed(event.id, &e.to_string(), event.retry_count + 1)
                            .await;
                    }
                }
            }
        }

        // Mark processed events
        if !processed_ids.is_empty() {
            self.outbox.mark_events_processed(&processed_ids).await?;
        }

        Ok(event_count)
    }

    /// Publish single event to Redis Streams
    async fn publish_event(&self, event: &super::outbox::OutboxEvent) -> AppResult<()> {
        if let Some(redis_client) = &self.redis_client {
            // Get Redis connection
            let mut con = redis_client.get_async_connection().await.map_err(|e| {
                AppError::internal_error(format!("Failed to get Redis connection: {}", e))
            })?;

            // Prepare event data for Redis Streams
            let event_data = vec![
                ("event_id", event.event_id.to_string()),
                ("aggregate_id", event.aggregate_id.clone()),
                ("aggregate_type", event.aggregate_type.clone()),
                ("event_type", event.event_type.clone()),
                ("payload", event.event_payload.to_string()),
                (
                    "created_at",
                    event.created_at.to_rfc3339(),
                ),
            ];

            // Publish to Redis Stream using XADD
            let _: String = redis::cmd("XADD")
                .arg(&self.config.redis_stream_name)
                .arg("*") // Auto-generate stream ID
                .arg(&event_data)
                .query_async(&mut con)
                .await
                .map_err(|e| {
                    AppError::internal_error(format!("Failed to publish to Redis Stream: {}", e))
                })?;

            Ok(())
        } else {
            // No Redis configured - just log (for development)
            debug!(
                "No Redis configured - event {} would be published to stream '{}'",
                event.event_id, self.config.redis_stream_name
            );
            Ok(())
        }
    }

    /// Get dispatcher statistics
    pub async fn get_stats(&self) -> AppResult<DispatcherStats> {
        let outbox_stats = self.outbox.get_stats().await?;
        let is_running = *self.is_running.read().await;

        Ok(DispatcherStats {
            is_running,
            pending_events: outbox_stats.pending_count,
            processed_events: outbox_stats.processed_count,
            failed_events: outbox_stats.failed_count,
            retry_events: outbox_stats.retry_count,
            config: self.config.clone(),
        })
    }

    /// Health check for dispatcher
    pub async fn health_check(&self) -> DispatcherHealth {
        let is_running = *self.is_running.read().await;

        // Check Redis connectivity if configured
        let redis_healthy = if let Some(redis_client) = &self.redis_client {
            redis_client.get_async_connection().await.is_ok()
        } else {
            true // No Redis = considered healthy
        };

        // Check outbox stats
        let outbox_stats = self.outbox.get_stats().await;
        let outbox_healthy = outbox_stats.is_ok();

        let overall_healthy = is_running && redis_healthy && outbox_healthy;

        DispatcherHealth {
            overall_healthy,
            is_running,
            redis_healthy,
            outbox_healthy,
        }
    }
}

/// Dispatcher statistics
#[derive(Debug, Clone)]
pub struct DispatcherStats {
    pub is_running: bool,
    pub pending_events: i64,
    pub processed_events: i64,
    pub failed_events: i64,
    pub retry_events: i64,
    pub config: EventDispatcherConfig,
}

/// Dispatcher health status
#[derive(Debug, Clone)]
pub struct DispatcherHealth {
    pub overall_healthy: bool,
    pub is_running: bool,
    pub redis_healthy: bool,
    pub outbox_healthy: bool,
}

#[cfg(test)]
mod tests {
    // Integration tests will be added when test database is available
}
