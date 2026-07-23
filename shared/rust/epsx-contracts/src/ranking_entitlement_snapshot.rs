//! Raw, owner-neutral ranking-entitlement snapshot contract.
//!
//! The core database adapter supplies these facts in one observation. Policy
//! remains in the identity service, so this contract intentionally contains no
//! SQL, clock, retry, cache, or ranking decision logic.

use async_trait::async_trait;

/// One atomic repository observation for one normalized wallet.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RankingEntitlementSnapshot {
    pub normalized_wallet: String,
    /// Repository observation time as Unix epoch microseconds.
    pub observed_at: i64,
    pub assignments: Vec<RawPlanRankingEntitlement>,
}

/// Raw assignment, plan, legacy metadata, and plan-permission facts.
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

/// Presence and raw integer shape of the legacy `plan_metadata` offset.
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

/// Narrow input port for one atomic storage observation.
///
/// The caller supplies an already-normalized wallet. Implementations must
/// return every raw assignment, expiry, plan, metadata, and permission fact
/// from one atomic observation. A read or decode failure must
/// be returned as an error and must never be converted to an empty snapshot.
#[async_trait]
pub trait RankingEntitlementSnapshotRepository: Send + Sync {
    async fn load_snapshot(
        &self,
        normalized_wallet: &str,
    ) -> Result<RankingEntitlementSnapshot, RankingEntitlementSnapshotError>;
}
