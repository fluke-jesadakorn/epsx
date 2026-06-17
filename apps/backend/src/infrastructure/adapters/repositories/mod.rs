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
pub mod payment_repository_adapter_cross_pool; // Wave 11 / Track A — cross-pool port impls
pub mod mappers;

pub mod wallet_user;

pub mod permission_plan_repository_adapter;
pub mod plan_repository_adapter; // NEW
pub mod developer_portal; // Developer portal API keys and modules

// Payment-bounded-context repository adapters.
//
// Pre-wave-11 these all sat in this central layer. Wave 11 /
// Track B moves ONLY `subscription_repository_adapter` into
// `payment/` (see audit-payments.md §3 row 3, the "strongest
// outward leak"). The other three
// (`payment_repository_adapter`,
// `payment_context_repository_adapter`,
// `credit_repository_adapter`) stay at the central layer for
// one more wave; the `payment/` subdir re-exports them as a
// forward-move marker so future call sites can use the
// destination path today.
pub mod payment_context_repository_adapter;
pub mod credit_repository_adapter;

pub mod payment;

// `use payment::*` keeps the legacy flat re-exports working
// for any pre-wave-11 callers (e.g.
// `use crate::infrastructure::adapters::repositories::SubscriptionRepositoryAdapter`).
// The real type now lives at
// `repositories::payment::PaymentSubscriptionRepositoryAdapter`.
pub use payment::{
    CreditRepositoryAdapter, NewPaymentContextDb, PaymentContextDb,
    PaymentContextRepositoryAdapter, PaymentContextSearchCriteria,
    PaymentRepositoryAdapter, PaymentSubscriptionRepositoryAdapter,
    SubscriptionSearchCriteria, UpdatePaymentContextDb, is_context_usable,
};

// wave11(track-b) deprecation shim: pre-wave-11 callers used
// `SubscriptionRepositoryAdapter` (without the `Payment` prefix).
// The wave-11 task brief renames it to
// `PaymentSubscriptionRepositoryAdapter` to make ownership
// explicit. This alias lets any pre-wave-11 import keep
// compiling for one minor version. Drop after the next wave.
#[deprecated(
    since = "0.2.0",
    note = "Use `PaymentSubscriptionRepositoryAdapter` (in `infrastructure::adapters::repositories::payment::subscription_repository_adapter`) — wave11(track-b) renamed the type to make ownership explicit."
)]
pub use payment::PaymentSubscriptionRepositoryAdapter as SubscriptionRepositoryAdapter;


pub use base_repository::{ BaseRepository, DieselBaseRepository };
pub use database_types::*;
pub use notification_repository_adapter::NotificationRepositoryAdapter;
pub use stock_analysis_repository_adapter::StockAnalysisRepositoryAdapter;
pub use tradingview_eps_repository::TradingViewEPSRepository;

pub use wallet_user::WalletUserRepositoryAdapter;
pub use plan_repository_adapter::PostgresPlanRepositoryAdapter;

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
