use axum::{
    extract::{Query, State},
    response::IntoResponse,
};
use chrono::{DateTime, Utc};
use tracing::{error, info};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use crate::web::auth::AppState;
use crate::web::responses::wrappers::AdminResponse;
use super::types::*;

/**
 * Get permission analytics
 * GET /admin/analytics/permissions
 */
pub async fn get_permission_analytics_handler(
    Query(_query): Query<AnalyticsQuery>,
    State(app_state): State<AppState>,
) -> axum::response::Response {
    info!("Admin: Getting permission analytics");

    let mut conn = match app_state.db_pool.get().await {
        Ok(conn) => conn,
        Err(e) => {
            error!("Admin: Failed to get database connection: {}", e);
            return AdminResponse::server_error("Database error").into_response();
        }
    };

    #[derive(QueryableByName)]
    struct PlanStatsRow {
        #[diesel(sql_type = diesel::sql_types::Text)]
        plan_name: String,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::BigInt>)]
        member_count: Option<i64>,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::BigInt>)]
        active_members: Option<i64>,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Numeric>)]
        revenue: Option<bigdecimal::BigDecimal>,
    }

    #[derive(QueryableByName)]
    struct TotalPlansRow {
        #[diesel(sql_type = diesel::sql_types::BigInt)]
        total_plans: i64,
    }

    // Get total plans count
    let total_plans = match diesel::sql_query(
        "SELECT COUNT(*)::bigint as total_plans FROM plans"
    )
    .get_result::<TotalPlansRow>(&mut conn)
    .await
    {
        Ok(result) => result.total_plans as i32,
        Err(_) => 0,
    };

    // Get permission plan stats with revenue
    let plan_stats = match diesel::sql_query(
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
        "#
    )
    .load::<PlanStatsRow>(&mut conn)
    .await
    {
        Ok(stats) => stats
            .into_iter()
            .map(|stat| PlanAssignmentStats {
                plan_name: stat.plan_name,
                member_count: stat.member_count.unwrap_or(0) as i32,
                active_members: stat.active_members.unwrap_or(0) as i32,
                revenue_contribution: stat.revenue.map(|r| r.to_string().parse::<f64>().unwrap_or(0.0)).unwrap_or(0.0),
            })
            .collect(),
        Err(_) => Vec::new(),
    };

    // Get real permission usage data
    #[derive(QueryableByName)]
    struct PermissionUsageRow {
        #[diesel(sql_type = diesel::sql_types::Text)]
        permission_string: String,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::BigInt>)]
        users_count: Option<i64>,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::BigInt>)]
        active_count: Option<i64>,
    }

    let permission_usage = match diesel::sql_query(
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
        "#
    )
    .load::<PermissionUsageRow>(&mut conn)
    .await
    {
        Ok(results) => results
            .into_iter()
            .map(|row| PermissionUsageStats {
                permission: row.permission_string,
                users_count: row.users_count.unwrap_or(0) as i32,
                active_count: row.active_count.unwrap_or(0) as i32,
                usage_frequency: if row.users_count.unwrap_or(0) > 0 {
                    (row.active_count.unwrap_or(0) as f64 / row.users_count.unwrap_or(1) as f64) * 100.0
                } else {
                    0.0
                },
            })
            .collect(),
        Err(_) => vec![], // Return empty if query fails
    };

    #[derive(QueryableByName)]
    struct TrendRow {
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>)]
        trend_date: Option<DateTime<Utc>>,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::BigInt>)]
        permission_count: Option<i64>,
    }

    // Get permission trends (last 30 days) - count permission grants over time
    let permission_trends = match diesel::sql_query(
        r#"
        SELECT
            DATE_TRUNC('day', granted_at) as trend_date,
            COUNT(*)::bigint as permission_count
        FROM wallet_direct_permissions
        WHERE granted_at >= NOW() - INTERVAL '30 days'
        GROUP BY DATE_TRUNC('day', granted_at)
        ORDER BY trend_date ASC
        "#
    )
    .load::<TrendRow>(&mut conn)
    .await
    {
        Ok(results) => results
            .into_iter()
            .map(|row| TimeSeriesPoint {
                timestamp: row.trend_date.unwrap_or_else(Utc::now),
                value: row.permission_count.unwrap_or(0) as f64,
                label: row.trend_date
                    .unwrap_or_else(Utc::now)
                    .format("%Y-%m-%d")
                    .to_string(),
            })
            .collect(),
        Err(_) => Vec::new(),
    };

    #[derive(QueryableByName)]
    struct ExpiringRow {
        #[diesel(sql_type = diesel::sql_types::Text)]
        wallet_address: String,
        #[diesel(sql_type = diesel::sql_types::Text)]
        permission_string: String,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>)]
        expires_at: Option<DateTime<Utc>>,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Integer>)]
        days_until_expiry: Option<i32>,
    }

    // Get expiring permissions (next 30 days) - use read model for denormalized permission_string
    let expiring_permissions = match diesel::sql_query(
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
        "#
    )
    .load::<ExpiringRow>(&mut conn)
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
    AdminResponse::success_with_message(response, "Permission analytics retrieved successfully").into_response()
}
