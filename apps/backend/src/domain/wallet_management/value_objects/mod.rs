// Wallet Management Value Objects
// Immutable objects that represent concepts in the user management domain

pub mod user_id;
pub mod wallet_address;
pub mod permission;
pub mod session_id;

pub use user_id::UserId; // Legacy - to be replaced with WalletAddress
pub use wallet_address::WalletAddress; // PRIMARY - Web3 identity
pub use permission::{ Permission, PermissionType }; // Enhanced for Web3 permission types
pub use session_id::SessionId;
