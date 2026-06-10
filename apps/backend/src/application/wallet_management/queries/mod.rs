// Wallet Management Queries
// Read operations following CQRS pattern

pub mod models;
pub mod handlers;
pub mod admin_models;    // Admin-specific query models (market analytics pattern)
pub mod admin_handlers;  // Admin query handlers

pub use models::*;
pub use handlers::*;
// Don't glob-export admin modules to avoid naming conflicts
// Use explicit paths: wallet_management::queries::admin_models::*