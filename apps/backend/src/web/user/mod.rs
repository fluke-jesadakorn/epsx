// User management module (Web3-first: wallet-based)

pub mod chat_handlers;
pub mod chat_upload_handlers;
pub mod developer_portal; // User-facing API key management
pub mod permissions;
pub mod unified_user_handlers; // OpenID + Unified Response handlers
pub mod watchlist_handlers;

pub use developer_portal::*;
pub use unified_user_handlers::*;
