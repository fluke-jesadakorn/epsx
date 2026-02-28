use axum::{extract::State, response::IntoResponse};
use tracing::info;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use crate::web::auth::AppState;
use crate::web::responses::wrappers::AdminResponse;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct AdminAnalyticsDashboardResponse {
    pub user_stats: Option<serde_json::Value>,
    pub permission_analytics: Option<serde_json::Value>,
    pub plan_stats: Option<serde_json::Value>,
    pub system_metrics: Option<serde_json::Value>,
    pub developer_portal: Option<serde_json::Value>,
}

pub async fn get_admin_analytics_dashboard_handler(
    State(app_state): State<AppState>,
) -> axum::response::Response {
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
        system_metrics: Some(serde_json::json!({
            "health_percentage": 99.9,
            "uptime": "99.9%",
            "avg_response_time": "120ms",
            "api_response_time": 120.0,
            "memory_usage": 45.2,
            "active_users": 0,
        })),
        developer_portal: dev_stats.ok(),
    };

    AdminResponse::success_with_message(response, "Analytics dashboard retrieved").into_response()
}

async fn fetch_user_stats(app_state: &AppState) -> Result<serde_json::Value, String> {
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

    Ok(serde_json::json!({
        "total": result.total_users,
        "active": result.active_users,
        "today_connections": result.today_connections,
        "total_users": result.total_users,
        "active_users": result.active_users,
    }))
}

async fn fetch_permission_stats(app_state: &AppState) -> Result<serde_json::Value, String> {
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

    let result = diesel::sql_query(
        "SELECT
            (SELECT COUNT(*)::bigint FROM plans) as total_plans,
            COUNT(*)::bigint as total_permissions,
            COUNT(*) FILTER (WHERE expires_at IS NULL OR expires_at > NOW())::bigint as active_permissions
         FROM user_effective_permissions"
    )
    .get_result::<PermStats>(&mut conn)
    .await
    .map_err(|e| e.to_string())?;

    Ok(serde_json::json!({
        "total": result.total_permissions,
        "total_plans": result.total_plans,
        "total_permissions": result.total_permissions,
        "active_permissions": result.active_permissions,
        "pending_notifications": 0,
        "expiring_soon": 0,
        "health_score": 100.0,
    }))
}

async fn fetch_plan_stats(app_state: &AppState) -> Result<serde_json::Value, String> {
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
        #[diesel(sql_type = diesel::sql_types::BigInt)]
        recent_removals: i64,
    }

    let result = diesel::sql_query(
        "SELECT
            COUNT(*)::bigint as total_plans,
            COUNT(*) FILTER (WHERE is_active = true)::bigint as active_plans,
            (SELECT COUNT(*)::bigint FROM wallet_plan_assignments) as total_memberships,
            (SELECT COUNT(*)::bigint FROM wallet_plan_assignments WHERE is_active = true) as active_memberships,
            (SELECT COUNT(*)::bigint FROM wallet_plan_assignments WHERE created_at >= NOW() - INTERVAL '30 days') as recent_assignments,
            0::bigint as recent_removals
         FROM plans"
    )
    .get_result::<PlanCounts>(&mut conn)
    .await
    .map_err(|e| e.to_string())?;

    Ok(serde_json::json!({
        "total_plans": result.total_plans,
        "active_plans": result.active_plans,
        "total_memberships": result.total_memberships,
        "active_memberships": result.active_memberships,
        "recent_assignments": result.recent_assignments,
        "recent_removals": result.recent_removals,
        "by_plan": {},
    }))
}

async fn fetch_developer_stats(app_state: &AppState) -> Result<serde_json::Value, String> {
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
         FROM api_keys"
    )
    .get_result::<DevStats>(&mut conn)
    .await
    .map_err(|e| e.to_string())?;

    Ok(serde_json::json!({
        "total_api_keys": result.total_api_keys,
        "active_api_keys": result.active_api_keys,
        "revoked_api_keys": 0,
        "expired_api_keys": 0,
        "total_modules": 0,
        "active_modules": 0,
        "total_requests_today": 0,
        "total_requests_this_month": 0,
        "top_modules_by_usage": [],
    }))
}
