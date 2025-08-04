// Application services - orchestrate use cases and handle cross-cutting concerns

use std::sync::Arc;
use async_trait::async_trait;

use crate::dom::values::UserId;
use crate::app::use_cases::{AuthUC, UserMgmtUC, PayUC, StockUC};

// Main application service that coordinates use cases
pub struct AppService {
    auth_uc: Arc<AuthUC>,
    user_mgmt_uc: Arc<UserMgmtUC>,
    pay_uc: Arc<PayUC>,
    stock_uc: Arc<StockUC>,
}

impl AppService {
    pub fn new(
        auth_uc: Arc<AuthUC>,
        user_mgmt_uc: Arc<UserMgmtUC>,
        pay_uc: Arc<PayUC>,
        stock_uc: Arc<StockUC>,
    ) -> Self {
        Self {
            auth_uc,
            user_mgmt_uc,
            pay_uc,
            stock_uc,
        }
    }
    
    pub fn auth(&self) -> &AuthUC {
        &self.auth_uc
    }
    
    pub fn user_mgmt(&self) -> &UserMgmtUC {
        &self.user_mgmt_uc
    }
    
    pub fn payments(&self) -> &PayUC {
        &self.pay_uc
    }
    
    pub fn stocks(&self) -> &StockUC {
        &self.stock_uc
    }
}

// AuthorizationService is now re-exported from domain layer as UnifiedPermissionService

// Re-export comprehensive services from domain layer
pub use crate::dom::services::audit_service::AuditService;
pub use crate::dom::services::permission_resolver::{UnifiedPermissionService as AuthorizationService, AuthorizationError};

// Rate limiting service
pub struct RateLimitService;

impl RateLimitService {
    pub fn check_rate_limit(user_id: &UserId, operation: &str) -> Result<(), RateLimitError> {
        // TODO: Implement proper rate limiting with Redis or in-memory cache
        tracing::debug!(
            user_id = %user_id,
            operation = operation,
            "Rate limit check passed"
        );
        Ok(())
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
        T: serde::Serialize;
        
    async fn delete(&self, key: &str) -> Result<(), CacheError>;
    
    async fn clear_pattern(&self, pattern: &str) -> Result<u64, CacheError>;
}

// In-memory cache implementation
pub struct InMemoryCache {
    // TODO: Implement with dashmap or similar
}

#[async_trait]
impl CacheService for InMemoryCache {
    async fn get<T>(&self, _key: &str) -> Result<Option<T>, CacheError>
    where
        T: serde::de::DeserializeOwned,
    {
        // TODO: Implement
        Ok(None)
    }
    
    async fn set<T>(&self, _key: &str, _value: &T, _ttl_seconds: u64) -> Result<(), CacheError>
    where
        T: serde::Serialize,
    {
        // TODO: Implement
        Ok(())
    }
    
    async fn delete(&self, _key: &str) -> Result<(), CacheError> {
        // TODO: Implement
        Ok(())
    }
    
    async fn clear_pattern(&self, _pattern: &str) -> Result<u64, CacheError> {
        // TODO: Implement
        Ok(0)
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