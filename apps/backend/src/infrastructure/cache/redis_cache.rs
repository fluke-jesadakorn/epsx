use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use super::{Cache, CacheConfig, MemoryCache};
use tokio::sync::Mutex;

/// Redis cache implementation with automatic fallback to memory cache
pub struct RedisCache {
    redis_client: Option<Arc<Mutex<redis::Client>>>,
    fallback_cache: Arc<MemoryCache>,
    #[allow(dead_code)]
    config: CacheConfig,
    is_redis_available: Arc<AtomicBool>,
}

impl RedisCache {
    pub async fn new(
        connection_url: String,
        _pool_size: u32,
        config: CacheConfig,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let client = redis::Client::open(connection_url.clone())?;
        let client_clone = client.clone();

        let redis_client = match tokio::task::spawn_blocking(move || {
            match client_clone.get_connection() {
                Ok(mut conn) => {
                    match redis::cmd("PING").query::<String>(&mut conn) {
                        Ok(_) => Some(client_clone),
                        Err(_) => None
                    }
                }
                Err(_) => None
            }
        }).await {
            Ok(Some(client)) => Some(Arc::new(Mutex::new(client))),
            _ => None
        };

        let available = redis_client.is_some();

        Ok(Self {
            redis_client,
            fallback_cache: Arc::new(MemoryCache::with_config(config.clone())),
            config,
            is_redis_available: Arc::new(AtomicBool::new(available)),
        })
    }

    pub async fn get_raw(&self, key: &str) -> Result<Option<String>, Box<dyn std::error::Error + Send + Sync>> {
        if let Some(client) = &self.redis_client {
            if self.is_redis_available.load(Ordering::Relaxed) {
                let client = client.lock().await;
                match client.get_connection() {
                    Ok(mut conn) => {
                        match redis::cmd("GET").arg(key).query::<Option<String>>(&mut conn) {
                            Ok(value) => return Ok(value),
                            Err(e) => {
                                tracing::warn!("Redis GET failed for key '{}': {}", key, e);
                                self.mark_unavailable();
                            }
                        }
                    }
                    Err(e) => {
                        tracing::warn!("Redis connection failed: {}", e);
                        self.mark_unavailable();
                    }
                }
            }
        }

        Ok(self.fallback_cache.get(key))
    }

    pub async fn set_raw(&self, key: &str, value: &str, ttl: Option<i64>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if let Some(client) = &self.redis_client {
            if self.is_redis_available.load(Ordering::Relaxed) {
                let client = client.lock().await;
                match client.get_connection() {
                    Ok(mut conn) => {
                        let result = if let Some(seconds) = ttl {
                            redis::cmd("SETEX").arg(key).arg(seconds).arg(value).query::<String>(&mut conn)
                        } else {
                            redis::cmd("SET").arg(key).arg(value).query::<String>(&mut conn)
                        };

                        match result {
                            Ok(_) => return Ok(()),
                            Err(e) => {
                                tracing::warn!("Redis SET failed for key '{}': {}", key, e);
                                self.mark_unavailable();
                            }
                        }
                    }
                    Err(e) => {
                        tracing::warn!("Redis connection failed: {}", e);
                        self.mark_unavailable();
                    }
                }
            }
        }

        self.fallback_cache.set(key, value.to_string(), ttl.map(|t| t as u64));
        Ok(())
    }

    fn mark_unavailable(&self) {
        self.is_redis_available.store(false, Ordering::Relaxed);
        tracing::warn!("Redis marked as unavailable, using memory cache fallback");
    }

    pub fn is_redis_available(&self) -> bool {
        self.is_redis_available.load(Ordering::Relaxed)
    }

    pub async fn health_check(&self) -> bool {
        if let Some(client) = &self.redis_client {
            let client = client.lock().await;
            match client.get_connection() {
                Ok(mut conn) => {
                    match redis::cmd("PING").query::<String>(&mut conn) {
                        Ok(_) => {
                            if !self.is_redis_available.load(Ordering::Relaxed) {
                                self.is_redis_available.store(true, Ordering::Relaxed);
                                tracing::info!("Redis connection restored");
                            }
                            true
                        }
                        Err(_) => false
                    }
                }
                Err(_) => false
            }
        } else {
            false
        }
    }
}

impl Cache for RedisCache {
    fn get(&self, key: &str) -> Option<String> {
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                match self.get_raw(key).await {
                    Ok(value) => value,
                    Err(e) => {
                        tracing::warn!("Cache get failed for key '{}': {}", key, e);
                        None
                    }
                }
            })
        })
    }

    fn set(&self, key: &str, value: String, ttl: Option<u64>) {
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                if let Err(e) = self.set_raw(key, &value, ttl.map(|t| t as i64)).await {
                    tracing::warn!("Cache set failed for key '{}': {}", key, e);
                }
            })
        })
    }

    fn delete(&self, key: &str) {
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                if let Some(client) = &self.redis_client {
                    if self.is_redis_available.load(Ordering::Relaxed) {
                        let client = client.lock().await;
                        match client.get_connection() {
                            Ok(mut conn) => {
                                if let Err(e) = redis::cmd("DEL").arg(key).query::<i32>(&mut conn) {
                                    tracing::warn!("Redis DELETE failed for key '{}': {}", key, e);
                                    self.mark_unavailable();
                                }
                            }
                            Err(e) => {
                                tracing::warn!("Redis connection failed: {}", e);
                                self.mark_unavailable();
                            }
                        }
                    }
                }

                self.fallback_cache.delete(key);
            })
        })
    }

    fn clear(&self) {
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                if let Some(client) = &self.redis_client {
                    if self.is_redis_available.load(Ordering::Relaxed) {
                        let client = client.lock().await;
                        match client.get_connection() {
                            Ok(mut conn) => {
                                if let Err(e) = redis::cmd("FLUSHDB").query::<String>(&mut conn) {
                                    tracing::warn!("Redis FLUSHDB failed: {}", e);
                                    self.mark_unavailable();
                                }
                            }
                            Err(e) => {
                                tracing::warn!("Redis connection failed: {}", e);
                                self.mark_unavailable();
                            }
                        }
                    }
                }

                self.fallback_cache.clear();
            })
        })
    }

    fn health_check(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if self.redis_client.is_none() {
            return Err("Redis client not configured".into());
        }

        if !self.is_redis_available.load(Ordering::Relaxed) {
            return Err("Redis marked as unavailable".into());
        }

        Ok(())
    }
}
