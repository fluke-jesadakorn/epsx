// Infrastructure Layer
// Implements ports defined in domain layer with concrete adapters

pub mod adapters;
pub mod blockchain; // Blockchain payment infrastructure
pub mod cache;
pub mod container;
pub mod cqrs; // NEW: Event sourcing and CQRS infrastructure
pub mod database;
pub mod event_bus;
pub mod logger; // Environment-specific logger configuration
pub mod models; // Re-added - contains Diesel database models
pub mod redis; // Redis connection pool for notification pub/sub
pub mod repositories; // NEW: DDD repositories
pub mod security;
pub mod services; // Background services
pub mod storage; // S3-compatible object storage (MinIO)

// Re-export infrastructure components with explicit imports to avoid conflicts
pub use adapters::{repositories as adapter_repositories, services as adapter_services};

// Re-export commonly needed services for backward compatibility
pub use blockchain::{BscEventListener, PaymentEvent, PaymentVerifier};
pub use cache::{MemoryCache, RedisCache, UnifiedPermissionCache};
pub use container::DomainContainer;
pub use cqrs::{
    EventDispatcher, EventDispatcherConfig, EventStore, PostgresEventStore, ProjectionManager,
    TransactionalOutbox, WalletReadModelProjection,
};
pub use event_bus::SimpleEventBus;
pub use security::threat_detection as ThreatDetection;
pub use services::BlockchainMonitor;
