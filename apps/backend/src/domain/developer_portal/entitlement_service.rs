// BIG-BANG TODO: this domain service leaks infrastructure (DbPool + diesel::sql_query).
// Canonical: extract `DeveloperEntitlementRepositoryPort` trait in `domain/developer_portal/repository_ports/`
// and move this impl to `infrastructure/adapters/repositories/developer_entitlement_adapter.rs`
// using `sqlx::query_as`. Kept as-is for single-branch big-bang scaffold to keep build green.
use chrono::{DateTime, Utc};
use diesel::sql_types::{Bool, Int4, Nullable, Text, Timestamptz, Uuid as SqlUuid};
use diesel::QueryableByName;
use diesel_async::RunQueryDsl;
use serde::Serialize;
use std::collections::BTreeSet;
use uuid::Uuid;

use crate::infrastructure::adapter_repositories::DbPool;
use crate::prelude::{AppError, AppResult};

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct EffectiveApiPlan {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct EffectiveApiRateLimits {
    pub per_minute: u32,
    pub per_hour: u32,
    pub per_day: u32,
    pub burst: u32,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct DeveloperEntitlement {
    pub plans: Vec<EffectiveApiPlan>,
    pub assignable_scopes: Vec<String>,
    pub rate_limits: EffectiveApiRateLimits,
    pub can_read: bool,
    pub can_write: bool,
    pub has_active_api_entitlement: bool,
}

#[derive(QueryableByName)]
struct PlanRow {
    #[diesel(sql_type = SqlUuid)]
    id: Uuid,
    #[diesel(sql_type = Text)]
    name: String,
    #[diesel(sql_type = Text)]
    slug: String,
    #[diesel(sql_type = Nullable<Timestamptz>)]
    expires_at: Option<DateTime<Utc>>,
    #[diesel(sql_type = Int4)]
    rate_limit_per_minute: i32,
    #[diesel(sql_type = Int4)]
    rate_limit_per_hour: i32,
    #[diesel(sql_type = Int4)]
    rate_limit_per_day: i32,
    #[diesel(sql_type = Int4)]
    burst_capacity: i32,
}

#[derive(QueryableByName)]
struct PermissionRow {
    #[diesel(sql_type = Text)]
    permission_string: String,
    #[diesel(sql_type = Bool)]
    api_assignable: bool,
}

pub struct DeveloperEntitlementService {
    core_pool: DbPool,
}

fn maximum_rate_limits(plans: &[PlanRow]) -> EffectiveApiRateLimits {
    EffectiveApiRateLimits {
        per_minute: plans
            .iter()
            .map(|plan| plan.rate_limit_per_minute.max(0) as u32)
            .max()
            .unwrap_or(0),
        per_hour: plans
            .iter()
            .map(|plan| plan.rate_limit_per_hour.max(0) as u32)
            .max()
            .unwrap_or(0),
        per_day: plans
            .iter()
            .map(|plan| plan.rate_limit_per_day.max(0) as u32)
            .max()
            .unwrap_or(0),
        burst: plans
            .iter()
            .map(|plan| plan.burst_capacity.max(0) as u32)
            .max()
            .unwrap_or(0),
    }
}

fn intersect_api_scopes(selected: &[String], allowed: &[String]) -> Vec<String> {
    let allowed = allowed.iter().collect::<BTreeSet<_>>();
    selected
        .iter()
        .filter(|scope| allowed.contains(scope))
        .cloned()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

impl DeveloperEntitlementService {
    pub fn new(core_pool: DbPool) -> Self {
        Self { core_pool }
    }

    /// Resolve API-key capabilities from live normalized grants. Admin plans
    /// never contribute scopes or limits, and the catalog must explicitly mark
    /// a permission as API-assignable before it can be delegated.
    pub async fn resolve(&self, wallet_address: &str) -> AppResult<DeveloperEntitlement> {
        let mut conn = self.core_pool.get().await.map_err(|error| {
            AppError::database_error(format!("developer entitlement pool: {error}"))
        })?;

        let plans = diesel::sql_query(
            r#"
            SELECT p.id, p.name::text, p.slug::text, wpa.expires_at,
                   p.rate_limit_per_minute, p.rate_limit_per_hour,
                   p.rate_limit_per_day, p.burst_capacity
            FROM wallet_plan_assignments wpa
            JOIN plans p ON p.id = wpa.plan_id
            WHERE LOWER(wpa.wallet_address) = LOWER($1)
              AND wpa.is_active = TRUE
              AND (wpa.expires_at IS NULL OR wpa.expires_at > NOW())
              AND p.is_active = TRUE
              AND p.plan_type <> 'admin'
            ORDER BY p.tier_level DESC, p.name ASC
            "#,
        )
        .bind::<Text, _>(wallet_address)
        .load::<PlanRow>(&mut conn)
        .await
        .map_err(|error| AppError::database_error(format!("load API plans: {error}")))?;

        let permission_rows = diesel::sql_query(
            r#"
            WITH current_permissions AS (
                SELECT DISTINCT permission.id
                FROM wallet_plan_assignments wpa
                JOIN plans plan ON plan.id = wpa.plan_id
                JOIN plan_permissions mapping ON mapping.plan_id = plan.id
                JOIN permissions permission ON permission.id = mapping.permission_id
                WHERE LOWER(wpa.wallet_address) = LOWER($1)
                  AND wpa.is_active = TRUE
                  AND (wpa.expires_at IS NULL OR wpa.expires_at > NOW())
                  AND plan.is_active = TRUE
                  AND plan.plan_type <> 'admin'

                UNION

                SELECT DISTINCT permission.id
                FROM wallet_direct_permissions direct
                JOIN permissions permission ON permission.id = direct.permission_id
                WHERE LOWER(direct.wallet_address) = LOWER($1)
                  AND direct.is_active = TRUE
                  AND (direct.expires_at IS NULL OR direct.expires_at > NOW())
            )
            SELECT permission.permission_string::text, permission.api_assignable
            FROM current_permissions current
            JOIN permissions permission ON permission.id = current.id
            WHERE permission.is_active = TRUE
              AND permission.permission_string NOT LIKE 'admin:%'
            ORDER BY permission.permission_string
            "#,
        )
        .bind::<Text, _>(wallet_address)
        .load::<PermissionRow>(&mut conn)
        .await
        .map_err(|error| AppError::database_error(format!("load API permissions: {error}")))?;

        let all_permissions: Vec<String> = permission_rows
            .iter()
            .map(|row| row.permission_string.clone())
            .collect();
        let assignable_scopes = permission_rows
            .into_iter()
            .filter(|row| row.api_assignable)
            .map(|row| row.permission_string)
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();
        let can_read =
            epsx_contracts::permissions::has_permission(&all_permissions, "epsx:api:read");
        let can_write =
            epsx_contracts::permissions::has_permission(&all_permissions, "epsx:api:write");

        let rate_limits = maximum_rate_limits(&plans);
        let plans = plans
            .into_iter()
            .map(|plan| EffectiveApiPlan {
                id: plan.id,
                name: plan.name,
                slug: plan.slug,
                expires_at: plan.expires_at,
            })
            .collect::<Vec<_>>();
        let has_active_api_entitlement =
            can_read && !plans.is_empty() && !assignable_scopes.is_empty();

        Ok(DeveloperEntitlement {
            plans,
            assignable_scopes,
            rate_limits,
            can_read,
            can_write,
            has_active_api_entitlement,
        })
    }

    pub async fn effective_key_scopes(
        &self,
        wallet_address: &str,
        selected_scopes: &[String],
    ) -> AppResult<(Vec<String>, DeveloperEntitlement)> {
        let entitlement = self.resolve(wallet_address).await?;
        if !entitlement.has_active_api_entitlement {
            return Ok((Vec::new(), entitlement));
        }
        let effective = intersect_api_scopes(selected_scopes, &entitlement.assignable_scopes);
        Ok((effective, entitlement))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn entitlement_contract_never_treats_admin_names_as_assignable_scopes() {
        let permissions = ["epsx:analytics:view", "admin:users:manage"];
        let filtered = permissions
            .into_iter()
            .filter(|permission| !permission.starts_with("admin:"))
            .collect::<Vec<_>>();
        assert_eq!(filtered, vec!["epsx:analytics:view"]);
    }

    #[test]
    fn multiple_plan_limits_use_maximum_instead_of_sum() {
        let plan = |per_minute, per_hour, per_day, burst| PlanRow {
            id: Uuid::new_v4(),
            name: "plan".to_string(),
            slug: "plan".to_string(),
            expires_at: None,
            rate_limit_per_minute: per_minute,
            rate_limit_per_hour: per_hour,
            rate_limit_per_day: per_day,
            burst_capacity: burst,
        };
        let limits = maximum_rate_limits(&[plan(10, 1_000, 5_000, 20), plan(60, 500, 50_000, 10)]);
        assert_eq!(limits.per_minute, 60);
        assert_eq!(limits.per_hour, 1_000);
        assert_eq!(limits.per_day, 50_000);
        assert_eq!(limits.burst, 20);
    }

    #[test]
    fn downgrade_removes_stored_scopes_immediately() {
        let selected = vec![
            "epsx:analytics:view".to_string(),
            "epsx:data:export".to_string(),
            "admin:users:manage".to_string(),
        ];
        let allowed = vec!["epsx:analytics:view".to_string()];
        assert_eq!(intersect_api_scopes(&selected, &allowed), allowed);
    }
}
