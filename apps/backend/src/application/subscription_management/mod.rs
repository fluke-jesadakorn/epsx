// Subscription Management Application Layer
// Commands and queries for plan and subscription operations

pub mod commands;
pub mod queries;
pub mod controllers; // HTTP controllers (inbound adapters)
pub mod dtos; // Request/Response DTOs

// Re-export command models
pub use commands::{
    CreatePlanCommand,
    CreatePlanResponse,
    UpdatePlanCommand,
    UpdatePlanResponse,
    DeletePlanCommand,
    DeletePlanResponse,
    CreateSubscriptionCommand,
    CreateSubscriptionResponse,
    CancelSubscriptionCommand,
    CancelSubscriptionResponse,
};

// Re-export command handlers
pub use commands::{
    CreatePlanCommandHandler,
    UpdatePlanCommandHandler,
    DeletePlanCommandHandler,
    CreateSubscriptionCommandHandler,
    CancelSubscriptionCommandHandler,
};

// Re-export query models
pub use queries::{
    GetPlanQuery,
    GetPlanResponse,
    ListPlansQuery,
    ListPlansResponse,
    GetSubscriptionQuery,
    GetSubscriptionResponse,
    ListSubscriptionsQuery,
    ListSubscriptionsResponse,
};

// Re-export query handlers
pub use queries::{
    GetPlanQueryHandler,
    ListPlansQueryHandler,
    GetSubscriptionQueryHandler,
    ListSubscriptionsQueryHandler,
};
