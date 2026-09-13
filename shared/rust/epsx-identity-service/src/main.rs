//! `epsx-identity-service` binary — single-port gRPC server entry point.
//!
//! The binary exposes `GetWalletRankingOffset` over tonic on
//! `BIND_ADDR` (default `0.0.0.0:50051`). The current adapter delegates
//! to a fail-closed ranking authority while the verified, database-backed
//! entitlement implementation is migrated.
//!
//! Event publishing helpers remain available in library test builds for
//! historical coverage, but the production binary does not construct them.

use std::net::SocketAddr;
use std::sync::Arc;

use anyhow::Context;
use tonic::transport::Server;
use tracing::info;
use tracing_subscriber::EnvFilter;

use epsx_contracts::wallet_ranking_offset_query::WalletRankingOffsetQuery;
use epsx_identity_service::generated::identity_server::IdentityServer;
use epsx_identity_service::identity_service::{
    map_app_error_to_status, UnavailableRankingOffsetService,
};

// ============================================================================
// gRPC service impl
// ============================================================================
//
// The tonic-build-generated `IdentityServer` is a
// `tonic::server::Server`-trait shim. We wrap the port impl
// (`UnavailableRankingOffsetService`) in a small adapter that
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
/// `UnavailableRankingOffsetService` until the authoritative adapter is wired.
pub struct GrpcIdentityService {
    inner: Arc<dyn WalletRankingOffsetQuery>,
}

impl GrpcIdentityService {
    /// Construct a gRPC service backed by an arbitrary
    /// `WalletRankingOffsetQuery` port impl. Day 1 passes
    /// `UnavailableRankingOffsetService`; future waves pass a
    /// `TierAwareRankingOffsetService` that reads from
    /// `wallet_plan_assignments`.
    pub fn new(inner: Arc<dyn WalletRankingOffsetQuery>) -> Self {
        Self { inner }
    }
}

#[tonic::async_trait]
impl epsx_identity_service::generated::identity_server::Identity for GrpcIdentityService {
    async fn get_wallet_ranking_offset(
        &self,
        request: tonic::Request<epsx_identity_service::generated::GetWalletRankingOffsetRequest>,
    ) -> Result<
        tonic::Response<epsx_identity_service::generated::GetWalletRankingOffsetResponse>,
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

    // Default: 0.0.0.0:50051 (tonic convention). Override with
    // `BIND_ADDR` so a container can bind to a different address.
    let bind_addr = std::env::var("BIND_ADDR").unwrap_or_else(|_| DEFAULT_BIND_ADDR.to_string());
    let grpc_addr: SocketAddr = bind_addr
        .parse()
        .with_context(|| format!("parsing BIND_ADDR={bind_addr}"))?;

    print_startup_banner(grpc_addr);

    // ---- DI ----
    // Keep the standalone authority fail-closed until a verified,
    // database-backed tier-aware implementation is wired.
    let port_impl: Arc<dyn WalletRankingOffsetQuery> = Arc::new(UnavailableRankingOffsetService);
    let grpc_service = GrpcIdentityService::new(port_impl);

    info!(%grpc_addr, "epsx-identity-service: tonic gRPC server listening");
    Server::builder()
        .add_service(IdentityServer::new(grpc_service))
        .serve(grpc_addr)
        .await
        .context("serving the identity gRPC endpoint")?;

    Ok(())
}

fn print_startup_banner(grpc_addr: SocketAddr) {
    info!("============================================================");
    info!("  {} v{}", BINARY_NAME, BINARY_VERSION);
    info!("  Identity gRPC: {}", grpc_addr);
    info!("  gRPC methods (1):");
    info!("    rpc GetWalletRankingOffset(GetWalletRankingOffsetRequest)");
    info!("        returns GetWalletRankingOffsetResponse");
    info!("  ranking behavior: unavailable until entitlement authority is wired");
    info!("============================================================");
}

#[cfg(test)]
mod tests {
    use super::*;
    use epsx_identity_service::generated::identity_server::Identity;
    use epsx_identity_service::generated::GetWalletRankingOffsetRequest;
    use prost::Message;

    fn service() -> GrpcIdentityService {
        GrpcIdentityService::new(Arc::new(UnavailableRankingOffsetService))
    }

    #[tokio::test]
    async fn ranking_authority_is_fail_closed_until_wired() {
        let error = Identity::get_wallet_ranking_offset(
            &service(),
            tonic::Request::new(GetWalletRankingOffsetRequest {
                wallet: "0xdeadbeef".to_owned(),
            }),
        )
        .await
        .expect_err("unwired ranking authority must not manufacture an offset");
        assert_eq!(error.code(), tonic::Code::Unavailable);
    }

    #[tokio::test]
    async fn a2_9_proto_wire_round_trip_remains_field_compatible() {
        let request = GetWalletRankingOffsetRequest {
            wallet: "0xabcdef".to_owned(),
        };
        let request_bytes = request.encode_to_vec();
        assert_eq!(request_bytes.first(), Some(&0x0a), "wallet remains field 1");
        let decoded_request = GetWalletRankingOffsetRequest::decode(request_bytes.as_slice())
            .expect("request wire payload must decode");

        let error =
            Identity::get_wallet_ranking_offset(&service(), tonic::Request::new(decoded_request))
                .await
                .expect_err("unwired authority must reject the request");
        assert_eq!(error.code(), tonic::Code::Unavailable);
    }

    #[test]
    fn a2_9_production_main_contains_only_grpc_listener_surface() {
        let source = include_str!("main.rs");
        let production = source
            .split_once("#[cfg(test)]")
            .expect("main must keep tests after the production entry point")
            .0;

        for forbidden in [
            "BIND_ADDR_SSE",
            "DEFAULT_BIND_ADDR_SSE",
            "50052",
            "axum",
            "axum::",
            "Router",
            "/v1/",
            "/v1/stream/ranking-offsets",
            "/v1/emit",
            "RankingOffsetEventBus",
            "EVENT_BUS_CAPACITY",
            "emit_ranking_offset",
            "stream_ranking_offsets",
            "try_join!",
            "TcpListener",
            "TcpSocket",
            "http_server",
        ] {
            assert!(
                !production.contains(forbidden),
                "production main unexpectedly contains {forbidden}"
            );
        }

        for required in [
            "DEFAULT_BIND_ADDR",
            "0.0.0.0:50051",
            "IdentityServer::new",
            ".serve(grpc_addr)",
        ] {
            assert!(
                production.contains(required),
                "production main must retain {required}"
            );
        }

        assert_eq!(
            production.matches(".serve(").count(),
            1,
            "production main must have exactly one server listener"
        );
    }
}
