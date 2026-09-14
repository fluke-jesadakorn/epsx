// Wallet Management Queries
// Read operations following CQRS pattern

pub mod admin_handlers;
pub mod admin_models; // Admin-specific query models (market analytics pattern)
pub mod handlers;
pub mod models; // Admin query handlers

pub use handlers::*;
pub use models::*;
// Don't glob-export admin modules to avoid naming conflicts
// Use explicit paths: wallet_management::queries::admin_models::*
