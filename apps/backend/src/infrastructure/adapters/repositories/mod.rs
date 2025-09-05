// Repository Adapters
// Concrete implementations of repository ports using Diesel ORM

pub mod user_repository_adapter;
pub mod session_repository_adapter;
pub mod stock_analysis_repository_adapter;
pub mod market_data_repository_adapter;
pub mod notification_repository_adapter;
pub mod payment_repository_adapter;
pub mod transaction_repository_adapter;
pub mod crypto_address_repository_adapter;
pub mod payment_method_repository_adapter;
pub mod realtime_event_repository_adapter;
pub mod connection_repository_adapter;
pub mod mappers;

pub use user_repository_adapter::UserRepositoryAdapter;
pub use session_repository_adapter::SessionRepositoryAdapter;
pub use stock_analysis_repository_adapter::StockAnalysisRepositoryAdapter;
pub use market_data_repository_adapter::{MarketDataRepositoryAdapter, MarketDataCriteria, MarketStatistics};
pub use notification_repository_adapter::NotificationRepositoryAdapter;
pub use payment_repository_adapter::PaymentRepositoryAdapter;
pub use transaction_repository_adapter::TransactionRepositoryAdapter;
pub use crypto_address_repository_adapter::CryptoAddressRepositoryAdapter;
pub use payment_method_repository_adapter::PaymentMethodRepositoryAdapter;
pub use realtime_event_repository_adapter::RealtimeEventRepositoryAdapter;
pub use connection_repository_adapter::ConnectionRepositoryAdapter;
pub use mappers::*;