// Application Layer - Orchestrates domain operations and handles use cases
// This layer contains the business application logic, commands, queries, and ports
// It depends on the domain layer but is independent of infrastructure concerns

pub mod shared;
pub mod user_management; 
pub mod payment;
pub mod ports;
pub mod services;
pub mod authentication;

// Convenience re-exports for legacy compatibility
pub mod auth;
pub mod user;

// Re-export commonly used types
pub use shared::{
    ApplicationError, 
    ApplicationResult,
    Command, 
    CommandHandler, 
    Query, 
    QueryHandler,
    PaginationParams,
    SortParams,
    SortDirection
};