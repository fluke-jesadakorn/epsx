// Container Module - Focused modules replacing the God Object pattern
// Follows Single Responsibility Principle with clear separation of concerns

pub mod database_module;
pub mod services_module;
pub mod cache_module;
pub mod container_builder;

// Re-export the focused modules for easy access
pub use database_module::DatabaseModule;
pub use services_module::{ServicesModule, PermissionSystems};
pub use cache_module::CacheModule;
pub use container_builder::{AppContainer, AppContainerBuilder};