// Re-export the existing InMemoryEventBus from domain_event module
// This module exists to provide a consistent import path for the event bus
pub use crate::domain::shared_kernel::domain_event::InMemoryEventBus;
