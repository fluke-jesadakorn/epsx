// DDD Container Module
// Dependency injection container for the new DDD architecture
// Follows hexagonal architecture principles with clean port/adapter separation

pub mod ddd_container;
pub mod app_container;

pub use ddd_container::*;
pub use app_container::*;