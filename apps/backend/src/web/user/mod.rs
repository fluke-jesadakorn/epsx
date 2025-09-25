// User management module

pub mod handlers;
pub mod routes;
// Legacy permissions module removed for Web3-first migration
// pub mod permissions;

// Pure Web3 wallet-first handlers and routes
// pub mod wallet_user_handlers; // Removed - compilation issues
// pub mod pure_web3_user_routes; // Removed - compilation issues

pub use handlers::*;
pub use routes::*;

// Export pure Web3 routes
// pub use pure_web3_user_routes::{
//     create_pure_web3_user_routes,
//     create_pure_web3_user_public_routes
// }; // Removed - compilation issues