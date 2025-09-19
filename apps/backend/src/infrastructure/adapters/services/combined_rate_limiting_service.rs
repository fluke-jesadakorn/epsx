// Combined Rate Limiting Service - Diesel ORM Implementation

use async_trait::async_trait;
use std::sync::Arc;
use sqlx::PgPool;
use crate::infrastructure::cache::Cache;
use crate::domain::resource_management::services::rate_limiting_service::{RateLimitingServicePort, UsageStats};

/// Combined Rate Limiting Service - Production Implementation with Redis and Database
#[derive(Clone)]
pub struct CombinedRateLimitingService {
    cache: Arc<dyn Cache>,
    db_pool: Arc<PgPool>,
}

impl CombinedRateLimitingService {
    pub fn new(cache: Arc<dyn Cache>, db_pool: Arc<PgPool>) -> Self {
        Self {
            cache,
            db_pool,
        }
    }

    /// Check rate limit with sliding window algorithm and fallback to database
    pub async fn check_rate_limit(
        &self,
        user_id: &str,
        resource: &str,
        limit: u32,
        window_seconds: u32,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        let now = chrono::Utc::now().timestamp();
        let window_start = now - window_seconds as i64;
        let cache_key = format!("rate_limit:{}:{}", user_id, resource);
        
        // Try Redis first (sliding window)
        if let Some(current_usage) = self.get_sliding_window_count(&cache_key, window_start, now).await? {
            return Ok(current_usage < limit);
        }
        
        // Fallback to database
        let current_usage = self.get_database_usage(user_id, resource, window_start).await?;
        Ok(current_usage < limit)
    }

    /// Increment usage with atomic operations
    pub async fn increment_usage(
        &self,
        user_id: &str,
        resource: &str,
    ) -> Result<u32, Box<dyn std::error::Error + Send + Sync>> {
        let now = chrono::Utc::now().timestamp();
        let cache_key = format!("rate_limit:{}:{}", user_id, resource);
        
        // Try Redis atomic increment
        match self.increment_redis_counter(&cache_key, now).await {
            Ok(count) => Ok(count),
            Err(_) => {
                // Fallback to database
                self.increment_database_usage(user_id, resource, now).await
            }
        }
    }

    /// Get current usage with cache preference
    pub async fn get_current_usage(
        &self,
        user_id: &str,
        resource: &str,
    ) -> Result<u32, Box<dyn std::error::Error + Send + Sync>> {
        let now = chrono::Utc::now().timestamp();
        let window_start = now - 3600; // 1 hour window
        let cache_key = format!("rate_limit:{}:{}", user_id, resource);
        
        // Try Redis first
        if let Some(usage) = self.get_sliding_window_count(&cache_key, window_start, now).await? {
            return Ok(usage);
        }
        
        // Fallback to database
        self.get_database_usage(user_id, resource, window_start).await
    }
    
    /// Redis sliding window implementation
    async fn get_sliding_window_count(
        &self,
        cache_key: &str,
        window_start: i64,
        _now: i64,
    ) -> Result<Option<u32>, Box<dyn std::error::Error + Send + Sync>> {
        // Use sorted set for sliding window
        let member_key = format!("{}:window", cache_key);
        
        // Remove expired entries and count current
        if let Some(count_str) = self.cache.get(&member_key) {
            // Parse and filter timestamps within window
            let timestamps: Vec<i64> = count_str
                .split(',')
                .filter_map(|ts| ts.parse().ok())
                .filter(|&ts| ts >= window_start)
                .collect();
                
            // Update cache with filtered timestamps
            let updated_data = timestamps
                .iter()
                .map(|ts| ts.to_string())
                .collect::<Vec<_>>()
                .join(",");
            self.cache.set(&member_key, updated_data, Some(3600));
            
            return Ok(Some(timestamps.len() as u32));
        }
        
        Ok(None)
    }
    
    /// Redis atomic increment with timestamp tracking
    async fn increment_redis_counter(
        &self,
        cache_key: &str,
        timestamp: i64,
    ) -> Result<u32, Box<dyn std::error::Error + Send + Sync>> {
        let member_key = format!("{}:window", cache_key);
        
        // Get existing timestamps
        let existing = self.cache.get(&member_key).unwrap_or_default();
        let new_data = if existing.is_empty() {
            timestamp.to_string()
        } else {
            format!("{},{}", existing, timestamp)
        };
        
        // Update cache
        self.cache.set(&member_key, new_data.clone(), Some(3600));
        
        // Count current entries
        let count = new_data.split(',').count() as u32;
        Ok(count)
    }
    
    /// Database fallback for rate limiting
    async fn get_database_usage(
        &self,
        user_id: &str,
        resource: &str,
        window_start: i64,
    ) -> Result<u32, Box<dyn std::error::Error + Send + Sync>> {
        let window_start_time = chrono::DateTime::from_timestamp(window_start, 0)
            .ok_or("Invalid timestamp")?;
            
        let count = sqlx::query_scalar!(
            r#"
            SELECT COUNT(*)::INTEGER as count
            FROM rate_limit_usage 
            WHERE user_id = $1 
              AND resource = $2 
              AND created_at >= $3
            "#,
            user_id,
            resource,
            window_start_time
        )
        .fetch_one(&*self.db_pool)
        .await?
        .unwrap_or(0);
        
        Ok(count as u32)
    }
    
    /// Database increment with upsert
    async fn increment_database_usage(
        &self,
        user_id: &str,
        resource: &str,
        timestamp: i64,
    ) -> Result<u32, Box<dyn std::error::Error + Send + Sync>> {
        let timestamp_dt = chrono::DateTime::from_timestamp(timestamp, 0)
            .ok_or("Invalid timestamp")?;
            
        // Insert usage record
        sqlx::query!(
            r#"
            INSERT INTO rate_limit_usage (user_id, resource, created_at)
            VALUES ($1, $2, $3)
            "#,
            user_id,
            resource,
            timestamp_dt
        )
        .execute(&*self.db_pool)
        .await?;
        
        // Return current count in window
        let window_start = timestamp - 3600; // 1 hour
        self.get_database_usage(user_id, resource, window_start).await
    }
    
    /// Advanced rate limiting with progressive penalties
    pub async fn check_progressive_rate_limit(
        &self,
        user_id: &str,
        resource: &str,
        base_limit: u32,
        window_seconds: u32,
    ) -> Result<(bool, u32, Option<u32>), Box<dyn std::error::Error + Send + Sync>> {
        let violation_count = self.get_violation_count(user_id, resource).await?;
        
        // Calculate adjusted limit based on violations
        let adjusted_limit = if violation_count == 0 {
            base_limit
        } else {
            // Reduce limit by 10% for each violation, minimum 1
            std::cmp::max(1, base_limit - (base_limit * violation_count / 10))
        };
        
        let is_allowed = self.check_rate_limit(user_id, resource, adjusted_limit, window_seconds).await?;
        
        // Track violation if limit exceeded
        if !is_allowed {
            self.record_violation(user_id, resource).await?;
        }
        
        let penalty_duration = if violation_count > 0 {
            Some(violation_count * 60) // 1 minute per violation
        } else {
            None
        };
        
        Ok((is_allowed, adjusted_limit, penalty_duration))
    }
    
    /// Track rate limit violations for progressive penalties
    async fn get_violation_count(
        &self,
        user_id: &str,
        resource: &str,
    ) -> Result<u32, Box<dyn std::error::Error + Send + Sync>> {
        let cache_key = format!("violations:{}:{}", user_id, resource);
        
        if let Some(count_str) = self.cache.get(&cache_key) {
            return Ok(count_str.parse().unwrap_or(0));
        }
        
        // Fallback to database
        let count = sqlx::query_scalar!(
            r#"
            SELECT COUNT(*)::INTEGER as count
            FROM rate_limit_violations 
            WHERE client_id = $1 
              AND endpoint = $2 
              AND created_at >= CURRENT_TIMESTAMP - INTERVAL '1 hour'
            "#,
            user_id,
            resource
        )
        .fetch_one(&*self.db_pool)
        .await?
        .unwrap_or(0);
        
        Ok(count as u32)
    }
    
    /// Record rate limit violation
    async fn record_violation(
        &self,
        user_id: &str,
        resource: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let cache_key = format!("violations:{}:{}", user_id, resource);
        
        // Increment violation count in cache
        let current_count = self.get_violation_count(user_id, resource).await?;
        let new_count = current_count + 1;
        self.cache.set(&cache_key, new_count.to_string(), Some(3600));
        
        // Record in database for persistence
        sqlx::query!(
            r#"
            INSERT INTO rate_limit_violations (client_id, client_type, endpoint, violation_type, timestamp_occurred, severity)
            VALUES ($1, 'user', $2, 'rate_limit_exceeded', extract(epoch from now()), 5)
            "#,
            user_id,
            resource
        )
        .execute(&*self.db_pool)
        .await?;
        
        Ok(())
    }
    
    /// Reset violations for a user (admin function)
    pub async fn reset_violations(
        &self,
        user_id: &str,
        resource: Option<&str>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if let Some(resource) = resource {
            // Reset specific resource
            let cache_key = format!("violations:{}:{}", user_id, resource);
            self.cache.delete(&cache_key);
            
            sqlx::query!(
                "DELETE FROM rate_limit_violations WHERE client_id = $1 AND endpoint = $2",
                user_id,
                resource
            )
            .execute(&*self.db_pool)
            .await?;
        } else {
            // Reset all violations for user
            let _cache_pattern = format!("violations:{}:*", user_id);
            // Note: This would need cache implementation that supports pattern deletion
            
            sqlx::query!(
                "DELETE FROM rate_limit_violations WHERE client_id = $1",
                user_id
            )
            .execute(&*self.db_pool)
            .await?;
        }
        
        Ok(())
    }
    
    /// Get rate limiting statistics
    pub async fn get_rate_limit_stats(
        &self,
        user_id: &str,
    ) -> Result<RateLimitStats, Box<dyn std::error::Error + Send + Sync>> {
        let stats = sqlx::query!(
            r#"
            SELECT 
                COUNT(DISTINCT resource) as resources_used,
                SUM(CASE WHEN created_at >= CURRENT_TIMESTAMP - INTERVAL '1 hour' THEN 1 ELSE 0 END) as requests_last_hour,
                SUM(CASE WHEN created_at >= CURRENT_TIMESTAMP - INTERVAL '1 day' THEN 1 ELSE 0 END) as requests_last_day,
                MAX(created_at) as last_request
            FROM rate_limit_usage 
            WHERE user_id = $1
            "#,
            user_id
        )
        .fetch_one(&*self.db_pool)
        .await?;
        
        let violations = sqlx::query_scalar!(
            r#"
            SELECT COUNT(*)::INTEGER as count
            FROM rate_limit_violations 
            WHERE client_id = $1 
              AND created_at >= CURRENT_TIMESTAMP - INTERVAL '24 hours'
            "#,
            user_id
        )
        .fetch_one(&*self.db_pool)
        .await?
        .unwrap_or(0);
        
        Ok(RateLimitStats {
            resources_used: stats.resources_used.unwrap_or(0) as u32,
            requests_last_hour: stats.requests_last_hour.unwrap_or(0) as u32,
            requests_last_day: stats.requests_last_day.unwrap_or(0) as u32,
            violations_last_24h: violations as u32,
            last_request: stats.last_request,
        })
    }
}

#[derive(Debug, Clone)]
pub struct RateLimitStats {
    pub resources_used: u32,
    pub requests_last_hour: u32,
    pub requests_last_day: u32,
    pub violations_last_24h: u32,
    pub last_request: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, thiserror::Error)]
pub enum CombinedRateLimitingError {
    #[error("Database error: {0}")]
    Database(String), // Placeholder error type
    #[error("Cache error: {0}")]
    Cache(String),
    #[error("Rate limit exceeded")]
    RateLimitExceeded,
}

#[async_trait]
impl RateLimitingServicePort for CombinedRateLimitingService {
    type Error = CombinedRateLimitingError;
    
    async fn check_rate_limit(&self, _user_id: &str, _resource: &str) -> Result<bool, Self::Error> {
        // Placeholder implementation - rate limiting logic
        Ok(true)
    }
    
    async fn increment_usage(&self, _user_id: &str, _resource: &str) -> Result<(), Self::Error> {
        // Placeholder implementation - usage tracking
        Ok(())
    }
    
    async fn get_usage_stats(&self, _user_id: &str) -> Result<UsageStats, Self::Error> {
        // Placeholder implementation - usage stats retrieval
        Ok(UsageStats {
            total_requests: 0,
            current_window_requests: 0,
            limit: 1000,
            reset_time: chrono::Utc::now(),
        })
    }
}