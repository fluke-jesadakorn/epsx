use crate::web::admin::responses::{AdminApiResponse, AdminMetadata};
use crate::web::auth::AppState;
use crate::web::middleware::{OpenIDUserContext, RequestId};
use axum::{
    extract::{Extension, State},
    http::HeaderValue,
    response::{IntoResponse, Response},
};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use serde::Serialize;
use tracing::info;

const ADMIN_AUDIENCE: &str = "epsx-admin";
const ANALYTICS_VIEW_PERMISSION: &str = "admin:analytics:view";
const ADMIN_PERMISSION_STATS_SQL: &str = r#"
    WITH permission_grants AS (
        SELECT
            wdp.is_active
                AND permission.is_active
                AND (wdp.expires_at IS NULL OR wdp.expires_at > NOW()) AS is_effective
        FROM wallet_direct_permissions AS wdp
        INNER JOIN permissions AS permission ON permission.id = wdp.permission_id

        UNION ALL

        SELECT
            assignment.is_active
                AND plan.is_active
                AND permission.is_active
                AND (
                    assignment.expires_at IS NULL
                    OR assignment.expires_at > NOW()
                    OR assignment.expires_at
                        + (plan.grace_period_hours || ' hours')::INTERVAL > NOW()
                ) AS is_effective
        FROM wallet_plan_assignments AS assignment
        INNER JOIN plans AS plan ON plan.id = assignment.plan_id
        INNER JOIN plan_permissions AS plan_permission ON plan_permission.plan_id = plan.id
        INNER JOIN permissions AS permission ON permission.id = plan_permission.permission_id
    )
    SELECT
        (SELECT COUNT(*)::bigint FROM plans) AS total_plans,
        COUNT(*)::bigint AS total_permissions,
        COUNT(*) FILTER (WHERE is_effective)::bigint AS active_permissions
    FROM permission_grants
"#;

#[derive(Debug, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AdminAnalyticsDashboardResponse {
    pub user_stats: Option<AdminAnalyticsUserStats>,
    pub permission_analytics: Option<AdminAnalyticsPermissionStats>,
    pub plan_stats: Option<AdminAnalyticsPlanStats>,
    pub system_metrics: Option<AdminAnalyticsSystemMetrics>,
    pub developer_portal: Option<AdminAnalyticsDeveloperStats>,
}

#[derive(Debug, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AdminAnalyticsUserStats {
    pub total: i64,
    pub active: i64,
    pub today_connections: i64,
    pub total_users: i64,
    pub active_users: i64,
}

#[derive(Debug, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AdminAnalyticsPermissionStats {
    pub total: i64,
    pub total_plans: i64,
    pub total_permissions: i64,
    pub active_permissions: i64,
}

#[derive(Debug, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AdminAnalyticsPlanStats {
    pub total_plans: i64,
    pub active_plans: i64,
    pub total_memberships: i64,
    pub active_memberships: i64,
    pub recent_assignments: i64,
}

#[derive(Debug, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AdminAnalyticsDeveloperStats {
    pub total_api_keys: i64,
    pub active_api_keys: i64,
}

/// Operational telemetry is not available in this read model. Keeping this
/// type explicit prevents fabricated health/latency fields from entering the
/// dashboard contract.
#[derive(Debug, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AdminAnalyticsSystemMetrics {}

fn exact_admin_analytics_read(context: &OpenIDUserContext) -> bool {
    matches!(
        context.token_audiences.as_deref(),
        Some([audience]) if audience == ADMIN_AUDIENCE
    ) && epsx_contracts::permissions::has_permission(
        &context.permissions,
        ANALYTICS_VIEW_PERMISSION,
    )
}

fn with_request_id(mut response: Response, request_id: &RequestId) -> Response {
    if let Ok(value) = HeaderValue::from_str(&request_id.0) {
        response.headers_mut().insert("x-request-id", value);
    }
    response
}

pub async fn get_admin_analytics_dashboard_handler(
    State(app_state): State<AppState>,
    Extension(context): Extension<OpenIDUserContext>,
    Extension(request_id): Extension<RequestId>,
) -> Response {
    if !matches!(
        context.token_audiences.as_deref(),
        Some([audience]) if audience == ADMIN_AUDIENCE
    ) {
        return with_request_id(
            AdminApiResponse::<()>::auth_error().into_response(),
            &request_id,
        );
    }
    if !exact_admin_analytics_read(&context) {
        return with_request_id(
            AdminApiResponse::<()>::permission_error(ANALYTICS_VIEW_PERMISSION).into_response(),
            &request_id,
        );
    }
    info!("Admin: Getting analytics dashboard batch");

    let (user_stats, perm_stats, plan_stats, dev_stats) = tokio::join!(
        fetch_user_stats(&app_state),
        fetch_permission_stats(&app_state),
        fetch_plan_stats(&app_state),
        fetch_developer_stats(&app_state),
    );

    let response = AdminAnalyticsDashboardResponse {
        user_stats: user_stats.ok(),
        permission_analytics: perm_stats.ok(),
        plan_stats: plan_stats.ok(),
        // Operational health, uptime, latency, and memory are not available
        // from this read model. Do not turn an absent source into a fabricated
        // green status that the admin UI could mistake for telemetry.
        system_metrics: None,
        developer_portal: dev_stats.ok(),
    };

    AdminApiResponse::success_with_meta(
        response,
        "Analytics dashboard retrieved",
        AdminMetadata::crud_operation("get_admin_analytics_dashboard", None),
    )
    .into_response()
}

async fn fetch_user_stats(app_state: &AppState) -> Result<AdminAnalyticsUserStats, String> {
    let mut conn = app_state.db_pool.get().await.map_err(|e| e.to_string())?;

    #[derive(QueryableByName)]
    struct UserCounts {
        #[diesel(sql_type = diesel::sql_types::BigInt)]
        total_users: i64,
        #[diesel(sql_type = diesel::sql_types::BigInt)]
        active_users: i64,
        #[diesel(sql_type = diesel::sql_types::BigInt)]
        today_connections: i64,
    }

    let result = diesel::sql_query(
        "SELECT COUNT(*)::bigint as total_users,
                COUNT(*) FILTER (WHERE is_active = true)::bigint as active_users,
                COUNT(*) FILTER (WHERE last_auth_at >= NOW() - INTERVAL '24 hours')::bigint as today_connections
         FROM wallet_users"
    )
    .get_result::<UserCounts>(&mut conn)
    .await
    .map_err(|e| e.to_string())?;

    Ok(AdminAnalyticsUserStats {
        total: result.total_users,
        active: result.active_users,
        today_connections: result.today_connections,
        total_users: result.total_users,
        active_users: result.active_users,
    })
}

async fn fetch_permission_stats(
    app_state: &AppState,
) -> Result<AdminAnalyticsPermissionStats, String> {
    let mut conn = app_state.db_pool.get().await.map_err(|e| e.to_string())?;

    #[derive(QueryableByName)]
    struct PermStats {
        #[diesel(sql_type = diesel::sql_types::BigInt)]
        total_plans: i64,
        #[diesel(sql_type = diesel::sql_types::BigInt)]
        total_permissions: i64,
        #[diesel(sql_type = diesel::sql_types::BigInt)]
        active_permissions: i64,
    }

    let result = diesel::sql_query(ADMIN_PERMISSION_STATS_SQL)
        .get_result::<PermStats>(&mut conn)
        .await
        .map_err(|e| e.to_string())?;

    Ok(AdminAnalyticsPermissionStats {
        total: result.total_permissions,
        total_plans: result.total_plans,
        total_permissions: result.total_permissions,
        active_permissions: result.active_permissions,
    })
}

async fn fetch_plan_stats(app_state: &AppState) -> Result<AdminAnalyticsPlanStats, String> {
    let mut conn = app_state.db_pool.get().await.map_err(|e| e.to_string())?;

    #[derive(QueryableByName)]
    struct PlanCounts {
        #[diesel(sql_type = diesel::sql_types::BigInt)]
        total_plans: i64,
        #[diesel(sql_type = diesel::sql_types::BigInt)]
        active_plans: i64,
        #[diesel(sql_type = diesel::sql_types::BigInt)]
        total_memberships: i64,
        #[diesel(sql_type = diesel::sql_types::BigInt)]
        active_memberships: i64,
        #[diesel(sql_type = diesel::sql_types::BigInt)]
        recent_assignments: i64,
    }

    let result = diesel::sql_query(
        "SELECT
            COUNT(*)::bigint as total_plans,
            COUNT(*) FILTER (WHERE is_active = true)::bigint as active_plans,
            (SELECT COUNT(*)::bigint FROM wallet_plan_assignments) as total_memberships,
            (SELECT COUNT(*)::bigint FROM wallet_plan_assignments WHERE is_active = true) as active_memberships,
            (SELECT COUNT(*)::bigint FROM wallet_plan_assignments WHERE created_at >= NOW() - INTERVAL '30 days') as recent_assignments
         FROM plans"
    )
    .get_result::<PlanCounts>(&mut conn)
    .await
    .map_err(|e| e.to_string())?;

    Ok(AdminAnalyticsPlanStats {
        total_plans: result.total_plans,
        active_plans: result.active_plans,
        total_memberships: result.total_memberships,
        active_memberships: result.active_memberships,
        recent_assignments: result.recent_assignments,
    })
}

async fn fetch_developer_stats(
    app_state: &AppState,
) -> Result<AdminAnalyticsDeveloperStats, String> {
    let pool = if let Some(analytics) = &app_state.analytics_db_pool {
        analytics
    } else {
        &app_state.db_pool
    };

    let mut conn = pool.get().await.map_err(|e| e.to_string())?;

    #[derive(QueryableByName)]
    struct DevStats {
        #[diesel(sql_type = diesel::sql_types::BigInt)]
        total_api_keys: i64,
        #[diesel(sql_type = diesel::sql_types::BigInt)]
        active_api_keys: i64,
    }

    let result = diesel::sql_query(
        "SELECT COUNT(*)::bigint as total_api_keys,
                COUNT(*) FILTER (WHERE status = 'active')::bigint as active_api_keys
         FROM api_keys",
    )
    .get_result::<DevStats>(&mut conn)
    .await
    .map_err(|e| e.to_string())?;

    Ok(AdminAnalyticsDeveloperStats {
        total_api_keys: result.total_api_keys,
        active_api_keys: result.active_api_keys,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn permission_stats_use_current_authoritative_grant_tables() {
        for relation in [
            "wallet_direct_permissions",
            "wallet_plan_assignments",
            "plan_permissions",
            "permissions",
        ] {
            assert!(ADMIN_PERMISSION_STATS_SQL.contains(relation));
        }
        assert!(!ADMIN_PERMISSION_STATS_SQL.contains("user_effective_permissions"));
        assert!(ADMIN_PERMISSION_STATS_SQL.contains("COUNT(*) FILTER (WHERE is_effective)"));
    }

    fn context(audiences: Option<Vec<&str>>, permissions: &[&str]) -> OpenIDUserContext {
        OpenIDUserContext {
            sub: "admin-subject".to_string(),
            wallet_address: "0xadmin".to_string(),
            permissions: permissions
                .iter()
                .map(|value| (*value).to_string())
                .collect(),
            token_audiences: audiences
                .map(|values| values.into_iter().map(str::to_string).collect()),
            api_key: None,
            auth_method: "oidc".to_string(),
            jti: "request-token".to_string(),
            exp: 2_000_000_000,
            iat: 1_900_000_000,
            auth_time: 1_900_000_000,
        }
    }

    #[test]
    fn dashboard_read_requires_exact_admin_audience_and_view_permission() {
        assert!(exact_admin_analytics_read(&context(
            Some(vec![ADMIN_AUDIENCE]),
            &[ANALYTICS_VIEW_PERMISSION],
        )));
        assert!(!exact_admin_analytics_read(&context(
            Some(vec!["epsx-admin", "epsx-frontend"]),
            &[ANALYTICS_VIEW_PERMISSION],
        )));
        assert!(!exact_admin_analytics_read(&context(
            Some(vec![ADMIN_AUDIENCE]),
            &["admin:analytics:read"],
        )));
    }
}
