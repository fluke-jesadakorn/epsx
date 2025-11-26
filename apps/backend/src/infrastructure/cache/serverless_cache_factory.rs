// Serverless Cache Factory - Redis-Only Caching for Serverless Environments
// Eliminates memory cache fallbacks that are incompatible with serverless

use std::sync::Arc;
use anyhow::Result;
use tracing::{info, error};

use crate::infrastructure::cache::{Cache, RedisCache, CacheConfig};

/// Serverless-specific cache factory - NO memory cache fallbacks
pub struct ServerlessCacheFactory;

impl ServerlessCacheFactory {
    /// Create Redis cache for serverless - FAILS if Redis unavailable
    pub async fn redis_only() -> Result<Box<dyn Cache>> {
        let redis_cache = Self::create_redis_cache().await
            .map_err(|e| anyhow::anyhow!("Redis is REQUIRED for serverless deployment: {}", e))?;
        
        info!("✅ Serverless Redis cache initialized (no fallback)");
        Ok(Box::new(redis_cache))
    }

    /// Create Redis cache as Arc for serverless - FAILS if Redis unavailable
    pub async fn redis_only_arc() -> Result<Arc<dyn Cache>> {
        let redis_cache = Self::create_redis_cache().await
            .map_err(|e| anyhow::anyhow!("Redis is REQUIRED for serverless deployment: {}", e))?;
        
        info!("✅ Serverless Redis cache (Arc) initialized (no fallback)");
        Ok(Arc::new(redis_cache))
    }

    /// Create Redis cache with specific URL - FAILS if Redis unavailable
    pub async fn redis_with_url(redis_url: String) -> Result<Arc<dyn Cache>> {
        let pool_size = Self::get_serverless_pool_size();
        let config = Self::get_serverless_cache_config();
        
        let redis_cache = RedisCache::new(redis_url.clone(), pool_size, config).await
            .map_err(|e| anyhow::anyhow!("Failed to connect to Redis at {}: {}", redis_url, e))?;
        
        info!("✅ Serverless Redis cache created with custom URL: {}", redis_url);
        Ok(Arc::new(redis_cache))
    }

    /// Create Redis cache with environment configuration
    async fn create_redis_cache() -> Result<RedisCache> {
        let redis_url = std::env::var("REDIS_URL")
            .map_err(|_| anyhow::anyhow!("REDIS_URL environment variable is required for serverless"))?;
        
        let pool_size = Self::get_serverless_pool_size();
        let config = Self::get_serverless_cache_config();
        
        info!("🔗 Connecting to Redis for serverless cache: {}", redis_url);
        info!("   Pool size: {}", pool_size);
        info!("   Default TTL: {}s", config.default_ttl);
        
        RedisCache::new(redis_url, pool_size, config).await
            .map_err(|e| anyhow::anyhow!("Redis connection failed: {}", e))
    }

    /// Get serverless-optimized pool size
    fn get_serverless_pool_size() -> u32 {
        std::env::var("REDIS_POOL_SIZE")
            .unwrap_or_else(|_| "5".to_string()) // Smaller pool for serverless
            .parse::<u32>()
            .unwrap_or(5)
    }

    /// Get serverless-optimized cache configuration
    fn get_serverless_cache_config() -> CacheConfig {
        CacheConfig {
            default_ttl: std::env::var("CACHE_DEFAULT_TTL")
                .unwrap_or_else(|_| "1800".to_string()) // 30 minutes default
                .parse::<u64>()
                .unwrap_or(1800),
            max_size: std::env::var("CACHE_MAX_SIZE")
                .unwrap_or_else(|_| "500".to_string()) // Smaller cache for serverless
                .parse::<usize>()
                .unwrap_or(500),
        }
    }

    /// Health check for Redis connectivity
    pub async fn health_check() -> bool {
        match Self::create_redis_cache().await {
            Ok(cache) => {
                // Use the async health_check method from RedisCache directly
                let is_healthy = cache.health_check().await;
                if is_healthy {
                    info!("✅ Redis cache health check passed");
                } else {
                    error!("❌ Redis cache health check failed");
                }
                is_healthy
            }
            Err(e) => {
                error!("❌ Failed to create Redis cache for health check: {}", e);
                false
            }
        }
    }

    /// Validate Redis configuration for serverless
    pub fn validate_serverless_config() -> Result<ServerlessCacheConfig> {
        let redis_url = std::env::var("REDIS_URL")
            .map_err(|_| anyhow::anyhow!("REDIS_URL is required for serverless caching"))?;
        
        let pool_size = Self::get_serverless_pool_size();
        let cache_config = Self::get_serverless_cache_config();
        
        Ok(ServerlessCacheConfig {
            redis_url,
            pool_size,
            cache_config,
        })
    }
}

/// Serverless cache configuration
#[derive(Debug, Clone)]
pub struct ServerlessCacheConfig {
    pub redis_url: String,
    pub pool_size: u32,
    pub cache_config: CacheConfig,
}

impl ServerlessCacheConfig {
    /// Create from environment variables
    pub fn from_env() -> Result<Self> {
        ServerlessCacheFactory::validate_serverless_config()
    }

    /// Display configuration for logging
    pub fn display(&self) -> String {
        format!(
            "Redis: {}, Pool: {}, TTL: {}s",
            self.redis_url,
            self.pool_size,
            self.cache_config.default_ttl
        )
    }
}

/// Helper functions for quick cache access
pub async fn get_serverless_cache() -> Result<Arc<dyn Cache>> {
    ServerlessCacheFactory::redis_only_arc().await
}

/// Health check function for serverless cache
pub async fn serverless_cache_health_check() -> bool {
    ServerlessCacheFactory::health_check().await
}

