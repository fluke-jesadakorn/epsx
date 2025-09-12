// Real-time cache adapter for rate limiting
// Implements RealTimeCachePort using Redis/in-memory cache for performance

use std::sync::Arc;
use chrono::{DateTime, Utc};
use async_trait::async_trait;
use serde::{Serialize, Deserialize};

use crate::{
    domain::{
        shared_kernel::domain_error::DomainError,
        resource_management::{
            services::{resource_tracking_service::RealTimeCachePort},
            value_objects::usage_metrics::{RealTimeUsageTracker, TimePeriod},
        },
    },
    infrastructure::cache::Cache,
};

/// Cache adapter that implements real-time usage tracking for rate limiting
pub struct RealTimeCacheAdapter {
    cache: Arc<dyn Cache>,
}

impl RealTimeCacheAdapter {
    pub fn new(cache: Arc<dyn Cache>) -> Self {
        Self { cache }
    }
    
    /// Generate cache key for usage tracker
    fn cache_key(identifier: &str) -> String {
        format!("rate_limit:usage:{}", identifier)
    }
    
    /// Generate cache key for rate limit window
    fn rate_limit_window_key(identifier: &str, time_window: &TimePeriod) -> String {
        match time_window {
            TimePeriod::Minute { timestamp } => {
                format!("rate_limit:minute:{}:{}", identifier, timestamp.timestamp())
            }
            TimePeriod::Hour { timestamp } => {
                format!("rate_limit:hour:{}:{}", identifier, timestamp.timestamp())
            }
            TimePeriod::Day { date } => {
                format!("rate_limit:day:{}:{}", identifier, date.format("%Y-%m-%d"))
            }
            TimePeriod::Month { year, month } => {
                format!("rate_limit:month:{}:{}-{:02}", identifier, year, month)
            }
            TimePeriod::Year { year } => {
                format!("rate_limit:year:{}:{}", identifier, year)
            }
        }
    }
}

#[async_trait]
impl RealTimeCachePort for RealTimeCacheAdapter {
    async fn get_real_time_usage(&self, identifier: &str) -> Result<Option<RealTimeUsageTracker>, DomainError> {
        let cache_key = Self::cache_key(identifier);
        
        match self.cache.get(&cache_key) {
            Some(json_data) => {
                // Deserialize from JSON
                match serde_json::from_str::<RealTimeUsageTracker>(&json_data) {
                    Ok(tracker) => Ok(Some(tracker)),
                    Err(e) => {
                        tracing::warn!(
                            "Failed to deserialize usage tracker for {}: {}",
                            identifier, e
                        );
                        // Return None instead of error to handle corrupted cache gracefully
                        Ok(None)
                    }
                }
            }
            None => Ok(None),
        }
    }

    async fn update_real_time_usage(&self, tracker: &RealTimeUsageTracker) -> Result<(), DomainError> {
        let cache_key = Self::cache_key(&tracker.identifier);
        
        // Serialize to JSON
        let json_data = serde_json::to_string(tracker)
            .map_err(|e| DomainError::infrastructure(format!(
                "Failed to serialize usage tracker: {}", e
            )))?;
        
        // Set with 1 hour expiration (will be refreshed by continuous usage)
        let expiration_seconds = 3600; // 1 hour
        
        self.cache.set(&cache_key, json_data, Some(expiration_seconds));
        
        tracing::debug!(
            "Updated real-time usage tracker for identifier: {}",
            tracker.identifier
        );
        
        Ok(())
    }

    async fn get_current_rate_limit_usage(&self, identifier: &str, time_window: &TimePeriod) -> Result<u64, DomainError> {
        // Get the full usage tracker first
        if let Some(tracker) = self.get_real_time_usage(identifier).await? {
            let usage_count = match time_window {
                TimePeriod::Minute { .. } => tracker.current_minute.total_requests,
                TimePeriod::Hour { .. } => tracker.current_hour.total_requests,
                TimePeriod::Day { .. } => tracker.current_day.total_requests,
                TimePeriod::Month { .. } => tracker.current_month.total_requests,
                TimePeriod::Year { .. } => {
                    // For year, we estimate based on month
                    tracker.current_month.total_requests * 12
                }
            };
            Ok(usage_count)
        } else {
            // No usage tracked yet
            Ok(0)
        }
    }

    async fn cleanup_expired_trackers(&self) -> Result<u32, DomainError> {
        // This is a simplified implementation
        // In a production system, you'd use Redis SCAN or similar to find expired keys
        // For now, we rely on Redis TTL to automatically expire keys
        
        tracing::debug!("Cache cleanup triggered - relying on TTL for automatic expiration");
        
        // Return 0 as we're not actively cleaning up in this implementation
        Ok(0)
    }
}

/// Helper structure for more granular cache control
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedUsageWindow {
    pub requests: u64,
    pub window_start: DateTime<Utc>,
    pub last_updated: DateTime<Utc>,
}

impl RealTimeCacheAdapter {
    /// Increment usage in a specific time window
    pub async fn increment_usage_window(
        &self,
        identifier: &str,
        time_window: &TimePeriod,
        increment: u64,
    ) -> Result<u64, DomainError> {
        let cache_key = Self::rate_limit_window_key(identifier, time_window);
        
        // Try to get existing window
        let mut window = match self.cache.get(&cache_key) {
            Some(json_data) => {
                serde_json::from_str::<CachedUsageWindow>(&json_data)
                    .unwrap_or_else(|_| CachedUsageWindow {
                        requests: 0,
                        window_start: Utc::now(),
                        last_updated: Utc::now(),
                    })
            }
            None => CachedUsageWindow {
                requests: 0,
                window_start: Utc::now(),
                last_updated: Utc::now(),
            }
        };
        
        // Update the window
        window.requests += increment;
        window.last_updated = Utc::now();
        
        // Serialize and store back
        let json_data = serde_json::to_string(&window)
            .map_err(|e| DomainError::infrastructure(format!(
                "Failed to serialize usage window: {}", e
            )))?;
        
        // Set appropriate expiration based on time window
        let expiration_seconds = match time_window {
            TimePeriod::Minute { .. } => 120,    // 2 minutes
            TimePeriod::Hour { .. } => 3720,     // ~1 hour + buffer
            TimePeriod::Day { .. } => 86400,     // 1 day
            TimePeriod::Month { .. } => 2678400, // ~31 days
            TimePeriod::Year { .. } => 31536000, // 365 days
        };
        
        self.cache.set(&cache_key, json_data, Some(expiration_seconds));
        
        Ok(window.requests)
    }
    
    /// Get current usage for a specific time window (direct cache access)
    pub async fn get_usage_window(
        &self,
        identifier: &str,
        time_window: &TimePeriod,
    ) -> Result<u64, DomainError> {
        let cache_key = Self::rate_limit_window_key(identifier, time_window);
        
        match self.cache.get(&cache_key) {
            Some(json_data) => {
                let window: CachedUsageWindow = serde_json::from_str(&json_data)
                    .unwrap_or(CachedUsageWindow {
                        requests: 0,
                        window_start: Utc::now(),
                        last_updated: Utc::now(),
                    });
                Ok(window.requests)
            }
            None => Ok(0)
        }
    }
}