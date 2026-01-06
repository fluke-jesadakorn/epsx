//! Multi-Level Rate Limiter
//!
//! Hierarchical rate limiting with 3 levels:
//! 1. Global - Protects entire API from overload
//! 2. Plan-based - Limits per subscription plan
//! 3. API Key - Per-key limits (uses existing UnifiedRateLimiter)

use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};
use tracing::{debug, warn};

use crate::infrastructure::cache::Cache;
use crate::config::Config;
use super::rate_limiter::{UnifiedRateLimiter, RateLimitConfig, ClientId, RateLimitError, RateLimitResult};

// ============================================================================
// Global Rate Limit Configuration
// ============================================================================

/// Global rate limit configuration - protects the entire API
#[derive(Debug, Clone)]
pub struct GlobalRateLimitConfig {
    /// Maximum requests per second across all clients
    pub requests_per_second: u32,
    /// Maximum requests per minute across all clients  
    pub requests_per_minute: u32,
    /// Whether global rate limiting is enabled
    pub enabled: bool,
}

impl Default for GlobalRateLimitConfig {
    fn default() -> Self {
        Self {
            requests_per_second: 10000,   // 10k/sec default
            requests_per_minute: 500000,  // 500k/min default
            enabled: true,
        }
    }
}

impl GlobalRateLimitConfig {
    /// Load from environment variables
    pub fn from_env() -> Self {
        Self {
            requests_per_second: std::env::var("GLOBAL_RATE_LIMIT_PER_SECOND")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(10000),
            requests_per_minute: std::env::var("GLOBAL_RATE_LIMIT_PER_MINUTE")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(500000),
            enabled: std::env::var("GLOBAL_RATE_LIMIT_ENABLED")
                .map(|s| s.to_lowercase() != "false")
                .unwrap_or(true),
        }
    }
}

// ============================================================================
// Plan-Based Rate Limits
// ============================================================================

/// Rate limits associated with a subscription plan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanRateLimits {
    pub plan_id: Option<i32>,
    pub plan_name: String,
    pub requests_per_minute: u32,
    pub requests_per_hour: u32,
    pub requests_per_day: u32,
    pub burst_capacity: u32,
}

impl Default for PlanRateLimits {
    fn default() -> Self {
        Self {
            plan_id: None,
            plan_name: "free".to_string(),
            requests_per_minute: 60,
            requests_per_hour: 1000,
            requests_per_day: 10000,
            burst_capacity: 10,
        }
    }
}

impl PlanRateLimits {
    /// Create rate limits for free tier
    pub fn free() -> Self {
        Self::default()
    }

    /// Create rate limits for basic tier
    pub fn basic() -> Self {
        Self {
            plan_id: None,
            plan_name: "basic".to_string(),
            requests_per_minute: 120,
            requests_per_hour: 3000,
            requests_per_day: 50000,
            burst_capacity: 20,
        }
    }

    /// Create rate limits for premium tier
    pub fn premium() -> Self {
        Self {
            plan_id: None,
            plan_name: "premium".to_string(),
            requests_per_minute: 300,
            requests_per_hour: 10000,
            requests_per_day: 200000,
            burst_capacity: 50,
        }
    }

    /// Create rate limits for enterprise tier
    pub fn enterprise() -> Self {
        Self {
            plan_id: None,
            plan_name: "enterprise".to_string(),
            requests_per_minute: 1000,
            requests_per_hour: 50000,
            requests_per_day: 1000000,
            burst_capacity: 200,
        }
    }

    /// Create rate limits for elite tier (alias for enterprise)
    pub fn elite() -> Self {
        Self {
            plan_id: None,
            plan_name: "elite".to_string(),
            requests_per_minute: 1000,
            requests_per_hour: 50000,
            requests_per_day: 1000000,
            burst_capacity: 200,
        }
    }

    /// Convert to RateLimitConfig for UnifiedRateLimiter
    pub fn to_rate_limit_config(&self) -> RateLimitConfig {
        RateLimitConfig {
            requests_per_minute: Some(self.requests_per_minute),
            requests_per_hour: Some(self.requests_per_hour),
            requests_per_day: Some(self.requests_per_day),
        }
    }
}

// ============================================================================
// Global Rate Limiter (Level 1)
// ============================================================================

/// Global rate limit entry for tracking aggregate request counts
#[derive(Debug, Clone, Serialize, Deserialize)]
struct GlobalRateLimitEntry {
    second_count: u32,
    minute_count: u32,
    second_window: u64,
    minute_window: u64,
    last_updated: u64,
}

impl GlobalRateLimitEntry {
    fn new(timestamp: u64) -> Self {
        Self {
            second_count: 0,
            minute_count: 0,
            second_window: timestamp,
            minute_window: timestamp / 60,
            last_updated: timestamp,
        }
    }

    fn update(&mut self, timestamp: u64) {
        let current_second = timestamp;
        let current_minute = timestamp / 60;

        // Reset second counter if new second
        if current_second != self.second_window {
            self.second_count = 0;
            self.second_window = current_second;
        }

        // Reset minute counter if new minute
        if current_minute != self.minute_window {
            self.minute_count = 0;
            self.minute_window = current_minute;
        }

        self.second_count += 1;
        self.minute_count += 1;
        self.last_updated = timestamp;
    }

    fn is_allowed(&self, config: &GlobalRateLimitConfig) -> bool {
        self.second_count <= config.requests_per_second
            && self.minute_count <= config.requests_per_minute
    }
}

/// Global rate limiter - Level 1 protection
pub struct GlobalRateLimiter {
    cache: Arc<dyn Cache>,
    config: GlobalRateLimitConfig,
}

impl GlobalRateLimiter {
    const CACHE_KEY: &'static str = "rate_limit:global:v1";

    pub fn new(cache: Arc<dyn Cache>, config: GlobalRateLimitConfig) -> Self {
        Self { cache, config }
    }

    /// Check if global rate limit allows the request
    pub async fn check(&self) -> Result<bool, RateLimitError> {
        if !self.config.enabled {
            return Ok(true);
        }

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| RateLimitError::SystemTimeError(e.to_string()))?
            .as_secs();

        // Get or create entry
        let mut entry = match self.cache.get(Self::CACHE_KEY) {
            Some(data) => serde_json::from_str::<GlobalRateLimitEntry>(&data)
                .unwrap_or_else(|_| GlobalRateLimitEntry::new(now)),
            None => GlobalRateLimitEntry::new(now),
        };

        // Update counters
        entry.update(now);

        // Check limits
        let allowed = entry.is_allowed(&self.config);

        // Save updated entry
        if let Ok(json) = serde_json::to_string(&entry) {
            self.cache.set(Self::CACHE_KEY, json, Some(120)); // 2 min TTL
        }

        if !allowed {
            warn!(
                "Global rate limit exceeded: {}/{} per second, {}/{} per minute",
                entry.second_count, self.config.requests_per_second,
                entry.minute_count, self.config.requests_per_minute
            );
        }

        Ok(allowed)
    }

    /// Get current global rate limit statistics
    pub async fn get_stats(&self) -> Option<(u32, u32, u32, u32)> {
        self.cache.get(Self::CACHE_KEY).and_then(|data| {
            serde_json::from_str::<GlobalRateLimitEntry>(&data).ok().map(|entry| {
                (
                    entry.second_count,
                    self.config.requests_per_second,
                    entry.minute_count,
                    self.config.requests_per_minute,
                )
            })
        })
    }
}

// ============================================================================
// Plan Rate Limiter (Level 2)
// ============================================================================

/// Plan-based rate limiter - Level 2 protection
pub struct PlanRateLimiter {
    cache: Arc<dyn Cache>,
}

impl PlanRateLimiter {
    pub fn new(cache: Arc<dyn Cache>) -> Self {
        Self { cache }
    }

    fn cache_key(user_id: &str) -> String {
        format!("rate_limit:plan:v1:{}", user_id)
    }

    /// Check if plan rate limit allows the request
    pub async fn check(
        &self,
        user_id: &str,
        plan_limits: &PlanRateLimits,
    ) -> Result<RateLimitResult, RateLimitError> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| RateLimitError::SystemTimeError(e.to_string()))?
            .as_secs();

        let cache_key = Self::cache_key(user_id);
        let config = plan_limits.to_rate_limit_config();

        // Reuse RateLimitEntry logic from UnifiedRateLimiter
        #[derive(Debug, Clone, Serialize, Deserialize)]
        struct PlanEntry {
            minute_count: u32,
            hour_count: u32,
            day_count: u32,
            minute_window: u64,
            hour_window: u64,
            day_window: u64,
            last_updated: u64,
        }

        let mut entry = match self.cache.get(&cache_key) {
            Some(data) => serde_json::from_str::<PlanEntry>(&data).unwrap_or(PlanEntry {
                minute_count: 0,
                hour_count: 0,
                day_count: 0,
                minute_window: now / 60,
                hour_window: now / 3600,
                day_window: now / 86400,
                last_updated: now,
            }),
            None => PlanEntry {
                minute_count: 0,
                hour_count: 0,
                day_count: 0,
                minute_window: now / 60,
                hour_window: now / 3600,
                day_window: now / 86400,
                last_updated: now,
            },
        };

        // Reset counters on window change
        let current_minute = now / 60;
        let current_hour = now / 3600;
        let current_day = now / 86400;

        if current_minute != entry.minute_window {
            entry.minute_count = 0;
            entry.minute_window = current_minute;
        }
        if current_hour != entry.hour_window {
            entry.hour_count = 0;
            entry.hour_window = current_hour;
        }
        if current_day != entry.day_window {
            entry.day_count = 0;
            entry.day_window = current_day;
        }

        // Increment counters
        entry.minute_count += 1;
        entry.hour_count += 1;
        entry.day_count += 1;
        entry.last_updated = now;

        // Check limits
        let allowed = config.requests_per_minute.map_or(true, |l| entry.minute_count <= l)
            && config.requests_per_hour.map_or(true, |l| entry.hour_count <= l)
            && config.requests_per_day.map_or(true, |l| entry.day_count <= l);

        // Save updated entry
        if let Ok(json) = serde_json::to_string(&entry) {
            self.cache.set(&cache_key, json, Some(86400)); // 24h TTL
        }

        let limit = config.requests_per_minute.unwrap_or(0);
        let _remaining = limit.saturating_sub(entry.minute_count);

        if !allowed {
            debug!(
                "Plan rate limit exceeded for user {}: plan={}, minute={}/{}, hour={}/{}, day={}/{}",
                user_id,
                plan_limits.plan_name,
                entry.minute_count,
                config.requests_per_minute.unwrap_or(0),
                entry.hour_count,
                config.requests_per_hour.unwrap_or(0),
                entry.day_count,
                config.requests_per_day.unwrap_or(0)
            );
        }

        Ok(RateLimitResult {
            allowed,
            reason: if allowed {
                format!("Plan {} rate limit OK", plan_limits.plan_name)
            } else {
                format!("Plan {} rate limit exceeded", plan_limits.plan_name)
            },
            retry_after_seconds: if allowed { None } else { Some(60) },
            window: super::rate_limiter::TimeWindow::Minute,
            current_count: entry.minute_count,
            limit,
        })
    }
}

// ============================================================================
// Multi-Level Rate Limiter (Combined)
// ============================================================================

/// Multi-level rate limiting result
#[derive(Debug, Clone)]
pub struct MultiLevelRateLimitResult {
    pub allowed: bool,
    pub blocked_at_level: Option<RateLimitLevel>,
    pub global_remaining: Option<(u32, u32)>,  // (current, limit)
    pub plan_remaining: Option<(u32, u32)>,    // (current, limit)
    pub api_key_remaining: Option<(u32, u32)>, // (current, limit)
    pub retry_after_seconds: Option<u64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RateLimitLevel {
    Global,
    Plan,
    ApiKey,
}

impl std::fmt::Display for RateLimitLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RateLimitLevel::Global => write!(f, "global"),
            RateLimitLevel::Plan => write!(f, "plan"),
            RateLimitLevel::ApiKey => write!(f, "api_key"),
        }
    }
}

/// Multi-level rate limiter combining all 3 levels
pub struct MultiLevelRateLimiter {
    global: GlobalRateLimiter,
    plan: PlanRateLimiter,
    api_key: UnifiedRateLimiter,
}

impl MultiLevelRateLimiter {
    /// Create a new multi-level rate limiter
    pub fn new(cache: Arc<dyn Cache>, _config: Arc<Config>) -> Self {
        let global_config = GlobalRateLimitConfig::from_env();
        


        Self {
            global: GlobalRateLimiter::new(cache.clone(), global_config),
            plan: PlanRateLimiter::new(cache.clone()),
            api_key: UnifiedRateLimiter::new(cache),
        }
    }

    /// Check all rate limit levels
    ///
    /// # Arguments
    /// * `user_id` - Optional user ID (for plan-based limiting)
    /// * `plan_limits` - Optional plan rate limits (if user has a subscription)
    /// * `api_key_id` - Optional API key (for API key-based limiting)
    /// * `api_key_config` - Optional custom API key rate limit config
    /// * `endpoint` - The endpoint being accessed
    /// * `method` - HTTP method
    pub async fn check(
        &self,
        user_id: Option<&str>,
        plan_limits: Option<&PlanRateLimits>,
        api_key_id: Option<&str>,
        api_key_config: Option<&RateLimitConfig>,
        endpoint: &str,
        method: &str,
    ) -> Result<MultiLevelRateLimitResult, RateLimitError> {
        // Level 1: Global rate limit
        let global_allowed = self.global.check().await?;
        if !global_allowed {
            return Ok(MultiLevelRateLimitResult {
                allowed: false,
                blocked_at_level: Some(RateLimitLevel::Global),
                global_remaining: self.global.get_stats().await.map(|(c, l, _, _)| (c, l)),
                plan_remaining: None,
                api_key_remaining: None,
                retry_after_seconds: Some(1), // Retry after 1 second for global
            });
        }

        // Level 2: Plan-based rate limit (if user authenticated with plan)
        let mut plan_remaining = None;
        if let (Some(uid), Some(limits)) = (user_id, plan_limits) {
            let plan_result = self.plan.check(uid, limits).await?;
            plan_remaining = Some((plan_result.current_count, plan_result.limit));
            
            if !plan_result.allowed {
                return Ok(MultiLevelRateLimitResult {
                    allowed: false,
                    blocked_at_level: Some(RateLimitLevel::Plan),
                    global_remaining: self.global.get_stats().await.map(|(c, l, _, _)| (c, l)),
                    plan_remaining,
                    api_key_remaining: None,
                    retry_after_seconds: plan_result.retry_after_seconds,
                });
            }
        }

        // Level 3: API key rate limit (if using API key)
        let mut api_key_remaining = None;
        if let Some(key_id) = api_key_id {
            let config = api_key_config.cloned().unwrap_or_default();
            let client_id = ClientId::ApiKey(key_id.to_string());
            let key_result = self.api_key.check_client_rate_limit(&client_id, endpoint, method, &config).await?;
            api_key_remaining = Some((key_result.current_count, key_result.limit));

            if !key_result.allowed {
                return Ok(MultiLevelRateLimitResult {
                    allowed: false,
                    blocked_at_level: Some(RateLimitLevel::ApiKey),
                    global_remaining: self.global.get_stats().await.map(|(c, l, _, _)| (c, l)),
                    plan_remaining,
                    api_key_remaining,
                    retry_after_seconds: key_result.retry_after_seconds,
                });
            }
        }

        // All levels passed
        Ok(MultiLevelRateLimitResult {
            allowed: true,
            blocked_at_level: None,
            global_remaining: self.global.get_stats().await.map(|(c, l, _, _)| (c, l)),
            plan_remaining,
            api_key_remaining,
            retry_after_seconds: None,
        })
    }

    /// Check only global rate limit (for unauthenticated requests)
    pub async fn check_global_only(&self) -> Result<bool, RateLimitError> {
        self.global.check().await
    }

    /// Check global + plan limits (for authenticated users without API key)
    pub async fn check_user(
        &self,
        user_id: &str,
        plan_limits: &PlanRateLimits,
    ) -> Result<MultiLevelRateLimitResult, RateLimitError> {
        self.check(
            Some(user_id),
            Some(plan_limits),
            None,
            None,
            "*",
            "*",
        ).await
    }

    /// Check all levels for API key requests
    pub async fn check_api_key(
        &self,
        user_id: &str,
        plan_limits: &PlanRateLimits,
        api_key_id: &str,
        api_key_config: &RateLimitConfig,
        endpoint: &str,
        method: &str,
    ) -> Result<MultiLevelRateLimitResult, RateLimitError> {
        self.check(
            Some(user_id),
            Some(plan_limits),
            Some(api_key_id),
            Some(api_key_config),
            endpoint,
            method,
        ).await
    }

    /// Get global rate limit statistics
    pub async fn global_stats(&self) -> Option<(u32, u32, u32, u32)> {
        self.global.get_stats().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::cache::MemoryCache;

    #[tokio::test]
    async fn test_global_rate_limiter() {
        let cache: Arc<dyn Cache> = Arc::new(MemoryCache::new());
        let config = GlobalRateLimitConfig {
            requests_per_second: 5,
            requests_per_minute: 100,
            enabled: true,
        };
        let limiter = GlobalRateLimiter::new(cache, config);

        // First 5 requests should pass
        for i in 1..=5 {
            let result = limiter.check().await.unwrap();
            assert!(result, "Request {} should be allowed", i);
        }

        // 6th request should fail (exceeds per-second limit)
        let result = limiter.check().await.unwrap();
        assert!(!result, "6th request should be denied");
    }

    #[tokio::test]
    async fn test_plan_rate_limiter() {
        let cache: Arc<dyn Cache> = Arc::new(MemoryCache::new());
        let limiter = PlanRateLimiter::new(cache);
        let limits = PlanRateLimits {
            plan_id: Some(1),
            plan_name: "test".to_string(),
            requests_per_minute: 3,
            requests_per_hour: 100,
            requests_per_day: 1000,
            burst_capacity: 5,
        };

        // First 3 requests should pass
        for i in 1..=3 {
            let result = limiter.check("user123", &limits).await.unwrap();
            assert!(result.allowed, "Request {} should be allowed", i);
        }

        // 4th request should fail
        let result = limiter.check("user123", &limits).await.unwrap();
        assert!(!result.allowed, "4th request should be denied");
    }

    #[tokio::test]
    async fn test_multi_level_rate_limiter() {
        let cache: Arc<dyn Cache> = Arc::new(MemoryCache::new());
        let config = Arc::new(Config::from_env().expect("Config"));
        let limiter = MultiLevelRateLimiter::new(cache, config);

        let plan_limits = PlanRateLimits::free();
        
        // Check user rate limit
        let result = limiter.check_user("user456", &plan_limits).await.unwrap();
        assert!(result.allowed);
        assert!(result.blocked_at_level.is_none());
    }
}
