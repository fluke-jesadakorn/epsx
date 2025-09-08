// Domain Layer - Core business logic following DDD principles
// This layer contains bounded contexts, aggregates, entities, value objects, and domain services
// It has no dependencies on external concerns (infrastructure, application, web layers)

pub mod shared_kernel;
pub mod user_management;
pub mod trading_analytics;
pub mod notification;
pub mod payment;
pub mod authentication;
pub mod session_management;
pub mod realtime_events;

// Re-export shared kernel for easy access
pub use shared_kernel::{
    AggregateRoot, 
    DomainEvent, 
    DomainEventBus, 
    Specification, 
    ValueObject, 
    DomainError,
    DomainResult
};