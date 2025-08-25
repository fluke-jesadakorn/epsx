// Application services - orchestrate use cases and handle cross-cutting concerns

use std::sync::Arc;
use async_trait::async_trait;
use crate::infra::cache::Cache;

use crate::dom::values::UserId;
use crate::app::use_cases::{AuthUC, UserMgmtUC, StockUC};

// Main application service that coordinates use cases
pub struct AppService {
    auth_uc: Arc<AuthUC>,
    user_mgmt_uc: Arc<UserMgmtUC>,
    stock_uc: Arc<StockUC>,
}

impl AppService {
    pub fn new(
        auth_uc: Arc<AuthUC>,
        user_mgmt_uc: Arc<UserMgmtUC>,
        stock_uc: Arc<StockUC>,
    ) -> Self {
        Self {
            auth_uc,
            user_mgmt_uc,
            stock_uc,
        }
    }
    
    pub fn auth(&self) -> &AuthUC {
        &self.auth_uc
    }
    
    pub fn user_mgmt(&self) -> &UserMgmtUC {
        &self.user_mgmt_uc
    }
    
    
    pub fn stocks(&self) -> &StockUC {
        &self.stock_uc
    }
}

// AuthorizationService is now re-exported from domain layer as UnifiedPermissionService

// Re-export comprehensive services from domain layer
pub use crate::dom::services::audit_service::AuditService;
pub use crate::dom::services::permission_resolver::PermissionResolver;

// Rate limiting service with Redis backend
pub struct RateLimitService {
    rate_limiter: Arc<crate::web::middleware::rate_limiter::UnifiedRateLimiter>,
}

impl RateLimitService {
    pub async fn new(config: Arc<crate::config::Config>) -> Self {
        use crate::infra::cache::CacheFactory;
        let cache = CacheFactory::with_fallback().await;
        Self {
            rate_limiter: Arc::new(crate::web::middleware::rate_limiter::UnifiedRateLimiter::with_config(cache, config)),
        }
    }

    pub async fn check_rate_limit(&self, user_id: &UserId, operation: &str) -> Result<(), RateLimitError> {
        use crate::web::middleware::rate_limiter::{RateLimitConfig, ClientId};
        
        let config = RateLimitConfig {
            requests_per_minute: Some(60),
            requests_per_hour: Some(1000),
            requests_per_day: Some(10000),
        };
        
        let client_id = ClientId::User(user_id.clone());
        let result = self.rate_limiter.check_client_rate_limit(&client_id, operation, "POST", &config).await
            .map_err(|_e| RateLimitError::ServiceUnavailable)?;
            
        if !result.allowed {
            return Err(RateLimitError::LimitExceeded {
                user_id: user_id.to_string(),
                operation: operation.to_string(),
            });
        }
        
        tracing::debug!(
            user_id = %user_id,
            operation = operation,
            "Rate limit check passed: {}/{}",
            result.current_count,
            result.limit
        );
        Ok(())
    }
    
    pub async fn check_ip_rate_limit(&self, client_ip: &str, endpoint: &str) -> Result<(), RateLimitError> {
        self.rate_limiter.check_ip_rate_limit(client_ip, endpoint).await
            .map_err(|_| RateLimitError::ServiceUnavailable)
    }
    
    pub async fn reset_user_limits(&self, user_id: &UserId) -> Result<u32, RateLimitError> {
        self.rate_limiter.reset_user_limits(user_id).await
            .map_err(|_| RateLimitError::ServiceUnavailable)
    }
}

// Caching service interface
#[async_trait]
pub trait CacheService: Send + Sync {
    async fn get<T>(&self, key: &str) -> Result<Option<T>, CacheError>
    where
        T: serde::de::DeserializeOwned;
        
    async fn set<T>(&self, key: &str, value: &T, ttl_seconds: u64) -> Result<(), CacheError>
    where
        T: serde::Serialize + Sync;
        
    async fn delete(&self, key: &str) -> Result<(), CacheError>;
    
    async fn clear_pattern(&self, pattern: &str) -> Result<u64, CacheError>;
}

// Redis cache implementation
pub struct RedisCache {
    cache: Arc<crate::infra::cache::redis_cache::RedisCache>,
}

impl RedisCache {
    pub async fn new(connection_url: String, pool_size: u32, config: crate::infra::cache::CacheConfig) -> Result<Self, CacheError> {
        let cache = crate::infra::cache::redis_cache::RedisCache::new(connection_url, pool_size, config).await
            .map_err(|e| CacheError::OperationFailed(e.to_string()))?;
        Ok(Self {
            cache: Arc::new(cache),
        })
    }
}

#[async_trait]
impl CacheService for RedisCache {
    async fn get<T>(&self, key: &str) -> Result<Option<T>, CacheError>
    where
        T: serde::de::DeserializeOwned,
    {
        match self.cache.as_ref().get_raw(key).await.map_err(|e| CacheError::OperationFailed(e.to_string()))? {
            Some(ref value) => {
                let deserialized = serde_json::from_str(value)
                    .map_err(|e| CacheError::SerializationError(e.to_string()))?;
                Ok(Some(deserialized))
            }
            None => Ok(None)
        }
    }
    
    async fn set<T>(&self, key: &str, value: &T, ttl_seconds: u64) -> Result<(), CacheError>
    where
        T: serde::Serialize + Sync,
    {
        let serialized = serde_json::to_string(value)
            .map_err(|e| CacheError::SerializationError(e.to_string()))?;
        self.cache.as_ref().set_raw(key, &serialized, Some(ttl_seconds as i64)).await
            .map_err(|e| CacheError::OperationFailed(e.to_string()))
    }
    
    async fn delete(&self, key: &str) -> Result<(), CacheError> {
        self.cache.as_ref().delete(key).await
            .map_err(|e| CacheError::OperationFailed(e.to_string()))?;
        Ok(())
    }
    
    async fn clear_pattern(&self, pattern: &str) -> Result<u64, CacheError> {
        // Redis doesn't have direct pattern delete, but we can scan and delete
        // For now, return 0 as a placeholder implementation
        tracing::warn!("Pattern clear not fully implemented for Redis cache: {}", pattern);
        Ok(0)
    }
}

// In-memory cache implementation as fallback
pub struct InMemoryCache {
    cache: Arc<std::sync::RwLock<std::collections::HashMap<String, (String, std::time::Instant)>>>,
    default_ttl: std::time::Duration,
}

impl InMemoryCache {
    pub fn new() -> Self {
        Self {
            cache: Arc::new(std::sync::RwLock::new(std::collections::HashMap::new())),
            default_ttl: std::time::Duration::from_secs(3600), // 1 hour default
        }
    }
}

#[async_trait]
impl CacheService for InMemoryCache {
    async fn get<T>(&self, key: &str) -> Result<Option<T>, CacheError>
    where
        T: serde::de::DeserializeOwned,
    {
        let cache = self.cache.read().unwrap();
        if let Some((value, timestamp)) = cache.get(key) {
            // Check if expired
            if timestamp.elapsed() > self.default_ttl {
                return Ok(None);
            }
            let deserialized = serde_json::from_str(value)
                .map_err(|e| CacheError::SerializationError(e.to_string()))?;
            Ok(Some(deserialized))
        } else {
            Ok(None)
        }
    }
    
    async fn set<T>(&self, key: &str, value: &T, _ttl_seconds: u64) -> Result<(), CacheError>
    where
        T: serde::Serialize + Sync,
    {
        let serialized = serde_json::to_string(value)
            .map_err(|e| CacheError::SerializationError(e.to_string()))?;
        let mut cache = self.cache.write().unwrap();
        cache.insert(key.to_string(), (serialized, std::time::Instant::now()));
        Ok(())
    }
    
    async fn delete(&self, key: &str) -> Result<(), CacheError> {
        let mut cache = self.cache.write().unwrap();
        cache.remove(key);
        Ok(())
    }
    
    async fn clear_pattern(&self, pattern: &str) -> Result<u64, CacheError> {
        let mut cache = self.cache.write().unwrap();
        let keys_to_remove: Vec<String> = cache.keys()
            .filter(|key| key.contains(pattern))
            .cloned()
            .collect();
        let count = keys_to_remove.len() as u64;
        for key in keys_to_remove {
            cache.remove(&key);
        }
        Ok(count)
    }
}

// AuthorizationError is now re-exported from domain layer

#[derive(Debug, thiserror::Error)]
pub enum RateLimitError {
    #[error("Rate limit exceeded for user {user_id} on operation {operation}")]
    LimitExceeded { user_id: String, operation: String },
    
    #[error("Rate limit service unavailable")]
    ServiceUnavailable,
}

#[derive(Debug, thiserror::Error)]
pub enum CacheError {
    #[error("Cache operation failed: {0}")]
    OperationFailed(String),
    
    #[error("Serialization error: {0}")]
    SerializationError(String),
    
    #[error("Cache service unavailable")]
    ServiceUnavailable,
}