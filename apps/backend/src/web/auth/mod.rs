// Web3-First Authentication module

// Core authentication components
pub mod routes;
pub mod web3_handlers;
// pub mod group_routes; // Removed - references non-existent tables
// pub mod web3_assignment_routes; // Removed - references non-existent services
// pub mod providers; // Removed - legacy OIDC/Firebase providers
// pub mod token_broker; // Removed - legacy multi-provider authentication
pub mod modern_routes;
// pub mod web3_routes; // Removed - references non-existent services
// pub mod oidc_compat_routes; // Removed - references non-existent services

// Pure Web3 authentication routes (wallet-first, no sessions)
// pub mod pure_web3_auth_routes; // Removed - compilation issues

// Main exports
pub use routes::AppState;
// API key service moved to crate::infrastructure::adapters::services::api_key_service
// pub use providers::*; // Removed - legacy OIDC/Firebase providers
// pub use token_broker::*; // Removed - legacy multi-provider authentication

// Export pure Web3 auth routes
// pub use pure_web3_auth_routes::create_pure_web3_auth_routes; // Removed - compilation issues