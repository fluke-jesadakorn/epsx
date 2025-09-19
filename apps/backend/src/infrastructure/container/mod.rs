// Unified Container Module
// Dependency injection containers for clean architecture
// Phase 3 Refactoring: Consolidating into ServiceContainer

// NEW UNIFIED CONTAINER (Phase 3 Refactoring)
pub mod service_container;

// LEGACY CONTAINERS (to be phased out)
pub mod ddd_container;
pub mod app_container;

// Exports - prioritize new unified container
pub use service_container::*;
pub use ddd_container::*;
pub use app_container::*;