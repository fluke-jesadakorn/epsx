// Repository Adapters
// Modern repository implementations with unified database access

pub mod base_repository;
pub mod user_repository_adapter;
pub mod realtime_event_repository_adapter;
pub mod connection_repository_adapter;
pub mod user_permission_repository_adapter;
pub mod direct_db_client;
pub mod database_types;
pub mod notification_repository_adapter;
pub mod eps_repository_adapter;
pub mod stock_analysis_repository_adapter;
pub mod session_repository_adapter;
pub mod market_data_repository_adapter;
pub mod payment_repository_adapter;
pub mod transaction_repository_adapter;
pub mod crypto_address_repository_adapter;
pub mod payment_method_repository_adapter;
pub mod plan_repository_adapter;
pub mod mappers;

pub use base_repository::{ BaseRepository, SqlxBaseRepository };
pub use user_repository_adapter::UserRepositoryAdapter;
pub use realtime_event_repository_adapter::RealtimeEventRepositoryAdapter;
pub use connection_repository_adapter::ConnectionRepositoryAdapter;
pub use user_permission_repository_adapter::UserPermissionRepositoryAdapter;
pub use direct_db_client::DirectDbClient;
pub use database_types::*;
pub use notification_repository_adapter::NotificationRepositoryAdapter;
pub use eps_repository_adapter::EPSRepositoryAdapter;
pub use stock_analysis_repository_adapter::StockAnalysisRepositoryAdapter;

// Database connection pool type - SQLx PostgreSQL pool
pub type DbPool = sqlx::PgPool;

/// Create a database connection pool for production use
pub async fn create_pool() -> Result<DbPool, sqlx::Error> {
  let database_url = std::env
    ::var("DATABASE_URL")
    .expect("DATABASE_URL must be set");

  sqlx::PgPool::connect(&database_url).await
}

/// Create a test database connection pool
pub async fn create_test_pool() -> Result<DbPool, sqlx::Error> {
  let database_url = std::env
    ::var("DATABASE_URL")
    .unwrap_or_else(|_|
      "postgresql://postgres:password@localhost:5432/epsx_test_db".to_string()
    );

  sqlx::PgPool::connect(&database_url).await
}
