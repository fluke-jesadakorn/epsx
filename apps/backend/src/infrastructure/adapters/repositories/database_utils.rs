// ============================================================================
// SHARED DATABASE UTILITIES
// ============================================================================
// Common database patterns, error handling macros, and utilities for the EPSX backend
// Reduces code duplication across all repository implementations

use crate::prelude::*;
use diesel_async::{AsyncPgConnection, RunQueryDsl, pooled_connection::deadpool::Pool};
use std::pin::Pin;
use std::future::Future;
use tracing::error;

// ============================================================================
// DATABASE ERROR HANDLING MACROS
// ============================================================================

/// Macro for consistent database error handling with component and operation tracking
#[macro_export]
macro_rules! handle_db_error {
    ($error:expr, $component:expr, $operation:expr) => {
        {
            error!("Database error in {}::{}: {}", $component, $operation, $error);
            AppError::database_error($error.to_string())
                .with_component($component)
                .with_operation($operation)
        }
    };
    ($error:expr, $component:expr, $operation:expr, $entity:expr) => {
        {
            error!("Database error in {}::{} for {}: {}", $component, $operation, $entity, $error);
            AppError::database_error($error.to_string())
                .with_component($component)
                .with_operation($operation)
        }
    };
}

/// Macro for handling database pool connection errors
#[macro_export]
macro_rules! get_db_connection {
    ($pool:expr, $component:expr, $operation:expr) => {
        $pool.get().await.map_err(|e| {
            error!("Pool error in {}::{}: {}", $component, $operation, e);
            AppError::database_error(format!("Pool error: {}", e))
                .with_component($component)
                .with_operation($operation)
        })
    };
}

/// Macro for consistent error handling in database operations that return None
#[macro_export]
macro_rules! handle_not_found {
    ($component:expr, $operation:expr, $entity:expr) => {
        AppError::not_found(&format!("{} not found in {}", $entity, $operation))
            .with_component($component)
            .with_operation($operation)
    };
}

/// Macro for handling validation errors consistently
#[macro_export]
macro_rules! handle_validation_error {
    ($message:expr, $component:expr, $operation:expr) => {
        AppError::validation_error($message)
            .with_component($component)
            .with_operation($operation)
    };
}

// ============================================================================
// REPOSITORY BASE TRAIT AND COMMON PATTERNS
// ============================================================================

/// Base trait for all repository implementations with common functionality
#[async_trait::async_trait]
pub trait RepositoryBase {
    /// Get a database connection from the pool
    async fn get_connection(&self) -> Result<AsyncPgConnection, AppError>;

    /// Perform a health check on the database connection
    async fn health_check(&self) -> Result<(), AppError>;

    /// Get the component name for error reporting
    fn component_name() -> &'static str;
}

/// Common pagination parameters
#[derive(Debug, Clone)]
pub struct PaginationParams {
    pub offset: Option<u32>,
    pub limit: Option<u32>,
    pub order_by: Option<String>,
}

impl Default for PaginationParams {
    fn default() -> Self {
        Self {
            offset: None,
            limit: Some(50), // Default limit
            order_by: None,
        }
    }
}

impl PaginationParams {
    pub fn new(limit: u32, offset: u32) -> Self {
        Self {
            limit: Some(limit),
            offset: Some(offset),
            order_by: None,
        }
    }

    pub fn with_order(mut self, order_by: String) -> Self {
        self.order_by = Some(order_by);
        self
    }
}

/// Paginated result wrapper
#[derive(Debug, Clone)]
pub struct PaginatedResult<T> {
    pub items: Vec<T>,
    pub total_count: u64,
    pub offset: u64,
    pub limit: u64,
    pub has_next: bool,
    pub has_prev: bool,
}

impl<T> PaginatedResult<T> {
    pub fn new(items: Vec<T>, total_count: u64, offset: u64, limit: u64) -> Self {
        let has_next = (offset + limit) < total_count;
        let has_prev = offset > 0;

        Self {
            items,
            total_count,
            offset,
            limit,
            has_next,
            has_prev,
        }
    }

    pub fn empty() -> Self {
        Self {
            items: Vec::new(),
            total_count: 0,
            offset: 0,
            limit: 0,
            has_next: false,
            has_prev: false,
        }
    }
}

/// Common database operations utility
pub struct DatabaseOperations;

impl DatabaseOperations {
    /// Execute a health check query
    pub async fn health_check_query(
        conn: &mut AsyncPgConnection,
        component: &str,
    ) -> Result<(), AppError> {
        use diesel::dsl::sql;

        let _: i32 = diesel::select(sql::<diesel::sql_types::Integer>("SELECT 1"))
            .get_result(conn)
            .await
            .map_err(|e| {
                error!("Health check failed: {}", e);
                AppError::database_error(format!("Database health check error: {}", e))
                    .with_component(component)
                    .with_operation("health_check")
            })?;

        Ok(())
    }
}

// ============================================================================
// CONNECTION POOL MANAGEMENT
// ============================================================================

/// Wrapper for database connection pool with common operations
#[derive(Clone)]
pub struct ConnectionPoolManager {
    pool: &'static Pool<AsyncPgConnection>,
    component: &'static str,
}

impl ConnectionPoolManager {
    pub fn new(pool: &'static Pool<AsyncPgConnection>, component: &'static str) -> Self {
        Self { pool, component }
    }

    /// Execute a database operation with automatic connection management
    pub async fn execute<F, R>(&self, operation: &str, f: F) -> Result<R, AppError>
    where
        F: FnOnce(&mut AsyncPgConnection) -> Pin<Box<dyn Future<Output = Result<R, AppError>> + Send>>,
    {
        let mut conn_obj = self.pool.get().await.map_err(|e| {
            error!("Pool error in {}::{}: {}", self.component, operation, e);
            AppError::database_error(format!("Pool error: {}", e))
                .with_component(self.component)
                .with_operation(operation)
        })?;

        f(&mut conn_obj).await
    }

    /// Check pool health
    pub async fn health_check(&self) -> Result<(), AppError> {
        let mut conn_obj = self.pool.get().await.map_err(|e| {
            error!("Pool error in {}::health_check: {}", self.component, e);
            AppError::database_error(format!("Pool error: {}", e))
                .with_component(self.component)
                .with_operation("health_check")
        })?;

        DatabaseOperations::health_check_query(&mut conn_obj, self.component).await
    }
}

// ============================================================================
// UTILITY FUNCTIONS
// ============================================================================

/// Convert wallet address to lowercase consistently
pub fn normalize_wallet_address(address: &str) -> String {
    address.to_lowercase()
}

/// Validate wallet address format (basic validation)
pub fn validate_wallet_address(address: &str) -> Result<(), AppError> {
    if address.len() < 42 || address.len() > 50 {
        return Err(handle_validation_error!(
            "Invalid wallet address length",
            "validation",
            "wallet_address"
        ));
    }

    if !address.starts_with("0x") && address.len() == 42 {
        return Err(handle_validation_error!(
            "Wallet address must start with 0x",
            "validation",
            "wallet_address"
        ));
    }

    Ok(())
}

/// Create a timestamp for database operations
pub fn current_timestamp() -> chrono::DateTime<chrono::Utc> {
    chrono::Utc::now()
}

/// Create a timestamp for expiration calculations
pub fn add_hours_to_timestamp(hours: i64) -> chrono::DateTime<chrono::Utc> {
    chrono::Utc::now() + chrono::Duration::hours(hours)
}

// ============================================================================
// TESTING UTILITIES
// ============================================================================

#[cfg(test)]
pub mod testing {
    use super::*;
    use diesel_async::pooled_connection::AsyncDieselConnectionManager;
    use diesel_async::AsyncConnection;
    use tracing::info;

    /// Create a mock connection pool for testing
    pub async fn create_test_pool() -> Arc<Pool<AsyncPgConnection>> {
        // Use in-memory SQLite for testing or connect to test database
        // For this implementation, we'll create a simple test configuration

        // Get test database URL from environment or use default test configuration
        let database_url = std::env::var("TEST_DATABASE_URL")
            .unwrap_or_else(|_| "postgresql://postgres:password@localhost:5432/epsx_test_db".to_string());

        // Create connection pool configuration for testing
        let config = AsyncDieselConnectionManager::<AsyncPgConnection>::new(database_url);

        let pool = Pool::builder(config)
            .max_size(5) // Small pool for testing
            .build()
            .expect("Failed to create test database pool");

        Arc::new(pool)
    }

    /// Create test database utilities
    pub struct TestDatabase {
        pool: Arc<Pool<AsyncPgConnection>>,
    }

    impl TestDatabase {
        pub async fn new() -> Self {
            Self {
                pool: create_test_pool().await,
            }
        }

        pub async fn cleanup(&self) -> Result<(), AppError> {
            // Clean up test data from all relevant tables
            let mut conn = self.pool.get().await
                .map_err(|e| AppError::database_error(e.to_string()))?;

            // Execute cleanup operations in a transaction
                conn.transaction::<_, AppError, _>(|conn| Box::pin(async move {
                    // Clean up test data in dependency order


                    // Clean up sessions first (foreign key dependencies)
                    diesel_async::RunQueryDsl::execute(diesel::sql_query("DELETE FROM user_sessions WHERE wallet_address LIKE 'test_%'"), conn)
                        .await
                        .map_err(|e| AppError::database_error(e.to_string()))?;

                    // Clean up permission assignments
                    diesel_async::RunQueryDsl::execute(diesel::sql_query("DELETE FROM user_permissions WHERE user_id LIKE 'test_%'"), conn)
                        .await
                        .map_err(|e| AppError::database_error(e.to_string()))?;

                    // Clean up test users
                    diesel_async::RunQueryDsl::execute(diesel::sql_query("DELETE FROM users WHERE wallet_address LIKE 'test_%'"), conn)
                        .await
                        .map_err(|e| AppError::database_error(e.to_string()))?;

                    // Clean up test permission groups
                    diesel_async::RunQueryDsl::execute(diesel::sql_query("DELETE FROM groups WHERE group_name LIKE 'test_%'"), conn)
                        .await
                        .map_err(|e| AppError::database_error(e.to_string()))?;

                    Ok(())
                })).await?;

            info!("Test database cleanup completed successfully");
            Ok(())
        }
    }
}

// ============================================================================
// EXAMPLE USAGE
// ============================================================================

/*
// Example repository using the shared utilities:

#[derive(Clone)]
pub struct ExampleRepository {
    db_pool: ConnectionPoolManager,
}

impl ExampleRepository {
    pub fn new(pool: &'static Pool<AsyncPgConnection>) -> Self {
        Self {
            db_pool: ConnectionPoolManager::new(pool, "example_repository"),
        }
    }

    pub async fn find_by_id(&self, id: &str) -> Result<Option<ExampleEntity>, AppError> {
        self.db_pool.execute("find_by_id", |conn| {
            let entity = example_table::table
                .filter(example_table::id.eq(id))
                .first::<ExampleEntity>(conn)
                .await
                .optional()
                .map_err(|e| handle_db_error!(e, Self::component_name(), "find_by_id", id))?;

            Ok(entity)
        }).await
    }

    pub async fn paginated_search(&self, params: PaginationParams) -> Result<PaginatedResult<ExampleEntity>, AppError> {
        self.db_pool.execute("paginated_search", |conn| {
            // Get total count
            let total_count = DatabaseOperations::count_query(
                conn,
                example_table::table,
                example_table::is_active.eq(true),
                Self::component_name(),
                "paginated_search_count"
            ).await?;

            // Get paginated results
            let offset = params.offset.unwrap_or(0) as i64;
            let limit = params.limit.unwrap_or(50) as i64;

            let items = example_table::table
                .filter(example_table::is_active.eq(true))
                .limit(limit)
                .offset(offset)
                .load::<ExampleEntity>(conn)
                .await
                .map_err(|e| handle_db_error!(e, Self::component_name(), "paginated_search"))?;

            Ok(PaginatedResult::new(items, total_count as u64, offset as u64, limit as u64))
        }).await
    }
}

impl RepositoryBase for ExampleRepository {
    async fn get_connection(&self) -> Result<AsyncPgConnection, AppError> {
        self.db_pool.get_connection("base_operation").await
    }

    async fn health_check(&self) -> Result<(), AppError> {
        self.db_pool.health_check().await
    }

    fn component_name() -> &'static str {
        "example_repository"
    }
}
*/