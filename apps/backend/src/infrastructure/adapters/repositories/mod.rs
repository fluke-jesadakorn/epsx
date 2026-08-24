// Repository Adapters
// Web3-first repository implementations with comprehensive blockchain integration

pub mod base_repository;
pub mod database_types;
pub mod database_utils; // NEW: Shared database utilities and error handling macros
pub mod mappers;
pub mod market_data_repository_adapter;
pub mod notification_repository_adapter;
pub mod payment_repository_adapter;
pub mod payment_repository_adapter_cross_pool; // Wave 11 / Track A — cross-pool port impls
pub mod ranking_entitlement_snapshot_repository;
pub mod stock_analysis_repository_adapter;
pub mod tradingview_eps_repository; // TradingView EPS data adapter

pub mod wallet_user;

pub mod developer_portal;
pub mod permission_plan_repository_adapter;
pub mod plan_repository_adapter; // NEW // Developer portal API keys and modules

// Payment-bounded-context repository adapters.
pub mod credit_repository_adapter;
pub mod payment_context_repository_adapter;
pub mod sqlx_credit_repository;
pub mod sqlx_notification_repository;
pub mod sqlx_payment_repository;
pub mod sqlx_plan_repository;
pub mod sqlx_ranking_entitlement_repository;
pub mod sqlx_stock_analysis_repository;

pub mod payment;

pub use payment::{
    is_context_usable, CreditRepositoryAdapter, NewPaymentContextDb, PaymentContextDb,
    PaymentContextRepositoryAdapter, PaymentContextSearchCriteria, PaymentRepositoryAdapter,
    PaymentSubscriptionRepositoryAdapter, SubscriptionSearchCriteria, UpdatePaymentContextDb,
};

#[deprecated(
    since = "0.2.0",
    note = "Use `PaymentSubscriptionRepositoryAdapter` (in `infrastructure::adapters::repositories::payment::subscription_repository_adapter`) — wave11(track-b) renamed the type to make ownership explicit."
)]
pub use payment::PaymentSubscriptionRepositoryAdapter as SubscriptionRepositoryAdapter;

pub use base_repository::{BaseRepository, DieselBaseRepository};
pub use database_types::*;
pub use notification_repository_adapter::NotificationRepositoryAdapter;
pub use ranking_entitlement_snapshot_repository::PostgresRankingEntitlementSnapshotRepository;
pub use stock_analysis_repository_adapter::StockAnalysisRepositoryAdapter;
pub use tradingview_eps_repository::TradingViewEPSRepository;

pub use plan_repository_adapter::PostgresPlanRepositoryAdapter;
pub use wallet_user::WalletUserRepositoryAdapter;

pub use permission_plan_repository_adapter::PlanRepositoryAdapter;

// Database connection pool type - alias for sqlx::PgPool (BIG-BANG migration)
pub type TlsPool = sqlx::PgPool;
pub type DbPool = sqlx::PgPool;

/// Create a database connection pool for production use (sqlx)
pub async fn create_pool() -> anyhow::Result<sqlx::PgPool> {
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await?;
    Ok(pool)
}

/// Create a test database connection pool (sqlx)
pub async fn create_test_pool() -> anyhow::Result<sqlx::PgPool> {
    let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        "postgresql://postgres:password@localhost:5432/epsx_test_db".to_string()
    });
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;
    Ok(pool)
}