// Authentication Performance Optimizations
// High-performance caching and query optimization for authentication operations

use std::sync::Arc;
use std::collections::HashMap;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc, Duration};
use uuid::Uuid;

/// High-performance permission cache with TTL
#[derive(Clone)]
pub struct PermissionCache {
    cache: Arc<RwLock<HashMap<String, CachedPermissions>>>,
    default_ttl: Duration,
}

#[derive(Clone)]
struct CachedPermissions {
    permissions: Vec<String>,
    expires_at: DateTime<Utc>,
    version: u32,
}

impl PermissionCache {
    pub fn new(default_ttl_seconds: i64) -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            default_ttl: Duration::seconds(default_ttl_seconds),
        }
    }
    
    /// Get cached permissions with automatic expiry
    pub async fn get(&self, user_id: &str) -> Option<Vec<String>> {
        let cache = self.cache.read().await;
        if let Some(cached) = cache.get(user_id) {
            if cached.expires_at > Utc::now() {
                return Some(cached.permissions.clone());
            }
        }
        None
    }
    
    /// Cache permissions with TTL
    pub async fn set(&self, user_id: &str, permissions: Vec<String>, version: u32) {
        let expires_at = Utc::now() + self.default_ttl;
        let cached = CachedPermissions {
            permissions,
            expires_at,
            version,
        };
        
        let mut cache = self.cache.write().await;
        cache.insert(user_id.to_string(), cached);
    }
    
    /// Invalidate cached permissions for a user
    pub async fn invalidate(&self, user_id: &str) {
        let mut cache = self.cache.write().await;
        cache.remove(user_id);
    }
    
    /// Clear expired entries
    pub async fn cleanup_expired(&self) {
        let now = Utc::now();
        let mut cache = self.cache.write().await;
        cache.retain(|_, cached| cached.expires_at > now);
    }
    
    /// Get cache statistics
    pub async fn stats(&self) -> CacheStats {
        let cache = self.cache.read().await;
        let now = Utc::now();
        let total_entries = cache.len();
        let expired_entries = cache.values()
            .filter(|cached| cached.expires_at <= now)
            .count();
        
        CacheStats {
            total_entries,
            active_entries: total_entries - expired_entries,
            expired_entries,
        }
    }
}

#[derive(Debug)]
pub struct CacheStats {
    pub total_entries: usize,
    pub active_entries: usize,
    pub expired_entries: usize,
}

/// Optimized permission lookup service
pub struct OptimizedPermissionService {
    cache: PermissionCache,
    db_pool: Arc<sqlx::PgPool>,
}

impl OptimizedPermissionService {
    pub fn new(db_pool: Arc<sqlx::PgPool>) -> Self {
        Self {
            cache: PermissionCache::new(300), // 5 minute cache
            db_pool,
        }
    }
    
    /// Get user permissions with caching and optimized queries
    pub async fn get_user_permissions(&self, user_id: Uuid) -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>> {
        let user_id_str = user_id.to_string();
        
        // Try cache first
        if let Some(cached_permissions) = self.cache.get(&user_id_str).await {
            return Ok(cached_permissions);
        }
        
        // Query database with optimized single query
        let permissions = sqlx::query_scalar!(
            r#"
            SELECT permission 
            FROM user_permissions 
            WHERE user_id = $1 
            AND is_active = true 
            AND (expires_at IS NULL OR expires_at > CURRENT_TIMESTAMP)
            ORDER BY permission
            "#,
            user_id
        )
        .fetch_all(&*self.db_pool)
        .await?;
        
        // Cache the result
        self.cache.set(&user_id_str, permissions.clone(), 1).await;
        
        Ok(permissions)
    }
    
    /// Batch get permissions for multiple users (avoids N+1 queries)
    pub async fn get_batch_permissions(&self, user_ids: &[Uuid]) -> Result<HashMap<Uuid, Vec<String>>, Box<dyn std::error::Error + Send + Sync>> {
        let mut result = HashMap::new();
        let mut uncached_users = Vec::new();
        
        // Check cache for all users first
        for user_id in user_ids {
            let user_id_str = user_id.to_string();
            if let Some(cached_permissions) = self.cache.get(&user_id_str).await {
                result.insert(*user_id, cached_permissions);
            } else {
                uncached_users.push(*user_id);
            }
        }
        
        // Batch query for uncached users
        if !uncached_users.is_empty() {
            let batch_results = sqlx::query!(
                r#"
                SELECT user_id, permission 
                FROM user_permissions 
                WHERE user_id = ANY($1) 
                AND is_active = true 
                AND (expires_at IS NULL OR expires_at > CURRENT_TIMESTAMP)
                ORDER BY user_id, permission
                "#,
                &uncached_users
            )
            .fetch_all(&*self.db_pool)
            .await?;
            
            // Group by user_id
            let mut user_permissions: HashMap<Uuid, Vec<String>> = HashMap::new();
            for row in batch_results {
                user_permissions.entry(row.user_id)
                    .or_insert_with(Vec::new)
                    .push(row.permission);
            }
            
            // Cache results and add to final result
            for (user_id, permissions) in user_permissions {
                let user_id_str = user_id.to_string();
                self.cache.set(&user_id_str, permissions.clone(), 1).await;
                result.insert(user_id, permissions);
            }
        }
        
        Ok(result)
    }
    
    /// Invalidate user permissions cache when permissions change
    pub async fn invalidate_user_cache(&self, user_id: Uuid) {
        let user_id_str = user_id.to_string();
        self.cache.invalidate(&user_id_str).await;
    }
    
    /// Periodic cache cleanup (call from background task)
    pub async fn cleanup_cache(&self) {
        self.cache.cleanup_expired().await;
    }
    
    /// Get cache performance metrics
    pub async fn cache_stats(&self) -> CacheStats {
        self.cache.stats().await
    }
}

/// Connection pool optimization for authentication queries
pub struct AuthDatabaseOptimizer;

impl AuthDatabaseOptimizer {
    /// Create optimized connection pool for authentication workloads
    pub async fn create_optimized_pool(database_url: &str) -> Result<sqlx::PgPool, sqlx::Error> {
        let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(50) // Higher for auth-heavy workload
            .min_connections(10) // Keep warm connections
            .max_lifetime(std::time::Duration::from_secs(1800)) // 30 minutes
            .idle_timeout(std::time::Duration::from_secs(600)) // 10 minutes
            .acquire_timeout(std::time::Duration::from_secs(5)) // Fast timeout for auth
            .test_before_acquire(false) // Skip ping for performance
            .connect(database_url)
            .await?;
            
        Ok(pool)
    }
    
    /// Optimize database for authentication queries
    pub async fn optimize_auth_tables(pool: &sqlx::PgPool) -> Result<(), sqlx::Error> {
        // Create optimized indexes for common auth queries
        sqlx::query(r#"
            -- Composite index for active user permissions lookup
            CREATE INDEX CONCURRENTLY IF NOT EXISTS 
            idx_user_permissions_active_lookup 
            ON user_permissions (user_id, is_active, expires_at) 
            WHERE is_active = true;
        "#).execute(pool).await?;
        
        sqlx::query(r#"
            -- Partial index for non-expiring permissions (most common case)
            CREATE INDEX CONCURRENTLY IF NOT EXISTS 
            idx_user_permissions_permanent 
            ON user_permissions (user_id, permission) 
            WHERE is_active = true AND expires_at IS NULL;
        "#).execute(pool).await?;
        
        sqlx::query(r#"
            -- Index for JWT token lookups
            CREATE INDEX CONCURRENTLY IF NOT EXISTS 
            idx_revoked_tokens_jti_active 
            ON revoked_tokens (jti) 
            WHERE expires_at > CURRENT_TIMESTAMP;
        "#).execute(pool).await?;
        
        Ok(())
    }
}

/// Background service for permission cache management
pub struct PermissionCacheService {
    optimizer: OptimizedPermissionService,
}

impl PermissionCacheService {
    pub fn new(db_pool: Arc<sqlx::PgPool>) -> Self {
        Self {
            optimizer: OptimizedPermissionService::new(db_pool),
        }
    }
    
    /// Start background cache cleanup task
    pub async fn start_cleanup_task(&self) {
        let optimizer = self.optimizer.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(300)); // 5 minutes
            loop {
                interval.tick().await;
                optimizer.cleanup_cache().await;
                
                let stats = optimizer.cache_stats().await;
                tracing::info!(
                    "Permission cache cleanup completed. Active: {}, Expired: {}, Total: {}",
                    stats.active_entries,
                    stats.expired_entries,
                    stats.total_entries
                );
            }
        });
    }
    
    /// Preload permissions for active users
    pub async fn preload_active_users(&self) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
        // Get recently active users (last 24 hours)
        let active_users = sqlx::query_scalar!(
            r#"
            SELECT DISTINCT user_id 
            FROM sessions 
            WHERE created_at > CURRENT_TIMESTAMP - INTERVAL '24 hours'
            AND is_active = true
            LIMIT 1000
            "#
        )
        .fetch_all(&*self.optimizer.db_pool)
        .await?;
        
        // Batch load their permissions
        let user_ids: Vec<Uuid> = active_users.into_iter().collect();
        let loaded_permissions = self.optimizer.get_batch_permissions(&user_ids).await?;
        
        Ok(loaded_permissions.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_permission_cache_basic_operations() {
        let cache = PermissionCache::new(300);
        let user_id = "test_user";
        let permissions = vec!["admin:users:read".to_string(), "admin:users:write".to_string()];
        
        // Cache should be empty initially
        assert!(cache.get(user_id).await.is_none());
        
        // Set permissions
        cache.set(user_id, permissions.clone(), 1).await;
        
        // Should retrieve cached permissions
        let cached = cache.get(user_id).await.unwrap();
        assert_eq!(cached, permissions);
        
        // Invalidate cache
        cache.invalidate(user_id).await;
        assert!(cache.get(user_id).await.is_none());
    }
    
    #[tokio::test]
    async fn test_permission_cache_expiry() {
        let cache = PermissionCache::new(1); // 1 second TTL
        let user_id = "test_user";
        let permissions = vec!["admin:users:read".to_string()];
        
        cache.set(user_id, permissions.clone(), 1).await;
        
        // Should be available immediately
        assert!(cache.get(user_id).await.is_some());
        
        // Wait for expiry
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        
        // Should be expired
        assert!(cache.get(user_id).await.is_none());
    }
    
    #[tokio::test]
    async fn test_cache_cleanup() {
        let cache = PermissionCache::new(1); // 1 second TTL
        let permissions = vec!["admin:users:read".to_string()];
        
        // Add multiple entries
        cache.set("user1", permissions.clone(), 1).await;
        cache.set("user2", permissions.clone(), 1).await;
        cache.set("user3", permissions.clone(), 1).await;
        
        let stats_before = cache.stats().await;
        assert_eq!(stats_before.total_entries, 3);
        
        // Wait for expiry
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        
        // Cleanup expired entries
        cache.cleanup_expired().await;
        
        let stats_after = cache.stats().await;
        assert_eq!(stats_after.total_entries, 0);
    }
}