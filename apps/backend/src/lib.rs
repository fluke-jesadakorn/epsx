// Clean architecture library exports
#![allow(dead_code)]

pub mod core; // Shared kernel
pub mod dom; // Domain layer (legacy, partially migrated to DDD bounded contexts)
pub mod domain; // New DDD Domain layer with bounded contexts (User Management, Trading Analytics, Notification, Payment)
pub mod app; // Application layer (legacy, partially migrated to CQRS)
pub mod application; // New Application layer with CQRS command/query handlers
pub mod infra; // Infrastructure layer (legacy)
pub mod infrastructure; // New Infrastructure layer with adapters and DDD patterns
pub mod web; // Web/API layer (maintains same endpoints, uses DDD internally)
pub mod config; // Configuration
pub mod auth; // Modern authentication with OIDC
// permissions module removed - replaced by auth/roles.rs

// Selective re-exports for clean namespace
pub use core::{ errors, types, telemetry };
pub use infra::AppContainer;
pub use web::create_router;
