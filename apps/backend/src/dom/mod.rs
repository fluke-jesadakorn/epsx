// Domain layer - Core business entities, value objects, and domain services
// This layer contains the core business logic and is independent of external concerns

pub mod entities;
pub mod values; 
pub mod services;
pub mod events;
pub mod error;
pub mod ports;
// pub mod notification; // Removed - migrated to DDD

// Selective re-exports to prevent namespace pollution
pub use entities::{AggregateRoot, UnitOfWork};
pub use values::{UserId, SessId, PayId, Email, SubscriptionTier, Subscription, Currency, PayStatus, Network};  // Explicit re-exports

pub use services::{AuditService, EPSCacheService, FirebaseUserService};  // Explicit re-exports
pub use events::*;
