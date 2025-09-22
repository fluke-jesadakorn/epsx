// Infrastructure Layer
// Implements ports defined in domain layer with concrete adapters

pub mod adapters;
pub mod event_bus;
pub mod container;
pub mod integration;
pub mod cache;
pub mod models;
pub mod oidc;
pub mod security;
pub mod config;

// Re-export infrastructure components with explicit imports to avoid conflicts
pub use adapters::{
    repositories, services, cache as adapter_cache
};

// Re-export commonly needed services for backward compatibility
pub use event_bus::{SimpleEventBus};
pub use container::{AppContainer, DDDContainer};
pub use integration::{
    AuthenticationServiceIntegration, PaymentServiceIntegration, 
    RealtimeEventsServiceIntegration
};
pub use cache::{
    MemoryCache, RedisCache, UnifiedCache, AffiliateCache, 
    PlanCache, PromotionCache, PermissionCache
};
pub use models::{MarketingModels};
pub use oidc::granular_service as GranularService;
pub use security::{
    key_management as KeyManagement, threat_detection as ThreatDetection
};
pub use config::{FeatureFlags};

// Additional exports for backward compatibility
pub use container::{AppContainer as InfraFactory};
