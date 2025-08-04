// Rate limiting implementation for API endpoint access control

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;
use axum::{http::StatusCode, response::{IntoResponse, Response}, Json};
use serde_json::json;
use crate::dom::values::UserId;
use crate::config::Config;

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

/// Client identifier for rate limiting
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ClientId {
    User(UserId),
    IpAddress(String),
    ApiKey(String),
}

impl std::fmt::Display for ClientId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ClientId::User(user_id) => write!(f, "user:{}", user_id),
            ClientId::IpAddress(ip) => write!(f, "ip:{}", ip),
            ClientId::ApiKey(key) => write!(f, "api_key:{}", &key[..std::cmp::min(8, key.len())]),
        }
    }
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

/// Unified rate limiter supporting both user and IP-based rate limiting
pub struct UnifiedRateLimiter {
    entries: Arc<RwLock<HashMap<String, RateLimitEntry>>>,
    config: Arc<Config>,
}

/// In-memory rate limiter (in production, use Redis for distributed rate limiting)
pub type InMemoryRateLimiter = UnifiedRateLimiter;

impl UnifiedRateLimiter {
    pub fn new() -> Self {
        Self {
            entries: Arc::new(RwLock::new(HashMap::new())),
            config: Arc::new(Config::from_env()),
        }
    }
    
    pub fn with_config(config: Arc<Config>) -> Self {
        Self {
            entries: Arc::new(RwLock::new(HashMap::new())),
            config,
        }
    }
    
    /// Check and update rate limits for any client type (user, IP, API key)
    pub async fn check_client_rate_limit(
        &self,
        client_id: &ClientId,
        endpoint: &str,
        method: &str,
        config: &RateLimitConfig,
    ) -> Result<RateLimitResult, RateLimitError> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| RateLimitError::SystemTimeError(e.to_string()))?
            .as_secs();
        
        let key = format!("{}:{}:{}", client_id, method, endpoint);
        
        let mut entries = self.entries.write().await;
        
        let entry = entries.entry(key.clone()).or_insert_with(|| {
            RateLimitEntry::new(now)
        });
        
        // Update the entry with current timestamp
        entry.update(now);
        
        // Check limits
        let result = entry.check_limits(config);
        
        // Clean up old entries periodically
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
    
    /// Check and update rate limits for IP address using configuration-based limits
    pub async fn check_ip_rate_limit(
        &self,
        client_ip: &str,
        endpoint: &str,
    ) -> Result<(), RateLimitError> {
        let (limit, window_seconds) = self.get_rate_limit_config(endpoint);
        let config = RateLimitConfig {
            requests_per_minute: Some(limit),
            requests_per_hour: None,
            requests_per_day: None,
        };
        
        let client_id = ClientId::IpAddress(client_ip.to_string());
        let result = self.check_client_rate_limit(&client_id, endpoint, "ANY", &config).await?;
        
        if !result.allowed {
            return Err(RateLimitError::RateLimitExceeded {
                message: format!("Rate limit exceeded for endpoint {}", endpoint),
                retry_after: result.retry_after_seconds.unwrap_or(window_seconds),
                limit,
                window: window_seconds,
            });
        }
        
        Ok(())
    }
    
    /// Get rate limit configuration for an endpoint (from rate_limit.rs functionality)
    fn get_rate_limit_config(&self, endpoint: &str) -> (u32, u64) {
        // Check for endpoint-specific limits first
        if let Some(endpoint_config) = self.config.rate_limiting.endpoint_specific.get(endpoint) {
            return (endpoint_config.per_minute, 60);
        }
        
        // Check for pattern-based limits
        if endpoint.contains("/login") {
            return (5, 60); // 5 requests per minute for login
        }
        
        if endpoint.contains("/payment") {
            return (10, 60); // 10 requests per minute for payments
        }
        
        if endpoint.contains("/admin") {
            return (20, 60); // 20 requests per minute for admin endpoints
        }
        
        // Default rate limit
        (self.config.rate_limiting.default_per_minute, 60)
    }
    
    /// Check and update rate limits for a user-endpoint combination (backward compatibility)
    pub async fn check_rate_limit(
        &self,
        user_id: &UserId,
        endpoint: &str,
        method: &str,
        config: &RateLimitConfig,
    ) -> Result<RateLimitResult, RateLimitError> {
        let client_id = ClientId::User(user_id.clone());
        self.check_client_rate_limit(&client_id, endpoint, method, config).await
    }
    
    /// Get current rate limit status for any client type
    pub async fn get_client_status(
        &self,
        client_id: &ClientId,
        endpoint: &str,
        method: &str,
    ) -> Option<RateLimitStatus> {
        let key = format!("{}:{}:{}", client_id, method, endpoint);
        let entries = self.entries.read().await;
        
        entries.get(&key).map(|entry| RateLimitStatus {
            minute_count: entry.minute_count,
            hour_count: entry.hour_count,
            day_count: entry.day_count,
            last_updated: entry.last_updated,
        })
    }
    
    /// Get current rate limit status for a user-endpoint combination (backward compatibility)
    pub async fn get_status(
        &self,
        user_id: &UserId,
        endpoint: &str,
        method: &str,
    ) -> Option<RateLimitStatus> {
        let client_id = ClientId::User(user_id.clone());
        self.get_client_status(&client_id, endpoint, method).await
    }
    
    /// Reset rate limits for any client type (admin function)
    pub async fn reset_client_limits(&self, client_id: &ClientId) -> Result<u32, RateLimitError> {
        let mut entries = self.entries.write().await;
        let prefix = format!("{}:", client_id);
        
        let keys_to_remove: Vec<String> = entries
            .keys()
            .filter(|key| key.starts_with(&prefix))
            .cloned()
            .collect();
        
        let count = keys_to_remove.len() as u32;
        for key in keys_to_remove {
            entries.remove(&key);
        }
        
        tracing::info!("Reset {} rate limit entries for client {}", count, client_id);
        Ok(count)
    }
    
    /// Reset rate limits for a user (backward compatibility)
    pub async fn reset_user_limits(&self, user_id: &UserId) -> Result<u32, RateLimitError> {
        let client_id = ClientId::User(user_id.clone());
        self.reset_client_limits(&client_id).await
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
    
    /// Clean up expired entries (public method)
    pub async fn cleanup_expired_entries(&self) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
            
        let mut entries = self.entries.write().await;
        self.cleanup_old_entries(&mut entries, now).await;
    }
    
    /// Get statistics about rate limiter usage
    pub async fn get_stats(&self) -> HashMap<String, u32> {
        let entries = self.entries.read().await;
        let mut stats = HashMap::new();
        
        stats.insert("total_entries".to_string(), entries.len() as u32);
        
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
            
        let active_entries = entries
            .values()
            .filter(|entry| now - entry.last_updated < 300) // 5 minutes
            .count();
            
        stats.insert("active_entries".to_string(), active_entries as u32);
        
        stats
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
    
    #[error("Rate limit exceeded: {message}")]
    RateLimitExceeded {
        message: String,
        retry_after: u64,
        limit: u32,
        window: u64,
    },
}

impl IntoResponse for RateLimitError {
    fn into_response(self) -> Response {
        match self {
            RateLimitError::RateLimitExceeded { message, retry_after, limit, window } => {
                let status = StatusCode::TOO_MANY_REQUESTS;
                let body = Json(json!({
                    "error": "rate_limit_exceeded",
                    "message": message,
                    "retry_after": retry_after,
                    "limit": limit,
                    "window": window
                }));
                (status, body).into_response()
            }
            _ => {
                let status = StatusCode::INTERNAL_SERVER_ERROR;
                let body = Json(json!({
                    "error": "internal_error",
                    "message": self.to_string()
                }));
                (status, body).into_response()
            }
        }
    }
}

/// Trait for different rate limiter implementations
#[async_trait::async_trait]
pub trait RateLimiter: Send + Sync {
    /// Check rate limits for any client type
    async fn check_client_rate_limit(
        &self,
        client_id: &ClientId,
        endpoint: &str,
        method: &str,
        config: &RateLimitConfig,
    ) -> Result<RateLimitResult, RateLimitError>;
    
    /// Get status for any client type
    async fn get_client_status(
        &self,
        client_id: &ClientId,
        endpoint: &str,
        method: &str,
    ) -> Option<RateLimitStatus>;
    
    /// Reset limits for any client type
    async fn reset_client_limits(&self, client_id: &ClientId) -> Result<u32, RateLimitError>;
    
    /// Check IP-based rate limits using configuration
    async fn check_ip_rate_limit(&self, client_ip: &str, endpoint: &str) -> Result<(), RateLimitError>;
    
    // Backward compatibility methods
    async fn check_rate_limit(
        &self,
        user_id: &UserId,
        endpoint: &str,
        method: &str,
        config: &RateLimitConfig,
    ) -> Result<RateLimitResult, RateLimitError> {
        let client_id = ClientId::User(user_id.clone());
        self.check_client_rate_limit(&client_id, endpoint, method, config).await
    }
    
    async fn get_status(
        &self,
        user_id: &UserId,
        endpoint: &str,
        method: &str,
    ) -> Option<RateLimitStatus> {
        let client_id = ClientId::User(user_id.clone());
        self.get_client_status(&client_id, endpoint, method).await
    }
    
    async fn reset_user_limits(&self, user_id: &UserId) -> Result<u32, RateLimitError> {
        let client_id = ClientId::User(user_id.clone());
        self.reset_client_limits(&client_id).await
    }
}

#[async_trait::async_trait]
impl RateLimiter for UnifiedRateLimiter {
    async fn check_client_rate_limit(
        &self,
        client_id: &ClientId,
        endpoint: &str,
        method: &str,
        config: &RateLimitConfig,
    ) -> Result<RateLimitResult, RateLimitError> {
        self.check_client_rate_limit(client_id, endpoint, method, config).await
    }
    
    async fn get_client_status(
        &self,
        client_id: &ClientId,
        endpoint: &str,
        method: &str,
    ) -> Option<RateLimitStatus> {
        self.get_client_status(client_id, endpoint, method).await
    }
    
    async fn reset_client_limits(&self, client_id: &ClientId) -> Result<u32, RateLimitError> {
        self.reset_client_limits(client_id).await
    }
    
    async fn check_ip_rate_limit(&self, client_ip: &str, endpoint: &str) -> Result<(), RateLimitError> {
        self.check_ip_rate_limit(client_ip, endpoint).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use tokio::time::sleep;
    
    #[tokio::test]
    async fn test_rate_limiting_per_minute() {
        let limiter = UnifiedRateLimiter::new();
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
    async fn test_ip_based_rate_limiting() {
        let limiter = UnifiedRateLimiter::new();
        let client_id = ClientId::IpAddress("192.168.1.100".to_string());
        let config = RateLimitConfig {
            requests_per_minute: Some(3),
            requests_per_hour: None,
            requests_per_day: None,
        };
        
        // First few requests should pass
        for i in 1..=3 {
            let result = limiter.check_client_rate_limit(&client_id, "/api/test", "GET", &config).await.unwrap();
            assert!(result.allowed, "Request {} should be allowed", i);
            assert_eq!(result.current_count, i);
        }
        
        // Fourth request should fail
        let result = limiter.check_client_rate_limit(&client_id, "/api/test", "GET", &config).await.unwrap();
        assert!(!result.allowed);
        assert_eq!(result.current_count, 4);
    }
    
    #[tokio::test]
    async fn test_different_endpoints_have_separate_limits() {
        let limiter = UnifiedRateLimiter::new();
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
        let limiter = UnifiedRateLimiter::new();
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
    
    #[tokio::test]
    async fn test_different_client_types() {
        let limiter = UnifiedRateLimiter::new();
        let user_id = ClientId::User(UserId::new("test_user".to_string()));
        let ip_id = ClientId::IpAddress("192.168.1.100".to_string());
        let api_key_id = ClientId::ApiKey("ak_test123".to_string());
        
        let config = RateLimitConfig {
            requests_per_minute: Some(1),
            requests_per_hour: None,
            requests_per_day: None,
        };
        
        // Each client type should have separate counters
        let result1 = limiter.check_client_rate_limit(&user_id, "/api/test", "GET", &config).await.unwrap();
        assert!(result1.allowed);
        
        let result2 = limiter.check_client_rate_limit(&ip_id, "/api/test", "GET", &config).await.unwrap();
        assert!(result2.allowed);
        
        let result3 = limiter.check_client_rate_limit(&api_key_id, "/api/test", "GET", &config).await.unwrap();
        assert!(result3.allowed);
        
        // Each should now be rate limited independently
        let result4 = limiter.check_client_rate_limit(&user_id, "/api/test", "GET", &config).await.unwrap();
        assert!(!result4.allowed);
        
        let result5 = limiter.check_client_rate_limit(&ip_id, "/api/test", "GET", &config).await.unwrap();
        assert!(!result5.allowed);
        
        let result6 = limiter.check_client_rate_limit(&api_key_id, "/api/test", "GET", &config).await.unwrap();
        assert!(!result6.allowed);
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