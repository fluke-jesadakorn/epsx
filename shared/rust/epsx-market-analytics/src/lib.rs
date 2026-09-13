//! `epsx-market-analytics` — market analytics domain (big-bang scaffold).
//!
//! BIG-BANG Phase C: This crate is the future home for the market analytics
//! bounded context currently in `apps/backend/src/domain/market_analytics/`.
//! On the single branch `migration/dioxus-microservices`, `apps/analytics`
//! still depends on `epsx` (the monolith) for `EPSRankingService`,
//! `MarketRankingsProviderPort`, `TradingView` adapters, etc.
//! Next phase will move those types here so `apps/analytics` can depend only
//! on `epsx-market-analytics` + `epsx-analytics-protocol` and drop `epsx`.
//!
//! For now this crate re-exports the protocol and provides a placeholder
//! so the workspace builds. Follow `docs/plans/2026-08-23-epsx-bigbang-sqlx.md` §6.
//!
//! TODO(bigbang-next):
//!   - Move `domain/market_analytics/{aggregates,value_objects,repository_ports,services}`
//!     from `apps/backend` to this crate
//!   - Move `infrastructure/adapters/services/tradingview/*` + `cache` adapters
//!     to this crate or a separate `epsx-market-analytics-infra` crate
//!   - Update `apps/analytics/Cargo.toml` to replace `epsx` with `epsx-market-analytics`

pub use epsx_analytics_protocol::{RankingOffsetChange, WalletRankingOffsetQuery};

/// Placeholder — real `EPSRanking` will be moved from `apps/backend/src/domain/market_analytics/`
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MarketAnalyticsPlaceholder {
    pub note: String,
}

impl Default for MarketAnalyticsPlaceholder {
    fn default() -> Self {
        Self {
            note: "big-bang scaffold: market analytics domain will be moved here".to_string(),
        }
    }
}
