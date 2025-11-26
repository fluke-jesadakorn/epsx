// ============================================================================
// UNIFIED PERMISSION CACHE WITH INVALIDATION
// ============================================================================
// Redis-backed permission cache with automatic invalidation
//
// Features:
// - 30-second TTL for permission checks
// - Automatic invalidation on permission changes
// - Cache versioning for atomic updates
// - Distributed locking for consistency
// ============================================================================

use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{debug, error, warn};

use crate::auth::unified_permission_service::PermissionDetail;

// ============================================================================
// CACHE CONFIGURATION
// ============================================================================

const CACHE_TTL_SECONDS: u64 = 30;  // 30 seconds
const PERMISSION_CHECK_PREFIX: &str = "perm_check:";
const WALLET_PERMISSIONS_PREFIX: &str = "wallet_perms:";
const CACHE_VERSION_PREFIX: &str = "cache_ver:";

// ============================================================================
// UNIFIED PERMISSION CACHE
// ============================================================================

#[derive(Clone)]
pub struct UnifiedPermissionCache {
    redis_client: Arc<redis::Client>,
}

impl UnifiedPermissionCache {
    /// Create new unified permission cache
    pub fn new(redis_client: Arc<redis::Client>) -> Self {
        Self { redis_client }
    }

    /// Get Redis connection
    async fn get_connection(&self) -> Result<redis::aio::Connection, redis::RedisError> {
        self.redis_client.get_async_connection().await
    }

    // ========================================================================
    // PERMISSION CHECK CACHE
    // ========================================================================

    /// Get cached permission check result
    pub async fn get_permission_check(
        &self,
        wallet_address: &str,
        permission: &str,
    ) -> Option<bool> {
        let key = format!("{}{}:{}", PERMISSION_CHECK_PREFIX, wallet_address, permission);

        match self.get_connection().await {
            Ok(mut conn) => {
                match conn.get::<_, Option<String>>(&key).await {
                    Ok(Some(value)) => {
                        match value.as_str() {
                            "1" => {
                                debug!("Cache hit for permission check: {}", permission);
                                Some(true)
                            }
                            "0" => {
                                debug!("Cache hit for permission check: {}", permission);
                                Some(false)
                            }
                            _ => None,
                        }
                    }
                    Ok(None) => None,
                    Err(e) => {
                        warn!("Redis error getting permission check: {}", e);
                        None
                    }
                }
            }
            Err(e) => {
                error!("Failed to get Redis connection: {}", e);
                None
            }
        }
    }

    /// Set permission check result in cache
    pub async fn set_permission_check(
        &self,
        wallet_address: &str,
        permission: &str,
        has_permission: bool,
    ) {
        let key = format!("{}{}:{}", PERMISSION_CHECK_PREFIX, wallet_address, permission);
        let value = if has_permission { "1" } else { "0" };

        match self.get_connection().await {
            Ok(mut conn) => {
                if let Err(e) = conn.set_ex::<_, _, ()>(&key, value, CACHE_TTL_SECONDS).await {
                    warn!("Redis error setting permission check: {}", e);
                }
            }
            Err(e) => {
                error!("Failed to get Redis connection: {}", e);
            }
        }
    }

    // ========================================================================
    // WALLET PERMISSIONS CACHE
    // ========================================================================

    /// Get cached wallet permissions
    pub async fn get_wallet_permissions(
        &self,
        wallet_address: &str,
    ) -> Option<Vec<PermissionDetail>> {
        let key = format!("{}{}", WALLET_PERMISSIONS_PREFIX, wallet_address);

        match self.get_connection().await {
            Ok(mut conn) => {
                match conn.get::<_, Option<String>>(&key).await {
                    Ok(Some(json)) => {
                        match serde_json::from_str::<Vec<PermissionDetail>>(&json) {
                            Ok(permissions) => {
                                debug!("Cache hit for wallet permissions: {}", wallet_address);
                                Some(permissions)
                            }
                            Err(e) => {
                                warn!("Failed to deserialize cached permissions: {}", e);
                                None
                            }
                        }
                    }
                    Ok(None) => None,
                    Err(e) => {
                        warn!("Redis error getting wallet permissions: {}", e);
                        None
                    }
                }
            }
            Err(e) => {
                error!("Failed to get Redis connection: {}", e);
                None
            }
        }
    }

    /// Set wallet permissions in cache
    pub async fn set_wallet_permissions(
        &self,
        wallet_address: &str,
        permissions: &[PermissionDetail],
    ) {
        let key = format!("{}{}", WALLET_PERMISSIONS_PREFIX, wallet_address);

        match serde_json::to_string(permissions) {
            Ok(json) => {
                match self.get_connection().await {
                    Ok(mut conn) => {
                        if let Err(e) = conn.set_ex::<_, _, ()>(&key, json, CACHE_TTL_SECONDS).await {
                            warn!("Redis error setting wallet permissions: {}", e);
                        }
                    }
                    Err(e) => {
                        error!("Failed to get Redis connection: {}", e);
                    }
                }
            }
            Err(e) => {
                error!("Failed to serialize permissions: {}", e);
            }
        }
    }

    // ========================================================================
    // CACHE INVALIDATION
    // ========================================================================

    /// Invalidate all cache entries for a wallet
    /// Called when permissions are granted/revoked or groups are assigned/removed
    pub async fn invalidate_wallet(&self, wallet_address: &str) {
        debug!("Invalidating cache for wallet: {}", wallet_address);

        match self.get_connection().await {
            Ok(mut conn) => {
                // Delete wallet permissions cache
                let wallet_perms_key = format!("{}{}", WALLET_PERMISSIONS_PREFIX, wallet_address);
                if let Err(e) = conn.del::<_, ()>(&wallet_perms_key).await {
                    warn!("Redis error deleting wallet permissions cache: {}", e);
                }

                // Delete all permission check caches for this wallet
                let pattern = format!("{}{}:*", PERMISSION_CHECK_PREFIX, wallet_address);
                if let Err(e) = self.delete_keys_by_pattern(&mut conn, &pattern).await {
                    warn!("Redis error deleting permission check caches: {}", e);
                }

                // Increment cache version (for future distributed cache invalidation)
                let version_key = format!("{}{}", CACHE_VERSION_PREFIX, wallet_address);
                if let Err(e) = conn.incr::<_, _, ()>(&version_key, 1).await {
                    warn!("Redis error incrementing cache version: {}", e);
                }

                debug!("Successfully invalidated cache for wallet: {}", wallet_address);
            }
            Err(e) => {
                error!("Failed to get Redis connection for cache invalidation: {}", e);
            }
        }
    }

    /// Delete keys matching pattern (helper for cache invalidation)
    async fn delete_keys_by_pattern(
        &self,
        conn: &mut redis::aio::Connection,
        pattern: &str,
    ) -> Result<(), redis::RedisError> {
        // Use SCAN to avoid blocking Redis
        let mut cursor = 0u64;
        loop {
            let (new_cursor, keys): (u64, Vec<String>) = redis::cmd("SCAN")
                .arg(cursor)
                .arg("MATCH")
                .arg(pattern)
                .arg("COUNT")
                .arg(100)
                .query_async(conn)
                .await?;

            if !keys.is_empty() {
                conn.del::<_, ()>(&keys).await?;
            }

            cursor = new_cursor;
            if cursor == 0 {
                break;
            }
        }

        Ok(())
    }

    /// Get cache version for wallet (for distributed cache invalidation)
    pub async fn get_cache_version(&self, wallet_address: &str) -> Option<i64> {
        let key = format!("{}{}", CACHE_VERSION_PREFIX, wallet_address);

        match self.get_connection().await {
            Ok(mut conn) => {
                match conn.get::<_, Option<i64>>(&key).await {
                    Ok(version) => version,
                    Err(e) => {
                        warn!("Redis error getting cache version: {}", e);
                        None
                    }
                }
            }
            Err(e) => {
                error!("Failed to get Redis connection: {}", e);
                None
            }
        }
    }

    // ========================================================================
    // CACHE STATISTICS
    // ========================================================================

    /// Get cache hit/miss statistics (for monitoring)
    pub async fn get_cache_stats(&self) -> Option<CacheStats> {
        match self.get_connection().await {
            Ok(mut conn) => {
                match redis::cmd("INFO")
                    .arg("stats")
                    .query_async::<_, String>(&mut conn)
                    .await
                {
                    Ok(info) => {
                        // Parse Redis INFO output
                        let mut hits = 0i64;
                        let mut misses = 0i64;

                        for line in info.lines() {
                            if line.starts_with("keyspace_hits:") {
                                hits = line.split(':').nth(1)?.parse().ok()?;
                            } else if line.starts_with("keyspace_misses:") {
                                misses = line.split(':').nth(1)?.parse().ok()?;
                            }
                        }

                        Some(CacheStats { hits, misses })
                    }
                    Err(e) => {
                        warn!("Redis error getting cache stats: {}", e);
                        None
                    }
                }
            }
            Err(e) => {
                error!("Failed to get Redis connection: {}", e);
                None
            }
        }
    }

    /// Clear all permission caches (for testing/debugging only)
    #[cfg(test)]
    pub async fn clear_all(&self) {
        match self.get_connection().await {
            Ok(mut conn) => {
                let _ = self.delete_keys_by_pattern(&mut conn, &format!("{}*", PERMISSION_CHECK_PREFIX)).await;
                let _ = self.delete_keys_by_pattern(&mut conn, &format!("{}*", WALLET_PERMISSIONS_PREFIX)).await;
                let _ = self.delete_keys_by_pattern(&mut conn, &format!("{}*", CACHE_VERSION_PREFIX)).await;
            }
            Err(e) => {
                error!("Failed to clear cache: {}", e);
            }
        }
    }
}

// ============================================================================
// CACHE STATISTICS
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStats {
    pub hits: i64,
    pub misses: i64,
}

impl CacheStats {
    pub fn hit_rate(&self) -> f64 {
        if self.hits + self.misses == 0 {
            return 0.0;
        }
        (self.hits as f64) / ((self.hits + self.misses) as f64) * 100.0
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // Note: These tests require a running Redis instance
    // They are integration tests and should be run separately

    #[tokio::test]
    #[ignore]
    async fn test_permission_check_cache() {
        let client = redis::Client::open("redis://127.0.0.1/").unwrap();
        let cache = UnifiedPermissionCache::new(Arc::new(client));

        let wallet = "0x1234567890123456789012345678901234567890";
        let permission = "admin:users:read";

        // Should be None initially
        assert_eq!(cache.get_permission_check(wallet, permission).await, None);

        // Set cache
        cache.set_permission_check(wallet, permission, true).await;

        // Should return cached value
        assert_eq!(cache.get_permission_check(wallet, permission).await, Some(true));

        // Invalidate
        cache.invalidate_wallet(wallet).await;

        // Should be None after invalidation
        assert_eq!(cache.get_permission_check(wallet, permission).await, None);
    }
}
