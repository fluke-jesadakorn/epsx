// Database Infrastructure Module
// Provides serverless-optimized database connection management

pub mod serverless_connection_manager;

// Re-export commonly used types
pub use serverless_connection_manager::{
    ServerlessConnectionManager,
    ServerlessConnectionConfig,
    DatabaseExecutor,
    PoolStats,
    get_db_pool,
    db_health_check,
};