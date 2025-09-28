// Application Layer - Orchestrates domain operations and handles use cases
// This layer contains the business application logic, commands, queries, and ports
// It depends on the domain layer but is independent of infrastructure concerns

pub mod shared;
pub mod user_management; 
// pub mod payment; // Temporarily disabled due to aggregate implementation issues
pub mod ports;

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