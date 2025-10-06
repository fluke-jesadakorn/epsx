// Wallet Management Query Models
// These represent requests for data without side effects

pub mod get_wallet;
pub mod get_wallet_permissions;
pub mod search_wallets;
pub mod list_wallets;
pub mod get_session;
pub mod get_wallet_sessions;
pub mod list_wallet_sessions;
pub mod get_token_info;

pub use get_wallet::{GetWalletQuery, GetWalletResponse, WalletStats};
pub use get_wallet_permissions::{GetWalletPermissionsQuery, GetWalletPermissionsResponse};
pub use search_wallets::{SearchWalletsQuery, SearchWalletsResponse};
pub use list_wallets::{ListWalletsQuery, ListWalletsResponse, WalletSummary};
pub use get_session::{GetSessionQuery, GetSessionResponse};
pub use get_wallet_sessions::{GetWalletSessionsQuery, GetWalletSessionsResponse};
pub use list_wallet_sessions::{ListWalletSessionsQuery, ListWalletSessionsResponse, WalletSessionInfo};
pub use get_token_info::{GetTokenInfoQuery, GetTokenInfoResponse, TokenInfo};