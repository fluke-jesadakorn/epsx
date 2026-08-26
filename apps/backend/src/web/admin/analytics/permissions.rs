use super::types::*;
use crate::web::auth::AppState;
use crate::web::responses::wrappers::AdminResponse;
use axum::{
    extract::{Query, State},
    response::IntoResponse,
};
use chrono::{DateTime, Utc};
use tracing::info;

/**
 * Get permission analytics
 * GET /admin/analytics/permissions
 */
pub async fn get_permission_analytics_handler(
    Query(_query): Query<AnalyticsQuery>,
    State(app_state): State<AppState>,
) -> axum::response::Response {
    info!("Admin: Getting permission analytics");

    #[derive(sqlx::FromRow)]
    struct PlanStatsRow {
        plan_name: String,
        member_count: Option<i64>,
        active_members: Option<i64>,
        revenue: Option<bigdecimal::BigDecimal>,
    }

    #[derive(sqlx::FromRow)]
    struct TotalPlansRow {
        total_plans: i64,
    }

    // Get total plans count
    let total_plans = match sqlx::query_as::<_, TotalPlansRow>("SELECT COUNT(*)::bigint as total_plans FROM plans")
        .fetch_one(app_state.db_pool.as_ref())
        .await
    {
        Ok(result) => result.total_plans as i32,
        Err(_) => 0,
    };

    // Get permission plan stats with revenue
    let plan_stats = match sqlx::query_as::<_, PlanStatsRow>(
        r#"
        SELECT
            pg.name as plan_name,
            COUNT(wga.id)::bigint as member_count,
            COUNT(wga.id) FILTER (WHERE wga.is_active = true)::bigint as active_members,
            COALESCE(SUM(CASE WHEN wga.is_active THEN pg.price ELSE 0 END), 0.0) as revenue
         FROM plans pg
         LEFT JOIN wallet_plan_assignments wga ON pg.id = wga.plan_id
         GROUP BY pg.id, pg.name
         ORDER BY member_count DESC
        "#,
    )
    .fetch_all(app_state.db_pool.as_ref())
    .await
    {
        Ok(stats) => stats
            .into_iter()
            .map(|stat| PlanAssignmentStats {
                plan_name: stat.plan_name,
                member_count: stat.member_count.unwrap_or(0) as i32,
                active_members: stat.active_members.unwrap_or(0) as i32,
                revenue_contribution: stat
                    .revenue
                    .map(|r| r.to_string().parse::<f64>().unwrap_or(0.0))
                    .unwrap_or(0.0),
            })
            .collect(),
        Err(_) => Vec::new(),
    };

    // Get real permission usage data
    #[derive(sqlx::FromRow)]
    struct PermissionUsageRow {
        permission_string: String,
        users_count: Option<i64>,
        active_count: Option<i64>,
    }

    let permission_usage = match sqlx::query_as::<_, PermissionUsageRow>(
        r#"
        SELECT
            dp.permission_string,
            COUNT(DISTINCT u.wallet_address) as users_count,
            COUNT(DISTINCT u.wallet_address) FILTER (WHERE u.is_active = true) as active_count
        FROM (
            SELECT DISTINCT permission_string
            FROM user_effective_permissions
            WHERE permission_string IS NOT NULL
        ) dp
        LEFT JOIN user_effective_permissions uep ON dp.permission_string = uep.permission_string
        LEFT JOIN wallet_users u ON uep.wallet_address = u.wallet_address
        GROUP BY dp.permission_string
        ORDER BY users_count DESC
        "#,
    )
    .fetch_all(app_state.db_pool.as_ref())
    .await
    {
        Ok(results) => results
            .into_iter()
            .map(|row| PermissionUsageStats {
                permission: row.permission_string,
                users_count: row.users_count.unwrap_or(0) as i32,
                active_count: row.active_count.unwrap_or(0) as i32,
                usage_frequency: if row.users_count.unwrap_or(0) > 0 {
                    (row.active_count.unwrap_or(0) as f64 / row.users_count.unwrap_or(1) as f64)
                        * 100.0
                } else {
                    0.0
                },
            })
            .collect(),
        Err(_) => vec![],
    };

    #[derive(sqlx::FromRow)]
    struct TrendRow {
        trend_date: Option<DateTime<Utc>>,
        permission_count: Option<i64>,
    }

    // Get permission trends (last 30 days) - count permission grants over time
    let permission_trends = match sqlx::query_as::<_, TrendRow>(
        r#"
        SELECT
            DATE_TRUNC('day', granted_at) as trend_date,
            COUNT(*)::bigint as permission_count
        FROM wallet_direct_permissions
        WHERE granted_at >= NOW() - INTERVAL '30 days'
        GROUP BY DATE_TRUNC('day', granted_at)
        ORDER BY trend_date ASC
        "#,
    )
    .fetch_all(app_state.db_pool.as_ref())
    .await
    {
        Ok(results) => results
            .into_iter()
            .map(|row| TimeSeriesPoint {
                timestamp: row.trend_date.unwrap_or_else(Utc::now),
                value: row.permission_count.unwrap_or(0) as f64,
                label: row
                    .trend_date
                    .unwrap_or_else(Utc::now)
                    .format("%Y-%m-%d")
                    .to_string(),
            })
            .collect(),
        Err(_) => Vec::new(),
    };

    #[derive(sqlx::FromRow)]
    struct ExpiringRow {
        wallet_address: String,
        permission_string: String,
        expires_at: Option<DateTime<Utc>>,
        days_until_expiry: Option<i32>,
    }

    // Get expiring permissions (next 30 days) - use read model for denormalized permission_string
    let expiring_permissions = match sqlx::query_as::<_, ExpiringRow>(
        r#"
        SELECT
            wallet_address,
            permission_string,
            expires_at,
            EXTRACT(DAY FROM (expires_at - NOW()))::int as days_until_expiry
        FROM user_effective_permissions
        WHERE expires_at IS NOT NULL
          AND expires_at > NOW()
          AND expires_at <= NOW() + INTERVAL '30 days'
        ORDER BY expires_at ASC
        LIMIT 100
        "#,
    )
    .fetch_all(app_state.db_pool.as_ref())
    .await
    {
        Ok(results) => results
            .into_iter()
            .map(|row| ExpiringPermission {
                wallet_address: row.wallet_address,
                permission: row.permission_string,
                expires_at: row.expires_at.unwrap_or_else(Utc::now),
                days_until_expiry: row.days_until_expiry.unwrap_or(0),
            })
            .collect(),
        Err(_) => Vec::new(),
    };

    let response = PermissionAnalyticsResponse {
        total_plans,
        total_permissions: permission_usage.iter().map(|p| p.users_count).sum(),
        active_permissions: permission_usage.iter().map(|p| p.active_count).sum(),
        permission_usage,
        plan_assignment: plan_stats,
        permission_trends,
        expiring_permissions,
    };

    info!("Admin: Successfully retrieved permission analytics");
    AdminResponse::success_with_message(response, "Permission analytics retrieved successfully")
        .into_response()
}
