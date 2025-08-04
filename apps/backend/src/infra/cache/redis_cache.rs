// Redis cache implementation for production use

use super::{Cache, CacheConfig, CacheStats, CacheError};
use async_trait::async_trait;
use redis::{Client, AsyncCommands, RedisError};
use redis::aio::ConnectionManager;
use redis::cmd;

/// Redis cache implementation
pub struct RedisCache {
    connection_manager: ConnectionManager,
    config: CacheConfig,
}

impl RedisCache {
    pub async fn new(
        connection_url: String,
        _pool_size: u32,
        config: CacheConfig,
    ) -> Result<Self, CacheError> {
        tracing::info!("Connecting to Redis at: {}", connection_url);
        
        let client = Client::open(connection_url)
            .map_err(|e| CacheError::ConnectionError(e.to_string()))?;
        
        let connection_manager = ConnectionManager::new(client).await
            .map_err(|e| CacheError::ConnectionError(e.to_string()))?;
        
        // Test the connection with a simple command
        let mut conn = connection_manager.clone();
        let _: String = cmd("PING").query_async(&mut conn).await
            .map_err(|e| CacheError::ConnectionError(format!("Redis ping failed: {}", e)))?;
        
        tracing::info!("Redis cache connected successfully");
        
        Ok(Self {
            connection_manager,
            config,
        })
    }
}

#[async_trait]
impl Cache for RedisCache {
    async fn get_raw(&self, key: &str) -> Result<Option<String>, CacheError> {
        let mut conn = self.connection_manager.clone();
        let data: Option<String> = conn.get(key).await
            .map_err(|e: RedisError| CacheError::ConnectionError(e.to_string()))?;
        Ok(data)
    }

    async fn set_raw(&self, key: &str, value: &str, ttl_seconds: Option<i64>) -> Result<(), CacheError> {
        let mut conn = self.connection_manager.clone();
        let ttl = ttl_seconds.unwrap_or(self.config.default_ttl_seconds);
        
        if ttl > 0 {
            let _: () = conn.set_ex(key, value, ttl as usize).await
                .map_err(|e: RedisError| CacheError::ConnectionError(e.to_string()))?;
        } else {
            let _: () = conn.set(key, value).await
                .map_err(|e: RedisError| CacheError::ConnectionError(e.to_string()))?;
        }
        
        Ok(())
    }

    async fn delete(&self, key: &str) -> Result<bool, CacheError> {
        let mut conn = self.connection_manager.clone();
        let deleted: i32 = conn.del(key).await
            .map_err(|e: RedisError| CacheError::ConnectionError(e.to_string()))?;
        Ok(deleted > 0)
    }

    async fn exists(&self, key: &str) -> Result<bool, CacheError> {
        let mut conn = self.connection_manager.clone();
        let exists: bool = conn.exists(key).await
            .map_err(|e: RedisError| CacheError::ConnectionError(e.to_string()))?;
        Ok(exists)
    }

    async fn clear(&self) -> Result<(), CacheError> {
        let mut conn = self.connection_manager.clone();
        let _: () = cmd("FLUSHDB").query_async(&mut conn).await
            .map_err(|e: RedisError| CacheError::ConnectionError(e.to_string()))?;
        Ok(())
    }

    async fn stats(&self) -> Result<CacheStats, CacheError> {
        let mut conn = self.connection_manager.clone();
        let info: String = cmd("INFO").arg("memory").query_async(&mut conn).await
            .map_err(|e: RedisError| CacheError::ConnectionError(e.to_string()))?;
        
        // Parse Redis INFO output to extract stats
        let mut memory_usage = None;
        for line in info.lines() {
            if line.starts_with("used_memory:") {
                if let Some(value) = line.split(':').nth(1) {
                    memory_usage = value.parse::<u64>().ok();
                }
            }
        }
        
        // Get keyspace info for entry counts
        let keyspace_info: String = cmd("INFO").arg("keyspace").query_async(&mut conn).await
            .map_err(|e: RedisError| CacheError::ConnectionError(e.to_string()))?;
        
        let mut total_entries = 0u64;
        for line in keyspace_info.lines() {
            if line.starts_with("db0:") {
                // Parse format like "db0:keys=123,expires=45,avg_ttl=300"
                for part in line.split(',') {
                    if part.starts_with("keys=") {
                        if let Some(value) = part.split('=').nth(1) {
                            total_entries = value.parse::<u64>().unwrap_or(0);
                        }
                    }
                }
            }
        }
        
        Ok(CacheStats {
            total_entries,
            expired_entries: 0, // Redis handles expiration automatically
            active_entries: total_entries,
            memory_usage_bytes: memory_usage,
            hit_count: None, // Would need CONFIG GET for this
            miss_count: None,
            hit_rate: None,
        })
    }

    async fn delete_many(&self, keys: &[String]) -> Result<u64, CacheError> {
        if keys.is_empty() {
            return Ok(0);
        }
        
        let mut conn = self.connection_manager.clone();
        let deleted: u64 = conn.del(keys).await
            .map_err(|e: RedisError| CacheError::ConnectionError(e.to_string()))?;
        Ok(deleted)
    }

    async fn increment(&self, key: &str, delta: i64, ttl_seconds: Option<i64>) -> Result<i64, CacheError> {
        let mut conn = self.connection_manager.clone();
        
        let new_value: i64 = if delta == 1 {
            conn.incr(key, 1).await
        } else {
            conn.incr(key, delta).await
        }.map_err(|e: RedisError| CacheError::ConnectionError(e.to_string()))?;
        
        // Set TTL if specified
        if let Some(ttl) = ttl_seconds {
            if ttl > 0 {
                let _: bool = conn.expire(key, ttl as usize).await
                    .map_err(|e: RedisError| CacheError::ConnectionError(e.to_string()))?;
            }
        }
        
        Ok(new_value)
    }

    async fn expire(&self, key: &str, ttl_seconds: i64) -> Result<bool, CacheError> {
        let mut conn = self.connection_manager.clone();
        let result: bool = conn.expire(key, ttl_seconds as usize).await
            .map_err(|e: RedisError| CacheError::ConnectionError(e.to_string()))?;
        Ok(result)
    }
}