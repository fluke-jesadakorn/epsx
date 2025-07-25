// Rate limiting implementation for API endpoint access control

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;
use crate::dom::values::UserId;

/// Time window for rate limiting
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimeWindow {
    Minute,
    Hour, 
    Day,
}

impl TimeWindow {
    pub fn duration_seconds(&self) -> u64 {
        match self {
            TimeWindow::Minute => 60,
            TimeWindow::Hour => 3600,
            TimeWindow::Day => 86400,
        }
    }
    
    pub fn window_key(&self, timestamp: u64) -> u64 {
        timestamp / self.duration_seconds()
    }
}

/// Rate limit configuration
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    pub requests_per_minute: Option<u32>,
    pub requests_per_hour: Option<u32>,
    pub requests_per_day: Option<u32>,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            requests_per_minute: Some(60),
            requests_per_hour: Some(1000),
            requests_per_day: Some(10000),
        }
    }
}

/// Rate limiting entry tracking request counts
#[derive(Debug, Clone)]
struct RateLimitEntry {
    minute_count: u32,
    hour_count: u32,
    day_count: u32,
    minute_window: u64,
    hour_window: u64, 
    day_window: u64,
    last_updated: u64,
}

impl RateLimitEntry {
    fn new(timestamp: u64) -> Self {
        Self {
            minute_count: 0,
            hour_count: 0,
            day_count: 0,
            minute_window: TimeWindow::Minute.window_key(timestamp),
            hour_window: TimeWindow::Hour.window_key(timestamp),
            day_window: TimeWindow::Day.window_key(timestamp),
            last_updated: timestamp,
        }
    }
    
    fn update(&mut self, timestamp: u64) {
        let minute_window = TimeWindow::Minute.window_key(timestamp);
        let hour_window = TimeWindow::Hour.window_key(timestamp);
        let day_window = TimeWindow::Day.window_key(timestamp);
        
        // Reset counters if we've moved to a new window
        if minute_window != self.minute_window {
            self.minute_count = 0;
            self.minute_window = minute_window;
        }
        
        if hour_window != self.hour_window {
            self.hour_count = 0;
            self.hour_window = hour_window;
        }
        
        if day_window != self.day_window {
            self.day_count = 0;
            self.day_window = day_window;
        }
        
        // Increment counters
        self.minute_count += 1;
        self.hour_count += 1;
        self.day_count += 1;
        self.last_updated = timestamp;
    }
    
    fn check_limits(&self, config: &RateLimitConfig) -> RateLimitResult {
        // Check minute limit
        if let Some(limit) = config.requests_per_minute {
            if self.minute_count > limit {
                return RateLimitResult {
                    allowed: false,
                    reason: format!("Minute rate limit exceeded: {}/{}", self.minute_count, limit),
                    retry_after_seconds: Some(60 - (self.last_updated % 60)),
                    window: TimeWindow::Minute,
                    current_count: self.minute_count,
                    limit: limit,
                };
            }
        }
        
        // Check hour limit
        if let Some(limit) = config.requests_per_hour {
            if self.hour_count > limit {
                return RateLimitResult {
                    allowed: false,
                    reason: format!("Hour rate limit exceeded: {}/{}", self.hour_count, limit),
                    retry_after_seconds: Some(3600 - (self.last_updated % 3600)),
                    window: TimeWindow::Hour,
                    current_count: self.hour_count,
                    limit: limit,
                };
            }
        }
        
        // Check day limit
        if let Some(limit) = config.requests_per_day {
            if self.day_count > limit {
                return RateLimitResult {
                    allowed: false,
                    reason: format!("Day rate limit exceeded: {}/{}", self.day_count, limit),
                    retry_after_seconds: Some(86400 - (self.last_updated % 86400)),
                    window: TimeWindow::Day,
                    current_count: self.day_count,
                    limit: limit,
                };
            }
        }
        
        RateLimitResult {
            allowed: true,
            reason: "Rate limit check passed".to_string(),
            retry_after_seconds: None,
            window: TimeWindow::Minute, // Default window for success
            current_count: self.minute_count,
            limit: config.requests_per_minute.unwrap_or(0),
        }
    }
}

/// Result of rate limit check
#[derive(Debug)]
pub struct RateLimitResult {
    pub allowed: bool,
    pub reason: String,
    pub retry_after_seconds: Option<u64>,
    pub window: TimeWindow,
    pub current_count: u32,
    pub limit: u32,
}

/// In-memory rate limiter (in production, use Redis for distributed rate limiting)
pub struct InMemoryRateLimiter {
    entries: Arc<RwLock<HashMap<String, RateLimitEntry>>>,
}

impl InMemoryRateLimiter {
    pub fn new() -> Self {
        Self {
            entries: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Check and update rate limits for a user-endpoint combination
    pub async fn check_rate_limit(
        &self,
        user_id: &UserId,
        endpoint: &str,
        method: &str,
        config: &RateLimitConfig,
    ) -> Result<RateLimitResult, RateLimitError> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| RateLimitError::SystemTimeError(e.to_string()))?
            .as_secs();
        
        let key = format!("{}:{}:{}", user_id, method, endpoint);
        
        let mut entries = self.entries.write().await;
        
        let entry = entries.entry(key.clone()).or_insert_with(|| {
            RateLimitEntry::new(now)
        });
        
        // Update the entry with current timestamp
        entry.update(now);
        
        // Check limits
        let result = entry.check_limits(config);
        
        // Clean up old entries periodically (simple cleanup every 1000 requests)
        if entries.len() > 1000 {
            self.cleanup_old_entries(&mut entries, now).await;
        }
        
        tracing::debug!(
            "Rate limit check for {}: {} - {}/{}",
            key,
            if result.allowed { "ALLOWED" } else { "DENIED" },
            result.current_count,
            result.limit
        );
        
        Ok(result)
    }
    
    /// Get current rate limit status for a user-endpoint combination
    pub async fn get_status(
        &self,
        user_id: &UserId,
        endpoint: &str,
        method: &str,
    ) -> Option<RateLimitStatus> {
        let key = format!("{}:{}:{}", user_id, method, endpoint);
        let entries = self.entries.read().await;
        
        entries.get(&key).map(|entry| RateLimitStatus {
            minute_count: entry.minute_count,
            hour_count: entry.hour_count,
            day_count: entry.day_count,
            last_updated: entry.last_updated,
        })
    }
    
    /// Reset rate limits for a user (admin function)
    pub async fn reset_user_limits(&self, user_id: &UserId) -> Result<u32, RateLimitError> {
        let mut entries = self.entries.write().await;
        let prefix = format!("{}:", user_id);
        
        let keys_to_remove: Vec<String> = entries
            .keys()
            .filter(|key| key.starts_with(&prefix))
            .cloned()
            .collect();
        
        let count = keys_to_remove.len() as u32;
        for key in keys_to_remove {
            entries.remove(&key);
        }
        
        tracing::info!("Reset {} rate limit entries for user {}", count, user_id);
        Ok(count)
    }
    
    /// Clean up entries older than 24 hours
    async fn cleanup_old_entries(&self, entries: &mut HashMap<String, RateLimitEntry>, now: u64) {
        let cutoff = now - 86400; // 24 hours ago
        
        let keys_to_remove: Vec<String> = entries
            .iter()
            .filter(|(_, entry)| entry.last_updated < cutoff)
            .map(|(key, _)| key.clone())
            .collect();
        
        let count = keys_to_remove.len();
        for key in keys_to_remove {
            entries.remove(&key);
        }
        
        if count > 0 {
            tracing::info!("Cleaned up {} old rate limit entries", count);
        }
    }
}

/// Rate limit status for monitoring
#[derive(Debug, Clone)]
pub struct RateLimitStatus {
    pub minute_count: u32,
    pub hour_count: u32,
    pub day_count: u32,
    pub last_updated: u64,
}

/// Rate limiter errors
#[derive(Debug, thiserror::Error)]
pub enum RateLimitError {
    #[error("System time error: {0}")]
    SystemTimeError(String),
    
    #[error("Configuration error: {0}")]
    ConfigError(String),
    
    #[error("Internal error: {0}")]
    InternalError(String),
}

/// Trait for different rate limiter implementations
#[async_trait::async_trait]
pub trait RateLimiter: Send + Sync {
    async fn check_rate_limit(
        &self,
        user_id: &UserId,
        endpoint: &str,
        method: &str,
        config: &RateLimitConfig,
    ) -> Result<RateLimitResult, RateLimitError>;
    
    async fn get_status(
        &self,
        user_id: &UserId,
        endpoint: &str,
        method: &str,
    ) -> Option<RateLimitStatus>;
    
    async fn reset_user_limits(&self, user_id: &UserId) -> Result<u32, RateLimitError>;
}

#[async_trait::async_trait]
impl RateLimiter for InMemoryRateLimiter {
    async fn check_rate_limit(
        &self,
        user_id: &UserId,
        endpoint: &str,
        method: &str,
        config: &RateLimitConfig,
    ) -> Result<RateLimitResult, RateLimitError> {
        self.check_rate_limit(user_id, endpoint, method, config).await
    }
    
    async fn get_status(
        &self,
        user_id: &UserId,
        endpoint: &str,
        method: &str,
    ) -> Option<RateLimitStatus> {
        self.get_status(user_id, endpoint, method).await
    }
    
    async fn reset_user_limits(&self, user_id: &UserId) -> Result<u32, RateLimitError> {
        self.reset_user_limits(user_id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use tokio::time::sleep;
    
    #[tokio::test]
    async fn test_rate_limiting_per_minute() {
        let limiter = InMemoryRateLimiter::new();
        let user_id = UserId::new("test_user".to_string());
        let config = RateLimitConfig {
            requests_per_minute: Some(2),
            requests_per_hour: Some(10),
            requests_per_day: Some(100),
        };
        
        // First request should pass
        let result1 = limiter.check_rate_limit(&user_id, "/api/test", "GET", &config).await.unwrap();
        assert!(result1.allowed);
        assert_eq!(result1.current_count, 1);
        
        // Second request should pass
        let result2 = limiter.check_rate_limit(&user_id, "/api/test", "GET", &config).await.unwrap();
        assert!(result2.allowed);
        assert_eq!(result2.current_count, 2);
        
        // Third request should fail (exceeds limit of 2 per minute)
        let result3 = limiter.check_rate_limit(&user_id, "/api/test", "GET", &config).await.unwrap();
        assert!(!result3.allowed);
        assert_eq!(result3.current_count, 3);
        assert!(result3.retry_after_seconds.is_some());
    }
    
    #[tokio::test]
    async fn test_different_endpoints_have_separate_limits() {
        let limiter = InMemoryRateLimiter::new();
        let user_id = UserId::new("test_user".to_string());
        let config = RateLimitConfig {
            requests_per_minute: Some(1),
            requests_per_hour: None,
            requests_per_day: None,
        };
        
        // First endpoint should have its own counter
        let result1 = limiter.check_rate_limit(&user_id, "/api/test1", "GET", &config).await.unwrap();
        assert!(result1.allowed);
        
        // Second endpoint should have its own counter
        let result2 = limiter.check_rate_limit(&user_id, "/api/test2", "GET", &config).await.unwrap();
        assert!(result2.allowed);
        
        // First endpoint should now be rate limited
        let result3 = limiter.check_rate_limit(&user_id, "/api/test1", "GET", &config).await.unwrap();
        assert!(!result3.allowed);
        
        // Second endpoint should also be rate limited
        let result4 = limiter.check_rate_limit(&user_id, "/api/test2", "GET", &config).await.unwrap();
        assert!(!result4.allowed);
    }
    
    #[tokio::test]
    async fn test_rate_limit_reset() {
        let limiter = InMemoryRateLimiter::new();
        let user_id = UserId::new("test_user".to_string());
        let config = RateLimitConfig {
            requests_per_minute: Some(1),
            requests_per_hour: None,
            requests_per_day: None,
        };
        
        // Make request to hit rate limit
        let result1 = limiter.check_rate_limit(&user_id, "/api/test", "GET", &config).await.unwrap();
        assert!(result1.allowed);
        
        let result2 = limiter.check_rate_limit(&user_id, "/api/test", "GET", &config).await.unwrap();
        assert!(!result2.allowed);
        
        // Reset user limits
        let reset_count = limiter.reset_user_limits(&user_id).await.unwrap();
        assert_eq!(reset_count, 1);
        
        // Should be able to make request again
        let result3 = limiter.check_rate_limit(&user_id, "/api/test", "GET", &config).await.unwrap();
        assert!(result3.allowed);
    }
    
    #[test]
    fn test_time_window_calculations() {
        let timestamp = 1234567890; // Some arbitrary timestamp
        
        assert_eq!(TimeWindow::Minute.window_key(timestamp), timestamp / 60);
        assert_eq!(TimeWindow::Hour.window_key(timestamp), timestamp / 3600);
        assert_eq!(TimeWindow::Day.window_key(timestamp), timestamp / 86400);
    }
    
    #[test]
    fn test_rate_limit_entry_window_reset() {
        let mut entry = RateLimitEntry::new(1000);
        entry.minute_count = 5;
        entry.hour_count = 20;
        entry.day_count = 100;
        
        // Update with timestamp in same windows - counts should increment
        entry.update(1001);
        assert_eq!(entry.minute_count, 6);
        assert_eq!(entry.hour_count, 21);
        assert_eq!(entry.day_count, 101);
        
        // Update with timestamp in new minute window - minute count should reset
        entry.update(1060); // 60 seconds later
        assert_eq!(entry.minute_count, 1); // Reset to 1 (new request)
        assert_eq!(entry.hour_count, 22); // Continues incrementing
        assert_eq!(entry.day_count, 102); // Continues incrementing
    }
}