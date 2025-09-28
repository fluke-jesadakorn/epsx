// Domain Layer - Core business logic following DDD principles
// This layer contains bounded contexts, aggregates, entities, value objects, and domain services
// It has no dependencies on external concerns (infrastructure, application, web layers)

pub mod shared_kernel;
pub mod user_management;
pub mod trading_analytics;
pub mod notification;
// pub mod payment; // Temporarily disabled due to aggregate implementation issues
pub mod authorization;
pub mod realtime_events;
pub mod resource_management; // New domain for resource tracking and billing

// Re-export shared kernel for easy access
pub use shared_kernel::{
    AggregateRoot, 
    DomainEvent, 
    DomainEventBus, 
    Specification, 
    ValueObject
};