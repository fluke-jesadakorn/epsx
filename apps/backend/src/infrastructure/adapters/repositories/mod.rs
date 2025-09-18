// Repository Adapters
// SQLx-based repository implementations

pub mod user_repository_adapter;
pub mod realtime_event_repository_adapter;
pub mod connection_repository_adapter;
pub mod user_permission_repository_adapter;
pub mod direct_db_client;
pub mod diesel_types;
pub mod notification_repository_adapter;
pub mod eps_repository_adapter;
pub mod stock_analysis_repository_adapter;

// TODO: Migrate remaining repositories to SQLx
// pub mod session_repository_adapter;
// pub mod market_data_repository_adapter;
// pub mod payment_repository_adapter;
// pub mod transaction_repository_adapter;
// pub mod crypto_address_repository_adapter;
// pub mod payment_method_repository_adapter;
// pub mod plan_repository_adapter;
// pub mod mappers;

pub use user_repository_adapter::UserRepositoryAdapter;
pub use realtime_event_repository_adapter::RealtimeEventRepositoryAdapter;
pub use connection_repository_adapter::ConnectionRepositoryAdapter;
pub use user_permission_repository_adapter::{UserPermissionRepositoryAdapter, LegacyPermissionRepositoryError};
pub use direct_db_client::DirectDbClient;
pub use diesel_types::*;
pub use notification_repository_adapter::NotificationRepositoryAdapter;
pub use eps_repository_adapter::EPSRepositoryAdapter;
pub use stock_analysis_repository_adapter::StockAnalysisRepositoryAdapter;

// Re-export DbPool as SQLx type
use sqlx::PgPool;
pub type DbPool = PgPool;