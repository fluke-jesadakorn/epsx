// Shared Kernel Services - Web3-first architecture

pub mod eps_ranking_service;

// Re-export the working implementations
pub use eps_ranking_service::{EPSRankingService, EPSRankingParams, EPSRepository};