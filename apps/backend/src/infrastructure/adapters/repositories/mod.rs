// Repository Adapters
// Web3-first repository implementations with comprehensive blockchain integration

pub mod base_repository;
pub mod database_types;
pub mod notification_repository_adapter;
pub mod stock_analysis_repository_adapter;
pub mod market_data_repository_adapter;
pub mod tradingview_eps_repository; // TradingView EPS data adapter
// pub mod payment_repository_adapter; // Temporarily disabled
pub mod mappers;

pub mod wallet_user_repository_adapter;
pub mod session_repository_adapter;


pub use base_repository::{ BaseRepository, SqlxBaseRepository };
pub use database_types::*;
pub use notification_repository_adapter::NotificationRepositoryAdapter;
pub use stock_analysis_repository_adapter::StockAnalysisRepositoryAdapter;
pub use tradingview_eps_repository::TradingViewEPSRepository;

pub use wallet_user_repository_adapter::WalletUserRepositoryAdapter;


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
