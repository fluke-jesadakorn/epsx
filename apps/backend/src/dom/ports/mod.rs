// Domain ports module - abstractions for external dependencies
pub mod cache;
// Removed: notification - will be re-implemented

pub use cache::{DomainCache, DomainCacheError, DomainCacheStats};