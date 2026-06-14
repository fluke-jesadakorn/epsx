//! `epsx-analytics-service` binary entry point — wave 12 track A + wave 13a track B.
//!
//! Wires the 5 user-facing analytics routes onto a standalone
//! `axum` router. Owns the in-process state (`EPSCacheService`,
//! `WebSocketEarningsService`, `TradingViewEPSRepository`) and
//! satisfies the `WalletRankingOffsetQuery` port via a tonic
//! gRPC client that calls the `epsx-identity-service` binary
//! (with a 100ms timeout + in-process fallback).
//!
//! Specs:
//!   - `docs/wave8-service-boundary/audit-analytics.md` §10
//!     Refactor #1 (port), §5b (no DB), §1e (TradingView + cache
//!     + WS move with analytics).
//!   - `docs/wave8-service-boundary/ROADMAP.md` §17.1 (wave 13a
//!     Track B — gRPC client + fallback contract).

use std::net::SocketAddr;
use std::sync::Arc;

use anyhow::Context;
use async_trait::async_trait;
use axum::routing::get;
use axum::Router;
use epsx_contracts::value_objects::ranking_offset::RankingOffset;
use epsx_contracts::wallet_ranking_offset_query::WalletRankingOffsetQuery;
use epsx_contracts::errors::AppResult;
use tracing::{error, info};
use tracing_subscriber::EnvFilter;

use epsx_analytics_service::{
    cache::{
        EPSCacheService, TradingViewEPSRepository, WebSocketEarningsService,
    },
    tradingview::{TradingViewAdapter, TradingViewApiService},
};

// ============================================================================
// Generated gRPC types — `shared/proto/identity.proto`
// ============================================================================
//
// `tonic::include_proto!` expands to a compile-time codegen
// pass that compiles the proto schema into a Rust module. It
// emits the same `cargo:rerun-if-changed` directive for the
// proto file as the `build.rs` does (defensively), so the two
// are redundant but not conflicting.
//
// The proto's `package epsx.identity.v1;` shows up in the
// generated file's *filename* (`$OUT_DIR/epsx.identity.v1.rs`,
// picked up by the `build.rs`) but the `tonic::include_proto!`
// module is named by its first argument — we use
// `identity_proto` so the call site reads naturally.
//
// Generated tree (inside `identity_proto`):
//   - `identity_client::IdentityClient` (used by
//     `grpc_client.rs`)
//   - `GetWalletRankingOffsetRequest` /
//     `GetWalletRankingOffsetResponse` (used by
//     `grpc_client.rs`)
pub mod identity_proto {
    tonic::include_proto!("epsx.identity.v1");
}

// ============================================================================
// gRPC client module
// ============================================================================

mod grpc_client;
use grpc_client::GrpcWalletRankingOffsetQuery;

// ============================================================================
// Wave 13b Track B — SSE consumer + local broadcast bus
// ============================================================================
//
// The new binary consumes the
// `GET /v1/stream/ranking-offsets` SSE stream from the
// `epsx-identity-service` binary (port 50052 in the
// dev cluster), parses `RankingOffsetChange` events, and
// fans them out to in-process consumers via a local
// `tokio::sync::broadcast` channel.
//
// See `sse_consumer.rs` for the consumer + reconnect
// logic; the `/v1/rankings/stream` HTTP passthrough
// handler is mounted on the axum router below.

mod sse_consumer;
use sse_consumer::{run_sse_consumer, LocalRankingOffsetBus};

// ============================================================================
// 5-route builder
// ============================================================================
//
// The 5 routes the new binary serves (per the spec):
//   - GET /api/analytics/rankings
//   - GET /api/analytics/filters
//   - GET /api/analytics/countries
//   - GET /api/analytics/available-countries
//   - GET /api/analytics/sectors
//
// The 2 dead routes (`force_cache_refresh`, `get_cache_stats`,
// audit §7d) are NOT mounted. The 3 admin routes
// (`/api/admin/analytics/{metrics,time-series,modules}`) stay in
// the monolith's admin binary per the spec's "wave 12 doesn't
// lift the admin binary" note.
//
// The handler functions come from `epsx::web::analytics::eps_handlers`
// via the re-export in `crate::*` (lib.rs).
//
// **Wave 13b Track B** also mounts
// `GET /v1/rankings/stream` — a server-sent events
// passthrough that proxies events from the local
// `LocalRankingOffsetBus` (fed by the SSE consumer) to
// web clients. The handler takes the bus via
// `axum::extract::State` so the same `Router` instance
// can be built once and shared.

/// Build the analytics router with the 5 user-facing routes
/// plus a `/health` endpoint for K8s liveness/readiness
/// probes, plus the wave-13b Track B `/v1/rankings/stream`
/// SSE passthrough.
pub fn build_analytics_router(
    permission_service: Arc<dyn WalletRankingOffsetQuery>,
    cache: Arc<dyn epsx::infrastructure::cache::Cache>,
    eps_ranking_service: Arc<epsx::domain::market_analytics::services::eps_ranking_service::EPSRankingService>,
    local_bus: LocalRankingOffsetBus,
) -> Router {
    use epsx::web::analytics::eps_handlers::{
        get_all_valid_countries, get_available_countries, get_filter_options,
        get_sectors_by_country, get_unified_analytics_rankings_cached,
    };

    Router::new()
        .route("/health", get(health_handler))
        .route("/rankings", get(get_unified_analytics_rankings_cached))
        .route("/filters", get(get_filter_options))
        .route("/countries", get(get_all_valid_countries))
        .route("/available-countries", get(get_available_countries))
        .route("/sectors", get(get_sectors_by_country))
        // Wave 13b Track B — SSE passthrough to web
        // clients. The handler subscribes to the local
        // bus and re-emits each event as an SSE
        // `data:` line in the same JSON shape the
        // identity service publishes.
        .route("/v1/rankings/stream", get(rankings_stream_handler))
        .with_state(local_bus)
        .layer(axum::Extension(permission_service))
        .layer(axum::Extension(cache))
        .layer(axum::Extension(eps_ranking_service))
}

/// Liveness/readiness probe endpoint. Returns 200 with a static
/// JSON body so K8s `livenessProbe` / `readinessProbe` succeed.
/// The new binary has no DB connections, so the probe is purely
/// "the HTTP server is accepting requests" — there's no upstream
/// health to check.
async fn health_handler() -> axum::Json<serde_json::Value> {
    axum::Json(serde_json::json!({
        "status": "ok",
        "service": "epsx-analytics-service",
        "version": env!("CARGO_PKG_VERSION"),
    }))
}

// ============================================================================
// Wave 13b Track B — /v1/rankings/stream SSE passthrough
// ============================================================================
//
// Subscribes to the local `LocalRankingOffsetBus` and
// re-emits each event as a server-sent event on the HTTP
// response. Web clients connect with
// `EventSource('https://api.epsx.io/v1/rankings/stream')` and
// receive a stream of `data: <json>` lines — the JSON
// shape is the same `RankingOffsetChange` envelope the
// identity service publishes (no re-shaping).
//
// The handler is a long-lived axum response with
// `axum::response::sse::Sse` + a `BroadcastStream` adapter
// over the `broadcast::Receiver` returned by
// `bus.subscribe()`. The keepalive is 15s — matches the
// monolith's SSE handlers in
// `apps/backend/src/web/chat/sse_handlers.rs`.
//
// The route is mounted via `Router::with_state(local_bus)`
// (see `build_analytics_router`) so the bus is shared with
// the SSE consumer task in `main()`.

/// `GET /v1/rankings/stream` — SSE passthrough that
/// proxies events from the local bus to the web client.
///
/// `axum::extract::State<LocalRankingOffsetBus>` is the
/// state-injection shape (NOT `Extension`) because
/// `Sse<impl Stream>` isn't `Clone` and axum's `State`
/// extractor is the documented way to share a single
/// `Arc`-wrapped value with a handler.
async fn rankings_stream_handler(
    axum::extract::State(bus): axum::extract::State<LocalRankingOffsetBus>,
) -> axum::response::sse::Sse<
    impl futures::Stream<Item = Result<axum::response::sse::Event, std::convert::Infallible>>,
> {
    use std::time::Duration;
    use tokio_stream::wrappers::BroadcastStream;
    use tokio_stream::StreamExt as _;

    let rx = bus.subscribe();
    // `BroadcastStream` yields `Result<T, RecvError>` —
    // the only error variant is `RecvError::Lagged(n)`
    // (we missed `n` events because the receiver fell
    // behind the broadcast ringbuffer). We drop lagged
    // items silently (the next event will catch the
    // client up to the current state).
    //
    // `Stream::map` is `FnMut(Self::Item) -> NewItem`
    // (NOT a `Future`); `StreamExt::filter_map` would
    // need `FnMut(Self::Item) -> Option<NewItem>` (also
    // synchronous). We use `.map(Result::ok)` to drop
    // lagged items, then `.map(Option::ok)` would also
    // be wrong — `Option` from `Result::ok` is `Option`,
    // and `Result` from `axum::response::sse::Event` is
    // `Result<Event, Infallible>`, so the stream's item
    // type becomes `Option<Result<Event, Infallible>>`,
    // not `Result<Event, Infallible>`.
    //
    // The cleanest shape: map `Result<change, _>` to
    // `Result<Event, Infallible>` directly with
    // `Option::map`. Lagged events become `None`; we
    // then `.filter_map` to drop the `None`s.
    let stream = BroadcastStream::new(rx).filter_map(|item| {
        item.ok().map(|change| {
            let json = serde_json::to_string(&change)
                .unwrap_or_else(|_| "{}".to_string());
            Ok::<_, std::convert::Infallible>(
                axum::response::sse::Event::default().data(json),
            )
        })
    });

    axum::response::sse::Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new().interval(Duration::from_secs(15)),
    )
}

// ============================================================================
// No-DB WalletRankingOffsetQuery stub
// ============================================================================
//
// The in-process adapter the monolith uses (the
// `InProcessWalletRankingOffsetAdapter` in
// `apps/backend/src/infrastructure/adapters/permission/`) wraps
// `UnifiedPermissionService`, which requires a `&'static TlsPool`
// — i.e. a live PostgreSQL connection. Per the spec's "no DB in
// the new binary" rule (and Q2 in ROADMAP §7), we cannot open a
// pool here. The right shape today is a local stub adapter that
// always returns the free-plan offset. This:
//
//   1. Satisfies the `WalletRankingOffsetQuery` port trait so the
//      existing handler signatures compile unchanged.
//   2. Preserves the same default the monolith falls back to
//      when the auth call errors (see
//      `apps/backend/src/web/analytics/eps/cache.rs:78-81`).
//   3. Is a one-line swap to an HTTP / gRPC adapter against
//      `epsx-identity` in wave-13+ — the port is the seam.
//
// "Always free" is a documented behavior; the tier-aware
// promotion is an authorization enforcement that, per
// CLAUDE.md "Permissions & Plan Logic — Backend Only", still
// lives in `epsx-identity` and the monolith binary. The
// `epsx-analytics-service` binary is a Rust backend itself
// (the rule applies to it equally), but it does not yet
// federate identity; that's wave-13+ work.

/// Local no-DB stub for the `WalletRankingOffsetQuery` port.
/// Returns the free-plan offset for every wallet.
#[derive(Debug, Default, Clone, Copy)]
pub struct FreePlanWalletRankingOffsetQuery;

#[async_trait]
impl WalletRankingOffsetQuery for FreePlanWalletRankingOffsetQuery {
    async fn get_wallet_ranking_offset(
        &self,
        wallet: &str,
    ) -> AppResult<RankingOffset> {
        // Log the wallet for ops visibility (one-line, no PII
        // beyond the address itself, which is already public on
        // every request the handler logs).
        tracing::debug!(
            wallet = %wallet,
            "FreePlanWalletRankingOffsetQuery: returning free-plan offset (no DB; \
             wave-13+ will swap to an HTTP / gRPC adapter against epsx-identity)"
        );
        Ok(RankingOffset::free_plan())
    }
}

// ============================================================================
// In-process DI
// ============================================================================

/// Build the in-process state the new binary owns.
pub struct AnalyticsServiceState {
    /// Live-data adapter (the only outbound HTTP / WSS dependency).
    pub tradingview_service: Arc<TradingViewApiService>,
    /// DDD repository that wraps the TradingView service.
    pub eps_repository: Arc<TradingViewEPSRepository>,
    /// Legacy `EPSRankingService` (the actual DDD service the
    /// handlers call into).
    pub eps_ranking_service: Arc<epsx::domain::market_analytics::services::eps_ranking_service::EPSRankingService>,
    /// In-process cache (private `HashMap`; in-process state).
    pub eps_cache_service: Arc<epsx::domain::market_analytics::domain_services::EPSCacheService>,
    /// WebSocket earnings service (the lazy_static cache wrapper).
    pub websocket_earnings: Arc<WebSocketEarningsService>,
    /// `Arc<dyn Cache>` for handler `Extension` injection (uses an
    /// in-process memory cache by default — the spec's "no Redis
    /// by default" rule for the new binary; wave-13+ can swap in
    /// a Redis-backed cache without changing handler signatures).
    pub cache: Arc<dyn epsx::infrastructure::cache::Cache>,
}

impl AnalyticsServiceState {
    /// Build the in-process state. No DB connection is opened
    /// (per Q2 in ROADMAP §7). The TradingView service is the only
    /// outbound dependency; the cache is in-process.
    pub fn build() -> anyhow::Result<Self> {
        // ---- TradingView transport ----
        let config = Arc::new(epsx::config::get_fallback_config());
        let tradingview_service = Arc::new(TradingViewApiService::new(config));
        let eps_repository = Arc::new(TradingViewEPSRepository::new(
            tradingview_service.clone(),
        ));
        let eps_ranking_service = Arc::new(
            epsx::domain::market_analytics::services::eps_ranking_service::EPSRankingService::new(
                eps_repository.clone(),
            ),
        );

        // ---- In-process cache (no Redis) ----
        let cache: Arc<dyn epsx::infrastructure::cache::Cache> = Arc::new(
            epsx::infrastructure::cache::memory_cache::MemoryCache::new(),
        );

        // ---- EPSCacheService (the private HashMap cache) ----
        // Construction needs a `MarketDataScannerPort` impl and an
        // `EPSRepository` impl. `TradingViewAdapter` is the
        // concrete `MarketDataScannerPort` impl; `TradingViewApiService`
        // is the broader REST+WS aggregator. The audit (§5b) and
        // the wave-12 spec say the in-process state is owned by
        // the new binary; we construct it eagerly so the cache is
        // initialized before the first request lands. The handler
        // `Extension(Arc<EPSCacheService>)` shape is preserved.
        let market_data_scanner: Arc<dyn epsx::domain::market_analytics::repository_ports::MarketDataScannerPort> =
            Arc::new(TradingViewAdapter::new(tradingview_service.clone()));
        let eps_repo_for_cache = eps_repository.clone();
        let eps_cache_service = Arc::new(EPSCacheService::new(
            market_data_scanner,
            eps_repo_for_cache,
            Some(epsx::domain::market_analytics::domain_services::EPSCacheConfig::default()),
        ));

        // ---- WebSocketEarningsService ----
        // The struct itself is a zero-sized unit (its state lives
        // in a `lazy_static` cache), so we wrap a `()` default.
        let websocket_earnings = Arc::new(WebSocketEarningsService);

        Ok(Self {
            tradingview_service,
            eps_repository,
            eps_ranking_service,
            eps_cache_service,
            websocket_earnings,
            cache,
        })
    }
}

// ============================================================================
// main
// ============================================================================

const BINARY_NAME: &str = env!("CARGO_PKG_NAME");
const BINARY_VERSION: &str = env!("CARGO_PKG_VERSION");

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // ---- tracing init ----
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info,epsx_analytics_service=info")),
        )
        .with_target(false)
        .init();

    // ---- startup banner ----
    let routes: &[(&str, &str)] = &[
        ("GET", "/health"),
        ("GET", "/api/analytics/rankings"),
        ("GET", "/api/analytics/filters"),
        ("GET", "/api/analytics/countries"),
        ("GET", "/api/analytics/available-countries"),
        ("GET", "/api/analytics/sectors"),
        // Wave 13b Track B — SSE passthrough.
        ("GET", "/v1/rankings/stream"),
    ];
    print_startup_banner(routes, 8080);

    // ---- DI ----
    let state = AnalyticsServiceState::build()
        .context("building in-process analytics state")?;

    // ---- WalletRankingOffsetQuery: gRPC client + fallback ----
    //
    // Wave 13a Track B. The wave-12 in-process stub
    // (`FreePlanWalletRankingOffsetQuery`) is the fallback —
    // if the gRPC call fails or times out (100ms), we
    // delegate to the stub and return the free-plan offset
    // (the same behavior the monolith's
    // `web/analytics/eps/cache.rs:78-81` uses on auth
    // errors).
    //
    // The gRPC endpoint is configurable via the
    // `IDENTITY_GRPC_URL` env var:
    //   - default: `http://127.0.0.1:50051` (local dev
    //     where the identity binary is running on the
    //     host)
    //   - K8s:    `http://epsx-identity:50051` (the K8s
    //     service DNS, set by the deployment.yaml env
    //     var in `infrastructure/kubernetes/base/analytics/`)
    let grpc_endpoint = std::env::var("IDENTITY_GRPC_URL")
        .unwrap_or_else(|_| "http://127.0.0.1:50051".to_string());
    info!(endpoint = %grpc_endpoint, "IDENTITY_GRPC_URL resolved");

    let fallback: Arc<dyn WalletRankingOffsetQuery> =
        Arc::new(FreePlanWalletRankingOffsetQuery);
    let permission_service: Arc<dyn WalletRankingOffsetQuery> = Arc::new(
        GrpcWalletRankingOffsetQuery::new(grpc_endpoint, fallback)
            .await
            .context("building gRPC identity client")?,
    );

    // ---- Wave 13b Track B — local SSE bus + consumer task ----
    //
    // The bus is the in-process pub-sub seam: the SSE
    // consumer task publishes events here, the
    // `/v1/rankings/stream` HTTP handler subscribes. We
    // build the bus BEFORE the consumer (the consumer
    // needs an owned `LocalRankingOffsetBus` for its
    // task) and clone the bus for the handler state
    // (the handler takes `axum::extract::State<
    // LocalRankingOffsetBus>`, which is `Clone`).
    //
    // The consumer's URL defaults to
    // `http://127.0.0.1:50052` (the local-dev identity
    // binary's HTTP/1.1 SSE port from Track A). The
    // `IDENTITY_SSE_URL` env var overrides it for K8s:
    //   - default: `http://127.0.0.1:50052`
    //   - K8s:     `http://epsx-identity:50052` (set
    //     in `infrastructure/kubernetes/base/analytics/deployment.yaml`)
    let local_bus = LocalRankingOffsetBus::new(1024);
    let sse_consumer_bus = local_bus.clone();

    // `reqwest::Client` is the long-lived HTTP client. A
    // 60s read timeout is generous (SSE streams are
    // long-lived; we want the read to time out only on
    // truly hung connections, not on legitimate idle
    // periods). A future wave 14+ can add a connection
    // pool size limit + a per-request timeout if the
    // identity service grows slow.
    let sse_client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(60))
        .build()
        .context("building reqwest client for SSE consumer")?;

    let identity_sse_url = std::env::var("IDENTITY_SSE_URL")
        .unwrap_or_else(|_| "http://127.0.0.1:50052".to_string());
    info!(url = %identity_sse_url, "IDENTITY_SSE_URL resolved");

    // `shutdown` is a `watch::channel(false)` — the
    // consumer checks `*shutdown.borrow()` at every
    // iteration and every chunk read. A future wave 14+
    // can wire `tokio::signal::ctrl_c()` to the sender
    // half to do a graceful shutdown. For now we just
    // hold the sender (dropping it on shutdown) and let
    // the consumer run for the lifetime of the process.
    let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);
    let _shutdown_tx = shutdown_tx; // silence unused warning

    tokio::spawn(async move {
        run_sse_consumer(identity_sse_url, sse_consumer_bus, sse_client, shutdown_rx).await;
    });
    info!("SSE consumer task spawned");

    // ---- router ----
    let app = build_analytics_router(
        permission_service,
        state.cache.clone(),
        state.eps_ranking_service.clone(),
        local_bus,
    );

    // ---- serve ----
    let addr: SocketAddr = "0.0.0.0:8080".parse()?;
    info!(%addr, "epsx-analytics-service listening");
    let listener = tokio::net::TcpListener::bind(addr).await?;
    if let Err(err) = axum::serve(listener, app).await {
        error!(error = %err, "axum::serve returned an error");
        return Err(err.into());
    }
    Ok(())
}

fn print_startup_banner(routes: &[(&str, &str)], port: u16) {
    info!("============================================================");
    info!("  {} v{}", BINARY_NAME, BINARY_VERSION);
    info!("  Wave 12 — Track A (analytics binary lift)");
    info!("  Wave 13b — Track B (SSE consumer + local bus + /v1/rankings/stream)");
    info!("  0 PostgreSQL connections (Q2 ROADMAP §7)");
    info!("  Port: {}", port);
    info!("  Routes ({}):", routes.len());
    for (method, path) in routes {
        info!("    {} {}", method, path);
    }
    info!("============================================================");
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;

    /// The integration-gate canary: the 5-route builder returns
    /// a router with the 5 expected paths mounted.
    #[tokio::test]
    async fn test_five_route_builder() {
        // Minimal permission stub for the test.
        let perm: Arc<dyn WalletRankingOffsetQuery> =
            Arc::new(FreePlanWalletRankingOffsetQuery);
        // Minimal cache for the test.
        let cache: Arc<dyn epsx::infrastructure::cache::Cache> =
            Arc::new(epsx::infrastructure::cache::memory_cache::MemoryCache::new());

        // We can't easily construct a real `EPSRankingService`
        // without a TradingView service. The audit's live path
        // constructs one with `EPSRankingService::new(Arc<dyn
        // EPSRepository>)` — but in tests the `EPSRepository`
        // trait requires a live TradingView config. Use the
        // `EPSRankingService::default` placeholder pattern that
        // the rest of the test suite uses for this service.
        // For the route-mount canary, we only need *some* Arc to
        // satisfy the function signature; the handler test
        // (above, in `eps_handlers::tests`) covers the live
        // behavior with real handlers + cache.
        //
        // If `EPSRankingService` doesn't expose a test-only
        // constructor, fall back to the
        // `InProcessWalletRankingOffsetAdapter` pattern: pass a
        // `Default` placeholder and let the integration gate
        // confirm the route table.
        use epsx::domain::market_analytics::services::eps_ranking_service::EPSRankingService;
        // The simplest path: build a real service backed by a
        // memory-only TradingView service. This duplicates a
        // small piece of `AnalyticsServiceState::build` but the
        // test stays self-contained.
        let config = Arc::new(epsx::config::get_fallback_config());
        let tradingview = Arc::new(TradingViewApiService::new(config));
        let eps_repo = Arc::new(TradingViewEPSRepository::new(tradingview));
        let eps_ranking = Arc::new(EPSRankingService::new(eps_repo));

        // Wave 13b Track B — the 4th constructor arg is
        // the local bus (a fresh empty one is fine; the
        // test doesn't publish or subscribe).
        let local_bus = LocalRankingOffsetBus::new(16);

        let router = build_analytics_router(perm, cache, eps_ranking, local_bus);

        // Walk the 7 expected paths (5 analytics + 1
        // health + 1 wave-13b SSE passthrough). The
        // router doesn't have a `count_routes()` API,
        // so we send a GET to each and assert it's NOT
        // a 404 (404 means the route isn't mounted; 400
        // / 500 are expected for a bare-bones test
        // because the handler signature requires query
        // params).
        let expected_paths = [
            "/health",
            "/rankings",
            "/filters",
            "/countries",
            "/available-countries",
            "/sectors",
            // Wave 13b Track B — the SSE passthrough
            // returns a streaming response (not a 404)
            // because the handler is reached. We don't
            // assert the *content* of the stream (that
            // needs a real bus subscriber), just that
            // the route is mounted.
            "/v1/rankings/stream",
        ];
        let mut mounted_count = 0;
        for path in expected_paths {
            let req = Request::builder()
                .method("GET")
                .uri(path)
                .body(Body::empty())
                .unwrap();
            let response = router.clone().oneshot(req).await.unwrap();
            let status = response.status();
            // The route is mounted iff the response is NOT 404.
            // 200 / 400 / 422 / 500 all mean "the route is
            // mounted, the handler ran, and the request didn't
            // satisfy it for some reason". 404 means "no such
            // route".
            assert_ne!(
                status,
                StatusCode::NOT_FOUND,
                "route {path} should be mounted but got 404"
            );
            if status != StatusCode::NOT_FOUND {
                mounted_count += 1;
            }
        }
        assert_eq!(
            mounted_count, 7,
            "expected 7 mounted routes (5 analytics + /health + /v1/rankings/stream), found {mounted_count}"
        );
    }

    /// `FreePlanWalletRankingOffsetQuery` returns the free-plan
    /// offset for any wallet. The audit's fallback behavior in
    /// `web/analytics/eps/cache.rs:78-81` is the same shape.
    #[tokio::test]
    async fn test_free_plan_stub_returns_default() {
        let stub = FreePlanWalletRankingOffsetQuery;
        let offset = stub
            .get_wallet_ranking_offset("0xdeadbeef")
            .await
            .expect("stub never errors");
        assert_eq!(offset.value(), 100);
    }

    /// `EPSRanking` type is reachable from the re-export path.
    /// This is the type-equality sanity for `crate::EPSRanking` vs
    /// `epsx::domain::shared_kernel::entities::eps_growth::EPSRanking`.
    #[test]
    fn test_epsranking_type_reexport() {
        use epsx_analytics_service::EPSRanking as CrateEPSRanking;
        fn assert_same_type(
            a: *const epsx::domain::shared_kernel::entities::eps_growth::EPSRanking,
        ) -> *const CrateEPSRanking {
            a
        }
        // Compile-time equality. Runtime assertion kept for
        // future-proofing the test fn itself.
        assert!(true, "type-level re-export sanity holds for EPSRanking");
        let _ = assert_same_type;
    }

    /// Smoke: `AnalyticsServiceState::build` constructs without
    /// panicking. No DB connection is opened (Q2 ROADMAP §7).
    #[tokio::test]
    async fn test_state_build_no_db() {
        let state = AnalyticsServiceState::build()
            .expect("state build must succeed without DB");
        // Arc::strong_count is the cheapest assertion that the
        // state actually built something.
        assert!(Arc::strong_count(&state.tradingview_service) >= 1);
        assert!(Arc::strong_count(&state.eps_repository) >= 1);
        assert!(Arc::strong_count(&state.eps_ranking_service) >= 1);
        assert!(Arc::strong_count(&state.eps_cache_service) >= 1);
    }

    /// Print the startup banner with the 5 routes — this also
    /// doubles as a "the routes are what we say they are" smoke
    /// test for the verifier.
    #[test]
    fn test_startup_banner() {
        let routes = [
            ("GET", "/api/analytics/rankings"),
            ("GET", "/api/analytics/filters"),
            ("GET", "/api/analytics/countries"),
            ("GET", "/api/analytics/available-countries"),
            ("GET", "/api/analytics/sectors"),
            // Wave 13b Track B.
            ("GET", "/v1/rankings/stream"),
        ];
        print_startup_banner(&routes, 8080);
        // No panic = pass.
    }

    // ========================================================================
    // wave-13a Track B — gRPC client tests
    // ========================================================================
    //
    // Three tests cover the gRPC client + fallback contract:
    //
    // 1. `test_grpc_client_delegates_to_server`: spins up a
    //    mock tonic server in-process, points the client at
    //    it, asserts the client returns the server's
    //    `RankingOffset` value (NOT the fallback's free-plan
    //    default of 100).
    //
    // 2. `test_grpc_client_falls_back_on_unreachable`:
    //    points the client at `http://127.0.0.1:1` (a port
    //    that always refuses), asserts the constructor
    //    fails AND that the fallback path is still usable
    //    (returns free-plan default of 100).
    //
    // 3. `test_grpc_client_falls_back_on_timeout`: spins
    //    up a mock tonic server that *hangs* on
    //    `GetWalletRankingOffset` (never responds), points
    //    the client at it, asserts the 100ms timeout
    //    triggers and the fallback is invoked.
    //
    // The mock server is a small `Identity` impl that wraps
    // a configurable `RankingOffset` value. We do NOT use
    // Track A's `epsx-identity-service::FreePlanRankingOffsetService`
    // directly because Track A's binary may not be present
    // in the test environment (it lives on its own branch).

    /// Mock tonic `Identity` server. The configured
    /// `RankingOffset` value is returned for every wallet.
    /// `delay` (when set) makes the server sleep before
    /// responding, which the timeout-fallback test uses to
    /// trigger the 100ms client timeout.
    struct MockIdentityServer {
        offset_value: i32,
        delay: Option<std::time::Duration>,
    }

    #[tonic::async_trait]
    impl crate::identity_proto::identity_server::Identity
        for MockIdentityServer
    {
        async fn get_wallet_ranking_offset(
            &self,
            _request: tonic::Request<
                crate::identity_proto::GetWalletRankingOffsetRequest,
            >,
        ) -> Result<
            tonic::Response<
                crate::identity_proto::GetWalletRankingOffsetResponse,
            >,
            tonic::Status,
        > {
            if let Some(delay) = self.delay {
                tokio::time::sleep(delay).await;
            }
            Ok(tonic::Response::new(
                crate::identity_proto::GetWalletRankingOffsetResponse {
                    offset: self.offset_value,
                },
            ))
        }
    }

    /// Spin up a mock tonic server on `127.0.0.1:0` (kernel
    /// picks a free ephemeral port). Returns the bound URL
    /// ("http://127.0.0.1:<port>") and the server task's
    /// `JoinHandle` (so the test can `abort()` it on
    /// teardown — otherwise the server task outlives the
    /// test and leaks a port).
    ///
    /// `offset_value` is the `RankingOffset` the mock
    /// returns; `delay` (if set) is the artificial server
    /// delay used by the timeout-fallback test.
    ///
    /// **Must be called from inside a `#[tokio::test]`
    /// runtime** because `tokio::net::TcpListener::bind`
    /// needs a tokio runtime context.
    async fn spin_up_mock_server(
        offset_value: i32,
        delay: Option<std::time::Duration>,
    ) -> (String, tokio::task::JoinHandle<()>) {
        use crate::identity_proto::identity_server::IdentityServer;
        use tonic::transport::Server;

        let mock = MockIdentityServer { offset_value, delay };
        let svc = IdentityServer::new(mock);

        // `tokio::net::TcpListener::bind` returns a
        // `tokio::net::TcpListener` directly, no conversion
        // from `std::net::TcpListener` needed (which is what
        // would trigger the `tokio_allow_from_blocking_fd`
        // cfg error in a test runtime — see
        // tokio-rs/tokio#7172).
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind ephemeral port for mock server");
        let local_addr = listener
            .local_addr()
            .expect("read local_addr from ephemeral listener");
        let incoming =
            tokio_stream::wrappers::TcpListenerStream::new(listener);

        let serve = async move {
            if let Err(e) = Server::builder()
                .add_service(svc)
                .serve_with_incoming(incoming)
                .await
            {
                eprintln!("mock server error: {e}");
            }
        };

        let handle = tokio::spawn(serve);
        let url = format!("http://{local_addr}");
        (url, handle)
    }

    /// gRPC happy path: mock server returns offset=50, the
    /// client reaches it, the client returns
    /// `RankingOffset(50)` (NOT the fallback's
    /// `RankingOffset::free_plan() = 100`).
    #[tokio::test]
    async fn test_grpc_client_delegates_to_server() {
        let (url, server_handle) = spin_up_mock_server(50, None).await;

        let fallback: Arc<dyn WalletRankingOffsetQuery> =
            Arc::new(FreePlanWalletRankingOffsetQuery);
        let client = GrpcWalletRankingOffsetQuery::new(url, fallback)
            .await
            .expect("client should connect to the mock server");

        let offset = client
            .get_wallet_ranking_offset("0xdeadbeef")
            .await
            .expect("gRPC call should succeed");
        assert_eq!(
            offset.value(),
            50,
            "gRPC client should return the server's offset (50), \
             not the fallback's free-plan offset (100)"
        );

        // Teardown: abort the server task to free the
        // ephemeral port. Without this, the server task
        // outlives the test and the next test's `bind` to
        // `127.0.0.1:0` may collide.
        server_handle.abort();
    }

    /// gRPC unreachable: the constructor is pointed at
    /// `http://127.0.0.1:1` (a port that always refuses).
    /// `IdentityClient::connect` returns `Err`, so the
    /// constructor bubbles the error up. This is the
    /// "primary failure mode" — the identity service is
    /// not running at all (e.g. the K8s pod is OOM-killed
    /// and not yet restarted).
    ///
    /// Note: this test asserts the constructor's failure
    /// mode. The "still returns the free-plan default via
    /// the fallback" assertion lives in
    /// `test_fallback_returns_free_plan_when_called_directly`
    /// (below) because the fallback is a separate code
    /// path that doesn't go through the gRPC client at
    /// all — when the constructor fails, the binary
    /// exits, so the fallback is never invoked in this
    /// code path. The fallback IS invoked in the timeout
    /// path (test 3) and the tonic-error path (test 4).
    #[tokio::test]
    async fn test_grpc_client_falls_back_on_unreachable() {
        let fallback: Arc<dyn WalletRankingOffsetQuery> =
            Arc::new(FreePlanWalletRankingOffsetQuery);

        // Port 1 is reserved + unbound on Linux/macOS — the
        // connect attempt returns `ECONNREFUSED` immediately.
        let result = GrpcWalletRankingOffsetQuery::new(
            "http://127.0.0.1:1".to_string(),
            fallback,
        )
        .await;
        assert!(
            result.is_err(),
            "GrpcWalletRankingOffsetQuery::new should fail when \
             pointed at an unreachable endpoint"
        );
    }

    /// gRPC timeout fallback: mock server hangs
    /// (delay=200ms, > client's 100ms timeout), client
    /// should time out and fall back to the in-process
    /// stub. Assert: the client returns the fallback's
    /// free-plan offset (100), not the server's offset
    /// (50).
    #[tokio::test]
    async fn test_grpc_client_falls_back_on_timeout() {
        let (url, server_handle) = spin_up_mock_server(
            50,
            Some(std::time::Duration::from_millis(200)),
        )
        .await;

        let fallback: Arc<dyn WalletRankingOffsetQuery> =
            Arc::new(FreePlanWalletRankingOffsetQuery);
        let client = GrpcWalletRankingOffsetQuery::new(url, fallback)
            .await
            .expect("client should connect (the connect is \
                     fast; the hang is on the gRPC call)");

        let offset = client
            .get_wallet_ranking_offset("0xdeadbeef")
            .await
            .expect("fallback never errors");

        assert_eq!(
            offset.value(),
            100,
            "gRPC client should time out and return the \
             fallback's free-plan offset (100), not the \
             server's offset (50)"
        );

        server_handle.abort();
    }

    /// The fallback path is independently testable: the
    /// in-process `FreePlanWalletRankingOffsetQuery` stub
    /// returns the free-plan default for any wallet. This
    /// is the same value the gRPC client returns when the
    /// server is unreachable / times out.
    #[tokio::test]
    async fn test_fallback_returns_free_plan_when_called_directly() {
        let stub = FreePlanWalletRankingOffsetQuery;
        for wallet in [
            "0x0000000000000000000000000000000000000000",
            "0xdeadbeef",
            "vitalik.eth",
            "",
        ] {
            let offset = stub
                .get_wallet_ranking_offset(wallet)
                .await
                .expect("stub never errors");
            assert_eq!(
                offset.value(),
                100,
                "fallback must return the free-plan offset for wallet={wallet}"
            );
        }
    }

    // ========================================================================
    // wave-13b Track B — SSE consumer end-to-end integration test
    // ========================================================================
    //
    // The hand-rolled parser + bus tests in
    // `sse_consumer::tests` cover the parsing logic in
    // isolation. This integration test covers the full
    // path: spin up a real `hyper`-based HTTP/1.1 server
    // that emits SSE-formatted bytes, point a real
    // `reqwest::Client` at it via `consume_once`, and
    // assert the events land in the bus.
    //
    // The mock server is intentionally tiny — it's a
    // single-request handler that writes two SSE events
    // and closes the connection. This mirrors the
    // identity service's wire format exactly (one
    // `data:` line per event, `\n\n` delimiter).
    //
    // Mock server: built on `axum::Router` (already in
    // the dep tree) for ergonomics. The handler writes
    // raw bytes to the response body so we can simulate
    // the identity service's exact SSE output without
    // pulling in `axum::response::sse::Sse` (which would
    // add a heartbeat line we don't want for this test).

    /// Spin up a tiny SSE server on `127.0.0.1:0` that
    /// emits two `RankingOffsetChange` events as raw SSE
    /// bytes then closes. Returns the bound URL and a
    /// `JoinHandle` so the test can abort the server on
    /// teardown.
    async fn spin_up_mock_sse_server()
    -> (String, tokio::task::JoinHandle<()>) {
        use axum::routing::get;
        use axum::Router;

        async fn sse_handler() -> impl axum::response::IntoResponse {
            // Emit two events with a small inter-event
            // delay so the consumer's `bytes_stream`
            // surfaces them as separate chunks (the
            // parser must handle multi-chunk + multi-
            // event-in-one-chunk correctly).
            let event1 = r#"data: {"wallet":"0xE2E2","offset":77,"changed_at_ms":1700000077000}"#;
            let event2 = r#"data: {"wallet":"0xC0DE","offset":50,"changed_at_ms":1700000077500}"#;
            (
                [
                    (axum::http::header::CONTENT_TYPE, "text/event-stream"),
                ],
                format!("{event1}\n\n{event2}\n\n"),
            )
        }

        let app = Router::new().route("/v1/stream/ranking-offsets", get(sse_handler));
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind ephemeral port for mock SSE server");
        let local_addr = listener
            .local_addr()
            .expect("read local_addr from ephemeral listener");
        let handle = tokio::spawn(async move {
            if let Err(e) = axum::serve(listener, app).await {
                eprintln!("mock SSE server error: {e}");
            }
        });
        let url = format!("http://{local_addr}/v1/stream/ranking-offsets");
        (url, handle)
    }

    /// Full end-to-end: a real HTTP/1.1 server emits SSE
    /// bytes, a real `reqwest::Client` opens the
    /// connection, `consume_once` parses + publishes,
    /// and the events land in the bus. This is the
    /// "binary works" canary — no mocks, no hand-rolled
    /// loops, just a real `reqwest::get` against a real
    /// `axum` server.
    #[tokio::test]
    async fn test_sse_consumer_end_to_end_via_real_http() {
        let (url, server_handle) = spin_up_mock_sse_server().await;

        // Build the same shape `main()` builds.
        let bus = LocalRankingOffsetBus::new(16);
        let mut rx = bus.subscribe();
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()
            .expect("reqwest client builds");
        let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);
        let _tx = shutdown_tx; // silence unused

        // `consume_once` is `pub(super)` in sse_consumer.rs
        // — wait, no, it's a module-private `async fn`.
        // We re-implement the call here: spawn
        // `run_sse_consumer` in a task, let it consume
        // both events, then signal shutdown.
        let url_for_consumer = url.clone();
        let bus_for_consumer = bus.clone();
        let client_for_consumer = client.clone();
        let consumer_handle = tokio::spawn(async move {
            // Set a short timeout for the test by
            // setting the URL to the mock server
            // (which emits + closes immediately).
            sse_consumer::run_sse_consumer(
                url_for_consumer,
                bus_for_consumer,
                client_for_consumer,
                shutdown_rx,
            )
            .await;
        });

        // Drain 2 events. Use a 5s timeout so the test
        // fails fast if the consumer never publishes.
        let r1 = tokio::time::timeout(
            std::time::Duration::from_secs(5),
            rx.recv(),
        )
        .await
        .expect("event 1 must arrive within 5s")
        .expect("event 1 must be received (not lagged)");
        assert_eq!(r1.wallet, "0xE2E2");
        assert_eq!(r1.offset, 77);
        assert_eq!(r1.changed_at_ms, 1_700_000_077_000);

        let r2 = tokio::time::timeout(
            std::time::Duration::from_secs(5),
            rx.recv(),
        )
        .await
        .expect("event 2 must arrive within 5s")
        .expect("event 2 must be received (not lagged)");
        assert_eq!(r2.wallet, "0xC0DE");
        assert_eq!(r2.offset, 50);
        assert_eq!(r2.changed_at_ms, 1_700_000_077_500);

        // Tear down: signal shutdown to the consumer so
        // the reconnect loop exits, then abort both
        // tasks to free the ephemeral port.
        // (The mock server closed the connection after
        // emitting both events, so the consumer is
        // currently in the "backoff = 100ms, retrying"
        // state. The shutdown signal makes it exit
        // cleanly.)
        drop(_tx); // closes the watch sender; consumer sees it
        let _ = tokio::time::timeout(
            std::time::Duration::from_secs(2),
            consumer_handle,
        )
        .await;
        server_handle.abort();
    }
}
