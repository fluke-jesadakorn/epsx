use crate::prelude::TlsPool;
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

pub mod wallet_user;

pub mod permission_plan_repository_adapter;
pub mod plan_repository_adapter; // NEW
pub mod subscription_repository_adapter;
pub mod credit_repository_adapter; // Credit wallet system
pub mod developer_portal; // Developer portal API keys and modules
pub mod payment_context_repository_adapter; // V2 Dynamic payment contexts


pub use base_repository::{ BaseRepository, DieselBaseRepository };
pub use database_types::*;
pub use notification_repository_adapter::NotificationRepositoryAdapter;
pub use stock_analysis_repository_adapter::StockAnalysisRepositoryAdapter;
pub use tradingview_eps_repository::TradingViewEPSRepository;
pub use payment_repository_adapter::PaymentRepositoryAdapter;

pub use wallet_user::WalletUserRepositoryAdapter;
pub use plan_repository_adapter::PostgresPlanRepositoryAdapter;
pub use subscription_repository_adapter::SubscriptionRepositoryAdapter;
pub use credit_repository_adapter::CreditRepositoryAdapter;
pub use payment_context_repository_adapter::{PaymentContextRepositoryAdapter, PaymentContextSearchCriteria};

// Export both new and legacy names for backward compatibility
pub use permission_plan_repository_adapter::{PlanRepositoryAdapter, PermissionPlanRepositoryAdapter};

// Database connection pool type - Diesel async PostgreSQL pool
pub type DbPool = &'static TlsPool;

/// Create a database connection pool for production use
pub async fn create_pool() -> anyhow::Result<&'static TlsPool> {
  let database_url = std::env
    ::var("DATABASE_URL")
    .expect("DATABASE_URL must be set");

  let manager = crate::infrastructure::database::diesel_connection_manager::TlsConnectionManager::new(database_url);
  let pool = deadpool::managed::Pool::builder(manager)
    .max_size(10)
    .build()?;

  // Leak pool to make it 'static
  Ok(Box::leak(Box::new(pool)))
}

/// Create a test database connection pool
pub async fn create_test_pool() -> anyhow::Result<&'static TlsPool> {
  let database_url = std::env
    ::var("DATABASE_URL")
    .unwrap_or_else(|_|
      "postgresql://postgres:password@localhost:5432/epsx_test_db".to_string()
    );

  let manager = crate::infrastructure::database::diesel_connection_manager::TlsConnectionManager::new(database_url);
  let pool = deadpool::managed::Pool::builder(manager)
    .max_size(5)
    .build()?;

  // Leak pool to make it 'static
  Ok(Box::leak(Box::new(pool)))
}
