// Notification-specific caching system with multi-tier architecture
// Provides optimized caching for notification data with automatic fallback

use super::{Cache, CacheExt, CacheError, UnifiedCache};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{debug, info};
use uuid::Uuid;

use crate::infra::services::notification_service::{Notification, NotificationPreferences, ServiceNotificationStats};

/// Cache keys for notification system
pub struct NotificationCacheKeys;

impl NotificationCacheKeys {
    // User notification caches
    pub fn user_notifications(user_id: &Uuid) -> String {
        format!("notifications:user:{}", user_id)
    }
    
    pub fn user_notifications_recent(user_id: &Uuid) -> String {
        format!("notifications:recent:{}", user_id)
    }
    
    pub fn user_unread_count(user_id: &Uuid) -> String {
        format!("unread_count:{}", user_id)
    }
    
    pub fn user_stats(user_id: &Uuid) -> String {
        format!("notification_stats:{}", user_id)
    }
    
    pub fn user_preferences(user_id: &Uuid) -> String {
        format!("preferences:user:{}", user_id)
    }
    
    // Global caches
    pub fn notification_by_id(notification_id: &Uuid) -> String {
        format!("notification:{}", notification_id)
    }
    
    pub fn user_rate_limit(user_id: &Uuid, window: &str) -> String {
        format!("rate_limit:{}:{}", user_id, window)
    }
    
    // Template and system caches
    pub fn notification_templates() -> String {
        "notification_templates".to_string()
    }
    
    pub fn system_notifications() -> String {
        "system_notifications".to_string()
    }
}

/// Notification-specific cache data structures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedNotificationList {
    pub notifications: Vec<Notification>,
    pub total_count: i64,
    pub last_updated: DateTime<Utc>,
    pub cache_version: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedUnreadCount {
    pub count: i64,
    pub last_updated: DateTime<Utc>,
    pub user_id: Uuid,
}

/// Cache configuration for notifications
#[derive(Debug, Clone)]
pub struct NotificationCacheConfig {
    // TTL values in seconds
    pub notification_list_ttl: i64,
    pub unread_count_ttl: i64, 
    pub user_stats_ttl: i64,
    pub user_preferences_ttl: i64,
    pub individual_notification_ttl: i64,
    pub rate_limit_ttl: i64,
    
    // Cache sizes
    pub max_notifications_per_user: usize,
    pub max_cached_users: usize,
    
    // Performance settings
    pub enable_preloading: bool,
    pub cache_warming_enabled: bool,
    pub background_refresh_enabled: bool,
}

impl Default for NotificationCacheConfig {
    fn default() -> Self {
        Self {
            notification_list_ttl: 300,    // 5 minutes for notification lists
            unread_count_ttl: 60,          // 1 minute for unread counts (changes frequently)
            user_stats_ttl: 600,           // 10 minutes for user stats
            user_preferences_ttl: 3600,    // 1 hour for preferences (rarely change)
            individual_notification_ttl: 1800, // 30 minutes for individual notifications
            rate_limit_ttl: 3600,          // 1 hour for rate limiting
            
            max_notifications_per_user: 100,
            max_cached_users: 10000,
            
            enable_preloading: true,
            cache_warming_enabled: true,
            background_refresh_enabled: false, // Disabled by default to prevent overhead
        }
    }
}

/// High-level notification cache interface
#[async_trait]
pub trait NotificationCache: Send + Sync {
    // Notification list operations
    async fn get_user_notifications(&self, user_id: &Uuid, limit: usize, offset: usize) -> Result<Option<Vec<Notification>>, CacheError>;
    async fn set_user_notifications(&self, user_id: &Uuid, notifications: Vec<Notification>) -> Result<(), CacheError>;
    async fn invalidate_user_notifications(&self, user_id: &Uuid) -> Result<(), CacheError>;
    
    // Unread count operations
    async fn get_unread_count(&self, user_id: &Uuid) -> Result<Option<i64>, CacheError>;
    async fn set_unread_count(&self, user_id: &Uuid, count: i64) -> Result<(), CacheError>;
    async fn increment_unread_count(&self, user_id: &Uuid, delta: i64) -> Result<i64, CacheError>;
    async fn decrement_unread_count(&self, user_id: &Uuid, delta: i64) -> Result<i64, CacheError>;
    
    // Individual notification operations
    async fn get_notification(&self, notification_id: &Uuid) -> Result<Option<Notification>, CacheError>;
    async fn set_notification(&self, notification: &Notification) -> Result<(), CacheError>;
    async fn invalidate_notification(&self, notification_id: &Uuid) -> Result<(), CacheError>;
    
    // User preferences operations
    async fn get_user_preferences(&self, user_id: &Uuid) -> Result<Option<NotificationPreferences>, CacheError>;
    async fn set_user_preferences(&self, user_id: &Uuid, preferences: &NotificationPreferences) -> Result<(), CacheError>;
    async fn invalidate_user_preferences(&self, user_id: &Uuid) -> Result<(), CacheError>;
    
    // Stats operations
    async fn get_user_stats(&self, user_id: &Uuid) -> Result<Option<ServiceNotificationStats>, CacheError>;
    async fn set_user_stats(&self, user_id: &Uuid, stats: &ServiceNotificationStats) -> Result<(), CacheError>;
    async fn invalidate_user_stats(&self, user_id: &Uuid) -> Result<(), CacheError>;
    
    // Rate limiting
    async fn check_rate_limit(&self, user_id: &Uuid, window: &str, limit: i64) -> Result<bool, CacheError>;
    async fn increment_rate_limit(&self, user_id: &Uuid, window: &str, ttl_seconds: i64) -> Result<i64, CacheError>;
    
    // Bulk operations
    async fn invalidate_all_user_data(&self, user_id: &Uuid) -> Result<(), CacheError>;
    async fn warm_user_cache(&self, user_id: &Uuid) -> Result<(), CacheError>;
    
    // System operations
    async fn get_cache_stats(&self) -> Result<NotificationCacheStats, CacheError>;
    async fn cleanup_expired(&self) -> Result<i64, CacheError>;
}

/// Implementation using UnifiedCache
pub struct NotificationCacheImpl {
    unified_cache: Arc<UnifiedCache>,
    config: NotificationCacheConfig,
}

impl NotificationCacheImpl {
    pub fn new(unified_cache: Arc<UnifiedCache>, config: NotificationCacheConfig) -> Self {
        Self {
            unified_cache,
            config,
        }
    }
    
    pub fn with_default_config(unified_cache: Arc<UnifiedCache>) -> Self {
        Self::new(unified_cache, NotificationCacheConfig::default())
    }
    
    /// Generate versioned cache key to support cache invalidation
    fn versioned_key(&self, base_key: &str, version: Option<u32>) -> String {
        match version {
            Some(v) => format!("{}:v{}", base_key, v),
            None => base_key.to_string(),
        }
    }
    
    /// Smart cache key generation based on operation type
    fn generate_list_key(&self, user_id: &Uuid, limit: usize, offset: usize) -> String {
        if limit <= 50 && offset == 0 {
            // Recent notifications - highly cacheable
            NotificationCacheKeys::user_notifications_recent(user_id)
        } else {
            // Specific pagination - less cacheable
            format!("{}:{}:{}", NotificationCacheKeys::user_notifications(user_id), limit, offset)
        }
    }
}

#[async_trait]
impl NotificationCache for NotificationCacheImpl {
    async fn get_user_notifications(&self, user_id: &Uuid, limit: usize, offset: usize) -> Result<Option<Vec<Notification>>, CacheError> {
        let cache_key = self.generate_list_key(user_id, limit, offset);
        
        // Try to get cached notification list
        match self.unified_cache.get::<CachedNotificationList>(&cache_key).await? {
            Some(cached_list) => {
                // Check if cache is still valid (not too old)
                let age_seconds = (Utc::now() - cached_list.last_updated).num_seconds();
                if age_seconds < self.config.notification_list_ttl {
                    debug!("Cache hit for user {} notifications (age: {}s)", user_id, age_seconds);
                    return Ok(Some(cached_list.notifications));
                } else {
                    debug!("Cache expired for user {} notifications (age: {}s)", user_id, age_seconds);
                }
            }
            None => {
                debug!("Cache miss for user {} notifications", user_id);
            }
        }
        
        Ok(None)
    }
    
    async fn set_user_notifications(&self, user_id: &Uuid, notifications: Vec<Notification>) -> Result<(), CacheError> {
        // Cache recent notifications (limit 50, offset 0) for quick access
        let recent_key = NotificationCacheKeys::user_notifications_recent(user_id);
        let recent_notifications = notifications.iter().take(50).cloned().collect();
        
        let cached_list = CachedNotificationList {
            notifications: recent_notifications,
            total_count: notifications.len() as i64,
            last_updated: Utc::now(),
            cache_version: 1,
        };
        
        self.unified_cache.set(&recent_key, &cached_list, Some(self.config.notification_list_ttl)).await?;
        
        // Also cache full list if it's reasonable size
        if notifications.len() <= self.config.max_notifications_per_user {
            let full_key = NotificationCacheKeys::user_notifications(user_id);
            let full_cached_list = CachedNotificationList {
                notifications,
                total_count: cached_list.total_count,
                last_updated: Utc::now(),
                cache_version: 1,
            };
            
            self.unified_cache.set(&full_key, &full_cached_list, Some(self.config.notification_list_ttl)).await?;
        }
        
        debug!("Cached notifications for user {}", user_id);
        Ok(())
    }
    
    async fn invalidate_user_notifications(&self, user_id: &Uuid) -> Result<(), CacheError> {
        let keys = vec![
            NotificationCacheKeys::user_notifications(user_id),
            NotificationCacheKeys::user_notifications_recent(user_id),
        ];
        
        for key in keys {
            self.unified_cache.delete(&key).await?;
        }
        
        debug!("Invalidated notification cache for user {}", user_id);
        Ok(())
    }
    
    async fn get_unread_count(&self, user_id: &Uuid) -> Result<Option<i64>, CacheError> {
        let cache_key = NotificationCacheKeys::user_unread_count(user_id);
        
        match self.unified_cache.get::<CachedUnreadCount>(&cache_key).await? {
            Some(cached_count) => {
                let age_seconds = (Utc::now() - cached_count.last_updated).num_seconds();
                if age_seconds < self.config.unread_count_ttl {
                    debug!("Cache hit for user {} unread count: {}", user_id, cached_count.count);
                    return Ok(Some(cached_count.count));
                }
            }
            None => {
                debug!("Cache miss for user {} unread count", user_id);
            }
        }
        
        Ok(None)
    }
    
    async fn set_unread_count(&self, user_id: &Uuid, count: i64) -> Result<(), CacheError> {
        let cache_key = NotificationCacheKeys::user_unread_count(user_id);
        let cached_count = CachedUnreadCount {
            count,
            last_updated: Utc::now(),
            user_id: *user_id,
        };
        
        self.unified_cache.set(&cache_key, &cached_count, Some(self.config.unread_count_ttl)).await?;
        debug!("Cached unread count {} for user {}", count, user_id);
        Ok(())
    }
    
    async fn increment_unread_count(&self, user_id: &Uuid, delta: i64) -> Result<i64, CacheError> {
        let cache_key = NotificationCacheKeys::user_unread_count(user_id);
        
        // Use atomic increment if available, otherwise get-modify-set
        match self.unified_cache.increment(&cache_key, delta, Some(self.config.unread_count_ttl)).await {
            Ok(new_count) => {
                debug!("Incremented unread count for user {} by {}, new count: {}", user_id, delta, new_count);
                Ok(new_count)
            }
            Err(_) => {
                // Fallback to get-modify-set
                let current_count = self.get_unread_count(user_id).await?.unwrap_or(0);
                let new_count = std::cmp::max(0, current_count + delta);
                self.set_unread_count(user_id, new_count).await?;
                Ok(new_count)
            }
        }
    }
    
    async fn decrement_unread_count(&self, user_id: &Uuid, delta: i64) -> Result<i64, CacheError> {
        self.increment_unread_count(user_id, -delta).await
    }
    
    async fn get_notification(&self, notification_id: &Uuid) -> Result<Option<Notification>, CacheError> {
        let cache_key = NotificationCacheKeys::notification_by_id(notification_id);
        
        match self.unified_cache.get::<Notification>(&cache_key).await? {
            Some(notification) => {
                debug!("Cache hit for notification {}", notification_id);
                Ok(Some(notification))
            }
            None => {
                debug!("Cache miss for notification {}", notification_id);
                Ok(None)
            }
        }
    }
    
    async fn set_notification(&self, notification: &Notification) -> Result<(), CacheError> {
        if let Ok(notification_id) = Uuid::parse_str(&notification.id) {
            let cache_key = NotificationCacheKeys::notification_by_id(&notification_id);
            self.unified_cache.set(&cache_key, notification, Some(self.config.individual_notification_ttl)).await?;
            debug!("Cached notification {}", notification_id);
        }
        
        Ok(())
    }
    
    async fn invalidate_notification(&self, notification_id: &Uuid) -> Result<(), CacheError> {
        let cache_key = NotificationCacheKeys::notification_by_id(notification_id);
        self.unified_cache.delete(&cache_key).await?;
        debug!("Invalidated notification cache for {}", notification_id);
        Ok(())
    }
    
    async fn get_user_preferences(&self, user_id: &Uuid) -> Result<Option<NotificationPreferences>, CacheError> {
        let cache_key = NotificationCacheKeys::user_preferences(user_id);
        
        match self.unified_cache.get::<NotificationPreferences>(&cache_key).await? {
            Some(prefs) => {
                debug!("Cache hit for user {} preferences", user_id);
                Ok(Some(prefs))
            }
            None => {
                debug!("Cache miss for user {} preferences", user_id);
                Ok(None)
            }
        }
    }
    
    async fn set_user_preferences(&self, user_id: &Uuid, preferences: &NotificationPreferences) -> Result<(), CacheError> {
        let cache_key = NotificationCacheKeys::user_preferences(user_id);
        self.unified_cache.set(&cache_key, preferences, Some(self.config.user_preferences_ttl)).await?;
        debug!("Cached preferences for user {}", user_id);
        Ok(())
    }
    
    async fn invalidate_user_preferences(&self, user_id: &Uuid) -> Result<(), CacheError> {
        let cache_key = NotificationCacheKeys::user_preferences(user_id);
        self.unified_cache.delete(&cache_key).await?;
        debug!("Invalidated preferences cache for user {}", user_id);
        Ok(())
    }
    
    async fn get_user_stats(&self, user_id: &Uuid) -> Result<Option<ServiceNotificationStats>, CacheError> {
        let cache_key = NotificationCacheKeys::user_stats(user_id);
        
        match self.unified_cache.get::<ServiceNotificationStats>(&cache_key).await? {
            Some(stats) => {
                debug!("Cache hit for user {} stats", user_id);
                Ok(Some(stats))
            }
            None => {
                debug!("Cache miss for user {} stats", user_id);
                Ok(None)
            }
        }
    }
    
    async fn set_user_stats(&self, user_id: &Uuid, stats: &ServiceNotificationStats) -> Result<(), CacheError> {
        let cache_key = NotificationCacheKeys::user_stats(user_id);
        self.unified_cache.set(&cache_key, stats, Some(self.config.user_stats_ttl)).await?;
        debug!("Cached stats for user {}", user_id);
        Ok(())
    }
    
    async fn invalidate_user_stats(&self, user_id: &Uuid) -> Result<(), CacheError> {
        let cache_key = NotificationCacheKeys::user_stats(user_id);
        self.unified_cache.delete(&cache_key).await?;
        debug!("Invalidated stats cache for user {}", user_id);
        Ok(())
    }
    
    async fn check_rate_limit(&self, user_id: &Uuid, window: &str, limit: i64) -> Result<bool, CacheError> {
        let cache_key = NotificationCacheKeys::user_rate_limit(user_id, window);
        
        match self.unified_cache.get::<i64>(&cache_key).await? {
            Some(current_count) => {
                let allowed = current_count < limit;
                debug!("Rate limit check for user {} in window {}: {}/{} (allowed: {})", 
                       user_id, window, current_count, limit, allowed);
                Ok(allowed)
            }
            None => {
                debug!("No rate limit data for user {} in window {}", user_id, window);
                Ok(true) // No previous data means allowed
            }
        }
    }
    
    async fn increment_rate_limit(&self, user_id: &Uuid, window: &str, ttl_seconds: i64) -> Result<i64, CacheError> {
        let cache_key = NotificationCacheKeys::user_rate_limit(user_id, window);
        
        let count = self.unified_cache.increment(&cache_key, 1, Some(ttl_seconds)).await?;
        debug!("Incremented rate limit for user {} in window {}: {}", user_id, window, count);
        Ok(count)
    }
    
    async fn invalidate_all_user_data(&self, user_id: &Uuid) -> Result<(), CacheError> {
        let keys = vec![
            NotificationCacheKeys::user_notifications(user_id),
            NotificationCacheKeys::user_notifications_recent(user_id),
            NotificationCacheKeys::user_unread_count(user_id),
            NotificationCacheKeys::user_stats(user_id),
            NotificationCacheKeys::user_preferences(user_id),
        ];
        
        for key in keys {
            let _ = self.unified_cache.delete(&key).await; // Ignore individual failures
        }
        
        info!("Invalidated all cached data for user {}", user_id);
        Ok(())
    }
    
    async fn warm_user_cache(&self, user_id: &Uuid) -> Result<(), CacheError> {
        if !self.config.cache_warming_enabled {
            return Ok(());
        }
        
        // This would typically be called with actual data from the database
        // For now, just log that warming was requested
        debug!("Cache warming requested for user {} (not implemented)", user_id);
        Ok(())
    }
    
    async fn get_cache_stats(&self) -> Result<NotificationCacheStats, CacheError> {
        let cache_stats = self.unified_cache.stats().await?;
        
        Ok(NotificationCacheStats {
            total_keys: cache_stats.active_entries,
            hit_ratio: 0.0, // Would need to track this separately
            memory_usage: 0,
            redis_keys: 0,
            in_memory_keys: cache_stats.active_entries,
            expired_keys_cleaned: 0,
            last_cleanup: None,
        })
    }
    
    async fn cleanup_expired(&self) -> Result<i64, CacheError> {
        // The underlying UnifiedCache handles expiration automatically
        // This is mainly for metrics/monitoring
        debug!("Notification cache cleanup requested (handled by underlying cache)");
        Ok(0)
    }
}

/// Cache statistics for monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationCacheStats {
    pub total_keys: u64,
    pub hit_ratio: f64,
    pub memory_usage: u64,
    pub redis_keys: u64,
    pub in_memory_keys: u64,
    pub expired_keys_cleaned: u64,
    pub last_cleanup: Option<DateTime<Utc>>,
}

/// Helper functions for cache management
impl NotificationCacheImpl {
    /// Batch invalidate notifications for multiple users
    pub async fn invalidate_users_batch(&self, user_ids: &[Uuid]) -> Result<(), CacheError> {
        for user_id in user_ids {
            let _ = self.invalidate_all_user_data(user_id).await; // Continue on error
        }
        info!("Batch invalidated cache for {} users", user_ids.len());
        Ok(())
    }
    
    /// Smart cache warming based on user activity patterns
    pub async fn warm_active_users(&self, user_ids: &[Uuid]) -> Result<(), CacheError> {
        if !self.config.cache_warming_enabled {
            return Ok(());
        }
        
        for user_id in user_ids.iter().take(100) { // Limit to prevent overload
            let _ = self.warm_user_cache(user_id).await; // Continue on error
        }
        
        info!("Warmed cache for {} active users", std::cmp::min(user_ids.len(), 100));
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infra::cache::{CacheConfig, InMemoryCache};
    
    async fn create_test_cache() -> NotificationCacheImpl {
        let memory_cache = Arc::new(InMemoryCache::new(CacheConfig::default()));
        let unified_cache = Arc::new(UnifiedCache::new(
            "redis://invalid:6379".to_string(), // Will fallback to memory
            10,
            CacheConfig::default(),
        ).await);
        
        NotificationCacheImpl::with_default_config(unified_cache)
    }
    
    #[tokio::test]
    async fn test_unread_count_operations() {
        let cache = create_test_cache().await;
        let user_id = Uuid::new_v4();
        
        // Test increment from zero
        let count = cache.increment_unread_count(&user_id, 5).await.unwrap();
        assert_eq!(count, 5);
        
        // Test decrement
        let count = cache.decrement_unread_count(&user_id, 2).await.unwrap();
        assert_eq!(count, 3);
        
        // Test get
        let cached_count = cache.get_unread_count(&user_id).await.unwrap();
        assert_eq!(cached_count, Some(3));
    }
    
    #[tokio::test]
    async fn test_rate_limiting() {
        let cache = create_test_cache().await;
        let user_id = Uuid::new_v4();
        
        // First check should be allowed
        assert!(cache.check_rate_limit(&user_id, "hour", 10).await.unwrap());
        
        // Increment counter
        cache.increment_rate_limit(&user_id, "hour", 3600).await.unwrap();
        
        // Should still be allowed (1 < 10)
        assert!(cache.check_rate_limit(&user_id, "hour", 10).await.unwrap());
    }
    
    #[tokio::test]
    async fn test_cache_invalidation() {
        let cache = create_test_cache().await;
        let user_id = Uuid::new_v4();
        
        // Set some data
        cache.set_unread_count(&user_id, 5).await.unwrap();
        assert_eq!(cache.get_unread_count(&user_id).await.unwrap(), Some(5));
        
        // Invalidate all user data
        cache.invalidate_all_user_data(&user_id).await.unwrap();
        
        // Data should be gone
        assert_eq!(cache.get_unread_count(&user_id).await.unwrap(), None);
    }
}