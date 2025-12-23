// Clean architecture library exports

#![allow(improper_ctypes_definitions)]

// Diesel schema module (auto-generated from database)
pub mod schema;

pub mod prelude; // Common imports prelude
pub mod core; // Shared kernel
pub mod domain; // DDD Domain layer with bounded contexts (User Management, Trading Analytics, Notification, Payment)
pub mod application; // Application layer with CQRS command/query handlers
pub mod infrastructure; // Infrastructure layer with adapters and DDD patterns
pub mod web; // Web/API layer (maintains same endpoints, uses DDD internally)
pub mod config; // Configuration
pub mod auth; // Web3 wallet-first authentication system

#[cfg(test)]
pub mod __test__; // Test utilities and test modules
// permissions module removed - replaced by auth/groups.rs (conceptually)

// Selective re-exports for clean namespace
pub use core::{ errors, types, telemetry };
pub use infrastructure::container::DomainContainer;
pub use web::create_router;
