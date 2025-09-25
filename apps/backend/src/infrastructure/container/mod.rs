// Unified Container Module following DDD and Clean Architecture
// Single unified dependency injection container for domain-driven design

// SIMPLE CONTAINER (Temporary replacement)
pub mod simple_container;

// Exports - minimal container for compilation
pub use simple_container::{
    SimpleContainer,
    DomainContainer,
};