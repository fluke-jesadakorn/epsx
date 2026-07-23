//! Pure ranking-entitlement resolution behind [`WalletRankingOffsetQuery`].
//!
//! A2.7 deliberately stops at the backend policy boundary. The repository
//! supplies one raw, atomic observation; this module decides which plan facts
//! participate and computes the effective offset. There is no PostgreSQL
//! adapter, runtime wiring, clock, retry, cache, transport, or Free-Plan error
//! fallback in this slice.

use std::sync::Arc;

use async_trait::async_trait;
use epsx_contracts::{
    errors::{AppError, AppResult, ErrorKind},
    value_objects::ranking_offset::RankingOffset,
    wallet_ranking_offset_query::WalletRankingOffsetQuery,
};

/// Only permissions beginning with this exact prefix are ranking-offset
/// candidates. Direct, nested, view, and wildcard permissions do not
/// participate.
pub const RANKING_OFFSET_PERMISSION_PREFIX: &str = "epsx:rankings:offset:";

const AUTHORITY_UNAVAILABLE_MESSAGE: &str = "Ranking access authority unavailable";
const AUTHORITY_INVALID_MESSAGE: &str = "Ranking access authority returned invalid data";

/// One atomic repository observation for one normalized wallet.
///
/// `observed_at` comes from the same storage statement as `assignments`; the
/// resolver never reads the process clock. The source assignment model has no
/// effective-start field, so A2.7 intentionally does not invent one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RankingEntitlementSnapshot {
    pub normalized_wallet: String,
    /// Repository observation time as Unix epoch microseconds.
    pub observed_at: i64,
    pub assignments: Vec<RawPlanRankingEntitlement>,
}

/// Raw assignment, plan, legacy metadata, and plan-permission facts.
///
/// IDs are retained so a later storage adapter and reconciliation proof can
/// diagnose duplicate rows without putting identifiers into outward errors.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawPlanRankingEntitlement {
    pub assignment_id: String,
    pub plan_id: String,
    pub assignment_active: bool,
    /// Exclusive expiry boundary as Unix epoch microseconds.
    pub expires_at: Option<i64>,
    pub plan_present: bool,
    pub plan_active: bool,
    pub legacy_metadata_offset: RawLegacyRankingOffset,
    pub permissions: Vec<RawRankingPermission>,
}

/// Presence and raw integer-shape of the legacy `plan_metadata` offset.
///
/// `Invalid` is distinct from `Missing`: malformed configured authority data
/// must fail closed, while a genuinely absent legacy field can be supplemented
/// by a canonical plan permission.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RawLegacyRankingOffset {
    Missing,
    Integer(i64),
    Invalid,
}

/// One raw permission fact attached to a plan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawRankingPermission {
    pub permission_id: String,
    pub active: bool,
    pub permission_string: String,
}

/// Repository failures stay data-source-neutral and carry no database detail.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RankingEntitlementSnapshotError {
    Unavailable,
    Corrupt,
}

/// Narrow input port for the future storage adapter.
///
/// Implementations must return all facts from one atomic storage observation
/// and must not turn a read or decode failure into an empty snapshot.
#[async_trait]
pub trait RankingEntitlementSnapshotRepository: Send + Sync {
    async fn load_snapshot(
        &self,
        normalized_wallet: &str,
    ) -> Result<RankingEntitlementSnapshot, RankingEntitlementSnapshotError>;
}

/// Typed result of the pure entitlement decision.
///
/// `PlanGrant` retains the minimum configured plan offset separately from the
/// effective offset. Therefore a valid configured offset of `500` remains a
/// plan-derived decision while the Free baseline prevents it from reducing
/// access below offset `100`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RankingEntitlementDecision {
    NoEffectivePlan {
        effective_offset: RankingOffset,
    },
    EffectivePlansWithoutGrant {
        effective_offset: RankingOffset,
    },
    PlanGrant {
        minimum_plan_offset: RankingOffset,
        effective_offset: RankingOffset,
    },
}

impl RankingEntitlementDecision {
    pub fn effective_offset(self) -> RankingOffset {
        match self {
            Self::NoEffectivePlan { effective_offset }
            | Self::EffectivePlansWithoutGrant { effective_offset }
            | Self::PlanGrant {
                effective_offset, ..
            } => effective_offset,
        }
    }
}

/// Invalid authority facts discovered by the pure resolver.
///
/// The variants intentionally contain no IDs, wallet, permission text, or raw
/// values. Callers can classify the fault without leaking authority data.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RankingEntitlementResolutionError {
    MissingPlan,
    InvalidLegacyMetadataOffset,
    InvalidPermissionOffset,
}

/// Resolve one fixed-time snapshot without I/O or process-global state.
pub fn resolve_ranking_entitlement(
    snapshot: &RankingEntitlementSnapshot,
) -> Result<RankingEntitlementDecision, RankingEntitlementResolutionError> {
    let free_offset = RankingOffset::free_plan();
    let mut effective_plan_count = 0usize;
    let mut minimum_plan_offset: Option<RankingOffset> = None;

    for assignment in &snapshot.assignments {
        if !assignment_is_relevant(assignment, snapshot.observed_at) {
            continue;
        }

        if !assignment.plan_present {
            return Err(RankingEntitlementResolutionError::MissingPlan);
        }
        if !assignment.plan_active {
            continue;
        }

        effective_plan_count += 1;

        match assignment.legacy_metadata_offset {
            RawLegacyRankingOffset::Missing => {}
            RawLegacyRankingOffset::Integer(raw) => {
                record_candidate(
                    &mut minimum_plan_offset,
                    ranking_offset_from_i64(raw)
                        .ok_or(RankingEntitlementResolutionError::InvalidLegacyMetadataOffset)?,
                );
            }
            RawLegacyRankingOffset::Invalid => {
                return Err(RankingEntitlementResolutionError::InvalidLegacyMetadataOffset);
            }
        }

        for permission in &assignment.permissions {
            if !permission.active {
                continue;
            }
            let Some(raw) = permission
                .permission_string
                .strip_prefix(RANKING_OFFSET_PERMISSION_PREFIX)
            else {
                continue;
            };
            record_candidate(
                &mut minimum_plan_offset,
                parse_permission_offset(raw)
                    .ok_or(RankingEntitlementResolutionError::InvalidPermissionOffset)?,
            );
        }
    }

    match (effective_plan_count, minimum_plan_offset) {
        (0, _) => Ok(RankingEntitlementDecision::NoEffectivePlan {
            effective_offset: free_offset,
        }),
        (_, None) => Ok(RankingEntitlementDecision::EffectivePlansWithoutGrant {
            effective_offset: free_offset,
        }),
        (_, Some(minimum_plan_offset)) => {
            let effective_offset = minimum_plan_offset.min(free_offset);
            Ok(RankingEntitlementDecision::PlanGrant {
                minimum_plan_offset,
                effective_offset,
            })
        }
    }
}

fn assignment_is_relevant(assignment: &RawPlanRankingEntitlement, observed_at: i64) -> bool {
    if !assignment.assignment_active {
        return false;
    }
    match assignment.expires_at {
        Some(expires_at) => expires_at > observed_at,
        None => true,
    }
}

fn ranking_offset_from_i64(raw: i64) -> Option<RankingOffset> {
    i32::try_from(raw)
        .ok()
        .and_then(|raw| RankingOffset::new(raw).ok())
}

fn parse_permission_offset(raw: &str) -> Option<RankingOffset> {
    if raw.is_empty() || !raw.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    let parsed = raw.parse::<i32>().ok()?;
    RankingOffset::new(parsed).ok()
}

fn record_candidate(minimum: &mut Option<RankingOffset>, candidate: RankingOffset) {
    *minimum = Some(match *minimum {
        Some(current) => current.min(candidate),
        None => candidate,
    });
}

/// Adapter from the raw snapshot input port to the existing cross-service
/// ranking-offset query. It performs one normalized repository call and never
/// retries or converts failure into a Free-Plan success.
#[derive(Clone)]
pub struct SnapshotWalletRankingOffsetQuery {
    repository: Arc<dyn RankingEntitlementSnapshotRepository>,
}

impl SnapshotWalletRankingOffsetQuery {
    pub fn new(repository: Arc<dyn RankingEntitlementSnapshotRepository>) -> Self {
        Self { repository }
    }
}

#[async_trait]
impl WalletRankingOffsetQuery for SnapshotWalletRankingOffsetQuery {
    async fn get_wallet_ranking_offset(&self, wallet: &str) -> AppResult<RankingOffset> {
        let normalized_wallet = wallet.to_ascii_lowercase();
        let snapshot = self
            .repository
            .load_snapshot(&normalized_wallet)
            .await
            .map_err(map_repository_error)?;

        if snapshot.normalized_wallet != normalized_wallet {
            return Err(invalid_authority_error());
        }

        resolve_ranking_entitlement(&snapshot)
            .map(RankingEntitlementDecision::effective_offset)
            .map_err(|_| invalid_authority_error())
    }
}

fn map_repository_error(error: RankingEntitlementSnapshotError) -> AppError {
    match error {
        RankingEntitlementSnapshotError::Unavailable => {
            AppError::new(ErrorKind::ServiceUnavailable, AUTHORITY_UNAVAILABLE_MESSAGE)
        }
        RankingEntitlementSnapshotError::Corrupt => invalid_authority_error(),
    }
}

fn invalid_authority_error() -> AppError {
    AppError::new(ErrorKind::InternalServerError, AUTHORITY_INVALID_MESSAGE)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;
    use std::sync::{
        atomic::{AtomicUsize, Ordering},
        Mutex,
    };

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct FixtureLedger {
        observed_at: i64,
        normalized_wallet: String,
        cases: Vec<FixtureCase>,
    }

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct FixtureCase {
        id: String,
        assignments: Vec<FixtureAssignment>,
        expected: FixtureExpected,
    }

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct FixtureAssignment {
        assignment_id: String,
        plan_id: String,
        assignment_active: bool,
        expires_at: Option<i64>,
        plan_present: bool,
        plan_active: bool,
        metadata: FixtureMetadata,
        permissions: Vec<FixturePermission>,
    }

    #[derive(Debug, Deserialize)]
    #[serde(tag = "kind", rename_all = "snake_case")]
    enum FixtureMetadata {
        Missing,
        Integer { value: i64 },
        Invalid,
    }

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct FixturePermission {
        permission_id: String,
        active: bool,
        permission_string: String,
    }

    #[derive(Debug, Deserialize)]
    #[serde(tag = "kind", rename_all = "snake_case")]
    enum FixtureExpected {
        NoEffectivePlan {
            effective_offset: i32,
        },
        EffectivePlansWithoutGrant {
            effective_offset: i32,
        },
        PlanGrant {
            minimum_plan_offset: i32,
            effective_offset: i32,
        },
        Error {
            error: String,
        },
    }

    impl FixtureAssignment {
        fn into_raw(self) -> RawPlanRankingEntitlement {
            RawPlanRankingEntitlement {
                assignment_id: self.assignment_id,
                plan_id: self.plan_id,
                assignment_active: self.assignment_active,
                expires_at: self.expires_at,
                plan_present: self.plan_present,
                plan_active: self.plan_active,
                legacy_metadata_offset: match self.metadata {
                    FixtureMetadata::Missing => RawLegacyRankingOffset::Missing,
                    FixtureMetadata::Integer { value } => RawLegacyRankingOffset::Integer(value),
                    FixtureMetadata::Invalid => RawLegacyRankingOffset::Invalid,
                },
                permissions: self
                    .permissions
                    .into_iter()
                    .map(|permission| RawRankingPermission {
                        permission_id: permission.permission_id,
                        active: permission.active,
                        permission_string: permission.permission_string,
                    })
                    .collect(),
            }
        }
    }

    fn fixture_ledger() -> FixtureLedger {
        serde_json::from_str(include_str!(
            "../../../../docs/migration/fixtures/a2-7-ranking-entitlement-snapshot.json"
        ))
        .expect("A2.7 fixture ledger must be valid JSON")
    }

    fn assert_fixture_expected(
        case_id: &str,
        actual: Result<RankingEntitlementDecision, RankingEntitlementResolutionError>,
        expected: FixtureExpected,
    ) {
        match expected {
            FixtureExpected::NoEffectivePlan { effective_offset } => assert_eq!(
                actual,
                Ok(RankingEntitlementDecision::NoEffectivePlan {
                    effective_offset: RankingOffset::new(effective_offset).unwrap(),
                }),
                "fixture case {case_id}",
            ),
            FixtureExpected::EffectivePlansWithoutGrant { effective_offset } => assert_eq!(
                actual,
                Ok(RankingEntitlementDecision::EffectivePlansWithoutGrant {
                    effective_offset: RankingOffset::new(effective_offset).unwrap(),
                }),
                "fixture case {case_id}",
            ),
            FixtureExpected::PlanGrant {
                minimum_plan_offset,
                effective_offset,
            } => assert_eq!(
                actual,
                Ok(RankingEntitlementDecision::PlanGrant {
                    minimum_plan_offset: RankingOffset::new(minimum_plan_offset).unwrap(),
                    effective_offset: RankingOffset::new(effective_offset).unwrap(),
                }),
                "fixture case {case_id}",
            ),
            FixtureExpected::Error { error } => {
                let expected_error = match error.as_str() {
                    "missing_plan" => RankingEntitlementResolutionError::MissingPlan,
                    "invalid_legacy_metadata_offset" => {
                        RankingEntitlementResolutionError::InvalidLegacyMetadataOffset
                    }
                    "invalid_permission_offset" => {
                        RankingEntitlementResolutionError::InvalidPermissionOffset
                    }
                    other => panic!("unknown fixture error {other} in case {case_id}"),
                };
                assert_eq!(actual, Err(expected_error), "fixture case {case_id}");
            }
        }
    }

    #[test]
    fn a2_7_fixture_ledger_matches_pure_decisions() {
        let ledger = fixture_ledger();
        for case in ledger.cases {
            let snapshot = RankingEntitlementSnapshot {
                normalized_wallet: ledger.normalized_wallet.clone(),
                observed_at: ledger.observed_at,
                assignments: case
                    .assignments
                    .into_iter()
                    .map(FixtureAssignment::into_raw)
                    .collect(),
            };
            assert_fixture_expected(
                &case.id,
                resolve_ranking_entitlement(&snapshot),
                case.expected,
            );
        }
    }

    #[derive(Clone)]
    struct FakeRepository {
        result: Result<RankingEntitlementSnapshot, RankingEntitlementSnapshotError>,
        calls: Arc<AtomicUsize>,
        wallets: Arc<Mutex<Vec<String>>>,
    }

    impl FakeRepository {
        fn new(
            result: Result<RankingEntitlementSnapshot, RankingEntitlementSnapshotError>,
        ) -> Self {
            Self {
                result,
                calls: Arc::new(AtomicUsize::new(0)),
                wallets: Arc::new(Mutex::new(Vec::new())),
            }
        }
    }

    #[async_trait]
    impl RankingEntitlementSnapshotRepository for FakeRepository {
        async fn load_snapshot(
            &self,
            normalized_wallet: &str,
        ) -> Result<RankingEntitlementSnapshot, RankingEntitlementSnapshotError> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            self.wallets
                .lock()
                .unwrap()
                .push(normalized_wallet.to_string());
            self.result.clone()
        }
    }

    fn empty_snapshot(wallet: &str) -> RankingEntitlementSnapshot {
        RankingEntitlementSnapshot {
            normalized_wallet: wallet.to_string(),
            observed_at: 1_784_808_000_000_000,
            assignments: Vec::new(),
        }
    }

    #[tokio::test]
    async fn a2_7_repository_unavailable_maps_to_opaque_service_unavailable() {
        let secret = "postgres://authority-secret";
        let repository = Arc::new(FakeRepository::new(Err(
            RankingEntitlementSnapshotError::Unavailable,
        )));
        let query = SnapshotWalletRankingOffsetQuery::new(repository);

        let error = query
            .get_wallet_ranking_offset(secret)
            .await
            .expect_err("unavailable repository must fail");

        assert_eq!(error.kind, ErrorKind::ServiceUnavailable);
        assert_eq!(error.message, AUTHORITY_UNAVAILABLE_MESSAGE);
        assert!(!error.message.contains(secret));
    }

    #[tokio::test]
    async fn a2_7_repository_corrupt_maps_to_opaque_internal_error() {
        let secret = "corrupt-row-secret";
        let repository = Arc::new(FakeRepository::new(Err(
            RankingEntitlementSnapshotError::Corrupt,
        )));
        let query = SnapshotWalletRankingOffsetQuery::new(repository);

        let error = query
            .get_wallet_ranking_offset(secret)
            .await
            .expect_err("corrupt repository result must fail");

        assert_eq!(error.kind, ErrorKind::InternalServerError);
        assert_eq!(error.message, AUTHORITY_INVALID_MESSAGE);
        assert!(!error.message.contains(secret));
    }

    #[tokio::test]
    async fn a2_7_query_normalizes_wallet_and_calls_repository_once() {
        let repository = Arc::new(FakeRepository::new(Ok(empty_snapshot("0xabcdef"))));
        let calls = Arc::clone(&repository.calls);
        let wallets = Arc::clone(&repository.wallets);
        let query = SnapshotWalletRankingOffsetQuery::new(repository);

        let offset = query
            .get_wallet_ranking_offset("0xAbCdEf")
            .await
            .expect("empty authoritative snapshot is explicit Free success");

        assert_eq!(offset, RankingOffset::free_plan());
        assert_eq!(calls.load(Ordering::SeqCst), 1);
        assert_eq!(&*wallets.lock().unwrap(), &["0xabcdef".to_string()]);
    }

    #[tokio::test]
    async fn a2_7_snapshot_wallet_mismatch_fails_opaquely() {
        let repository = Arc::new(FakeRepository::new(Ok(empty_snapshot("0xother"))));
        let calls = Arc::clone(&repository.calls);
        let query = SnapshotWalletRankingOffsetQuery::new(repository);

        let error = query
            .get_wallet_ranking_offset("0xrequested")
            .await
            .expect_err("cross-wallet snapshot must fail");

        assert_eq!(calls.load(Ordering::SeqCst), 1);
        assert_eq!(error.kind, ErrorKind::InternalServerError);
        assert_eq!(error.message, AUTHORITY_INVALID_MESSAGE);
        assert!(!error.message.contains("0xrequested"));
        assert!(!error.message.contains("0xother"));
    }

    #[tokio::test]
    async fn a2_7_resolution_corruption_maps_to_opaque_internal_error() {
        let wallet = "0xsecretwallet";
        let mut snapshot = empty_snapshot(wallet);
        snapshot.assignments.push(RawPlanRankingEntitlement {
            assignment_id: "secret-assignment".to_string(),
            plan_id: "secret-missing-plan".to_string(),
            assignment_active: true,
            expires_at: None,
            plan_present: false,
            plan_active: false,
            legacy_metadata_offset: RawLegacyRankingOffset::Missing,
            permissions: Vec::new(),
        });
        let repository = Arc::new(FakeRepository::new(Ok(snapshot)));
        let calls = Arc::clone(&repository.calls);
        let query = SnapshotWalletRankingOffsetQuery::new(repository);

        let error = query
            .get_wallet_ranking_offset(wallet)
            .await
            .expect_err("invalid resolved snapshot must fail");

        assert_eq!(calls.load(Ordering::SeqCst), 1);
        assert_eq!(error.kind, ErrorKind::InternalServerError);
        assert_eq!(error.message, AUTHORITY_INVALID_MESSAGE);
        for secret in [wallet, "secret-assignment", "secret-missing-plan"] {
            assert!(!error.message.contains(secret));
        }
    }

    #[test]
    fn a2_7_query_is_dyn_compatible() {
        let repository = Arc::new(FakeRepository::new(Ok(empty_snapshot("0xabc"))));
        let query: Arc<dyn WalletRankingOffsetQuery> =
            Arc::new(SnapshotWalletRankingOffsetQuery::new(repository));
        let _ = query;
    }
}
