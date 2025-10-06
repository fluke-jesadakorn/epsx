// Infrastructure Layer
// Implements ports defined in domain layer with concrete adapters

pub mod adapters;
pub mod event_bus;
pub mod container;
// pub mod integration; // Removed - empty module with only commented-out payment service
pub mod cache;
// pub mod models; // Removed - was empty
pub mod security;
pub mod config;
pub mod database;
pub mod cqrs; // NEW: Event sourcing and CQRS infrastructure

// Re-export infrastructure components with explicit imports to avoid conflicts
pub use adapters::{
    repositories, services
};

// Re-export commonly needed services for backward compatibility
pub use event_bus::{SimpleEventBus};
pub use container::DomainContainer;
// pub use integration::{ PaymentServiceIntegration }; // Temporarily disabled
pub use cache::{
    MemoryCache, RedisCache, PermissionCache
};
pub use security::{
    key_management as KeyManagement, threat_detection as ThreatDetection
};
pub use config::{FeatureFlags};
pub use cqrs::{
    EventStore, PostgresEventStore, TransactionalOutbox,
    EventDispatcher, EventDispatcherConfig,
    ProjectionManager, WalletReadModelProjection
};

