// Query Handlers for Wallet Management
// These implement the QueryHandler trait and mediate between web layer and repositories

pub mod get_wallet_handler;
pub mod list_wallets_handler;
pub mod search_wallets_handler;
pub mod get_wallet_permissions_handler;
pub mod get_session_handler;
pub mod get_wallet_sessions_handler;
pub mod list_wallet_sessions_handler;
pub mod get_token_info_handler;

pub use get_wallet_handler::GetWalletQueryHandler;
pub use list_wallets_handler::ListWalletsQueryHandler;
pub use search_wallets_handler::SearchWalletsQueryHandler;
pub use get_wallet_permissions_handler::GetWalletPermissionsQueryHandler;
pub use get_session_handler::GetSessionQueryHandler;
pub use get_wallet_sessions_handler::GetWalletSessionsQueryHandler;
pub use list_wallet_sessions_handler::ListWalletSessionsQueryHandler;
pub use get_token_info_handler::GetTokenInfoQueryHandler;
