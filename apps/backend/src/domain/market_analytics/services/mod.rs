// EPS Ranking Service — moved here from
// apps/backend/src/domain/shared_kernel/services/eps_ranking_service.rs
// as part of wave 9 (kernel extraction R5).
//
// The 8 call sites that used to import this service from
// `crate::domain::shared_kernel::services::eps_ranking_service` are now
// expected to import from `crate::domain::market_analytics::services::eps_ranking_service`.

pub mod eps_ranking_service;

pub use eps_ranking_service::{EPSRankingService, EPSRankingParams, EPSRepository, CountryValidator, PermissionParser};
