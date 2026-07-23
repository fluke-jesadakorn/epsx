//! Fail-closed gRPC adapter for the identity ranking-offset authority.
//!
//! The channel is constructed lazily: parsing a valid endpoint never dials the
//! identity service. Anonymous ranking requests do not call this adapter, so an
//! identity outage does not prevent the analytics process from starting or
//! serving the anonymous path. Once a verified wallet needs an offset, the
//! single RPC attempt is bounded by one deadline and every transport, status,
//! timeout, or wire-contract failure is returned as one opaque authority error.

use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use epsx_contracts::{
    errors::{AppError, AppResult, ErrorKind},
    value_objects::ranking_offset::RankingOffset,
    wallet_ranking_offset_query::WalletRankingOffsetQuery,
};
use tonic::transport::{Channel, Endpoint};
use tracing::{debug, warn};

use super::identity_proto::identity_client::IdentityClient;
use super::identity_proto::GetWalletRankingOffsetRequest;

const GRPC_TIMEOUT: Duration = Duration::from_millis(100);
const AUTHORITY_UNAVAILABLE_MESSAGE: &str = "Ranking authority is unavailable";

/// Narrow transport seam used to keep authority failure tests hermetic.
///
/// The production implementation performs one tonic request. Tests inject a
/// fake implementation and therefore never bind or connect a socket.
#[async_trait]
trait IdentityRankingOffsetRpc: Send + Sync {
    async fn get_wallet_ranking_offset(&self, wallet: String) -> Result<i32, tonic::Status>;
}

#[derive(Clone)]
struct TonicIdentityRankingOffsetRpc {
    client: IdentityClient<Channel>,
}

#[async_trait]
impl IdentityRankingOffsetRpc for TonicIdentityRankingOffsetRpc {
    async fn get_wallet_ranking_offset(&self, wallet: String) -> Result<i32, tonic::Status> {
        let request = tonic::Request::new(GetWalletRankingOffsetRequest { wallet });
        self.client
            .clone()
            .get_wallet_ranking_offset(request)
            .await
            .map(|response| response.into_inner().offset)
    }
}

/// gRPC-backed implementation of the shared ranking-offset query port.
#[derive(Clone)]
pub struct GrpcWalletRankingOffsetQuery {
    rpc: Arc<dyn IdentityRankingOffsetRpc>,
}

impl GrpcWalletRankingOffsetQuery {
    /// Parse the endpoint and construct a lazy tonic channel without dialing.
    ///
    /// Connection establishment happens only when an authenticated ranking
    /// request invokes the query port. Endpoint parse failures are deliberately
    /// opaque so credentials accidentally embedded in configuration are not
    /// repeated in startup errors.
    pub fn new(endpoint: String) -> anyhow::Result<Self> {
        let endpoint = Endpoint::from_shared(endpoint)
            .map_err(|_| anyhow::anyhow!("invalid identity gRPC endpoint"))?;
        let client = IdentityClient::new(endpoint.connect_lazy());
        Ok(Self {
            rpc: Arc::new(TonicIdentityRankingOffsetRpc { client }),
        })
    }

    #[cfg(test)]
    fn from_rpc(rpc: Arc<dyn IdentityRankingOffsetRpc>) -> Self {
        Self { rpc }
    }

    fn unavailable() -> AppError {
        AppError::new(ErrorKind::ServiceUnavailable, AUTHORITY_UNAVAILABLE_MESSAGE)
    }
}

#[async_trait]
impl WalletRankingOffsetQuery for GrpcWalletRankingOffsetQuery {
    async fn get_wallet_ranking_offset(&self, wallet: &str) -> AppResult<RankingOffset> {
        let result = tokio::time::timeout(
            GRPC_TIMEOUT,
            self.rpc.get_wallet_ranking_offset(wallet.to_string()),
        )
        .await;

        match result {
            Ok(Ok(raw_offset)) => RankingOffset::new(raw_offset).map_err(|_| {
                warn!("Identity ranking authority returned an invalid offset");
                Self::unavailable()
            }),
            Ok(Err(_)) => {
                warn!("Identity ranking authority request failed");
                Err(Self::unavailable())
            }
            Err(_) => {
                warn!(
                    timeout_ms = GRPC_TIMEOUT.as_millis() as u64,
                    "Identity ranking authority request timed out"
                );
                Err(Self::unavailable())
            }
        }
        .inspect(|offset| debug!(offset = offset.value(), "Identity ranking offset resolved"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[derive(Clone, Copy)]
    enum FakeOutcome {
        Offset(i32),
        Status,
        Pending,
    }

    struct FakeRpc {
        outcome: FakeOutcome,
        calls: AtomicUsize,
    }

    impl FakeRpc {
        fn new(outcome: FakeOutcome) -> Arc<Self> {
            Arc::new(Self {
                outcome,
                calls: AtomicUsize::new(0),
            })
        }
    }

    #[async_trait]
    impl IdentityRankingOffsetRpc for FakeRpc {
        async fn get_wallet_ranking_offset(&self, _wallet: String) -> Result<i32, tonic::Status> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            match self.outcome {
                FakeOutcome::Offset(offset) => Ok(offset),
                FakeOutcome::Status => Err(tonic::Status::internal(
                    "secret authority status and wallet=0xleak",
                )),
                FakeOutcome::Pending => std::future::pending().await,
            }
        }
    }

    fn assert_opaque_unavailable(error: AppError) {
        assert_eq!(error.kind, ErrorKind::ServiceUnavailable);
        assert_eq!(error.message, AUTHORITY_UNAVAILABLE_MESSAGE);
        assert!(!error.message.contains("secret"));
        assert!(!error.message.contains("0xleak"));
        assert!(!error.message.to_ascii_lowercase().contains("internal"));
    }

    #[tokio::test]
    async fn a2_6_grpc_success_returns_strictly_validated_offset_once() {
        let rpc = FakeRpc::new(FakeOutcome::Offset(50));
        let client = GrpcWalletRankingOffsetQuery::from_rpc(rpc.clone());

        let offset = client
            .get_wallet_ranking_offset("0xverified")
            .await
            .expect("valid authority offset should pass");

        assert_eq!(offset.value(), 50);
        assert_eq!(rpc.calls.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn a2_6_grpc_status_is_opaque_without_fallback_or_retry() {
        let rpc = FakeRpc::new(FakeOutcome::Status);
        let client = GrpcWalletRankingOffsetQuery::from_rpc(rpc.clone());

        let error = client
            .get_wallet_ranking_offset("0xverified")
            .await
            .expect_err("authority status must fail closed");

        assert_opaque_unavailable(error);
        assert_eq!(rpc.calls.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn a2_6_grpc_timeout_is_opaque_without_fallback_or_retry() {
        let rpc = FakeRpc::new(FakeOutcome::Pending);
        let client = GrpcWalletRankingOffsetQuery::from_rpc(rpc.clone());

        let error = client
            .get_wallet_ranking_offset("0xverified")
            .await
            .expect_err("authority timeout must fail closed");

        assert_opaque_unavailable(error);
        assert_eq!(rpc.calls.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn a2_6_invalid_wire_offset_is_rejected_not_clamped() {
        for raw_offset in [-1, 1_001] {
            let rpc = FakeRpc::new(FakeOutcome::Offset(raw_offset));
            let client = GrpcWalletRankingOffsetQuery::from_rpc(rpc.clone());

            let error = client
                .get_wallet_ranking_offset("0xverified")
                .await
                .expect_err("invalid authority offset must fail closed");

            assert_opaque_unavailable(error);
            assert_eq!(rpc.calls.load(Ordering::SeqCst), 1);
        }
    }

    #[tokio::test]
    async fn a2_6_lazy_constructor_accepts_unreachable_uri_without_dialing() {
        let client = GrpcWalletRankingOffsetQuery::new("http://127.0.0.1:1".to_string());
        assert!(client.is_ok(), "lazy construction must not connect");
    }

    #[tokio::test]
    async fn a2_6_constructor_rejects_malformed_uri_opaquely() {
        let secret = "HOSTILE_SECRET";
        let error = match GrpcWalletRankingOffsetQuery::new(format!("not a uri {secret}")) {
            Ok(_) => panic!("malformed identity endpoint must be rejected"),
            Err(error) => error,
        };
        let message = error.to_string();
        assert_eq!(message, "invalid identity gRPC endpoint");
        assert!(!message.contains(secret));
    }
}
