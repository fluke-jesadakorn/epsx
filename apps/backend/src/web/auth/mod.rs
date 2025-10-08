// Web3-First Authentication module

// Active authentication components
pub mod app_state;
// pub mod routes; // DELETED - routes now managed by unified_router
pub mod web3_handlers;
pub mod openid_web3_handlers;
pub mod session_verification_handlers;

// Main exports
pub use app_state::AppState;