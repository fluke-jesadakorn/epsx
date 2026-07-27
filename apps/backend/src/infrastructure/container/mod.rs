// Unified Container Module following DDD and Clean Architecture
// Single unified dependency injection container for domain-driven design

// SIMPLE CONTAINER (Legacy stateful architecture)
pub mod simple_container;

// STATELESS SERVICE FACTORY (New serverless architecture)
pub mod stateless_service_factory;

// Exports - minimal container for compilation
pub use simple_container::{DomainContainer, SimpleContainer};

// New serverless exports
pub use stateless_service_factory::{
    HealthServices, RequestServices, ServiceFactory, StatelessConfig, StatelessHealthStatus,
    StatelessServiceFactory,
};
