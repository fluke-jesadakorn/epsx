// Application Layer - Orchestrates domain operations and handles use cases
// This layer contains the business application logic, commands, queries, and ports
// It depends on the domain layer but is independent of infrastructure concerns

pub mod shared;
pub mod wallet_management; // Web3-first: wallet-based operations
pub mod permission_management; // Permission groups, policies, and assignments
pub mod subscription_management; // Plans, subscriptions, and billing
pub mod market_analytics; // Stock analysis and EPS rankings
pub mod notification; // Multi-channel notification system with scheduling
pub mod realtime_events; // Real-time event delivery and retry system
pub mod resource_management; // Resource usage tracking and billing
pub mod payment;
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