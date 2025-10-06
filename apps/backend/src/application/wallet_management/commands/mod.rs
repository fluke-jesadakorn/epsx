// User Management Commands
// Write operations following CQRS pattern

pub mod models;
pub mod handlers;
pub mod admin_models;    // Admin-specific command models (trading analytics pattern)
pub mod admin_handlers;  // Admin command handlers

pub use models::*;
pub use handlers::*;
// Don't glob-export admin modules to avoid naming conflicts
// Use explicit paths: wallet_management::commands::admin_models::*