// Subscription Management Application Layer
// Commands and queries for plan operations (Subscription logic removed - Direct Payment model)

pub mod commands;
pub mod controllers; // HTTP controllers (inbound adapters)
pub mod dtos;
pub mod queries; // Request/Response DTOs

// Re-export command models (Plan only)
pub use commands::{
    CreatePlanCommand, CreatePlanResponse, DeletePlanCommand, DeletePlanResponse,
    UpdatePlanCommand, UpdatePlanResponse,
};

// Re-export command handlers (Plan only)
pub use commands::{
    CreatePlanCommandHandler,
    // UpdatePlanCommandHandler,
    // DeletePlanCommandHandler,
};

// Re-export query handlers (Plan only)
pub use queries::{GetPlanQueryHandler, ListPlansQueryHandler};
