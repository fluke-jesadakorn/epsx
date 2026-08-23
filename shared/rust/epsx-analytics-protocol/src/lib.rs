//! `epsx-analytics-protocol` — lightweight analytics DTOs + `WalletRankingOffsetQuery` trait.
//!
//! BIG-BANG Phase 3: Extracted from `apps/backend` so `apps/analytics` no longer
//! depends on the whole `epsx` monolith (`apps/analytics/Cargo.toml:21`).
//! Re-exports the canonical `WalletRankingOffsetQuery` from `epsx-contracts`
//! and adds analytics-specific DTOs that were previously inline in the monolith.

use serde::{Deserialize, Serialize};

pub use epsx_contracts::value_objects::ranking_offset::RankingOffset;
pub use epsx_contracts::wallet_ranking_offset_query::WalletRankingOffsetQuery;

/// Request for `GetWalletRankingOffset`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetWalletRankingOffsetRequest {
    pub wallet: String,
}

/// Response for `GetWalletRankingOffset`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetWalletRankingOffsetResponse {
    pub wallet: String,
    pub offset: i32,
}

/// SSE payload `RankingOffsetChange` — same shape as `shared/proto/identity.proto`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RankingOffsetChange {
    pub wallet: String,
    pub offset: i32,
    pub changed_at_ms: i64,
}

/// Analytics ranking query (mirrors `GET /api/analytics/rankings`)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsRankingsQuery {
    pub page: Option<u32>,
    pub limit: Option<u32>,
    pub country: Option<String>,
    pub sector: Option<String>,
    pub sort_by: Option<String>,
    pub min_eps: Option<f64>,
    pub min_growth: Option<f64>,
}

/// Lightweight analytics event (POST /api/analytics/track)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackEventRequest {
    pub event: String,
    pub properties: Option<serde_json::Value>,
    pub chain_id: Option<String>,
}

/// Health/status for analytics service
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsHealth {
    pub status: String,
    pub version: String,
}
