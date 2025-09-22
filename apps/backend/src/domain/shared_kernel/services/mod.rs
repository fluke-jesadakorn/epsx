// Shared Kernel Services - Web3-first architecture

pub mod eps_ranking_service;
pub mod eps_cache_service;

// Re-export the working implementations
pub use eps_ranking_service::{EPSRankingService, EPSRankingParams, EPSRepository};
pub use eps_cache_service::{EPSCacheService, EPSCacheConfig};