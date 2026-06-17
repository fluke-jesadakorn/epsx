//! `GrpcWalletRankingOffsetQuery` — tonic gRPC client that
//! calls the `epsx-identity-service` binary over the wire to
//! fetch a wallet's plan-tier ranking offset.
//!
//! This is the **wave-13a Track B** end-state for the
//! `epsx-analytics-service` binary: the wave-12 in-process
//! `FreePlanWalletRankingOffsetQuery` stub (in
//! `apps/analytics/src/main.rs:122-141`) is wrapped by this
//! gRPC client. Behavior is preserved — every wallet still
//! gets `RankingOffset::free_plan()` (the identity binary's
//! Track A stub returns the same value) — but the
//! *transport* is now a real gRPC call.
//!
//! ## Fallback contract
//!
//! Per `ROADMAP §17.1` (Track B spec): if the gRPC call fails
//! or times out, fall back to the in-process stub. The
//! fallback is a constructor argument (a
//! `Arc<dyn WalletRankingOffsetQuery>`) so the caller can
//! swap in any impl — the in-process stub is the day-1
//! choice, a future wave could swap to a cached / Redis-backed
//! fallback without changing the client.
//!
//! Two failure modes trigger fallback:
//!   1. **`Ok(Err(status))`** — the gRPC server returned a
//!      tonic `Status` (transient DB error, `UNAVAILABLE`,
//!      `DEADLINE_EXCEEDED`, etc.). The fallback is
//!      defensive: any server error is treated as "we don't
//!      know this wallet's tier, default to free".
//!   2. **`Err(_elapsed)`** — the 100ms timeout elapsed
//!      before the server responded. Most commonly a network
//!      partition or the identity pod being OOM-killed.
//!
//! A successful response (`Ok(Ok(resp))`) is wrapped in the
//! port's `RankingOffset` newtype via `RankingOffset(resp.offset)`
//! (out-of-range values would be clamped by
//! `RankingOffset::from(i32)`, but the contract is that the
//! server returns a valid `0..=1000` value).
//!
//! ## Why 100ms?
//!
//! The fallback timeout matches the monolith's
//! `web/analytics/eps/cache.rs:78-81` behavior of falling back
//! to the free-plan offset on auth errors (which are typically
//! fast network round-trips, not slow timeouts). 100ms is
//! well above the local-network RTT (sub-ms) and the identity
//! service's expected response time (sub-10ms for the
//! free-plan stub) but well below the analytics request
//! timeout (axum's default is no timeout; Cloudflare's is
//! 100s). Future waves can tune this if the identity service
//! grows slower.
//!
//! ## Object safety
//!
//! The trait method `get_wallet_ranking_offset(&self, wallet:
//! &str)` is `async` + takes `&self`, so the trait needs
//! `#[async_trait]` to be dyn-compatible. The
//! `WalletRankingOffsetQuery` trait in
//! `shared/rust/epsx-contracts` already has that — the
//! `Arc<dyn WalletRankingOffsetQuery>` constructor argument
//! compiles because the trait is object-safe (see the
//! `object_safety` test in `wallet_ranking_offset_query.rs`).

use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use epsx_contracts::{
    errors::AppResult,
    value_objects::ranking_offset::RankingOffset,
    wallet_ranking_offset_query::WalletRankingOffsetQuery,
};
use tonic::transport::Channel;
use tracing::{debug, warn};

// The `tonic::include_proto!` macro lives in `main.rs` (the
// spec wants it there so the generated module is a sibling
// of `grpc_client`). We re-import the types from `super`.
// The proto's `package epsx.identity.v1;` makes the module
// name `identity_proto` (any name works — `tonic::include_proto!`
// uses the first argument as the module name, not the
// proto's package).
use super::identity_proto::identity_client::IdentityClient;
use super::identity_proto::{
    GetWalletRankingOffsetRequest, GetWalletRankingOffsetResponse,
};

/// Timeout for the gRPC call. See module-level docs for the
/// rationale (matches the monolith's `web/analytics/eps/
/// cache.rs:78-81` fallback behavior).
const GRPC_TIMEOUT: Duration = Duration::from_millis(100);

/// gRPC client that calls the `epsx-identity-service` binary.
///
/// The day-1 shape is a **client** that wraps a **fallback**
/// (the wave-12 in-process `FreePlanWalletRankingOffsetQuery`
/// stub). If the gRPC call fails or times out, the fallback
/// is invoked transparently — the caller never sees the
/// transport error.
///
/// `Clone` is required because tonic's `IdentityClient` is
/// cheap to clone (it wraps an `Arc<Channel>` internally)
/// and the client is held inside an `Arc<dyn
/// WalletRankingOffsetQuery>` in the analytics binary's
/// `AppState`.
#[derive(Clone)]
pub struct GrpcWalletRankingOffsetQuery {
    /// The tonic client. Cheap to clone (internally
    /// `Arc<Channel>`).
    client: IdentityClient<Channel>,
    /// The fallback used when the gRPC call fails or times
    /// out. Held as `Arc<dyn WalletRankingOffsetQuery>` so
    /// the constructor can accept any impl (the day-1
    /// `FreePlanWalletRankingOffsetQuery`, a future cached
    /// variant, etc.).
    fallback: Arc<dyn WalletRankingOffsetQuery>,
}

impl GrpcWalletRankingOffsetQuery {
    /// Build a gRPC client. The endpoint is a tonic-formatted
    /// URL (e.g. `http://127.0.0.1:50051` for local dev,
    /// `http://epsx-identity:50051` for K8s).
    ///
    /// `IdentityClient::connect` is async (it dials the
    /// endpoint and does the HTTP/2 handshake), so this
    /// constructor is async too. The caller is expected to
    /// `await` it during the binary's startup phase.
    ///
    /// Returns `anyhow::Error` on connect failure so the
    /// caller can `?`-propagate up through `main()` and the
    /// binary exits with a clear error message ("building
    /// gRPC identity client: ...") instead of panicking
    /// mid-startup.
    pub async fn new(
        endpoint: String,
        fallback: Arc<dyn WalletRankingOffsetQuery>,
    ) -> anyhow::Result<Self> {
        let client = IdentityClient::connect(endpoint.clone())
            .await
            .map_err(|e| {
                anyhow::anyhow!("failed to connect to identity gRPC endpoint {endpoint}: {e}")
            })?;
        Ok(Self { client, fallback })
    }
}

#[async_trait]
impl WalletRankingOffsetQuery for GrpcWalletRankingOffsetQuery {
    async fn get_wallet_ranking_offset(
        &self,
        wallet: &str,
    ) -> AppResult<RankingOffset> {
        let req = tonic::Request::new(GetWalletRankingOffsetRequest {
            wallet: wallet.to_string(),
        });

        // Wrap the gRPC call in a 100ms timeout. The
        // `tokio::time::timeout` future returns:
        //   - `Ok(Ok(resp))` — server responded successfully
        //     within the timeout.
        //   - `Ok(Err(status))` — server responded with a
        //     tonic `Status` (transient DB error, etc.).
        //   - `Err(_elapsed)` — the timeout elapsed before
        //     the server responded (network partition, OOM,
        //     etc.).
        //
        // `self.client.clone()` is required because
        // `get_wallet_ranking_offset` takes `self` by value
        // (tonic's signature). The `IdentityClient` is cheap
        // to clone (it's an `Arc<Channel>` internally).
        let result = tokio::time::timeout(
            GRPC_TIMEOUT,
            self.client.clone().get_wallet_ranking_offset(req),
        )
        .await;

        match result {
            Ok(Ok(resp)) => {
                let inner: GetWalletRankingOffsetResponse = resp.into_inner();
                debug!(
                    wallet = %wallet,
                    offset = inner.offset,
                    "gRPC GetWalletRankingOffset OK"
                );
                // Use `RankingOffset::from(i32)` rather than
                // `RankingOffset::new(i32)` or
                // `RankingOffset::new_unchecked` because:
                //   - The struct's inner field is `pub struct
                //     RankingOffset(i32)` with a private `i32`
                //     field, so `RankingOffset(inner.offset)`
                //     won't compile (E0423). The constructor
                //     API is `new` (validated) or
                //     `new_unchecked` (unvalidated) or
                //     `From<i32>` (clamps out-of-range to
                //     free-plan default).
                //   - The server is supposed to return a
                //     valid `0..=1000` value, but `From<i32>`
                //     is defensive — a buggy server returning
                //     `-1` or `> 1000` falls back to the
                //     same free-plan default the in-process
                //     stub returns. No client crash on a
                //     wire-format violation.
                Ok(RankingOffset::from(inner.offset))
            }
            Ok(Err(status)) => {
                warn!(
                    wallet = %wallet,
                    code = ?status.code(),
                    message = %status.message(),
                    "gRPC GetWalletRankingOffset failed; falling back to in-process stub"
                );
                self.fallback.get_wallet_ranking_offset(wallet).await
            }
            Err(_elapsed) => {
                warn!(
                    wallet = %wallet,
                    timeout_ms = GRPC_TIMEOUT.as_millis() as u64,
                    "gRPC GetWalletRankingOffset timed out; falling back to in-process stub"
                );
                self.fallback.get_wallet_ranking_offset(wallet).await
            }
        }
    }
}
