// Domain Layer - Core business logic following DDD principles
// This layer contains bounded contexts, aggregates, entities, value objects, and domain services
// It has no dependencies on external concerns (infrastructure, application, web layers)

pub mod shared_kernel;
pub mod wallet_management; // Web3-first: wallet-based user management
pub mod payment; // Payment processing and validation
pub mod permission_management; // Permission groups, policies, and assignments
pub mod subscription_management; // Plans, subscriptions, and billing
pub mod trading_analytics;
pub mod notification;

pub mod realtime_events;
pub mod resource_management; // New domain for resource tracking and billing

// Re-export shared kernel for easy access
pub use shared_kernel::{
  AggregateRoot,
  DomainEvent,
  DomainEventBus,
  Specification,
  ValueObject,
};
