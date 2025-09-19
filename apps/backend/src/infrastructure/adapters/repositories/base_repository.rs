use async_trait::async_trait;
use sqlx::PgPool;
use std::sync::Arc;
use crate::domain::shared_kernel::{DomainResult, DomainError};

/// Base repository trait providing common database operations
#[async_trait]
pub trait BaseRepository<T, ID> {
    /// Find entity by ID
    async fn find_by_id(&self, id: &ID) -> DomainResult<Option<T>>;
    
    /// Save entity (insert or update)
    async fn save(&self, entity: &T) -> DomainResult<()>;
    
    /// Delete entity by ID
    async fn delete(&self, id: &ID) -> DomainResult<()>;
    
    /// Generate next identity
    async fn next_identity(&self) -> DomainResult<ID>;
    
    /// Health check for repository
    async fn health_check(&self) -> DomainResult<()>;
}

/// Base repository implementation with SQLx integration
/// Safe Send/Sync implementation - contains Arc<PgPool> which is thread-safe
#[derive(Clone)]
pub struct SqlxBaseRepository {
    pool: Arc<PgPool>,
}

impl SqlxBaseRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
    
    pub fn get_pool(&self) -> &PgPool {
        &self.pool
    }
    
    /// Standard health check implementation
    pub async fn health_check_impl(&self) -> DomainResult<()> {
        let _ = sqlx::query("SELECT 1")
            .fetch_one(self.get_pool())
            .await
            .map_err(|e| DomainError::invalid_operation(
                format!("Database health check failed: {}", e), 
                "BaseRepository"
            ))?;
        Ok(())
    }
}

// SqlxBaseRepository is automatically Send+Sync because:
// - Arc<PgPool> is Send+Sync 
// - PgPool is designed for concurrent access
// No unsafe implementations needed - Rust's type system handles this safely