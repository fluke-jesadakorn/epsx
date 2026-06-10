use axum::{extract::State, response::IntoResponse};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use serde::Serialize;
use tracing::info;

use crate::web::auth::AppState;
use crate::web::responses::wrappers::AdminResponse;

// ============================================================================
// A. Admin Dashboard Summary: GET /admin/dashboard/summary
// ============================================================================

#[derive(Debug, Serialize)]
pub struct AdminDashboardSummaryResponse {
    pub wallet_stats: serde_json::Value,
    pub permission_stats: serde_json::Value,
    pub system_health: serde_json::Value,
}

pub async fn admin_dashboard_summary_handler(
    State(app_state): State<AppState>,
) -> axum::response::Response {
    info!("Admin: Getting dashboard summary batch");

    let (wallet_stats, perm_stats) = tokio::join!(
        fetch_wallet_stats(&app_state),
        fetch_perm_system_stats(&app_state),
    );

    let response = AdminDashboardSummaryResponse {
        wallet_stats: wallet_stats.unwrap_or_else(|_| serde_json::json!({ "total": 0, "active": 0, "today_connections": 0 })),
        permission_stats: perm_stats.unwrap_or_else(|_| serde_json::json!({ "total": 0, "pending_notifications": 0 })),
        system_health: serde_json::json!({
            "health_percentage": 99.9,
            "uptime": "99.9%",
            "avg_response_time": "120ms",
        }),
    };

    AdminResponse::success_with_message(response, "Dashboard summary retrieved").into_response()
}

async fn fetch_wallet_stats(app_state: &AppState) -> Result<serde_json::Value, String> {
    let mut conn = app_state.db_pool.get().await.map_err(|e| e.to_string())?;

    #[derive(QueryableByName)]
    struct WalletCounts {
        #[diesel(sql_type = diesel::sql_types::BigInt)]
        total: i64,
        #[diesel(sql_type = diesel::sql_types::BigInt)]
        active: i64,
        #[diesel(sql_type = diesel::sql_types::BigInt)]
        today_connections: i64,
    }

    let result = diesel::sql_query(
        "SELECT COUNT(*)::bigint as total,
                COUNT(*) FILTER (WHERE is_active = true)::bigint as active,
                COUNT(*) FILTER (WHERE last_auth_at >= NOW() - INTERVAL '24 hours')::bigint as today_connections
         FROM wallet_users"
    )
    .get_result::<WalletCounts>(&mut conn)
    .await
    .map_err(|e| e.to_string())?;

    Ok(serde_json::json!({
        "total": result.total,
        "active": result.active,
        "today_connections": result.today_connections,
    }))
}

async fn fetch_perm_system_stats(app_state: &AppState) -> Result<serde_json::Value, String> {
    let mut conn = app_state.db_pool.get().await.map_err(|e| e.to_string())?;

    #[derive(QueryableByName)]
    struct PermCounts {
        #[diesel(sql_type = diesel::sql_types::BigInt)]
        total: i64,
    }

    let result = diesel::sql_query(
        "SELECT COUNT(*)::bigint as total FROM user_effective_permissions WHERE expires_at IS NULL OR expires_at > NOW()"
    )
    .get_result::<PermCounts>(&mut conn)
    .await
    .map_err(|e| e.to_string())?;

    Ok(serde_json::json!({
        "total": result.total,
        "pending_notifications": 0,
    }))
}

// ============================================================================
// B. Admin Notification Overview: GET /admin/notifications/overview
// ============================================================================

#[derive(Debug, Serialize)]
pub struct NotificationOverviewResponse {
    pub notifications: serde_json::Value,
    pub stats: serde_json::Value,
}

#[derive(Debug, serde::Deserialize)]
pub struct NotificationOverviewQuery {
    pub limit: Option<i64>,
}

pub async fn admin_notification_overview_handler(
    State(app_state): State<AppState>,
    axum::extract::Query(query): axum::extract::Query<NotificationOverviewQuery>,
) -> axum::response::Response {
    info!("Admin: Getting notification overview batch");
    let limit = query.limit.unwrap_or(20);

    let (notifications, stats) = tokio::join!(
        fetch_notifications(&app_state, limit),
        fetch_notification_stats(&app_state),
    );

    let response = NotificationOverviewResponse {
        notifications: notifications.unwrap_or_else(|_| serde_json::json!([])),
        stats: stats.unwrap_or_else(|_| serde_json::json!({ "total": 0, "unread": 0 })),
    };

    AdminResponse::success_with_message(response, "Notification overview retrieved").into_response()
}

async fn fetch_notifications(app_state: &AppState, limit: i64) -> Result<serde_json::Value, String> {
    let mut conn = app_state.db_pool.get().await.map_err(|e| e.to_string())?;

    #[derive(QueryableByName, serde::Serialize)]
    struct NotifRow {
        #[diesel(sql_type = diesel::sql_types::Text)]
        id: String,
        #[diesel(sql_type = diesel::sql_types::Text)]
        title: String,
        #[diesel(sql_type = diesel::sql_types::Text)]
        message: String,
        #[diesel(sql_type = diesel::sql_types::Text)]
        notification_type: String,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>)]
        created_at: Option<chrono::DateTime<chrono::Utc>>,
    }

    let results = diesel::sql_query(
        "SELECT id::text, title, message, notification_type, created_at FROM notifications ORDER BY created_at DESC LIMIT $1"
    )
    .bind::<diesel::sql_types::BigInt, _>(limit)
    .load::<NotifRow>(&mut conn)
    .await
    .map_err(|e| e.to_string())?;

    Ok(serde_json::to_value(results).unwrap_or_else(|_| serde_json::json!([])))
}

async fn fetch_notification_stats(app_state: &AppState) -> Result<serde_json::Value, String> {
    let mut conn = app_state.db_pool.get().await.map_err(|e| e.to_string())?;

    #[derive(QueryableByName)]
    struct StatsRow {
        #[diesel(sql_type = diesel::sql_types::BigInt)]
        total: i64,
        #[diesel(sql_type = diesel::sql_types::BigInt)]
        today: i64,
    }

    let result = diesel::sql_query(
        "SELECT COUNT(*)::bigint as total,
                COUNT(*) FILTER (WHERE created_at >= NOW() - INTERVAL '24 hours')::bigint as today
         FROM notifications"
    )
    .get_result::<StatsRow>(&mut conn)
    .await
    .map_err(|e| e.to_string())?;

    Ok(serde_json::json!({
        "total": result.total,
        "today": result.today,
        "unread": 0,
    }))
}

// ============================================================================
// C. Wallet Access Summary: GET /admin/wallets/{address}/access-summary
// ============================================================================

#[derive(Debug, Serialize)]
pub struct WalletAccessSummaryResponse {
    pub available_permissions: Vec<String>,
    pub available_plans: serde_json::Value,
    pub wallet_permissions: Vec<String>,
    pub wallet_assignments: serde_json::Value,
}

pub async fn wallet_access_summary_handler(
    State(app_state): State<AppState>,
    axum::extract::Path(wallet_address): axum::extract::Path<String>,
) -> axum::response::Response {
    info!("Admin: Getting wallet access summary for {}", wallet_address);

    let (avail_perms, avail_plans, wallet_perms, wallet_assignments) = tokio::join!(
        fetch_available_permissions(&app_state),
        fetch_available_plans(&app_state),
        fetch_wallet_permissions(&app_state, &wallet_address),
        fetch_wallet_plan_assignments(&app_state, &wallet_address),
    );

    let response = WalletAccessSummaryResponse {
        available_permissions: avail_perms.unwrap_or_default(),
        available_plans: avail_plans.unwrap_or_else(|_| serde_json::json!([])),
        wallet_permissions: wallet_perms.unwrap_or_default(),
        wallet_assignments: wallet_assignments.unwrap_or_else(|_| serde_json::json!([])),
    };

    AdminResponse::success_with_message(response, "Wallet access summary retrieved").into_response()
}

async fn fetch_available_permissions(app_state: &AppState) -> Result<Vec<String>, String> {
    let mut conn = app_state.db_pool.get().await.map_err(|e| e.to_string())?;

    #[derive(QueryableByName)]
    struct PermRow {
        #[diesel(sql_type = diesel::sql_types::Text)]
        permission_string: String,
    }

    let results = diesel::sql_query(
        "SELECT DISTINCT permission_string FROM user_effective_permissions WHERE permission_string IS NOT NULL ORDER BY permission_string"
    )
    .load::<PermRow>(&mut conn)
    .await
    .map_err(|e| e.to_string())?;

    Ok(results.into_iter().map(|r| r.permission_string).collect())
}

async fn fetch_available_plans(app_state: &AppState) -> Result<serde_json::Value, String> {
    let mut conn = app_state.db_pool.get().await.map_err(|e| e.to_string())?;

    #[derive(QueryableByName, serde::Serialize)]
    struct PlanRow {
        #[diesel(sql_type = diesel::sql_types::Text)]
        id: String,
        #[diesel(sql_type = diesel::sql_types::Text)]
        name: String,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
        description: Option<String>,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
        plan_group: Option<String>,
        #[diesel(sql_type = diesel::sql_types::BigInt)]
        member_count: i64,
    }

    let results = diesel::sql_query(
        "SELECT p.id::text, p.name, p.description, p.plan_group,
                COUNT(wpa.id)::bigint as member_count
         FROM plans p
         LEFT JOIN wallet_plan_assignments wpa ON p.id = wpa.plan_id AND wpa.is_active = true
         WHERE p.is_active = true
         GROUP BY p.id, p.name, p.description, p.plan_group
         ORDER BY p.name"
    )
    .load::<PlanRow>(&mut conn)
    .await
    .map_err(|e| e.to_string())?;

    Ok(serde_json::to_value(results).unwrap_or_else(|_| serde_json::json!([])))
}

async fn fetch_wallet_permissions(app_state: &AppState, wallet: &str) -> Result<Vec<String>, String> {
    let mut conn = app_state.db_pool.get().await.map_err(|e| e.to_string())?;

    #[derive(QueryableByName)]
    struct PermRow {
        #[diesel(sql_type = diesel::sql_types::Text)]
        permission_string: String,
    }

    let results = diesel::sql_query(
        "SELECT permission_string FROM user_effective_permissions WHERE wallet_address = $1 AND (expires_at IS NULL OR expires_at > NOW()) ORDER BY permission_string"
    )
    .bind::<diesel::sql_types::Text, _>(wallet)
    .load::<PermRow>(&mut conn)
    .await
    .map_err(|e| e.to_string())?;

    Ok(results.into_iter().map(|r| r.permission_string).collect())
}

async fn fetch_wallet_plan_assignments(app_state: &AppState, wallet: &str) -> Result<serde_json::Value, String> {
    let mut conn = app_state.db_pool.get().await.map_err(|e| e.to_string())?;

    #[derive(QueryableByName, serde::Serialize)]
    struct AssignRow {
        #[diesel(sql_type = diesel::sql_types::Text)]
        plan_id: String,
        #[diesel(sql_type = diesel::sql_types::Text)]
        plan_name: String,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>)]
        expires_at: Option<chrono::DateTime<chrono::Utc>>,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>)]
        granted_at: Option<chrono::DateTime<chrono::Utc>>,
    }

    let results = diesel::sql_query(
        "SELECT wpa.plan_id::text, p.name as plan_name, wpa.expires_at, wpa.created_at as granted_at
         FROM wallet_plan_assignments wpa
         INNER JOIN plans p ON wpa.plan_id = p.id
         WHERE wpa.wallet_address = $1 AND wpa.is_active = true
         ORDER BY wpa.created_at DESC"
    )
    .bind::<diesel::sql_types::Text, _>(wallet)
    .load::<AssignRow>(&mut conn)
    .await
    .map_err(|e| e.to_string())?;

    Ok(serde_json::to_value(results).unwrap_or_else(|_| serde_json::json!([])))
}
