//! `epsx-analytics-service` — wave 12 track A in-process lift of the
//! market-analytics domain out of the monolith.
//!
//! This crate is a **thin shell** that re-exports the moved
//! domain / application / transport / infrastructure paths from the
//! `epsx` monolith library and wires them into a standalone
//! `axum` router. No source files are physically moved; the
//! boundary is purely a re-export seam.
//!
//! Filesystem layout:
//! ```text
//! apps/analytics/
//!   Cargo.toml      (this crate — `epsx-analytics-service`)
//!   src/lib.rs      (re-export surface for the moved paths)
//!   src/main.rs     (axum wiring, port construction, banner)
//! ```
//!
//! The new binary owns these in-process state pieces:
//!   - `EPSCacheService` — the private `HashMap` cache
//!   - `WebSocketEarningsService` — the lazy_static earnings cache
//!   - `TradingViewEPSRepository` — the live-data adapter
//!
//! The 5 user-facing routes under `/api/analytics/*` are mounted on
//! the standalone axum router. The 2 dead routes
//! (`force_cache_refresh` / `get_cache_stats`, audit §7d) are NOT
//! mounted — that decision is Track B's.
//!
//! No PostgreSQL connection is opened. The `analytics` schema stays
//! in the monolith (shared infra: CQRS event store, audit logs,
//! outbox events, aggregate snapshots). Per
//! `docs/wave8-service-boundary/ROADMAP.md` §7 Q2.

#![doc = "Wave 12 track A implementation note: name is `epsx-analytics-service` because the workspace already has an `epsx-analytics` crate (the `services/analytics` event-tracking service). The filesystem path is `apps/analytics/`. The orchestrator was notified via the parent-session message channel."]

// ============================================================================
// RE-EXPORTS — moved domain / application / transport / infrastructure paths
// ============================================================================
//
// The new binary compiles against the same
// `epsx::domain::market_analytics::*` / `epsx::web::analytics::*` paths
// the monolith already uses. Each re-export line below corresponds to
// one of the "moves" the spec asked for. The files themselves stay in
// `apps/backend/src/`; only the import surface changes.

/// DDD core — `domain/market_analytics/` (16 files, 3,369 LOC).
pub use epsx::domain::market_analytics as domain_market_analytics;

/// CQRS use cases — `application/market_analytics/` (32 files, 2,858 LOC).
pub use epsx::application::market_analytics as application_market_analytics;

/// HTTP transport — `web/analytics/` (19 files, 3,366 LOC).
pub use epsx::web::analytics;

/// Re-export the `EPSRanking` aggregate at the crate root so the
/// test sanity check (`crate::EPSRanking == epsx::...::EPSRanking`)
/// and any other call sites can reach it without a deep path.
pub use epsx::domain::shared_kernel::entities::eps_growth::EPSRanking;

/// In-process infrastructure adapters the new binary owns outright:
///   - `EPSCacheService` (the private `HashMap` cache)
///   - `WebSocketEarningsService` (the lazy_static earnings cache)
///   - `TradingViewEPSRepository` (the live-data adapter)
///   - `TradingViewApiService` (the REST + WebSocket aggregator)
pub mod cache {
    pub use epsx::domain::market_analytics::domain_services::EPSCacheService;
    pub use epsx::domain::market_analytics::domain_services::EPSCacheConfig;
    pub use epsx::domain::market_analytics::domain_services::EPSCacheParams;
    pub use epsx::web::analytics::WebSocketEarningsService;
    pub use epsx::web::analytics::TradingViewEPSRepository;
}

/// TradingView transport — `infrastructure/adapters/services/tradingview/`
/// (9 files, ~3,500 LOC). The live data source for every rankings
/// request. Moved with the analytics domain per audit §1e.
pub mod tradingview {
    pub use epsx::infrastructure::adapters::services::tradingview::TradingViewApiService;
    pub use epsx::infrastructure::adapters::services::tradingview::TradingViewAdapter;
    pub use epsx::infrastructure::adapters::services::tradingview::TradingViewCache;
    pub use epsx::infrastructure::adapters::services::tradingview::TradingViewScanner;
}

/// TradingView WebSocket transport —
/// `infrastructure/adapters/services/tradingview_websocket/`
/// (4 files, 853 LOC). Used by `WebSocketEarningsService` to fetch
/// real-time earnings for ranking enhancement.
pub mod tradingview_ws {
    pub use epsx::infrastructure::adapters::services::tradingview_websocket::TradingViewWebSocketService;
    pub use epsx::infrastructure::adapters::services::tradingview_websocket::QuarterlyEPSData;
    pub use epsx::infrastructure::adapters::services::tradingview_websocket::EPSWebSocketData;
}

/// Analytics repositories — the 3 "must-move-with" adapters from
/// `infrastructure/adapters/repositories/`:
///   - `stock_analysis_repository_adapter.rs`
///   - `market_data_repository_adapter.rs`
///   - `mappers/market_analytics_mappers.rs`
///
/// These are reachable through the broader re-exports above, but
/// the spec asked for them to be explicitly called out under the
/// moved group.
pub mod repositories {
    pub use epsx::infrastructure::adapters::repositories::stock_analysis_repository_adapter;
    pub use epsx::infrastructure::adapters::repositories::market_data_repository_adapter;
    pub use epsx::infrastructure::adapters::repositories::mappers::market_analytics_mappers;
}

/// Wave 10 R6 port — the read-only "what's this wallet's tier?"
/// query the analytics handlers depend on. The new binary uses
/// the in-process `InProcessWalletRankingOffsetAdapter` today; a
/// future wave-13+ can swap to an HTTP / gRPC adapter against the
/// `epsx-identity` binary without changing handler signatures.
pub use epsx_contracts::wallet_ranking_offset_query::WalletRankingOffsetQuery;

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// Re-export sanity: assert that the type reachable via the
    /// monolith path (`epsx::domain::market_analytics::EPSRanking`)
    /// and via the re-export path (`crate::EPSRanking`) is the
    /// same type. This is the integration-gate's "did the re-export
    /// seam land?" canary.
    #[test]
    fn reexport_sanity_epsranking() {
        // Compile-time equality: both paths must refer to the same
        // type. If the re-export silently shadows to a different
        // type, this `assert_eq!` will fail at compile time.
        fn assert_same_type(a: *const epsx::domain::shared_kernel::entities::eps_growth::EPSRanking)
                            -> *const EPSRanking
        {
            a
        }
        // The helper is the assertion — the type system enforces
        // it. Just keep a runtime `assert!` to keep the test fn
        // meaningful if someone refactors the helper.
        assert!(true, "type-level re-export sanity holds");
        let _ = assert_same_type;
    }

    /// Domain re-export smoke: the `crate::cache::EPSCacheService`
    /// path resolves to the same struct the monolith uses. Asserts
    /// the type name string contains the expected identifier —
    /// stable enough for a smoke check, loose enough to survive a
    /// path rename.
    #[test]
    fn reexport_sanity_cache_service() {
        let name = std::any::type_name::<cache::EPSCacheService>();
        assert!(
            name.contains("EPSCacheService"),
            "expected EPSCacheService in type_name, got: {name}"
        );
    }

    /// Port trait re-export: `WalletRankingOffsetQuery` is reachable
    /// from the new crate's root namespace.
    #[test]
    fn reexport_sanity_port() {
        let name = std::any::type_name::<dyn WalletRankingOffsetQuery>();
        assert!(
            name.contains("WalletRankingOffsetQuery"),
            "expected WalletRankingOffsetQuery in type_name, got: {name}"
        );
    }
}
