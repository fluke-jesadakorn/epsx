use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::Json,
};
use chrono::{DateTime, Utc};
use tracing::{error, info};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use crate::web::auth::AppState;
use crate::web::admin::responses::{AdminApiResponse, AdminMetadata};
use super::types::*;

/**
 * Get revenue analytics
 * GET /admin/analytics/revenue
 */
pub async fn get_revenue_analytics_handler(
    Query(_query): Query<AnalyticsQuery>,
    State(app_state): State<AppState>,
) -> Result<Json<AdminApiResponse<RevenueAnalyticsResponse>>, StatusCode> {
    info!("💰 Admin: Getting revenue analytics");

    let mut conn = match app_state.db_pool.get().await {
        Ok(conn) => conn,
        Err(e) => {
            error!("❌ Admin: Failed to get database connection: {}", e);
            return Ok(Json(AdminApiResponse::server_error()));
        }
    };

    #[derive(QueryableByName)]
    struct RevenueResult {
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Numeric>)]
        revenue: Option<bigdecimal::BigDecimal>,
    }

    // Calculate total revenue from active subscriptions and lifetime packages
    let total_revenue = match diesel::sql_query(
        "SELECT COALESCE(SUM(pg.price), 0.0) as revenue FROM wallet_plan_assignments wga
         INNER JOIN plans pg ON wga.plan_id = pg.id
         WHERE wga.is_active = true AND pg.plan_type = 'subscription'"
    )
    .get_result::<RevenueResult>(&mut conn)
    .await
    {
        Ok(result) => result.revenue.map(|r| r.to_string().parse::<f64>().unwrap_or(0.0)).unwrap_or(0.0),
        Err(_) => 0.0
    };

    // Calculate monthly recurring revenue (only monthly/yearly subscriptions)
    let monthly_recurring_revenue = match diesel::sql_query(
        "SELECT COALESCE(SUM(
            CASE
                WHEN pg.billing_cycle = 'monthly' THEN pg.price
                WHEN pg.billing_cycle = 'yearly' THEN pg.price / 12.0
                ELSE 0.0
            END
        ), 0.0) as revenue FROM wallet_plan_assignments wga
         INNER JOIN plans pg ON wga.plan_id = pg.id
         WHERE wga.is_active = true AND pg.plan_type = 'subscription'
         AND pg.billing_cycle IN ('monthly', 'yearly')"
    )
    .get_result::<RevenueResult>(&mut conn)
    .await
    {
        Ok(result) => result.revenue.map(|r| r.to_string().parse::<f64>().unwrap_or(0.0)).unwrap_or(0.0),
        Err(_) => 0.0
    };

    #[derive(QueryableByName)]
    struct TierRevenueRow {
        #[diesel(sql_type = diesel::sql_types::Text)]
        tier_name: String,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Numeric>)]
        revenue: Option<bigdecimal::BigDecimal>,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::BigInt>)]
        subscriber_count: Option<i64>,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Double>)]
        average_revenue_per_user: Option<f64>,
    }

    // Get revenue by tier/permission plan
    let revenue_by_tier = match diesel::sql_query(
        "SELECT pg.name as tier_name,
                COALESCE(SUM(pg.price), 0.0) as revenue,
                COUNT(*)::bigint as subscriber_count,
                CASE
                    WHEN COUNT(*) > 0 THEN COALESCE(SUM(pg.price), 0.0) / COUNT(*)::float
                    ELSE 0.0
                END as average_revenue_per_user
         FROM wallet_plan_assignments wga
         INNER JOIN plans pg ON wga.plan_id = pg.id
         WHERE wga.is_active = true AND pg.plan_type = 'subscription'
         GROUP BY pg.id, pg.name
         ORDER BY revenue DESC"
    )
    .load::<TierRevenueRow>(&mut conn)
    .await
    {
        Ok(results) => results.into_iter().map(|row| TierRevenue {
            tier_name: row.tier_name,
            revenue: row.revenue.map(|r| r.to_string().parse::<f64>().unwrap_or(0.0)).unwrap_or(0.0),
            subscriber_count: row.subscriber_count.unwrap_or(0) as i32,
            average_revenue_per_user: row.average_revenue_per_user.unwrap_or(0.0),
        }).collect(),
        Err(_) => Vec::new()
    };

    #[derive(QueryableByName)]
    struct CountResult {
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::BigInt>)]
        count: Option<i64>,
    }

    // Calculate subscription metrics
    let active_subscriptions = match diesel::sql_query(
        "SELECT COUNT(*)::bigint as count FROM wallet_plan_assignments wga
         INNER JOIN plans pg ON wga.plan_id = pg.id
         WHERE wga.is_active = true AND pg.plan_type = 'subscription'"
    )
    .get_result::<CountResult>(&mut conn)
    .await
    {
        Ok(result) => result.count.unwrap_or(0) as i32,
        Err(_) => 0
    };

    let new_subscriptions = match diesel::sql_query(
        "SELECT COUNT(*)::bigint as count FROM wallet_plan_assignments wga
         INNER JOIN plans pg ON wga.plan_id = pg.id
         WHERE wga.is_active = true AND pg.plan_type = 'subscription'
         AND wga.created_at >= NOW() - INTERVAL '30 days'"
    )
    .get_result::<CountResult>(&mut conn)
    .await
    {
        Ok(result) => result.count.unwrap_or(0) as i32,
        Err(_) => 0
    };

    let cancelled_subscriptions = match diesel::sql_query(
        "SELECT COUNT(*)::bigint as count FROM wallet_plan_assignments wga
         INNER JOIN plans pg ON wga.plan_id = pg.id
         WHERE wga.is_active = false AND pg.plan_type = 'subscription'
         AND wga.updated_at >= NOW() - INTERVAL '30 days'"
    )
    .get_result::<CountResult>(&mut conn)
    .await
    {
        Ok(result) => result.count.unwrap_or(0) as i32,
        Err(_) => 0
    };

    let subscription_churn_rate = if active_subscriptions > 0 {
        (cancelled_subscriptions as f64 / active_subscriptions as f64) * 100.0
    } else {
        0.0
    };

    #[derive(QueryableByName)]
    struct RevenueTrendRow {
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>)]
        trend_date: Option<DateTime<Utc>>,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Numeric>)]
        daily_revenue: Option<bigdecimal::BigDecimal>,
    }

    // Get revenue trends (last 30 days)
    let revenue_trends = match diesel::sql_query(
        r#"
        SELECT
            DATE_TRUNC('day', wga.created_at) as trend_date,
            COALESCE(SUM(pg.price), 0.0) as daily_revenue
        FROM wallet_plan_assignments wga
        INNER JOIN plans pg ON wga.plan_id = pg.id
        WHERE wga.created_at >= NOW() - INTERVAL '30 days'
          AND pg.plan_type = 'subscription'
        GROUP BY DATE_TRUNC('day', wga.created_at)
        ORDER BY trend_date ASC
        "#
    )
    .load::<RevenueTrendRow>(&mut conn)
    .await
    {
        Ok(results) => results
            .into_iter()
            .map(|row| TimeSeriesPoint {
                timestamp: row.trend_date.unwrap_or_else(Utc::now),
                value: row.daily_revenue.map(|r| r.to_string().parse::<f64>().unwrap_or(0.0)).unwrap_or(0.0),
                label: row.trend_date
                    .unwrap_or_else(Utc::now)
                    .format("%Y-%m-%d")
                    .to_string(),
            })
            .collect(),
        Err(_) => Vec::new(),
    };

    let response = RevenueAnalyticsResponse {
        total_revenue,
        monthly_recurring_revenue,
        revenue_by_tier,
        revenue_trends,
        subscription_metrics: SubscriptionMetrics {
            active_subscriptions,
            new_subscriptions,
            cancelled_subscriptions,
            subscription_churn_rate,
            upgrade_rate: 0.0, // Tier upgrade tracking requires transaction history table
            downgrade_rate: 0.0, // Tier downgrade tracking requires transaction history table
        },
        churn_analysis: ChurnAnalysis {
            monthly_churn_rate: subscription_churn_rate,
            churn_reasons: Vec::new(), // Churn reason tracking requires user feedback/survey table
            at_risk_users: 0, // At-risk user identification requires engagement metrics table
            prevented_churn: 0, // Prevented churn tracking requires intervention tracking table
        },
    };

    let metadata = AdminMetadata::crud_operation("get_revenue_analytics", Some("admin".to_string()));

    info!("✅ Admin: Successfully retrieved revenue analytics");
    Ok(Json(AdminApiResponse::success_with_meta(
        response,
        "Revenue analytics retrieved successfully",
        metadata,
    )))
}
