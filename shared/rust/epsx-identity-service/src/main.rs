//! `epsx-identity-service` binary — tonic gRPC server entry point.
//!
//! Day 1: exposes one RPC, `GetWalletRankingOffset`, that returns
//! the free-plan offset (100) for every wallet. The impl is a
//! stub — it satisfies the kernel-level `WalletRankingOffsetQuery`
//! port (`shared/rust/epsx-contracts/src/wallet_ranking_offset_query.rs`)
//! via `FreePlanRankingOffsetService` in `identity_service.rs`.
//!
//! Behavior contract: the new binary is functionally identical to
//! the wave-12 in-process `FreePlanWalletRankingOffsetQuery` in
//! `apps/analytics/src/main.rs:122-141` but goes over gRPC instead
//! of in-process. The fallback contract (timeout → fall back to
//! free-plan offset) is implemented in **Track B** (the analytics
//! binary's gRPC client); Track A is just the server.
//!
//! Spec: `docs/wave8-service-boundary/ROADMAP.md` §17.1 (this
//! track creates that section).

use std::net::SocketAddr;
use std::sync::Arc;

use anyhow::Context;
use tonic::transport::Server;
use tracing::{error, info};
use tracing_subscriber::EnvFilter;

use epsx_contracts::wallet_ranking_offset_query::WalletRankingOffsetQuery;
use epsx_identity_service::generated::identity_server::IdentityServer;
use epsx_identity_service::identity_service::{
    map_app_error_to_status, FreePlanRankingOffsetService,
};

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

    // ---- bind address ----
    // Default: 0.0.0.0:50051 (tonic convention). Override with
    // `BIND_ADDR` env var (same pattern as the analytics
    // binary) so the dev container can bind to a different port
    // if the host's 50051 is already in use.
    let bind_addr = std::env::var("BIND_ADDR").unwrap_or_else(|_| DEFAULT_BIND_ADDR.to_string());
    let addr: SocketAddr = bind_addr
        .parse()
        .with_context(|| format!("parsing BIND_ADDR={bind_addr}"))?;

    // ---- startup banner ----
    print_startup_banner(addr);

    // ---- DI ----
    // Day 1: a single Arc<dyn WalletRankingOffsetQuery> backed
    // by the stub. Future waves can swap to a tier-aware impl
    // without changing the gRPC server scaffolding.
    let port_impl: Arc<dyn WalletRankingOffsetQuery> = Arc::new(FreePlanRankingOffsetService);
    let grpc_service = GrpcIdentityService::new(port_impl);

    // ---- serve ----
    info!(%addr, "epsx-identity-service listening on gRPC");
    let server = Server::builder().add_service(IdentityServer::new(grpc_service));

    if let Err(err) = server.serve(addr).await {
        error!(error = %err, "tonic::server::serve returned an error");
        return Err(err.into());
    }
    Ok(())
}

fn print_startup_banner(addr: SocketAddr) {
    info!("============================================================");
    info!("  {} v{}", BINARY_NAME, BINARY_VERSION);
    info!("  Wave 13a — Track A (identity binary, tonic gRPC server)");
    info!("  Bind: {}", addr);
    info!("  gRPC methods (1):");
    info!("    rpc GetWalletRankingOffset(GetWalletRankingOffsetRequest)");
    info!("        returns GetWalletRankingOffsetResponse");
    info!("  Day-1 behavior: always returns the free-plan offset");
    info!("  (matching the wave-12 in-process FreePlanWalletRankingOffsetQuery stub).");
    info!("============================================================");
}
