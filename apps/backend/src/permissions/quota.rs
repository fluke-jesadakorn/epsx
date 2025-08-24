// API Quota Management with Redis + in-memory fallback
// Manages API usage quotas for users, endpoints, and other resources

use std::sync::Arc;
use std::collections::HashMap;
use chrono::{DateTime, Utc, Duration};
use serde::{Serialize, Deserialize};
use tracing::{debug, warn, info};
use crate::infra::cache::{Cache, CacheExt};
use crate::dom::values::UserId;

/// Quota period definitions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum QuotaPeriod {
    Minute,
    Hour,
    Day,
    Week,
    Month,
    Year,
    Custom(i64), // Custom period in seconds
}

impl QuotaPeriod {
    /// Get period duration in seconds
    pub fn duration_seconds(&self) -> i64 {
        match self {
            QuotaPeriod::Minute => 60,
            QuotaPeriod::Hour => 3600,
            QuotaPeriod::Day => 86400,
            QuotaPeriod::Week => 604800,
            QuotaPeriod::Month => 2592000, // 30 days
            QuotaPeriod::Year => 31536000, // 365 days
            QuotaPeriod::Custom(seconds) => *seconds,
        }
    }

    /// Get the period key for the current time
    pub fn get_period_key(&self, timestamp: i64) -> i64 {
        let duration = self.duration_seconds();
        timestamp / duration
    }
}

/// Quota type definitions
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum QuotaType {
    ApiRequests,
    DataTransfer, // in bytes
    Compute,      // in compute units
    Storage,      // in bytes
    Custom(String),
}

impl std::fmt::Display for QuotaType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            QuotaType::ApiRequests => write!(f, "api_requests"),
            QuotaType::DataTransfer => write!(f, "data_transfer"),
            QuotaType::Compute => write!(f, "compute"),
            QuotaType::Storage => write!(f, "storage"),
            QuotaType::Custom(name) => write!(f, "custom_{}", name),
        }
    }
}

/// Client identifier for quota management
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum QuotaClientId {
    User(UserId),
    Organization(String),
    ApiKey(String),
    IpAddress(String),
}

impl std::fmt::Display for QuotaClientId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            QuotaClientId::User(user_id) => write!(f, "user:{}", user_id),
            QuotaClientId::Organization(org_id) => write!(f, "org:{}", org_id),
            QuotaClientId::ApiKey(key) => write!(f, "apikey:{}", &key[..std::cmp::min(8, key.len())]),
            QuotaClientId::IpAddress(ip) => write!(f, "ip:{}", ip),
        }
    }
}

/// Quota configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuotaConfig {
    pub quota_type: QuotaType,
    pub limit: u64,
    pub period: QuotaPeriod,
    pub reset_time: Option<DateTime<Utc>>, // For custom reset times
    pub burst_limit: Option<u64>, // Allow temporary bursts above limit
    pub grace_period_seconds: Option<i64>, // Grace period before enforcement
}

impl Default for QuotaConfig {
    fn default() -> Self {
        Self {
            quota_type: QuotaType::ApiRequests,
            limit: 1000,
            period: QuotaPeriod::Hour,
            reset_time: None,
            burst_limit: None,
            grace_period_seconds: None,
        }
    }
}

/// Quota usage tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuotaUsage {
    pub client_id: QuotaClientId,
    pub quota_type: QuotaType,
    pub current_usage: u64,
    pub limit: u64,
    pub period: QuotaPeriod,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub last_updated: DateTime<Utc>,
    pub burst_used: u64,
}

impl QuotaUsage {
    /// Check if quota is exceeded
    pub fn is_exceeded(&self) -> bool {
        self.current_usage >= self.limit
    }

    /// Check if burst limit is exceeded
    pub fn is_burst_exceeded(&self, burst_limit: Option<u64>) -> bool {
        if let Some(burst) = burst_limit {
            self.burst_used >= burst
        } else {
            false
        }
    }

    /// Get remaining quota
    pub fn remaining(&self) -> u64 {
        if self.current_usage >= self.limit {
            0
        } else {
            self.limit - self.current_usage
        }
    }

    /// Get usage percentage
    pub fn usage_percentage(&self) -> f64 {
        if self.limit == 0 {
            100.0
        } else {
            (self.current_usage as f64 / self.limit as f64) * 100.0
        }
    }
}

/// Quota check result
#[derive(Debug)]
pub struct QuotaCheckResult {
    pub allowed: bool,
    pub usage: QuotaUsage,
    pub reason: String,
    pub retry_after_seconds: Option<i64>,
    pub warning: Option<String>, // Warning when approaching limit
}

/// Quota management errors
#[derive(Debug, thiserror::Error)]
pub enum QuotaError {
    #[error("Quota exceeded for {client_id}: {message}")]
    QuotaExceeded { client_id: String, message: String },
    
    #[error("Configuration error: {0}")]
    ConfigError(String),
    
    #[error("Cache error: {0}")]
    CacheError(String),
    
    #[error("Invalid usage amount: {0}")]
    InvalidUsage(String),
    
    #[error("Time calculation error: {0}")]
    TimeError(String),
}

/// API Quota Management Service with Redis + in-memory fallback
pub struct QuotaManagementService {
    cache: Arc<dyn Cache>,
}

impl QuotaManagementService {
    /// Create new quota management service with cache support
    pub fn new(cache: Arc<dyn Cache>) -> Self {
        Self { cache }
    }

    /// Generate cache key for quota usage
    fn quota_usage_key(&self, client_id: &QuotaClientId, quota_type: &QuotaType, period_key: i64) -> String {
        format!("quota:{}:{}:{}", client_id, quota_type, period_key)
    }

    /// Generate cache key for quota configuration
    fn quota_config_key(&self, client_id: &QuotaClientId, quota_type: &QuotaType) -> String {
        format!("quota:config:{}:{}", client_id, quota_type)
    }

    /// Check and update quota usage
    pub async fn check_and_use_quota(
        &self,
        client_id: &QuotaClientId,
        quota_type: &QuotaType,
        amount: u64,
        config: &QuotaConfig,
    ) -> Result<QuotaCheckResult, QuotaError> {
        let now = Utc::now();
        let period_key = config.period.get_period_key(now.timestamp());
        let usage_key = self.quota_usage_key(client_id, quota_type, period_key);

        // Get or create quota usage
        let mut usage = self.get_quota_usage(&usage_key, client_id, quota_type, config, now).await?;

        // Check if we need to reset for new period
        if usage.period_start <= now.checked_sub_signed(Duration::seconds(config.period.duration_seconds())).unwrap_or(now) {
            usage = self.reset_quota_usage(client_id, quota_type, config, now);
        }

        // Check if adding this amount would exceed the quota
        let would_exceed = usage.current_usage + amount > config.limit;
        let burst_exceeded = config.burst_limit.map_or(false, |burst| usage.burst_used + amount > burst);

        if would_exceed && !burst_exceeded && config.burst_limit.is_some() {
            // Use burst capacity
            usage.burst_used += amount;
            usage.current_usage += amount;
            usage.last_updated = now;
            
            self.save_quota_usage(&usage_key, &usage).await?;
            
            Ok(QuotaCheckResult {
                allowed: true,
                usage: usage.clone(),
                reason: "Using burst capacity".to_string(),
                retry_after_seconds: None,
                warning: Some("Using burst capacity - regular quota exceeded".to_string()),
            })
        } else if would_exceed || burst_exceeded {
            // Quota exceeded
            let retry_after = (usage.period_end - now).num_seconds().max(0);
            
            Ok(QuotaCheckResult {
                allowed: false,
                usage,
                reason: format!("Quota exceeded: {}/{}", usage.current_usage, config.limit),
                retry_after_seconds: Some(retry_after),
                warning: None,
            })
        } else {
            // Within quota - update usage
            usage.current_usage += amount;
            usage.last_updated = now;
            
            self.save_quota_usage(&usage_key, &usage).await?;
            
            let warning = if usage.usage_percentage() > 80.0 {
                Some(format!("Warning: {}% of quota used", usage.usage_percentage() as u8))
            } else {
                None
            };
            
            Ok(QuotaCheckResult {
                allowed: true,
                usage,
                reason: "Within quota limits".to_string(),
                retry_after_seconds: None,
                warning,
            })
        }
    }

    /// Get quota usage from cache or create new
    async fn get_quota_usage(
        &self,
        usage_key: &str,
        client_id: &QuotaClientId,
        quota_type: &QuotaType,
        config: &QuotaConfig,
        now: DateTime<Utc>,
    ) -> Result<QuotaUsage, QuotaError> {
        match self.cache.get::<QuotaUsage>(usage_key).await {
            Ok(Some(usage)) => {
                debug!("Quota usage cache hit for key: {}", usage_key);
                Ok(usage)
            }
            Ok(None) => {
                debug!("Quota usage cache miss for key: {}", usage_key);
                Ok(self.reset_quota_usage(client_id, quota_type, config, now))
            }
            Err(e) => {
                warn!("Quota usage cache error for key {}: {}, creating new", usage_key, e);
                Ok(self.reset_quota_usage(client_id, quota_type, config, now))
            }
        }
    }

    /// Reset quota usage for new period
    fn reset_quota_usage(
        &self,
        client_id: &QuotaClientId,
        quota_type: &QuotaType,
        config: &QuotaConfig,
        now: DateTime<Utc>,
    ) -> QuotaUsage {
        let period_duration = Duration::seconds(config.period.duration_seconds());
        
        QuotaUsage {
            client_id: client_id.clone(),
            quota_type: quota_type.clone(),
            current_usage: 0,
            limit: config.limit,
            period: config.period,
            period_start: now,
            period_end: now + period_duration,
            last_updated: now,
            burst_used: 0,
        }
    }

    /// Save quota usage to cache
    async fn save_quota_usage(&self, usage_key: &str, usage: &QuotaUsage) -> Result<(), QuotaError> {
        let ttl_seconds = (usage.period_end - Utc::now()).num_seconds().max(60); // At least 60 seconds
        
        match self.cache.set(usage_key, usage, Some(ttl_seconds)).await {
            Ok(_) => {
                debug!("Successfully cached quota usage: {}", usage_key);
                Ok(())
            }
            Err(e) => {
                warn!("Failed to cache quota usage {}: {}", usage_key, e);
                Err(QuotaError::CacheError(e.to_string()))
            }
        }
    }

    /// Get current quota status
    pub async fn get_quota_status(
        &self,
        client_id: &QuotaClientId,
        quota_type: &QuotaType,
        config: &QuotaConfig,
    ) -> Result<QuotaUsage, QuotaError> {
        let now = Utc::now();
        let period_key = config.period.get_period_key(now.timestamp());
        let usage_key = self.quota_usage_key(client_id, quota_type, period_key);
        
        self.get_quota_usage(&usage_key, client_id, quota_type, config, now).await
    }

    /// Reset quota for a client
    pub async fn reset_quota(
        &self,
        client_id: &QuotaClientId,
        quota_type: &QuotaType,
    ) -> Result<bool, QuotaError> {
        let now = Utc::now();
        // For reset, we can use current time as period key
        let period_key = QuotaPeriod::Hour.get_period_key(now.timestamp());
        let usage_key = self.quota_usage_key(client_id, quota_type, period_key);
        
        match self.cache.delete(&usage_key).await {
            Ok(deleted) => {
                if deleted {
                    info!("Reset quota for {} - {}", client_id, quota_type);
                }
                Ok(deleted)
            }
            Err(e) => {
                warn!("Failed to reset quota for {} - {}: {}", client_id, quota_type, e);
                Err(QuotaError::CacheError(e.to_string()))
            }
        }
    }

    /// Get quota statistics
    pub async fn get_statistics(&self) -> Result<HashMap<String, u64>, QuotaError> {
        let mut stats = HashMap::new();
        
        match self.cache.stats().await {
            Ok(cache_stats) => {
                stats.insert("total_quota_entries".to_string(), cache_stats.active_entries);
                stats.insert("cache_hits".to_string(), cache_stats.hits);
                stats.insert("cache_misses".to_string(), cache_stats.misses);
                stats.insert("memory_usage_bytes".to_string(), cache_stats.memory_usage);
                
                debug!("Retrieved quota management statistics: {:?}", stats);
                Ok(stats)
            }
            Err(e) => {
                warn!("Failed to get quota management statistics: {}", e);
                Err(QuotaError::CacheError(e.to_string()))
            }
        }
    }

    /// Bulk check quotas for multiple types
    pub async fn check_multiple_quotas(
        &self,
        client_id: &QuotaClientId,
        checks: Vec<(QuotaType, u64, QuotaConfig)>, // (quota_type, amount, config)
    ) -> Result<Vec<QuotaCheckResult>, QuotaError> {
        let mut results = Vec::new();
        
        for (quota_type, amount, config) in checks {
            let result = self.check_and_use_quota(client_id, &quota_type, amount, &config).await?;
            results.push(result);
            
            // If any quota is exceeded, we might want to stop (depending on business logic)
            // For now, continue checking all quotas
        }
        
        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infra::cache::CacheFactory;
    
    #[tokio::test]
    async fn test_basic_quota_management() {
        let cache = CacheFactory::with_fallback().await;
        let quota_service = QuotaManagementService::new(cache);
        let client_id = QuotaClientId::User(UserId::new("test_user".to_string()));
        let quota_config = QuotaConfig {
            quota_type: QuotaType::ApiRequests,
            limit: 100,
            period: QuotaPeriod::Hour,
            reset_time: None,
            burst_limit: Some(20), // Allow 20 extra requests as burst
            grace_period_seconds: None,
        };

        // First check should pass
        let result1 = quota_service
            .check_and_use_quota(&client_id, &QuotaType::ApiRequests, 10, &quota_config)
            .await
            .unwrap();
        assert!(result1.allowed);
        assert_eq!(result1.usage.current_usage, 10);

        // Use up to limit
        for i in 1..=9 {
            let result = quota_service
                .check_and_use_quota(&client_id, &QuotaType::ApiRequests, 10, &quota_config)
                .await
                .unwrap();
            assert!(result.allowed);
            assert_eq!(result.usage.current_usage, 10 + (i * 10));
        }

        // This should use burst capacity
        let result_burst = quota_service
            .check_and_use_quota(&client_id, &QuotaType::ApiRequests, 10, &quota_config)
            .await
            .unwrap();
        assert!(result_burst.allowed);
        assert!(result_burst.warning.is_some());

        // This should exceed burst limit
        let result_exceeded = quota_service
            .check_and_use_quota(&client_id, &QuotaType::ApiRequests, 15, &quota_config)
            .await
            .unwrap();
        assert!(!result_exceeded.allowed);
        assert!(result_exceeded.retry_after_seconds.is_some());
    }

    #[tokio::test]
    async fn test_quota_reset() {
        let cache = CacheFactory::with_fallback().await;
        let quota_service = QuotaManagementService::new(cache);
        let client_id = QuotaClientId::User(UserId::new("test_user_reset".to_string()));
        let quota_config = QuotaConfig {
            quota_type: QuotaType::ApiRequests,
            limit: 10,
            period: QuotaPeriod::Hour,
            reset_time: None,
            burst_limit: None,
            grace_period_seconds: None,
        };

        // Use quota
        let _ = quota_service
            .check_and_use_quota(&client_id, &QuotaType::ApiRequests, 10, &quota_config)
            .await
            .unwrap();

        // Should be at limit
        let result = quota_service
            .check_and_use_quota(&client_id, &QuotaType::ApiRequests, 1, &quota_config)
            .await
            .unwrap();
        assert!(!result.allowed);

        // Reset quota
        let reset_result = quota_service
            .reset_quota(&client_id, &QuotaType::ApiRequests)
            .await
            .unwrap();
        assert!(reset_result);

        // Should be able to use quota again
        let result_after_reset = quota_service
            .check_and_use_quota(&client_id, &QuotaType::ApiRequests, 5, &quota_config)
            .await
            .unwrap();
        assert!(result_after_reset.allowed);
        assert_eq!(result_after_reset.usage.current_usage, 5);
    }

    #[tokio::test]
    async fn test_different_quota_types() {
        let cache = CacheFactory::with_fallback().await;
        let quota_service = QuotaManagementService::new(cache);
        let client_id = QuotaClientId::User(UserId::new("test_user_multi".to_string()));
        
        let api_config = QuotaConfig {
            quota_type: QuotaType::ApiRequests,
            limit: 100,
            period: QuotaPeriod::Hour,
            reset_time: None,
            burst_limit: None,
            grace_period_seconds: None,
        };
        
        let data_config = QuotaConfig {
            quota_type: QuotaType::DataTransfer,
            limit: 1000000, // 1MB
            period: QuotaPeriod::Day,
            reset_time: None,
            burst_limit: None,
            grace_period_seconds: None,
        };

        // Different quota types should be independent
        let api_result = quota_service
            .check_and_use_quota(&client_id, &QuotaType::ApiRequests, 50, &api_config)
            .await
            .unwrap();
        assert!(api_result.allowed);
        assert_eq!(api_result.usage.current_usage, 50);

        let data_result = quota_service
            .check_and_use_quota(&client_id, &QuotaType::DataTransfer, 500000, &data_config)
            .await
            .unwrap();
        assert!(data_result.allowed);
        assert_eq!(data_result.usage.current_usage, 500000);

        // Each should maintain its own state
        let api_status = quota_service
            .get_quota_status(&client_id, &QuotaType::ApiRequests, &api_config)
            .await
            .unwrap();
        assert_eq!(api_status.current_usage, 50);

        let data_status = quota_service
            .get_quota_status(&client_id, &QuotaType::DataTransfer, &data_config)
            .await
            .unwrap();
        assert_eq!(data_status.current_usage, 500000);
    }

    #[test]
    fn test_quota_period_calculations() {
        let timestamp = 1700000000; // Some timestamp
        
        assert_eq!(QuotaPeriod::Minute.get_period_key(timestamp), timestamp / 60);
        assert_eq!(QuotaPeriod::Hour.get_period_key(timestamp), timestamp / 3600);
        assert_eq!(QuotaPeriod::Day.get_period_key(timestamp), timestamp / 86400);
        assert_eq!(QuotaPeriod::Custom(300).get_period_key(timestamp), timestamp / 300);
    }

    #[test]
    fn test_quota_usage_calculations() {
        let client_id = QuotaClientId::User(UserId::new("test_user".to_string()));
        let usage = QuotaUsage {
            client_id,
            quota_type: QuotaType::ApiRequests,
            current_usage: 75,
            limit: 100,
            period: QuotaPeriod::Hour,
            period_start: Utc::now(),
            period_end: Utc::now() + Duration::hours(1),
            last_updated: Utc::now(),
            burst_used: 0,
        };

        assert_eq!(usage.remaining(), 25);
        assert_eq!(usage.usage_percentage(), 75.0);
        assert!(!usage.is_exceeded());
        
        let exceeded_usage = QuotaUsage {
            current_usage: 105,
            limit: 100,
            ..usage
        };
        
        assert_eq!(exceeded_usage.remaining(), 0);
        assert_eq!(exceeded_usage.usage_percentage(), 105.0);
        assert!(exceeded_usage.is_exceeded());
    }
}