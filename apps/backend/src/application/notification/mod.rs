pub mod commands;
pub mod queries;
pub mod controllers; // HTTP controllers (inbound adapters)
pub mod dtos; // Request/Response DTOs

// Re-export command and query types separately to avoid name conflicts
pub use commands::models as command_models;
pub use commands::handlers as command_handlers;
pub use queries::models as query_models;
pub use queries::handlers as query_handlers;
