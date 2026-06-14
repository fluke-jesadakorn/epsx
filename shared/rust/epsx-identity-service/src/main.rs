//! `epsx-identity-service` binary — dual-port server entry
//! point (wave 13a Track A + wave 13b Track A).
//!
//! ## Port layout
//!
//! - **Port 50051 (BIND_ADDR, gRPC, HTTP/2):** tonic gRPC
//!   server. Exposes ONE RPC, `GetWalletRankingOffset`, that
//!   returns the free-plan offset (100) for every wallet.
//!   The impl is a stub — it satisfies the kernel-level
//!   `WalletRankingOffsetQuery` port
//!   (`shared/rust/epsx-contracts/src/wallet_ranking_offset_query.rs`)
//!   via `FreePlanRankingOffsetService` in `identity_service.rs`.
//!   Behavior contract: the new binary is functionally
//!   identical to the wave-12 in-process
//!   `FreePlanWalletRankingOffsetQuery` in
//!   `apps/analytics/src/main.rs:122-141` but goes over gRPC
//!   instead of in-process. The fallback contract (timeout →
//!   fall back to free-plan offset) is implemented in
//!   **Track B** (the analytics binary's gRPC client); Track
//!   A is just the server.
//! - **Port 50052 (BIND_ADDR_SSE, HTTP/1.1, axum):** SSE
//!   stream + admin emit hook. Exposes:
//!     - `GET  /v1/stream/ranking-offsets` — SSE stream of
//!       `RankingOffsetChange` events.
//!     - `POST /v1/emit` — admin hook to publish a
//!       `RankingOffsetChange` to the bus.
//!   Both endpoints share a single `RankingOffsetEventBus`
//!   instance — the admin emit handler writes to it, the
//!   SSE handler reads from it.
//!
//! ## Why two ports, not one?
//!
//! The task spec suggested `tonic-web = "0.12"` as the
//! bridge. In practice, `tonic-web` is a **gRPC-Web**
//! protocol translator (browser → tonic, HTTP/1.1 +
//! `application/grpc-web` content type → native gRPC over
//! HTTP/2). It is NOT a general HTTP/1.1 router — the crate
//! docs explicitly state "is not expected to handle
//! arbitrary HTTP/x.x requests or bespoke protocols" and
//! "There is no support for web socket transports". There
//! is no `resource()` / `add_routes()` API in any
//! `tonic-web` version. The right primitive for hosting
//! arbitrary HTTP/1.1 routes alongside tonic is plain axum
//! on a separate port — the workspace already pins
//! `axum = "0.8"` with the `ws` feature (which transitively
//! enables `tokio` + `keep_alive`), and the wave-10
//! notifications SSE handler already uses the same axum
//! pattern. Two ports keeps the gRPC seam clean (no
//! content-type negotiation, no ALPN) at the cost of one
//! extra `tokio::spawn`.
//!
//! ## Spec
//!
//! - Wave 13a (Track A): `docs/wave8-service-boundary/ROADMAP.md`
//!   §17.1.
//! - Wave 13b (Track A): `docs/wave8-service-boundary/ROADMAP.md`
//!   §17.2 (this track creates that section).

use std::net::SocketAddr;
use std::sync::Arc;

use anyhow::Context;
use axum::{
    routing::{get, post},
    Router,
};
use tonic::transport::Server;
use tracing::{error, info};
use tracing_subscriber::EnvFilter;

use epsx_contracts::wallet_ranking_offset_query::WalletRankingOffsetQuery;
use epsx_identity_service::emit_handler::emit_ranking_offset;
use epsx_identity_service::event_bus::RankingOffsetEventBus;
use epsx_identity_service::generated::identity_server::IdentityServer;
use epsx_identity_service::identity_service::{
    map_app_error_to_status, FreePlanRankingOffsetService,
};
use epsx_identity_service::sse_handler::stream_ranking_offsets;

// ============================================================================
// gRPC service impl
// ============================================================================
//
// The tonic-build-generated `IdentityServer` is a
// `tonic::server::Server`-trait shim. We wrap the port impl
// (`FreePlanRankingOffsetService`) in a small adapter that
// implements the generated trait, mapping the gRPC request
// (`GetWalletRankingOffsetRequest`) → the port's
// `get_wallet_ranking_offset(wallet: &str)` call, then mapping
// the port's `AppResult<RankingOffset>` → the gRPC response
// (`GetWalletRankingOffsetResponse { offset: i32 }`).
//
// We do this as a separate struct (rather than implementing
// the trait on `FreePlanRankingOffsetService` directly) so the
// port impl stays port-only — it doesn't depend on tonic or
// prost at all. The adapter is a thin (~30 LOC) shim that
// lives in the binary, not the lib.

/// The tonic `Identity` service impl. Wraps any
/// `WalletRankingOffsetQuery` port impl and serves it over
/// gRPC. The current concrete impl is
/// `FreePlanRankingOffsetService` (the stub).
pub struct GrpcIdentityService {
    inner: Arc<dyn WalletRankingOffsetQuery>,
}

impl GrpcIdentityService {
    /// Construct a gRPC service backed by an arbitrary
    /// `WalletRankingOffsetQuery` port impl. Day 1 passes
    /// `FreePlanRankingOffsetService`; future waves pass a
    /// `TierAwareRankingOffsetService` that reads from
    /// `wallet_plan_assignments`.
    pub fn new(inner: Arc<dyn WalletRankingOffsetQuery>) -> Self {
        Self { inner }
    }
}

#[tonic::async_trait]
impl epsx_identity_service::generated::identity_server::Identity
    for GrpcIdentityService
{
    async fn get_wallet_ranking_offset(
        &self,
        request: tonic::Request<
            epsx_identity_service::generated::GetWalletRankingOffsetRequest,
        >,
    ) -> Result<
        tonic::Response<
            epsx_identity_service::generated::GetWalletRankingOffsetResponse,
        >,
        tonic::Status,
    > {
        use epsx_identity_service::generated as pb;

        let wallet = request.into_inner().wallet;
        match self.inner.get_wallet_ranking_offset(&wallet).await {
            Ok(offset) => {
                let resp = pb::GetWalletRankingOffsetResponse {
                    offset: offset.value(),
                };
                Ok(tonic::Response::new(resp))
            }
            Err(err) => Err(map_app_error_to_status(err)),
        }
    }
}

// ============================================================================
// main
// ============================================================================

const BINARY_NAME: &str = env!("CARGO_PKG_NAME");
const BINARY_VERSION: &str = env!("CARGO_PKG_VERSION");
const DEFAULT_BIND_ADDR: &str = "0.0.0.0:50051";
const DEFAULT_BIND_ADDR_SSE: &str = "0.0.0.0:50052";
/// 1024-slot broadcast channel. Plenty for the dev cluster;
/// production tuning can lift this in wave-14+ if needed.
const EVENT_BUS_CAPACITY: usize = 1024;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // ---- tracing init ----
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info,epsx_identity_service=info")),
        )
        .with_target(false)
        .init();

    // ---- bind addresses ----
    // Default: 0.0.0.0:50051 (tonic convention) +
    //          0.0.0.0:50052 (axum convention, separate port).
    // Override with `BIND_ADDR` (gRPC) and `BIND_ADDR_SSE`
    // (HTTP/1.1) env vars (same pattern as the analytics
    // binary) so the dev container can bind to a different
    // port if the host's 50051/50052 are already in use.
    let bind_addr = std::env::var("BIND_ADDR").unwrap_or_else(|_| DEFAULT_BIND_ADDR.to_string());
    let grpc_addr: SocketAddr = bind_addr
        .parse()
        .with_context(|| format!("parsing BIND_ADDR={bind_addr}"))?;
    let bind_addr_sse =
        std::env::var("BIND_ADDR_SSE").unwrap_or_else(|_| DEFAULT_BIND_ADDR_SSE.to_string());
    let sse_addr: SocketAddr = bind_addr_sse
        .parse()
        .with_context(|| format!("parsing BIND_ADDR_SSE={bind_addr_sse}"))?;

    // ---- startup banner ----
    print_startup_banner(grpc_addr, sse_addr);

    // ---- DI ----
    // Day 1: a single Arc<dyn WalletRankingOffsetQuery> backed
    // by the stub. Future waves can swap to a tier-aware impl
    // without changing the gRPC server scaffolding.
    let port_impl: Arc<dyn WalletRankingOffsetQuery> = Arc::new(FreePlanRankingOffsetService);
    let grpc_service = GrpcIdentityService::new(port_impl);

    // ---- pub-sub bus (shared between gRPC future
    //      publishers + the HTTP/1.1 SSE + emit handlers) ----
    let bus = Arc::new(RankingOffsetEventBus::new(EVENT_BUS_CAPACITY));

    // ---- gRPC server (port 50051, unchanged from wave 13a) ----
    let grpc_server = {
        let addr = grpc_addr;
        let svc = grpc_service;
        async move {
            info!(%addr, "epsx-identity-service: tonic gRPC server listening");
            if let Err(err) = Server::builder()
                .add_service(IdentityServer::new(svc))
                .serve(addr)
                .await
            {
                error!(error = %err, "tonic::server::serve returned an error");
                return Err(err.into());
            }
            Ok::<(), anyhow::Error>(())
        }
    };

    // ---- HTTP/1.1 server (port 50052, wave 13b) ----
    // axum router with two routes:
    //   - `GET  /v1/stream/ranking-offsets` — SSE stream
    //   - `POST /v1/emit` — admin emit hook
    // Both share the same `RankingOffsetEventBus` state via
    // `with_state(Arc::clone(&bus))`. The bus is the only
    // shared state the binary owns; the gRPC future-tier
    // impls (wave 13c+) will hold their own `Arc` to the
    // same bus.
    let http_server = {
        let addr = sse_addr;
        let bus = Arc::clone(&bus);
        async move {
            let app = Router::new()
                .route("/v1/stream/ranking-offsets", get(stream_ranking_offsets))
                .route("/v1/emit", post(emit_ranking_offset))
                .with_state((*bus).clone());

            info!(
                %addr,
                routes = "/v1/stream/ranking-offsets (GET, SSE), /v1/emit (POST, JSON)",
                "epsx-identity-service: axum HTTP/1.1 server listening"
            );

            let listener = tokio::net::TcpListener::bind(addr)
                .await
                .with_context(|| format!("binding axum listener to {addr}"))?;

            if let Err(err) = axum::serve(listener, app).await {
                error!(error = %err, "axum::serve returned an error");
                return Err(err.into());
            }
            Ok::<(), anyhow::Error>(())
        }
    };

    // ---- serve both concurrently ----
    // `tokio::try_join!` propagates the first error and
    // cancels the other future. If the gRPC server fails to
    // bind, the HTTP/1.1 server is also cancelled; if the
    // HTTP/1.1 server fails, the gRPC server is cancelled.
    // K8s liveness probes on both ports keep the failure
    // modes independent at the pod level.
    tokio::try_join!(grpc_server, http_server)?;

    Ok(())
}

fn print_startup_banner(grpc_addr: SocketAddr, sse_addr: SocketAddr) {
    info!("============================================================");
    info!("  {} v{}", BINARY_NAME, BINARY_VERSION);
    info!("  Wave 13a — Track A (gRPC, tonic) +");
    info!("  Wave 13b — Track A (SSE + admin emit, axum)");
    info!("  gRPC:        {}", grpc_addr);
    info!("  HTTP/1.1 SSE:{}", sse_addr);
    info!("  Event bus:   broadcast channel, capacity = {}", EVENT_BUS_CAPACITY);
    info!("  gRPC methods (1):");
    info!("    rpc GetWalletRankingOffset(GetWalletRankingOffsetRequest)");
    info!("        returns GetWalletRankingOffsetResponse");
    info!("  HTTP/1.1 routes (2):");
    info!("    GET  /v1/stream/ranking-offsets  (SSE)");
    info!("    POST /v1/emit                    (JSON, admin)");
    info!("  Day-1 behavior: always returns the free-plan offset");
    info!("  (matching the wave-12 in-process FreePlanWalletRankingOffsetQuery stub).");
    info!("============================================================");
}
