use async_trait::async_trait;
use diesel_async::{AsyncPgConnection, pooled_connection::deadpool::Pool};
use std::sync::Arc;
use crate::core::errors::{AppResult, AppError};

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

/// Base repository implementation with Diesel integration
/// Safe Send/Sync implementation - contains Arc<&'static Pool> which is thread-safe
#[derive(Clone)]
pub struct DieselBaseRepository {
    pool: Arc<&'static Pool<AsyncPgConnection>>,
}

impl DieselBaseRepository {
    pub fn new(pool: Arc<&'static Pool<AsyncPgConnection>>) -> Self {
        Self { pool }
    }

    pub fn get_pool(&self) -> &Pool<AsyncPgConnection> {
        &self.pool
    }

    /// Standard health check implementation
    pub async fn health_check_impl(&self) -> AppResult<()> {
        use diesel_async::RunQueryDsl;

        let mut conn = self.pool.get().await
            .map_err(|e| AppError::invalid_operation(
                format!("Failed to get database connection: {}", e)
            ))?;

        // Use Diesel DSL with a simple select statement
        diesel::select(diesel::dsl::sql::<diesel::sql_types::Integer>("SELECT 1"))
            .get_result::<i32>(&mut conn)
            .await
            .map_err(|e| AppError::invalid_operation(
                format!("Database health check failed: {}", e)
            ))?;
        Ok(())
    }
}

// DieselBaseRepository is automatically Send+Sync because:
// - Arc<&'static Pool> is Send+Sync
// - Pool is designed for concurrent access
// No unsafe implementations needed - Rust's type system handles this safely