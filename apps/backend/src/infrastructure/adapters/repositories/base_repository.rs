use async_trait::async_trait;
use epsx_contracts::errors::{AppError, AppResult};
use sqlx::PgPool;
use std::sync::Arc;

/// Base repository trait providing common database operations
#[async_trait]
pub trait BaseRepository<T, ID> {
    /// Find entity by ID
    async fn find_by_id(&self, id: &ID) -> AppResult<Option<T>>;
    /// Save entity (insert or update)
    async fn save(&self, entity: &T) -> AppResult<()>;
    /// Delete entity by ID
    async fn delete(&self, id: &ID) -> AppResult<()>;
    /// Generate next identity
    async fn next_identity(&self) -> AppResult<ID>;
    /// Health check for repository
    async fn health_check(&self) -> AppResult<()>;
}

/// Base repository implementation with sqlx integration.
/// Send/Sync: `Arc<PgPool>` is thread-safe.
#[derive(Clone)]
pub struct DieselBaseRepository {
    pool: Arc<PgPool>,
}

impl DieselBaseRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }

    pub fn get_pool(&self) -> &PgPool {
        &self.pool
    }

    /// Standard health check implementation
    pub async fn health_check_impl(&self) -> AppResult<()> {
        sqlx::query("SELECT 1::INTEGER")
            .execute(self.pool.as_ref())
            .await
            .map_err(|e| {
                AppError::invalid_operation(format!("Database health check failed: {}", e))
            })?;
        Ok(())
    }
}