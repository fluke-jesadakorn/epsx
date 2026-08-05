//! Unwired, workload-authenticated gRPC composition for ranking offsets.
//!
//! This module defines the fail-closed transport boundary that a future
//! identity runtime can compose with an authoritative ranking query. It does
//! not provide a credential verifier and is deliberately not constructed by
//! `main`: credential issuance, TLS, database activation, and deployment
//! remain separate readiness gates.

use std::sync::Arc;

use async_trait::async_trait;
use epsx_contracts::{
    errors::{AppError, ErrorKind},
    wallet_ranking_offset_query::WalletRankingOffsetQuery,
};
use tonic::{metadata::MetadataMap, Request, Response, Status};

use crate::generated::{
    identity_server::Identity, GetWalletRankingOffsetRequest, GetWalletRankingOffsetResponse,
};

pub const EXPECTED_WORKLOAD_SUBJECT: &str = "epsx-analytics-service";
pub const EXPECTED_WORKLOAD_AUDIENCE: &str = "epsx-identity-service";

const AUTHENTICATION_REQUIRED: &str = "workload authentication required";
const AUTHORIZATION_UNAVAILABLE: &str = "workload authorization unavailable";
const CALLER_FORBIDDEN: &str = "workload caller forbidden";
const INVALID_WALLET: &str = "invalid wallet address";
const AUTHORITY_UNAVAILABLE: &str = "ranking authority unavailable";
const AUTHORITY_INVALID: &str = "ranking authority returned invalid data";
const AUTHORITY_FAILED: &str = "ranking authority failed";

/// Cryptographically verified workload identity returned by a future adapter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifiedWorkload {
    pub subject: String,
    pub audience: String,
}

/// Opaque workload-verifier failures. No credential detail crosses the RPC.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkloadAuthorizationError {
    InvalidCredential,
    Unavailable,
}

/// Workload-specific authentication port.
///
/// Implementations must cryptographically verify the credential signature,
/// issuer, expiry, and intended workload claims. Browser access-token
/// verification is a different authority and must not implement this port by
/// merely widening its accepted audiences.
#[async_trait]
pub trait RankingWorkloadAuthorizer: Send + Sync {
    async fn authorize(&self, bearer: &str)
        -> Result<VerifiedWorkload, WorkloadAuthorizationError>;
}

/// Fail-closed gRPC adapter. It is exported for composition but remains
/// deliberately unwired from the production identity binary.
pub struct AuthenticatedRankingGrpcService {
    authorizer: Arc<dyn RankingWorkloadAuthorizer>,
    query: Arc<dyn WalletRankingOffsetQuery>,
}

impl AuthenticatedRankingGrpcService {
    pub fn new(
        authorizer: Arc<dyn RankingWorkloadAuthorizer>,
        query: Arc<dyn WalletRankingOffsetQuery>,
    ) -> Self {
        Self { authorizer, query }
    }
}

#[async_trait]
impl Identity for AuthenticatedRankingGrpcService {
    async fn get_wallet_ranking_offset(
        &self,
        request: Request<GetWalletRankingOffsetRequest>,
    ) -> Result<Response<GetWalletRankingOffsetResponse>, Status> {
        // Authenticate before reading, validating, or querying for the wallet.
        let bearer = parse_bearer(request.metadata())
            .map_err(|_| Status::unauthenticated(AUTHENTICATION_REQUIRED))?;
        let workload = self
            .authorizer
            .authorize(bearer)
            .await
            .map_err(map_authorization_error)?;

        if workload.subject != EXPECTED_WORKLOAD_SUBJECT
            || workload.audience != EXPECTED_WORKLOAD_AUDIENCE
        {
            return Err(Status::permission_denied(CALLER_FORBIDDEN));
        }

        let normalized_wallet = normalize_evm_wallet(&request.get_ref().wallet)
            .map_err(|_| Status::invalid_argument(INVALID_WALLET))?;
        let offset = self
            .query
            .get_wallet_ranking_offset(&normalized_wallet)
            .await
            .map_err(map_query_error)?;

        Ok(Response::new(GetWalletRankingOffsetResponse {
            offset: offset.value(),
        }))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct InvalidBearer;

fn parse_bearer(metadata: &MetadataMap) -> Result<&str, InvalidBearer> {
    let mut values = metadata.get_all("authorization").iter();
    let value = values.next().ok_or(InvalidBearer)?;
    if values.next().is_some() {
        return Err(InvalidBearer);
    }

    let value = value.to_str().map_err(|_| InvalidBearer)?;
    let bearer = value.strip_prefix("Bearer ").ok_or(InvalidBearer)?;
    if bearer.is_empty() || bearer.bytes().any(|byte| byte.is_ascii_whitespace()) {
        return Err(InvalidBearer);
    }
    Ok(bearer)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct InvalidWallet;

fn normalize_evm_wallet(wallet: &str) -> Result<String, InvalidWallet> {
    let bytes = wallet.as_bytes();
    if bytes.len() != 42 || &bytes[..2] != b"0x" || !bytes[2..].iter().all(u8::is_ascii_hexdigit) {
        return Err(InvalidWallet);
    }
    Ok(wallet.to_ascii_lowercase())
}

fn map_authorization_error(error: WorkloadAuthorizationError) -> Status {
    match error {
        WorkloadAuthorizationError::InvalidCredential => {
            Status::unauthenticated(AUTHENTICATION_REQUIRED)
        }
        WorkloadAuthorizationError::Unavailable => Status::unavailable(AUTHORIZATION_UNAVAILABLE),
    }
}

fn map_query_error(error: AppError) -> Status {
    match error.kind {
        ErrorKind::ServiceUnavailable => Status::unavailable(AUTHORITY_UNAVAILABLE),
        ErrorKind::InternalServerError => Status::internal(AUTHORITY_INVALID),
        _ => Status::internal(AUTHORITY_FAILED),
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{
        atomic::{AtomicUsize, Ordering},
        Mutex,
    };

    use epsx_contracts::{
        errors::{AppResult, ErrorKind},
        value_objects::ranking_offset::RankingOffset,
    };
    use prost::Message;
    use tonic::{metadata::MetadataValue, Code};

    use super::*;

    #[derive(Clone, Copy)]
    enum AuthorizationOutcome {
        Allowed,
        Invalid,
        Unavailable,
        WrongSubject,
        WrongAudience,
    }

    struct FakeAuthorizer {
        outcome: AuthorizationOutcome,
        calls: AtomicUsize,
    }

    impl FakeAuthorizer {
        fn new(outcome: AuthorizationOutcome) -> Self {
            Self {
                outcome,
                calls: AtomicUsize::new(0),
            }
        }

        fn calls(&self) -> usize {
            self.calls.load(Ordering::SeqCst)
        }
    }

    #[async_trait]
    impl RankingWorkloadAuthorizer for FakeAuthorizer {
        async fn authorize(
            &self,
            _bearer: &str,
        ) -> Result<VerifiedWorkload, WorkloadAuthorizationError> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            match self.outcome {
                AuthorizationOutcome::Allowed => Ok(allowed_workload()),
                AuthorizationOutcome::Invalid => Err(WorkloadAuthorizationError::InvalidCredential),
                AuthorizationOutcome::Unavailable => Err(WorkloadAuthorizationError::Unavailable),
                AuthorizationOutcome::WrongSubject => Ok(VerifiedWorkload {
                    subject: "not-the-market-service".into(),
                    audience: EXPECTED_WORKLOAD_AUDIENCE.into(),
                }),
                AuthorizationOutcome::WrongAudience => Ok(VerifiedWorkload {
                    subject: EXPECTED_WORKLOAD_SUBJECT.into(),
                    audience: "not-the-identity-service".into(),
                }),
            }
        }
    }

    #[derive(Clone, Copy)]
    enum QueryOutcome {
        Offset(i32),
        Unavailable,
        Corrupt,
        Unexpected,
    }

    struct FakeQuery {
        outcome: QueryOutcome,
        calls: AtomicUsize,
        wallets: Mutex<Vec<String>>,
    }

    impl FakeQuery {
        fn new(outcome: QueryOutcome) -> Self {
            Self {
                outcome,
                calls: AtomicUsize::new(0),
                wallets: Mutex::new(Vec::new()),
            }
        }

        fn calls(&self) -> usize {
            self.calls.load(Ordering::SeqCst)
        }

        fn wallets(&self) -> Vec<String> {
            self.wallets
                .lock()
                .expect("wallet recorder poisoned")
                .clone()
        }
    }

    #[async_trait]
    impl WalletRankingOffsetQuery for FakeQuery {
        async fn get_wallet_ranking_offset(&self, wallet: &str) -> AppResult<RankingOffset> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            self.wallets
                .lock()
                .expect("wallet recorder poisoned")
                .push(wallet.to_string());
            match self.outcome {
                QueryOutcome::Offset(offset) => {
                    Ok(RankingOffset::new(offset).expect("valid offset"))
                }
                QueryOutcome::Unavailable => Err(AppError::new(
                    ErrorKind::ServiceUnavailable,
                    "database address and credential must stay private",
                )),
                QueryOutcome::Corrupt => Err(AppError::internal_server_error(
                    "corrupt row identifier must stay private",
                )),
                QueryOutcome::Unexpected => Err(AppError::database_error(
                    "unexpected SQL detail must stay private",
                )),
            }
        }
    }

    fn allowed_workload() -> VerifiedWorkload {
        VerifiedWorkload {
            subject: EXPECTED_WORKLOAD_SUBJECT.into(),
            audience: EXPECTED_WORKLOAD_AUDIENCE.into(),
        }
    }

    fn request(wallet: &str) -> Request<GetWalletRankingOffsetRequest> {
        Request::new(GetWalletRankingOffsetRequest {
            wallet: wallet.to_string(),
        })
    }

    fn authorized_request(wallet: &str) -> Request<GetWalletRankingOffsetRequest> {
        let mut request = request(wallet);
        request.metadata_mut().insert(
            "authorization",
            MetadataValue::from_static("Bearer test-token"),
        );
        request
    }

    fn service(
        authorization: AuthorizationOutcome,
        query_outcome: QueryOutcome,
    ) -> (
        AuthenticatedRankingGrpcService,
        Arc<FakeAuthorizer>,
        Arc<FakeQuery>,
    ) {
        let authorizer = Arc::new(FakeAuthorizer::new(authorization));
        let query = Arc::new(FakeQuery::new(query_outcome));
        let service = AuthenticatedRankingGrpcService::new(authorizer.clone(), query.clone());
        (service, authorizer, query)
    }

    fn assert_status(status: Status, code: Code, message: &str) {
        assert_eq!(status.code(), code);
        assert_eq!(status.message(), message);
    }

    const VALID_WALLET: &str = "0xabcdefabcdefabcdefabcdefabcdefabcdefabcd";

    #[tokio::test]
    async fn a2_10_missing_metadata_is_unauthenticated_before_authorizer_or_query() {
        let (service, authorizer, query) =
            service(AuthorizationOutcome::Allowed, QueryOutcome::Offset(7));
        let status = service
            .get_wallet_ranking_offset(request("invalid-wallet"))
            .await
            .expect_err("missing metadata must fail");

        assert_status(status, Code::Unauthenticated, AUTHENTICATION_REQUIRED);
        assert_eq!(authorizer.calls(), 0);
        assert_eq!(query.calls(), 0);
    }

    #[tokio::test]
    async fn a2_10_duplicate_or_malformed_bearer_is_rejected_before_authorizer_or_query() {
        let (service, authorizer, query) =
            service(AuthorizationOutcome::Allowed, QueryOutcome::Offset(7));
        let mut duplicate = authorized_request(VALID_WALLET);
        duplicate.metadata_mut().append(
            "authorization",
            MetadataValue::from_static("Bearer second-token"),
        );
        let status = service
            .get_wallet_ranking_offset(duplicate)
            .await
            .expect_err("duplicate metadata must fail");
        assert_status(status, Code::Unauthenticated, AUTHENTICATION_REQUIRED);

        for value in ["bearer test-token", "Bearer ", "Bearer two tokens"] {
            let mut malformed = request(VALID_WALLET);
            malformed.metadata_mut().insert(
                "authorization",
                MetadataValue::try_from(value).expect("valid ASCII test metadata"),
            );
            let status = service
                .get_wallet_ranking_offset(malformed)
                .await
                .expect_err("malformed bearer must fail");
            assert_status(status, Code::Unauthenticated, AUTHENTICATION_REQUIRED);
        }

        let mut non_ascii = request(VALID_WALLET);
        non_ascii.metadata_mut().insert(
            "authorization",
            MetadataValue::try_from(&b"Bearer \xff"[..])
                .expect("opaque ASCII-metadata bytes are accepted by tonic"),
        );
        let status = service
            .get_wallet_ranking_offset(non_ascii)
            .await
            .expect_err("non-ASCII bearer must fail");
        assert_status(status, Code::Unauthenticated, AUTHENTICATION_REQUIRED);

        assert_eq!(authorizer.calls(), 0);
        assert_eq!(query.calls(), 0);
    }

    #[tokio::test]
    async fn a2_10_invalid_credential_precedes_wallet_validation_and_query() {
        let (service, authorizer, query) =
            service(AuthorizationOutcome::Invalid, QueryOutcome::Offset(7));
        let status = service
            .get_wallet_ranking_offset(authorized_request("invalid-wallet"))
            .await
            .expect_err("invalid credential must fail");

        assert_status(status, Code::Unauthenticated, AUTHENTICATION_REQUIRED);
        assert_eq!(authorizer.calls(), 1);
        assert_eq!(query.calls(), 0);
    }

    #[tokio::test]
    async fn a2_10_authorizer_unavailable_precedes_wallet_validation_and_query() {
        let (service, authorizer, query) =
            service(AuthorizationOutcome::Unavailable, QueryOutcome::Offset(7));
        let status = service
            .get_wallet_ranking_offset(authorized_request("invalid-wallet"))
            .await
            .expect_err("unavailable authorizer must fail");

        assert_status(status, Code::Unavailable, AUTHORIZATION_UNAVAILABLE);
        assert_eq!(authorizer.calls(), 1);
        assert_eq!(query.calls(), 0);
    }

    #[tokio::test]
    async fn a2_10_wrong_subject_or_audience_is_permission_denied_before_query() {
        for outcome in [
            AuthorizationOutcome::WrongSubject,
            AuthorizationOutcome::WrongAudience,
        ] {
            let (service, authorizer, query) = service(outcome, QueryOutcome::Offset(7));
            let status = service
                .get_wallet_ranking_offset(authorized_request(VALID_WALLET))
                .await
                .expect_err("wrong workload identity must fail");

            assert_status(status, Code::PermissionDenied, CALLER_FORBIDDEN);
            assert_eq!(authorizer.calls(), 1);
            assert_eq!(query.calls(), 0);
        }
    }

    #[tokio::test]
    async fn a2_10_invalid_evm_wallet_is_invalid_argument_without_query() {
        for wallet in [
            "",
            "0x1234",
            "0Xabcdefabcdefabcdefabcdefabcdefabcdefabcd",
            "0xabcdefabcdefabcdefabcdefabcdefabcdefabcg",
            " vitalik.eth",
        ] {
            let (service, authorizer, query) =
                service(AuthorizationOutcome::Allowed, QueryOutcome::Offset(7));
            let status = service
                .get_wallet_ranking_offset(authorized_request(wallet))
                .await
                .expect_err("invalid wallet must fail");

            assert_status(status, Code::InvalidArgument, INVALID_WALLET);
            assert_eq!(authorizer.calls(), 1);
            assert_eq!(query.calls(), 0);
        }
    }

    #[tokio::test]
    async fn a2_10_mixed_case_wallet_is_normalized_once_and_queried_once() {
        let mixed = "0xAbCdEfAbCdEfAbCdEfAbCdEfAbCdEfAbCdEfAbCd";
        let (service, authorizer, query) =
            service(AuthorizationOutcome::Allowed, QueryOutcome::Offset(7));
        let response = service
            .get_wallet_ranking_offset(authorized_request(mixed))
            .await
            .expect("authorized query succeeds")
            .into_inner();

        assert_eq!(response.offset, 7);
        assert_eq!(authorizer.calls(), 1);
        assert_eq!(query.calls(), 1);
        assert_eq!(query.wallets(), vec![mixed.to_ascii_lowercase()]);
    }

    #[tokio::test]
    async fn a2_10_query_unavailable_maps_to_sanitized_unavailable() {
        let (service, authorizer, query) =
            service(AuthorizationOutcome::Allowed, QueryOutcome::Unavailable);
        let status = service
            .get_wallet_ranking_offset(authorized_request(VALID_WALLET))
            .await
            .expect_err("unavailable authority must fail");

        assert_status(status, Code::Unavailable, AUTHORITY_UNAVAILABLE);
        assert_eq!(authorizer.calls(), 1);
        assert_eq!(query.calls(), 1);
    }

    #[tokio::test]
    async fn a2_10_corrupt_query_maps_to_sanitized_internal() {
        let (service, authorizer, query) =
            service(AuthorizationOutcome::Allowed, QueryOutcome::Corrupt);
        let status = service
            .get_wallet_ranking_offset(authorized_request(VALID_WALLET))
            .await
            .expect_err("corrupt authority must fail");

        assert_status(status, Code::Internal, AUTHORITY_INVALID);
        assert_eq!(authorizer.calls(), 1);
        assert_eq!(query.calls(), 1);
    }

    #[tokio::test]
    async fn a2_10_unexpected_query_error_is_sanitized_internal() {
        let (service, authorizer, query) =
            service(AuthorizationOutcome::Allowed, QueryOutcome::Unexpected);
        let status = service
            .get_wallet_ranking_offset(authorized_request(VALID_WALLET))
            .await
            .expect_err("unexpected authority failure must fail");

        assert_status(status, Code::Internal, AUTHORITY_FAILED);
        assert_eq!(authorizer.calls(), 1);
        assert_eq!(query.calls(), 1);
    }

    #[tokio::test]
    async fn a2_10_authorization_uses_metadata_without_proto_field_changes() {
        let proto = include_str!("../../../proto/identity.proto");
        assert!(!proto.contains("authorization"));
        assert!(!proto.contains("credential"));

        let message = GetWalletRankingOffsetRequest {
            wallet: VALID_WALLET.into(),
        };
        let encoded_before = message.encode_to_vec();
        let mut request = Request::new(message);
        request.metadata_mut().insert(
            "authorization",
            MetadataValue::from_static("Bearer test-token"),
        );
        assert_eq!(request.get_ref().encode_to_vec(), encoded_before);

        let (service, authorizer, query) =
            service(AuthorizationOutcome::Allowed, QueryOutcome::Offset(5));
        let response = service
            .get_wallet_ranking_offset(request)
            .await
            .expect("metadata-authenticated request succeeds")
            .into_inner();
        assert_eq!(response.offset, 5);
        assert_eq!(authorizer.calls(), 1);
        assert_eq!(query.calls(), 1);
    }
}
