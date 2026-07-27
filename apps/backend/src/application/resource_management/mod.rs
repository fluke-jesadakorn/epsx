// Resource Management Application Layer
// Commands and queries for resource usage tracking and billing

pub mod commands;
pub mod controllers; // HTTP controllers (inbound adapters)
pub mod dtos;
pub mod queries; // Request/Response DTOs

// Re-export command models
pub use commands::{
    IncrementResourceUsageCommand, IncrementResourceUsageResponse, UpdateResourceQuotaCommand,
    UpdateResourceQuotaResponse,
};

// Re-export command handlers
pub use commands::{IncrementResourceUsageCommandHandler, UpdateResourceQuotaCommandHandler};

// Re-export query models
pub use queries::{
    GetBillingPreviewQuery, GetBillingPreviewResponse, GetResourceUsageQuery,
    GetResourceUsageResponse,
};

// Re-export query handlers
pub use queries::{GetBillingPreviewQueryHandler, GetResourceUsageQueryHandler};
