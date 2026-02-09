// Infrastructure Layer
// Implements ports defined in domain layer with concrete adapters

pub mod adapters;
pub mod event_bus;
pub mod container;
pub mod cache;
pub mod models; // Re-added - contains Diesel database models
pub mod security;
pub mod config;
pub mod database;
pub mod cqrs; // NEW: Event sourcing and CQRS infrastructure
pub mod repositories; // NEW: DDD repositories
pub mod redis; // Redis connection pool for notification pub/sub
pub mod blockchain; // Blockchain payment infrastructure
pub mod services; // Background services

// Re-export infrastructure components with explicit imports to avoid conflicts
pub use adapters::{
    repositories as adapter_repositories, services as adapter_services
};

// Re-export commonly needed services for backward compatibility
pub use event_bus::{SimpleEventBus};
pub use container::DomainContainer;
pub use cache::{
    MemoryCache, RedisCache, UnifiedPermissionCache
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
pub use blockchain::{
    BscEventListener, PaymentEvent, PaymentVerifier
};
pub use services::BlockchainMonitor;

