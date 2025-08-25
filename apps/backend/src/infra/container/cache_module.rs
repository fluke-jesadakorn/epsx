// Cache Module - Handles cache systems creation and management
// Focused module for Redis cache, security cache, and unified caching

use std::sync::Arc;
use crate::infra::cache::{Cache, CacheFactory, SecurityCache};

/// Cache module responsible for cache systems creation and management
#[derive(Clone)]
pub struct CacheModule {
    pub cache: Arc<dyn Cache>,
    pub security_cache: Option<Arc<SecurityCache>>,
}

impl CacheModule {
    /// Create a new cache module with Redis/in-memory cache
    pub async fn new() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        // Initialize main cache (Redis or in-memory fallback)
        tracing::info!("🔧 Creating cache service...");
        let cache = CacheFactory::from_env().await?;

        // Initialize security cache
        tracing::info!("🔧 Creating security cache...");
        let security_cache = Some(Arc::new(
            SecurityCache::new(cache.clone(), 60, 10, 100, 3600)
        ));

        tracing::info!("✅ Cache module created successfully");

        Ok(CacheModule {
            cache,
            security_cache,
        })
    }

    /// Create cache module with specific configuration
    pub async fn with_config(
        _ttl_seconds: i64,
        _max_entries: usize,
        _cleanup_interval: usize,
        security_ttl: i64,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        tracing::info!("🔧 Creating cache service with custom config...");
        let cache = CacheFactory::from_env().await?;

        tracing::info!("🔧 Creating security cache with custom config...");
        let security_cache = Some(Arc::new(
            SecurityCache::new(cache.clone(), security_ttl, 10, 100, 3600)
        ));

        tracing::info!("✅ Cache module with custom config created successfully");

        Ok(CacheModule {
            cache,
            security_cache,
        })
    }

    /// Get cache implementation based on environment
    pub async fn get_cache_implementation() -> Result<Arc<dyn Cache>, Box<dyn std::error::Error + Send + Sync>> {
        // Try to create cache from environment, fallback to in-memory
        match std::env::var("REDIS_URL") {
            Ok(redis_url) if !redis_url.is_empty() => {
                // Try Redis cache
                match crate::infra::cache::redis_cache::RedisCache::new(
                    redis_url,
                    10, // pool size
                    crate::infra::cache::CacheConfig::default()
                ).await {
                    Ok(redis_cache) => {
                        tracing::info!("✅ Using Redis cache");
                        Ok(Arc::new(redis_cache))
                    }
                    Err(e) => {
                        tracing::warn!("Failed to create Redis cache: {}, falling back to in-memory", e);
                        Ok(Arc::new(crate::infra::cache::memory_cache::InMemoryCache::new(
                            crate::infra::cache::CacheConfig::default()
                        )))
                    }
                }
            }
            _ => {
                // Use in-memory cache
                tracing::info!("✅ Using in-memory cache");
                Ok(Arc::new(crate::infra::cache::memory_cache::InMemoryCache::new(
                    crate::infra::cache::CacheConfig::default()
                )))
            }
        }
    }

    /// Create a minimal cache module for testing
    pub fn create_test_cache() -> Self {
        let cache = Arc::new(crate::infra::cache::memory_cache::InMemoryCache::new(
            crate::infra::cache::CacheConfig::default()
        )) as Arc<dyn Cache>;

        let security_cache = Some(Arc::new(
            SecurityCache::new(cache.clone(), 60, 10, 100, 3600)
        ));

        CacheModule {
            cache,
            security_cache,
        }
    }
}