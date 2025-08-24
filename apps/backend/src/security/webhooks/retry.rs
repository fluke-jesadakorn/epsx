// Webhook Retry Logic
// Exponential backoff retry system for failed webhook deliveries

use std::collections::HashMap;
use std::sync::Arc;
use tokio::time::Duration;
use chrono::{DateTime, Utc};
use uuid::Uuid;
use tracing::{debug, error};

use super::models::*;

/// Retry manager for handling failed webhook deliveries
pub struct RetryManager {
    retry_queue: tokio::sync::RwLock<Vec<RetryQueueItem>>,
    retry_stats: tokio::sync::RwLock<HashMap<Uuid, RetryStats>>,
}

#[derive(Debug, Clone)]
pub struct RetryQueueItem {
    pub delivery_id: Uuid,
    pub webhook_id: Uuid,
    pub alert_id: Uuid,
    pub payload: serde_json::Value,
    pub attempt_count: i32,
    pub max_attempts: i32,
    pub next_retry_at: DateTime<Utc>,
    pub backoff_multiplier: f64,
    pub last_error: Option<String>,
    pub priority: WebhookPriority,
}

#[derive(Debug, Clone)]
pub struct RetryStats {
    pub webhook_id: Uuid,
    pub total_retries: i64,
    pub successful_retries: i64,
    pub failed_retries: i64,
    pub current_backoff_seconds: u64,
    pub last_retry_at: Option<DateTime<Utc>>,
    pub consecutive_failures: i32,
}

impl RetryManager {
    pub fn new() -> Self {
        Self {
            retry_queue: tokio::sync::RwLock::new(Vec::new()),
            retry_stats: tokio::sync::RwLock::new(HashMap::new()),
        }
    }

    /// Add delivery to retry queue
    pub async fn add_to_retry_queue(
        &self,
        delivery_id: Uuid,
        webhook_id: Uuid,
        alert_id: Uuid,
        payload: serde_json::Value,
        attempt_count: i32,
        max_attempts: i32,
        last_error: Option<String>,
        priority: WebhookPriority,
    ) -> WebhookResult<()> {
        let backoff_seconds = self.calculate_backoff(attempt_count);
        let next_retry_at = Utc::now() + chrono::Duration::seconds(backoff_seconds as i64);

        let retry_item = RetryQueueItem {
            delivery_id,
            webhook_id,
            alert_id,
            payload,
            attempt_count,
            max_attempts,
            next_retry_at,
            backoff_multiplier: 2.0,
            last_error,
            priority,
        };

        let mut queue = self.retry_queue.write().await;
        queue.push(retry_item);
        
        // Sort by retry time (earliest first) and priority (highest first)
        queue.sort_by(|a, b| {
            a.next_retry_at.cmp(&b.next_retry_at)
                .then_with(|| b.priority.cmp(&a.priority))
        });

        // Update retry stats
        let mut stats = self.retry_stats.write().await;
        let webhook_stats = stats.entry(webhook_id).or_insert_with(|| RetryStats {
            webhook_id,
            total_retries: 0,
            successful_retries: 0,
            failed_retries: 0,
            current_backoff_seconds: backoff_seconds,
            last_retry_at: None,
            consecutive_failures: 0,
        });
        webhook_stats.total_retries += 1;
        webhook_stats.consecutive_failures += 1;
        webhook_stats.current_backoff_seconds = backoff_seconds;

        debug!(
            "Added delivery {} to retry queue, attempt {}/{}, retry at {}",
            delivery_id, attempt_count, max_attempts, next_retry_at
        );

        Ok(())
    }

    /// Get items ready for retry
    pub async fn get_ready_retries(&self) -> Vec<RetryQueueItem> {
        let now = Utc::now();
        let mut queue = self.retry_queue.write().await;
        
        let mut ready_items = Vec::new();
        let mut remaining_items = Vec::new();

        for item in queue.drain(..) {
            if item.next_retry_at <= now {
                ready_items.push(item);
            } else {
                remaining_items.push(item);
            }
        }

        *queue = remaining_items;
        ready_items
    }

    /// Mark retry as successful
    pub async fn mark_retry_success(&self, webhook_id: Uuid, delivery_id: Uuid) -> WebhookResult<()> {
        let mut stats = self.retry_stats.write().await;
        if let Some(webhook_stats) = stats.get_mut(&webhook_id) {
            webhook_stats.successful_retries += 1;
            webhook_stats.consecutive_failures = 0;
            webhook_stats.last_retry_at = Some(Utc::now());
            webhook_stats.current_backoff_seconds = 0; // Reset backoff on success
        }

        debug!("Marked retry successful for delivery {}", delivery_id);
        Ok(())
    }

    /// Mark retry as failed
    pub async fn mark_retry_failed(&self, webhook_id: Uuid, delivery_id: Uuid) -> WebhookResult<()> {
        let mut stats = self.retry_stats.write().await;
        if let Some(webhook_stats) = stats.get_mut(&webhook_id) {
            webhook_stats.failed_retries += 1;
            webhook_stats.consecutive_failures += 1;
            webhook_stats.last_retry_at = Some(Utc::now());
        }

        debug!("Marked retry failed for delivery {}", delivery_id);
        Ok(())
    }

    /// Calculate exponential backoff delay
    fn calculate_backoff(&self, attempt_count: i32) -> u64 {
        let base_delay = 60; // 1 minute base delay
        let max_delay = 3600; // 1 hour max delay
        let multiplier: f64 = 2.0;

        let delay = base_delay as f64 * multiplier.powi(attempt_count - 1);
        (delay as u64).min(max_delay)
    }

    /// Calculate jittered backoff delay to avoid thundering herd
    fn calculate_jittered_backoff(&self, attempt_count: i32) -> u64 {
        use rand::Rng;
        
        let base_delay = self.calculate_backoff(attempt_count);
        let jitter_range = base_delay / 4; // 25% jitter
        
        let mut rng = rand::thread_rng();
        let jitter = rng.gen_range(0..=jitter_range);
        
        base_delay + jitter
    }

    /// Get retry configuration for webhook
    pub async fn get_retry_config(&self, webhook: &WebhookEndpointConfig) -> RetryConfig {
        RetryConfig {
            max_attempts: webhook.retry_attempts.unwrap_or(3),
            initial_delay_seconds: webhook.retry_backoff_seconds.unwrap_or(60),
            max_delay_seconds: 3600,
            backoff_multiplier: 2.0,
            retry_http_codes: vec![500, 502, 503, 504, 429],
            stop_on_codes: vec![400, 401, 403, 404],
        }
    }

    /// Should retry based on HTTP status code
    pub fn should_retry_status(&self, status_code: u16, retry_config: &RetryConfig) -> bool {
        if retry_config.stop_on_codes.contains(&status_code) {
            return false;
        }
        
        retry_config.retry_http_codes.contains(&status_code) || status_code >= 500
    }

    /// Check if delivery should be retried
    pub async fn should_retry_delivery(
        &self,
        delivery_result: &DeliveryResult,
        attempt_count: i32,
        max_attempts: i32,
    ) -> bool {
        // Don't retry if we've exceeded max attempts
        if attempt_count >= max_attempts {
            return false;
        }

        // Don't retry if explicitly marked as no-retry
        if !delivery_result.should_retry {
            return false;
        }

        // Retry on timeout, connection errors, or retryable HTTP codes
        if delivery_result.status_code.is_none() {
            return true; // Network/timeout error
        }

        let status_code = delivery_result.status_code.unwrap();
        match status_code {
            // Don't retry client errors (except 429)
            400..=499 if status_code != 429 => false,
            // Retry server errors and rate limits
            429 | 500..=599 => true,
            // Don't retry success codes
            200..=299 => false,
            // Retry everything else
            _ => true,
        }
    }

    /// Calculate next retry time with circuit breaker logic
    pub async fn calculate_next_retry(
        &self,
        webhook_id: Uuid,
        attempt_count: i32,
    ) -> DateTime<Utc> {
        let stats = self.retry_stats.read().await;
        let webhook_stats = stats.get(&webhook_id);

        let backoff_seconds = if let Some(stats) = webhook_stats {
            // Apply circuit breaker logic for consecutive failures
            if stats.consecutive_failures > 5 {
                // Longer backoff for consistently failing webhooks
                self.calculate_backoff(attempt_count) * 2
            } else {
                self.calculate_jittered_backoff(attempt_count)
            }
        } else {
            self.calculate_jittered_backoff(attempt_count)
        };

        Utc::now() + chrono::Duration::seconds(backoff_seconds as i64)
    }

    /// Get retry statistics for webhook
    pub async fn get_retry_stats(&self, webhook_id: Uuid) -> Option<RetryStats> {
        let stats = self.retry_stats.read().await;
        stats.get(&webhook_id).cloned()
    }

    /// Get overall retry queue statistics
    pub async fn get_queue_stats(&self) -> RetryQueueStats {
        let queue = self.retry_queue.read().await;
        let stats = self.retry_stats.read().await;

        let total_items = queue.len();
        let mut items_by_priority = HashMap::new();
        let mut items_ready_now = 0;
        let now = Utc::now();

        for item in queue.iter() {
            let priority_str = match item.priority {
                WebhookPriority::Low => "Low",
                WebhookPriority::Normal => "Normal",
                WebhookPriority::High => "High",
                WebhookPriority::Critical => "Critical",
            };
            
            *items_by_priority.entry(priority_str.to_string()).or_insert(0) += 1;
            
            if item.next_retry_at <= now {
                items_ready_now += 1;
            }
        }

        let total_webhooks_with_retries = stats.len();
        let total_retries_today = stats.values()
            .map(|s| s.total_retries)
            .sum();

        RetryQueueStats {
            total_items,
            items_ready_now,
            items_by_priority,
            total_webhooks_with_retries,
            total_retries_today,
            average_backoff_seconds: self.calculate_average_backoff(&stats).await,
        }
    }

    async fn calculate_average_backoff(&self, stats: &HashMap<Uuid, RetryStats>) -> f64 {
        if stats.is_empty() {
            return 0.0;
        }

        let total_backoff: u64 = stats.values()
            .map(|s| s.current_backoff_seconds)
            .sum();

        total_backoff as f64 / stats.len() as f64
    }

    /// Clean up old retry queue items and stats
    pub async fn cleanup_old_items(&self, max_age_hours: i64) -> WebhookResult<usize> {
        let cutoff_time = Utc::now() - chrono::Duration::hours(max_age_hours);
        
        let mut queue = self.retry_queue.write().await;
        let initial_count = queue.len();
        
        queue.retain(|item| item.next_retry_at > cutoff_time);
        
        let removed_count = initial_count - queue.len();
        
        if removed_count > 0 {
            debug!("Cleaned up {} old retry queue items", removed_count);
        }

        Ok(removed_count)
    }

    /// Start retry processor background task
    pub fn start_retry_processor<F>(self: Arc<Self>, retry_handler: F)
    where
        F: Fn(RetryQueueItem) -> std::pin::Pin<Box<dyn std::future::Future<Output = WebhookResult<DeliveryResult>> + Send>> + Send + Sync + 'static,
    {
        let retry_handler = Arc::new(retry_handler);
        let retry_manager = Arc::clone(&self);

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(30)); // Check every 30 seconds
            
            loop {
                interval.tick().await;
                
                let ready_items = retry_manager.get_ready_retries().await;
                
                for item in ready_items {
                    let handler = retry_handler.clone();
                    let manager = Arc::clone(&retry_manager);
                    let webhook_id = item.webhook_id;
                    let delivery_id = item.delivery_id;
                    
                    tokio::spawn(async move {
                        match handler(item).await {
                            Ok(result) => {
                                if result.success {
                                    if let Err(e) = manager.mark_retry_success(webhook_id, delivery_id).await {
                                        error!("Failed to mark retry success: {}", e);
                                    }
                                } else {
                                    if let Err(e) = manager.mark_retry_failed(webhook_id, delivery_id).await {
                                        error!("Failed to mark retry failed: {}", e);
                                    }
                                }
                            }
                            Err(e) => {
                                error!("Retry handler error: {}", e);
                                if let Err(e) = manager.mark_retry_failed(webhook_id, delivery_id).await {
                                    error!("Failed to mark retry failed: {}", e);
                                }
                            }
                        }
                    });
                }
            }
        });
    }

    /// Circuit breaker logic for temporarily disabling problematic webhooks
    pub async fn should_circuit_break(&self, webhook_id: Uuid) -> bool {
        let stats = self.retry_stats.read().await;
        
        if let Some(webhook_stats) = stats.get(&webhook_id) {
            // Circuit break if we have more than 10 consecutive failures
            if webhook_stats.consecutive_failures > 10 {
                return true;
            }
            
            // Circuit break if failure rate is very high
            if webhook_stats.total_retries > 20 {
                let failure_rate = webhook_stats.failed_retries as f64 / webhook_stats.total_retries as f64;
                if failure_rate > 0.9 {
                    return true;
                }
            }
        }

        false
    }

    /// Reset circuit breaker for webhook
    pub async fn reset_circuit_breaker(&self, webhook_id: Uuid) -> WebhookResult<()> {
        let mut stats = self.retry_stats.write().await;
        if let Some(webhook_stats) = stats.get_mut(&webhook_id) {
            webhook_stats.consecutive_failures = 0;
            webhook_stats.current_backoff_seconds = 0;
        }

        debug!("Reset circuit breaker for webhook {}", webhook_id);
        Ok(())
    }
}

#[derive(Debug)]
pub struct RetryQueueStats {
    pub total_items: usize,
    pub items_ready_now: usize,
    pub items_by_priority: HashMap<String, i32>,
    pub total_webhooks_with_retries: usize,
    pub total_retries_today: i64,
    pub average_backoff_seconds: f64,
}

/// Retry policy configuration
#[derive(Debug, Clone)]
pub struct RetryPolicy {
    pub max_attempts: i32,
    pub initial_delay_seconds: u64,
    pub max_delay_seconds: u64,
    pub backoff_multiplier: f64,
    pub use_jitter: bool,
    pub circuit_breaker_threshold: i32,
    pub circuit_breaker_timeout_minutes: i64,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_delay_seconds: 60,
            max_delay_seconds: 3600,
            backoff_multiplier: 2.0,
            use_jitter: true,
            circuit_breaker_threshold: 10,
            circuit_breaker_timeout_minutes: 30,
        }
    }
}