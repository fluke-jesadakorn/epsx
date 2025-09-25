// Clean architecture library exports

pub mod core; // Shared kernel
pub mod domain; // DDD Domain layer with bounded contexts (User Management, Trading Analytics, Notification, Payment)
pub mod application; // Application layer with CQRS command/query handlers
pub mod infrastructure; // Infrastructure layer with adapters and DDD patterns
pub mod web; // Web/API layer (maintains same endpoints, uses DDD internally)
pub mod config; // Configuration
pub mod auth; // Modern authentication with OIDC
// permissions module removed - replaced by auth/roles.rs

// Selective re-exports for clean namespace
pub use core::{ errors, types, telemetry };
pub use infrastructure::container::DomainContainer;
pub use web::create_router;
