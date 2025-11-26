// Realtime Events Application Layer
// Commands and queries for real-time event operations

pub mod commands;
pub mod queries;
pub mod controllers; // HTTP controllers (inbound adapters)
pub mod dtos; // Request/Response DTOs

// Re-export command models
pub use commands::{
    CreateRealtimeEventCommand,
    CreateRealtimeEventResponse,
    MarkEventDeliveredCommand,
    MarkEventDeliveredResponse,
    MarkEventFailedCommand,
    MarkEventFailedResponse,
};

// Re-export command handlers
pub use commands::{
    CreateRealtimeEventCommandHandler,
    MarkEventDeliveredCommandHandler,
    MarkEventFailedCommandHandler,
};

// Re-export query models
pub use queries::{
    GetRealtimeEventQuery,
    GetRealtimeEventResponse,
    ListPendingEventsQuery,
    ListPendingEventsResponse,
    PendingEventSummary,
};

// Re-export query handlers
pub use queries::{
    GetRealtimeEventQueryHandler,
    ListPendingEventsQueryHandler,
};
