// Wallet Management Query Models
// These represent requests for data without side effects

pub mod get_wallet;
pub mod get_wallet_permissions;
pub mod list_wallets;
pub mod search_wallets;

pub use get_wallet::{GetWalletQuery, GetWalletResponse, WalletStats};
pub use get_wallet_permissions::{GetWalletPermissionsQuery, GetWalletPermissionsResponse};
pub use list_wallets::{ListWalletsQuery, ListWalletsResponse, WalletSummary};
pub use search_wallets::{SearchWalletsQuery, SearchWalletsResponse};
