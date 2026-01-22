// Clean architecture library exports

#![allow(improper_ctypes_definitions)]
// Allow architectural patterns used throughout the codebase
#![allow(clippy::redundant_allocation)] // Arc<&'static Pool<AsyncPgConnection>> is used for DI
#![allow(clippy::result_large_err)] // AppError is intentionally rich for debugging
#![allow(clippy::too_many_arguments)] // Complex domain constructors require many params

// Diesel schema module (auto-generated from database)
// Diesel schema modules
pub mod schemas;

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
// permissions module removed - replaced by auth/plans.rs (conceptually)

// Selective re-exports for clean namespace
pub use core::{ errors, types, telemetry };
pub use infrastructure::container::DomainContainer;
pub use web::create_router;
