// Repository Adapters
// Web3-first repository implementations with comprehensive blockchain integration

pub mod base_repository;
pub mod database_types;
pub mod database_utils; // NEW: Shared database utilities and error handling macros
pub mod notification_repository_adapter;
pub mod stock_analysis_repository_adapter;
pub mod market_data_repository_adapter;
pub mod tradingview_eps_repository; // TradingView EPS data adapter
pub mod payment_repository_adapter;
pub mod mappers;

pub mod wallet_user_repository_adapter;
pub mod session_repository_adapter;
pub mod group_repository_adapter;
pub use group_repository_adapter as permission_group_repository_adapter;
pub mod plan_repository_adapter; // NEW
pub mod subscription_repository_adapter; 
pub mod developer_portal; // Developer portal API keys and modules
pub mod payment_context_repository_adapter; // V2 Dynamic payment contexts


pub use base_repository::{ BaseRepository, DieselBaseRepository };
pub use database_types::*;
pub use notification_repository_adapter::NotificationRepositoryAdapter;
pub use stock_analysis_repository_adapter::StockAnalysisRepositoryAdapter;
pub use tradingview_eps_repository::TradingViewEPSRepository;
pub use payment_repository_adapter::PaymentRepositoryAdapter;

pub use wallet_user_repository_adapter::WalletUserRepositoryAdapter;
pub use plan_repository_adapter::PostgresPlanRepositoryAdapter;
pub use subscription_repository_adapter::SubscriptionRepositoryAdapter;
pub use payment_context_repository_adapter::{PaymentContextRepositoryAdapter, PaymentContextSearchCriteria};

// Export both new and legacy names for backward compatibility
pub use group_repository_adapter::{GroupRepositoryAdapter, PermissionGroupRepositoryAdapter};

use diesel_async::{AsyncPgConnection, pooled_connection::AsyncDieselConnectionManager, pooled_connection::deadpool::Pool};

// Database connection pool type - Diesel async PostgreSQL pool
pub type DbPool = &'static Pool<AsyncPgConnection>;

/// Create a database connection pool for production use
pub async fn create_pool() -> Result<&'static Pool<AsyncPgConnection>, Box<dyn std::error::Error>> {
  let database_url = std::env
    ::var("DATABASE_URL")
    .expect("DATABASE_URL must be set");

  let config = AsyncDieselConnectionManager::<AsyncPgConnection>::new(&database_url);
  let pool = Pool::builder(config)
    .max_size(10)
    .build()?;

  // Leak pool to make it 'static
  Ok(Box::leak(Box::new(pool)))
}

/// Create a test database connection pool
pub async fn create_test_pool() -> Result<&'static Pool<AsyncPgConnection>, Box<dyn std::error::Error>> {
  let database_url = std::env
    ::var("DATABASE_URL")
    .unwrap_or_else(|_|
      "postgresql://postgres:password@localhost:5432/epsx_test_db".to_string()
    );

  let config = AsyncDieselConnectionManager::<AsyncPgConnection>::new(&database_url);
  let pool = Pool::builder(config)
    .max_size(5)
    .build()?;

  // Leak pool to make it 'static
  Ok(Box::leak(Box::new(pool)))
}
