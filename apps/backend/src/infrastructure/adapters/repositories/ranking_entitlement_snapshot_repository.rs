//! Core-owned PostgreSQL adapter for atomic ranking-entitlement snapshots.
//!
//! This adapter is compiled but deliberately not wired into either runtime.
//! Its single read-only statement observes assignment, plan, and permission
//! rows at one database timestamp; pure Rust then validates and groups them.

use std::{collections::BTreeMap, sync::Arc};

use async_trait::async_trait;
use diesel::QueryableByName;
use diesel_async::RunQueryDsl;
use epsx_contracts::ranking_entitlement_snapshot::{
    RankingEntitlementSnapshot, RankingEntitlementSnapshotError,
    RankingEntitlementSnapshotRepository, RawLegacyRankingOffset, RawPlanRankingEntitlement,
    RawRankingPermission,
};
use serde_json::Value;

use crate::prelude::TlsPool;

/// Exactly one schema-qualified, read-only observation statement.
///
/// There is intentionally no effective-row filter: inactive, expired, missing
/// plan, inactive permission, and unrelated permission facts all reach the
/// policy resolver unchanged. The observation CTE also guarantees a sentinel
/// row when a wallet has no assignments.
pub const RANKING_ENTITLEMENT_SNAPSHOT_SQL: &str = r#"
WITH observation AS MATERIALIZED (
    SELECT
        LOWER($1::text) AS normalized_wallet,
        statement_timestamp() AS observed_at
)
SELECT
    observation.normalized_wallet,
    (EXTRACT(EPOCH FROM observation.observed_at) * 1000000)::bigint AS observed_at_micros,
    assignment.wallet_address AS assignment_wallet,
    assignment.id::text AS assignment_id,
    assignment.plan_id::text AS assignment_plan_id,
    assignment.is_active AS assignment_active,
    (EXTRACT(EPOCH FROM assignment.expires_at) * 1000000)::bigint AS expires_at_micros,
    plan.id::text AS joined_plan_id,
    plan.is_active AS plan_active,
    plan.plan_metadata,
    plan_permission.id::text AS plan_permission_link_id,
    plan_permission.permission_id::text AS linked_permission_id,
    permission.id::text AS permission_id,
    permission.is_active AS permission_active,
    permission.permission_string
FROM observation
LEFT JOIN public.wallet_plan_assignments AS assignment
    ON LOWER(assignment.wallet_address) = observation.normalized_wallet
LEFT JOIN public.plans AS plan
    ON plan.id = assignment.plan_id
LEFT JOIN public.plan_permissions AS plan_permission
    ON plan_permission.plan_id = plan.id
LEFT JOIN public.permissions AS permission
    ON permission.id = plan_permission.permission_id
ORDER BY
    assignment.id NULLS FIRST,
    plan_permission.id NULLS FIRST,
    permission.id NULLS FIRST
"#;

#[derive(Debug, Clone, PartialEq, QueryableByName)]
#[cfg_attr(test, derive(serde::Deserialize))]
#[cfg_attr(test, serde(rename_all = "camelCase"))]
struct SnapshotRow {
    #[diesel(sql_type = diesel::sql_types::Text)]
    normalized_wallet: String,
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    observed_at_micros: i64,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    assignment_wallet: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    assignment_id: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    assignment_plan_id: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Bool>)]
    assignment_active: Option<bool>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::BigInt>)]
    expires_at_micros: Option<i64>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    joined_plan_id: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Bool>)]
    plan_active: Option<bool>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Jsonb>)]
    plan_metadata: Option<Value>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    plan_permission_link_id: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    linked_permission_id: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    permission_id: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Bool>)]
    permission_active: Option<bool>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    permission_string: Option<String>,
}

/// Unwired core-owned adapter using the existing primary-database TLS pool.
#[derive(Clone)]
pub struct PostgresRankingEntitlementSnapshotRepository {
    pool: Arc<&'static TlsPool>,
}

impl PostgresRankingEntitlementSnapshotRepository {
    pub fn new(pool: Arc<&'static TlsPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl RankingEntitlementSnapshotRepository for PostgresRankingEntitlementSnapshotRepository {
    async fn load_snapshot(
        &self,
        normalized_wallet: &str,
    ) -> Result<RankingEntitlementSnapshot, RankingEntitlementSnapshotError> {
        let normalized_wallet = normalized_wallet.to_ascii_lowercase();
        let mut connection = self
            .pool
            .get()
            .await
            .map_err(|_| RankingEntitlementSnapshotError::Unavailable)?;

        let rows = diesel::sql_query(RANKING_ENTITLEMENT_SNAPSHOT_SQL)
            .bind::<diesel::sql_types::Text, _>(&normalized_wallet)
            .load::<SnapshotRow>(&mut connection)
            .await
            .map_err(|_| RankingEntitlementSnapshotError::Unavailable)?;

        snapshot_from_rows(&normalized_wallet, rows)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PermissionAccumulator {
    link_id: String,
    permission: RawRankingPermission,
}

#[derive(Debug, Clone, PartialEq)]
struct AssignmentAccumulator {
    assignment_wallet: String,
    plan_id: String,
    assignment_active: bool,
    expires_at: Option<i64>,
    plan_present: bool,
    plan_active: bool,
    legacy_metadata_offset: RawLegacyRankingOffset,
    permission_link_targets: BTreeMap<String, String>,
    permissions: BTreeMap<String, PermissionAccumulator>,
}

fn snapshot_from_rows(
    expected_wallet: &str,
    rows: Vec<SnapshotRow>,
) -> Result<RankingEntitlementSnapshot, RankingEntitlementSnapshotError> {
    let Some(first) = rows.as_slice().first() else {
        return Err(RankingEntitlementSnapshotError::Corrupt);
    };
    let observed_at = first.observed_at_micros;
    let mut assignments = BTreeMap::<String, AssignmentAccumulator>::new();
    let mut saw_sentinel = false;

    for row in rows {
        if row.normalized_wallet != expected_wallet || row.observed_at_micros != observed_at {
            return Err(RankingEntitlementSnapshotError::Corrupt);
        }

        let Some(assignment_id) = row.assignment_id.clone() else {
            if !is_clean_sentinel(&row) || saw_sentinel || !assignments.is_empty() {
                return Err(RankingEntitlementSnapshotError::Corrupt);
            }
            saw_sentinel = true;
            continue;
        };
        if saw_sentinel {
            return Err(RankingEntitlementSnapshotError::Corrupt);
        }

        let assignment_wallet = row
            .assignment_wallet
            .clone()
            .ok_or(RankingEntitlementSnapshotError::Corrupt)?;
        if assignment_wallet.to_ascii_lowercase() != expected_wallet {
            return Err(RankingEntitlementSnapshotError::Corrupt);
        }
        let plan_id = row
            .assignment_plan_id
            .clone()
            .ok_or(RankingEntitlementSnapshotError::Corrupt)?;
        let assignment_active = row
            .assignment_active
            .ok_or(RankingEntitlementSnapshotError::Corrupt)?;

        let (plan_present, plan_active, legacy_metadata_offset) = match &row.joined_plan_id {
            None => {
                if row.plan_active.is_some()
                    || row.plan_metadata.is_some()
                    || has_any_permission_fact(&row)
                {
                    return Err(RankingEntitlementSnapshotError::Corrupt);
                }
                (false, false, RawLegacyRankingOffset::Missing)
            }
            Some(joined_plan_id) => {
                if joined_plan_id != &plan_id {
                    return Err(RankingEntitlementSnapshotError::Corrupt);
                }
                let plan_active = row
                    .plan_active
                    .ok_or(RankingEntitlementSnapshotError::Corrupt)?;
                let metadata = row
                    .plan_metadata
                    .as_ref()
                    .ok_or(RankingEntitlementSnapshotError::Corrupt)?;
                (true, plan_active, metadata_offset(metadata))
            }
        };

        let candidate = AssignmentAccumulator {
            assignment_wallet,
            plan_id,
            assignment_active,
            expires_at: row.expires_at_micros,
            plan_present,
            plan_active,
            legacy_metadata_offset,
            permission_link_targets: BTreeMap::new(),
            permissions: BTreeMap::new(),
        };
        let assignment = assignments
            .entry(assignment_id)
            .or_insert(candidate.clone());
        if !same_assignment_facts(assignment, &candidate) {
            return Err(RankingEntitlementSnapshotError::Corrupt);
        }
        absorb_permission(assignment, &row)?;
    }

    if saw_sentinel {
        return Ok(RankingEntitlementSnapshot {
            normalized_wallet: expected_wallet.to_string(),
            observed_at,
            assignments: Vec::new(),
        });
    }

    let assignments = assignments
        .into_iter()
        .map(|(assignment_id, assignment)| RawPlanRankingEntitlement {
            assignment_id,
            plan_id: assignment.plan_id,
            assignment_active: assignment.assignment_active,
            expires_at: assignment.expires_at,
            plan_present: assignment.plan_present,
            plan_active: assignment.plan_active,
            legacy_metadata_offset: assignment.legacy_metadata_offset,
            permissions: assignment
                .permissions
                .into_values()
                .map(|permission| permission.permission)
                .collect(),
        })
        .collect();

    Ok(RankingEntitlementSnapshot {
        normalized_wallet: expected_wallet.to_string(),
        observed_at,
        assignments,
    })
}

fn same_assignment_facts(left: &AssignmentAccumulator, right: &AssignmentAccumulator) -> bool {
    left.assignment_wallet == right.assignment_wallet
        && left.plan_id == right.plan_id
        && left.assignment_active == right.assignment_active
        && left.expires_at == right.expires_at
        && left.plan_present == right.plan_present
        && left.plan_active == right.plan_active
        && left.legacy_metadata_offset == right.legacy_metadata_offset
}

fn absorb_permission(
    assignment: &mut AssignmentAccumulator,
    row: &SnapshotRow,
) -> Result<(), RankingEntitlementSnapshotError> {
    let Some(link_id) = row.plan_permission_link_id.clone() else {
        if row.linked_permission_id.is_some()
            || row.permission_id.is_some()
            || row.permission_active.is_some()
            || row.permission_string.is_some()
        {
            return Err(RankingEntitlementSnapshotError::Corrupt);
        }
        return Ok(());
    };

    let linked_permission_id = row
        .linked_permission_id
        .clone()
        .ok_or(RankingEntitlementSnapshotError::Corrupt)?;
    let permission_id = row
        .permission_id
        .clone()
        .ok_or(RankingEntitlementSnapshotError::Corrupt)?;
    if linked_permission_id != permission_id {
        return Err(RankingEntitlementSnapshotError::Corrupt);
    }
    let permission = RawRankingPermission {
        permission_id: permission_id.clone(),
        active: row
            .permission_active
            .ok_or(RankingEntitlementSnapshotError::Corrupt)?,
        permission_string: row
            .permission_string
            .clone()
            .ok_or(RankingEntitlementSnapshotError::Corrupt)?,
    };

    if let Some(existing_permission_id) = assignment.permission_link_targets.get(&link_id) {
        if existing_permission_id != &permission_id {
            return Err(RankingEntitlementSnapshotError::Corrupt);
        }
    } else {
        assignment
            .permission_link_targets
            .insert(link_id.clone(), permission_id.clone());
    }

    let candidate = PermissionAccumulator {
        link_id,
        permission,
    };
    if let Some(existing) = assignment.permissions.get(&permission_id) {
        if existing != &candidate {
            return Err(RankingEntitlementSnapshotError::Corrupt);
        }
    } else {
        assignment.permissions.insert(permission_id, candidate);
    }
    Ok(())
}

fn metadata_offset(metadata: &Value) -> RawLegacyRankingOffset {
    let Some(object) = metadata.as_object() else {
        return RawLegacyRankingOffset::Invalid;
    };
    match object.get("ranking_offset") {
        None => RawLegacyRankingOffset::Missing,
        Some(Value::Number(number)) => number
            .as_i64()
            .map(RawLegacyRankingOffset::Integer)
            .unwrap_or(RawLegacyRankingOffset::Invalid),
        Some(_) => RawLegacyRankingOffset::Invalid,
    }
}

fn has_any_permission_fact(row: &SnapshotRow) -> bool {
    row.plan_permission_link_id.is_some()
        || row.linked_permission_id.is_some()
        || row.permission_id.is_some()
        || row.permission_active.is_some()
        || row.permission_string.is_some()
}

fn is_clean_sentinel(row: &SnapshotRow) -> bool {
    row.assignment_wallet.is_none()
        && row.assignment_plan_id.is_none()
        && row.assignment_active.is_none()
        && row.expires_at_micros.is_none()
        && row.joined_plan_id.is_none()
        && row.plan_active.is_none()
        && row.plan_metadata.is_none()
        && !has_any_permission_fact(row)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct FixtureLedger {
        normalized_wallet: String,
        observed_at_micros: i64,
        cases: Vec<FixtureCase>,
    }

    #[derive(Debug, Deserialize)]
    struct FixtureCase {
        id: String,
        rows: Vec<SnapshotRow>,
    }

    fn ledger() -> FixtureLedger {
        serde_json::from_str(include_str!(
            "../../../../../../docs/migration/fixtures/a2-8-ranking-entitlement-rows.json"
        ))
        .expect("A2.8 row fixture must be valid JSON")
    }

    fn rows(case_id: &str) -> (FixtureLedger, Vec<SnapshotRow>) {
        let mut ledger = ledger();
        let index = ledger
            .cases
            .iter()
            .position(|case| case.id == case_id)
            .unwrap_or_else(|| panic!("missing A2.8 fixture case {case_id}"));
        let case = ledger.cases.remove(index);
        (ledger, case.rows)
    }

    fn map_case(
        case_id: &str,
    ) -> Result<RankingEntitlementSnapshot, RankingEntitlementSnapshotError> {
        let (ledger, rows) = rows(case_id);
        assert!(rows.iter().all(|row| {
            row.observed_at_micros == ledger.observed_at_micros
                || case_id == "inconsistent-observation"
        }));
        snapshot_from_rows(&ledger.normalized_wallet, rows)
    }

    #[test]
    fn a2_8_sql_is_one_read_only_schema_qualified_statement() {
        let sql = RANKING_ENTITLEMENT_SNAPSHOT_SQL.to_ascii_lowercase();
        assert!(!sql.contains(';'));
        for forbidden in [" insert ", " update ", " delete ", " merge ", " for update"] {
            assert!(!format!(" {sql} ").contains(forbidden));
        }
        assert!(sql.contains("with observation as materialized"));
        assert!(sql.contains("statement_timestamp()"));
        assert!(sql.contains("lower($1::text)"));
        assert_eq!(sql.matches("left join public.").count(), 4);
        for table in [
            "public.wallet_plan_assignments",
            "public.plans",
            "public.plan_permissions",
            "public.permissions",
        ] {
            assert!(sql.contains(table));
        }
        assert!(!sql.contains(" where "));
        assert!(sql.contains("order by"));
    }

    #[test]
    fn a2_8_sentinel_empty_wallet_maps_to_empty_snapshot() {
        let snapshot = map_case("sentinel-empty").unwrap();
        assert_eq!(snapshot.normalized_wallet, "0xabcdef");
        assert_eq!(snapshot.observed_at, 1_784_808_000_000_001);
        assert!(snapshot.assignments.is_empty());
    }

    #[test]
    fn a2_8_grouping_and_permissions_are_stable() {
        let snapshot = map_case("grouping-stable-permissions").unwrap();
        let assignment_ids: Vec<_> = snapshot
            .assignments
            .iter()
            .map(|assignment| assignment.assignment_id.as_str())
            .collect();
        assert_eq!(assignment_ids, ["assignment-a", "assignment-b"]);
        let permission_ids: Vec<_> = snapshot.assignments[0]
            .permissions
            .iter()
            .map(|permission| permission.permission_id.as_str())
            .collect();
        assert_eq!(permission_ids, ["permission-a", "permission-z"]);
        assert_eq!(
            snapshot.assignments[0].permissions[0].permission_string,
            "epsx:rankings:view:5"
        );
    }

    #[test]
    fn a2_8_equivalent_duplicate_rows_are_idempotent() {
        let snapshot = map_case("equivalent-duplicates").unwrap();
        assert_eq!(snapshot.assignments.len(), 1);
        assert_eq!(snapshot.assignments[0].permissions.len(), 1);
    }

    #[test]
    fn a2_8_zero_rows_are_corrupt_not_empty_success() {
        assert_eq!(
            snapshot_from_rows("0xabcdef", Vec::new()),
            Err(RankingEntitlementSnapshotError::Corrupt)
        );
    }

    #[test]
    fn a2_8_conflicting_duplicate_rows_are_corrupt() {
        assert_eq!(
            map_case("conflicting-assignment-duplicates"),
            Err(RankingEntitlementSnapshotError::Corrupt)
        );
        assert_eq!(
            map_case("conflicting-permission-duplicates"),
            Err(RankingEntitlementSnapshotError::Corrupt)
        );
    }

    #[test]
    fn a2_8_missing_plan_is_preserved_without_invented_facts() {
        let snapshot = map_case("missing-plan").unwrap();
        let assignment = &snapshot.assignments[0];
        assert!(!assignment.plan_present);
        assert!(!assignment.plan_active);
        assert_eq!(
            assignment.legacy_metadata_offset,
            RawLegacyRankingOffset::Missing
        );
        assert!(assignment.permissions.is_empty());
    }

    #[test]
    fn a2_8_inactive_expired_and_inactive_permission_facts_are_preserved() {
        let snapshot = map_case("inactive-expired-facts").unwrap();
        let assignment = &snapshot.assignments[0];
        assert!(!assignment.assignment_active);
        assert_eq!(assignment.expires_at, Some(1_784_807_999_999_999));
        assert!(!assignment.plan_active);
        assert_eq!(assignment.permissions.len(), 1);
        assert!(!assignment.permissions[0].active);
        assert_eq!(
            assignment.permissions[0].permission_string,
            "epsx:rankings:offset:5"
        );
    }

    #[test]
    fn a2_8_metadata_shapes_remain_missing_integer_or_invalid() {
        let snapshot = map_case("metadata-lossless-shapes").unwrap();
        let shapes: Vec<_> = snapshot
            .assignments
            .iter()
            .map(|assignment| assignment.legacy_metadata_offset)
            .collect();
        assert_eq!(
            shapes,
            [
                RawLegacyRankingOffset::Missing,
                RawLegacyRankingOffset::Integer(5),
                RawLegacyRankingOffset::Invalid,
                RawLegacyRankingOffset::Invalid,
            ]
        );
    }

    #[test]
    fn a2_8_dangling_and_partial_rows_are_corrupt() {
        for case_id in [
            "dangling-permission",
            "partial-assignment",
            "partial-plan",
            "mismatched-joined-plan",
            "permission-fields-without-link",
            "linked-permission-mismatch",
            "null-present-plan-metadata",
            "missing-assignment-plan-id",
            "missing-assignment-active",
        ] {
            assert_eq!(
                map_case(case_id),
                Err(RankingEntitlementSnapshotError::Corrupt),
                "fixture case {case_id}"
            );
        }
    }

    #[test]
    fn a2_8_sentinel_cardinality_and_mixed_rows_are_corrupt() {
        for case_id in ["duplicate-sentinel", "mixed-sentinel-and-assignment"] {
            assert_eq!(
                map_case(case_id),
                Err(RankingEntitlementSnapshotError::Corrupt),
                "fixture case {case_id}"
            );
        }
    }

    #[test]
    fn a2_8_wallet_and_observation_inconsistency_are_corrupt() {
        for case_id in ["inconsistent-wallet", "inconsistent-observation"] {
            assert_eq!(
                map_case(case_id),
                Err(RankingEntitlementSnapshotError::Corrupt),
                "fixture case {case_id}"
            );
        }
    }
}
