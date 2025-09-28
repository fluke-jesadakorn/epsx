// Infrastructure Layer
// Implements ports defined in domain layer with concrete adapters

pub mod adapters;
pub mod event_bus;
pub mod container;
pub mod integration;
pub mod cache;
pub mod models;
pub mod security;
pub mod config;
pub mod database;

// Re-export infrastructure components with explicit imports to avoid conflicts
pub use adapters::{
    repositories, services, cache as adapter_cache
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

