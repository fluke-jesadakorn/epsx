// User management module (Web3-first: wallet-based)

pub mod unified_user_handlers; // OpenID + Unified Response handlers
pub mod developer_portal; // User-facing API key management
pub mod permissions;
pub mod watchlist_handlers;
pub mod chat_handlers;

pub use unified_user_handlers::*;
pub use developer_portal::*;