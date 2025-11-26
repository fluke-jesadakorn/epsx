// ============================================================================
// ADMIN ANALYTICS HANDLERS
// Data aggregation and business intelligence for admin interface
// ============================================================================

use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc, Duration};
use tracing::{error, info};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;

use crate::web::auth::AppState;
use crate::web::admin::responses::{AdminApiResponse, AdminMetadata};

// ============================================================================
// REQUEST/RESPONSE TYPES
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct AnalyticsQuery {
    pub period: Option<String>, // "7d", "30d", "90d", "1y"
    pub granularity: Option<String>, // "hour", "day", "week", "month"
    pub include_inactive: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct PlatformOverviewResponse {
    pub total_users: i32,
    pub active_users: i32,
    pub new_users_period: i32,
    pub retention_rate: f64,
    pub revenue_total: f64,
    pub revenue_period: f64,
    pub popular_features: Vec<FeatureUsage>,
    pub growth_metrics: GrowthMetrics,
    pub user_distribution: UserDistribution,
}

#[derive(Debug, Serialize)]
pub struct UserAnalyticsResponse {
    pub total_users: i32,
    pub active_users: i32,
    pub new_registrations: Vec<TimeSeriesPoint>,
    pub user_activity: Vec<TimeSeriesPoint>,
    pub tier_distribution: Vec<TierStats>,
    pub retention_cohorts: Vec<CohortData>,
    pub geographic_distribution: Vec<GeographicData>,
}

#[derive(Debug, Serialize)]
pub struct PermissionAnalyticsResponse {
    pub total_permissions: i32,
    pub active_permissions: i32,
    pub permission_usage: Vec<PermissionUsageStats>,
    pub group_membership: Vec<GroupMembershipStats>,
    pub permission_trends: Vec<TimeSeriesPoint>,
    pub expiring_permissions: Vec<ExpiringPermission>,
}

#[derive(Debug, Serialize)]
pub struct RevenueAnalyticsResponse {
    pub total_revenue: f64,
    pub monthly_recurring_revenue: f64,
    pub revenue_by_tier: Vec<TierRevenue>,
    pub revenue_trends: Vec<TimeSeriesPoint>,
    pub subscription_metrics: SubscriptionMetrics,
    pub churn_analysis: ChurnAnalysis,
}

#[derive(Debug, Serialize)]
pub struct FeatureUsage {
    pub feature_name: String,
    pub usage_count: i32,
    pub unique_users: i32,
    pub growth_rate: f64,
}

#[derive(Debug, Serialize)]
pub struct GrowthMetrics {
    pub daily_active_users: i32,
    pub weekly_active_users: i32,
    pub monthly_active_users: i32,
    pub user_growth_rate: f64,
    pub retention_7_day: f64,
    pub retention_30_day: f64,
}

#[derive(Debug, Serialize)]
pub struct UserDistribution {
    pub by_tier: Vec<TierStats>,
    pub by_region: Vec<RegionStats>,
    pub by_signup_date: Vec<TimeSeriesPoint>,
}

#[derive(Debug, Serialize)]
pub struct TimeSeriesPoint {
    pub timestamp: DateTime<Utc>,
    pub value: f64,
    pub label: String,
}

#[derive(Debug, Serialize)]
pub struct TierStats {
    pub tier_name: String,
    pub user_count: i32,
    pub percentage: f64,
    pub revenue: f64,
    pub growth_rate: f64,
}

#[derive(Debug, Serialize)]
pub struct RegionStats {
    pub region: String,
    pub user_count: i32,
    pub percentage: f64,
}

#[derive(Debug, Serialize)]
pub struct CohortData {
    pub cohort_period: String,
    pub cohort_size: i32,
    pub retention_periods: Vec<f64>,
}

#[derive(Debug, Serialize)]
pub struct GeographicData {
    pub country: String,
    pub user_count: i32,
    pub revenue: f64,
}

#[derive(Debug, Serialize)]
pub struct PermissionUsageStats {
    pub permission: String,
    pub users_count: i32,
    pub active_count: i32,
    pub usage_frequency: f64,
}

#[derive(Debug, Serialize)]
pub struct GroupMembershipStats {
    pub group_name: String,
    pub member_count: i32,
    pub active_members: i32,
    pub revenue_contribution: f64,
}

#[derive(Debug, Serialize)]
pub struct ExpiringPermission {
    pub wallet_address: String,
    pub permission: String,
    pub expires_at: DateTime<Utc>,
    pub days_until_expiry: i32,
}

#[derive(Debug, Serialize)]
pub struct TierRevenue {
    pub tier_name: String,
    pub revenue: f64,
    pub subscriber_count: i32,
    pub average_revenue_per_user: f64,
}

#[derive(Debug, Serialize)]
pub struct SubscriptionMetrics {
    pub active_subscriptions: i32,
    pub new_subscriptions: i32,
    pub cancelled_subscriptions: i32,
    pub subscription_churn_rate: f64,
    pub upgrade_rate: f64,
    pub downgrade_rate: f64,
}

#[derive(Debug, Serialize)]
pub struct ChurnAnalysis {
    pub monthly_churn_rate: f64,
    pub churn_reasons: Vec<ChurnReason>,
    pub at_risk_users: i32,
    pub prevented_churn: i32,
}

#[derive(Debug, Serialize)]
pub struct ChurnReason {
    pub reason: String,
    pub count: i32,
    pub percentage: f64,
}

// ============================================================================
// ANALYTICS HANDLERS
// ============================================================================

/**
 * Get platform overview analytics
 * GET /admin/analytics/overview
 */
pub async fn get_platform_overview_handler(
    Query(query): Query<AnalyticsQuery>,
    State(app_state): State<AppState>,
) -> Result<Json<AdminApiResponse<PlatformOverviewResponse>>, StatusCode> {
    info!("📊 Admin: Getting platform overview analytics");

    let mut conn = match app_state.db_pool.get().await {
        Ok(conn) => conn,
        Err(e) => {
            error!("❌ Admin: Failed to get database connection: {}", e);
            return Ok(Json(AdminApiResponse::server_error()));
        }
    };

    let period_days = match query.period.as_deref() {
        Some("7d") => 7,
        Some("30d") => 30,
        Some("90d") => 90,
        Some("1y") => 365,
        _ => 30,
    };

    let period_start = Utc::now() - Duration::days(period_days);

    #[derive(QueryableByName)]
    struct UserMetrics {
        #[diesel(sql_type = diesel::sql_types::BigInt)]
        total_users: i64,
        #[diesel(sql_type = diesel::sql_types::BigInt)]
        active_users: i64,
        #[diesel(sql_type = diesel::sql_types::BigInt)]
        new_users_period: i64,
    }

    // Get basic user metrics
    let user_metrics = match diesel::sql_query(
        "SELECT
            COUNT(*)::bigint as total_users,
            COUNT(*) FILTER (WHERE is_active = true)::bigint as active_users,
            COUNT(*) FILTER (WHERE created_at >= $1)::bigint as new_users_period
         FROM wallet_users"
    )
    .bind::<diesel::sql_types::Timestamptz, _>(period_start)
    .get_result::<UserMetrics>(&mut conn)
    .await
    {
        Ok(metrics) => metrics,
        Err(e) => {
            error!("❌ Admin: Failed to fetch user metrics: {}", e);
            return Ok(Json(AdminApiResponse::server_error()));
        }
    };

    // Tier distribution removed - tier_level column deleted in migration #023
    // Returns empty vector for backwards compatibility
    let tier_distribution: Vec<TierStats> = Vec::new();

    // Calculate retention rate (simplified)
    let retention_rate = if user_metrics.total_users > 0 {
        (user_metrics.active_users as f64 / user_metrics.total_users as f64) * 100.0
    } else {
        0.0
    };

    // Popular features (mock data)
    let popular_features = vec![
        FeatureUsage {
            feature_name: "Analytics Dashboard".to_string(),
            usage_count: user_metrics.active_users as i32,
            unique_users: user_metrics.active_users as i32,
            growth_rate: 15.2,
        },
        FeatureUsage {
            feature_name: "Permission Management".to_string(),
            usage_count: (user_metrics.active_users as f64 * 0.8) as i32,
            unique_users: (user_metrics.active_users as f64 * 0.8) as i32,
            growth_rate: 8.5,
        },
    ];

    // Growth metrics
    let growth_metrics = GrowthMetrics {
        daily_active_users: (user_metrics.active_users as f64 * 0.6) as i32,
        weekly_active_users: (user_metrics.active_users as f64 * 0.8) as i32,
        monthly_active_users: user_metrics.active_users as i32,
        user_growth_rate: if user_metrics.total_users > 0 {
            (user_metrics.new_users_period as f64 / user_metrics.total_users as f64) * 100.0
        } else {
            0.0
        },
        retention_7_day: retention_rate * 0.9,
        retention_30_day: retention_rate,
    };

    #[derive(QueryableByName)]
    struct SignupTrend {
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>)]
        signup_date: Option<DateTime<Utc>>,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::BigInt>)]
        user_count: Option<i64>,
    }

    // Generate signup trends from database
    let by_signup_date = match diesel::sql_query(
        r#"
        SELECT DATE_TRUNC('day', created_at) as signup_date,
               COUNT(*)::bigint as user_count
        FROM wallet_users
        WHERE created_at >= $1
        GROUP BY DATE_TRUNC('day', created_at)
        ORDER BY signup_date ASC
        "#
    )
    .bind::<diesel::sql_types::Timestamptz, _>(period_start)
    .load::<SignupTrend>(&mut conn)
    .await
    {
        Ok(results) => results
            .into_iter()
            .map(|row| TimeSeriesPoint {
                timestamp: row.signup_date.unwrap_or_else(|| Utc::now()),
                value: row.user_count.unwrap_or(0) as f64,
                label: row.signup_date
                    .unwrap_or_else(|| Utc::now())
                    .format("%Y-%m-%d")
                    .to_string(),
            })
            .collect(),
        Err(_) => Vec::new(),
    };

    let user_distribution = UserDistribution {
        by_tier: tier_distribution,
        by_region: vec![], // Geographic data not available (no IP/location tracking)
        by_signup_date,
    };

    #[derive(QueryableByName)]
    struct RevenueResult {
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Numeric>)]
        revenue: Option<bigdecimal::BigDecimal>,
    }

    let revenue_total = {
        // Calculate total revenue from all active subscription-based permission groups
        match diesel::sql_query(
            "SELECT COALESCE(SUM(pg.price), 0.0) as revenue FROM wallet_group_memberships wgm
             INNER JOIN permission_groups pg ON wgm.group_id = pg.id
             WHERE wgm.is_active = true AND pg.group_type = 'subscription'"
        )
        .get_result::<RevenueResult>(&mut conn)
        .await
        {
            Ok(result) => result.revenue.map(|r| r.to_string().parse::<f64>().unwrap_or(0.0)).unwrap_or(0.0),
            Err(_) => 0.0
        }
    };

    let revenue_period = {
        // Calculate revenue from new subscriptions in the last 30 days
        match diesel::sql_query(
            "SELECT COALESCE(SUM(pg.price), 0.0) as revenue FROM wallet_group_memberships wgm
             INNER JOIN permission_groups pg ON wgm.group_id = pg.id
             WHERE wgm.is_active = true AND pg.group_type = 'subscription'
             AND wgm.created_at >= NOW() - INTERVAL '30 days'"
        )
        .get_result::<RevenueResult>(&mut conn)
        .await
        {
            Ok(result) => result.revenue.map(|r| r.to_string().parse::<f64>().unwrap_or(0.0)).unwrap_or(0.0),
            Err(_) => 0.0
        }
    };

    let response = PlatformOverviewResponse {
        total_users: user_metrics.total_users as i32,
        active_users: user_metrics.active_users as i32,
        new_users_period: user_metrics.new_users_period as i32,
        retention_rate,
        revenue_total,
        revenue_period,
        popular_features,
        growth_metrics,
        user_distribution,
    };

    let metadata = AdminMetadata::crud_operation("get_platform_overview", Some("admin".to_string()));

    info!("✅ Admin: Successfully retrieved platform overview analytics");
    Ok(Json(AdminApiResponse::success_with_meta(
        response,
        "Platform overview analytics retrieved successfully",
        metadata,
    )))
}

/**
 * Get user analytics
 * GET /admin/analytics/users
 */
pub async fn get_user_analytics_handler(
    Query(query): Query<AnalyticsQuery>,
    State(app_state): State<AppState>,
) -> Result<Json<AdminApiResponse<UserAnalyticsResponse>>, StatusCode> {
    info!("👥 Admin: Getting user analytics");

    let mut conn = match app_state.db_pool.get().await {
        Ok(conn) => conn,
        Err(e) => {
            error!("❌ Admin: Failed to get database connection: {}", e);
            return Ok(Json(AdminApiResponse::server_error()));
        }
    };

    let period_days = match query.period.as_deref() {
        Some("7d") => 7,
        Some("30d") => 30,
        Some("90d") => 90,
        Some("1y") => 365,
        _ => 30,
    };

    #[derive(QueryableByName)]
    struct UserCounts {
        #[diesel(sql_type = diesel::sql_types::BigInt)]
        total_users: i64,
        #[diesel(sql_type = diesel::sql_types::BigInt)]
        active_users: i64,
    }

    // Get basic user counts
    let user_counts = match diesel::sql_query(
        "SELECT
            COUNT(*)::bigint as total_users,
            COUNT(*) FILTER (WHERE is_active = true)::bigint as active_users
         FROM wallet_users"
    )
    .get_result::<UserCounts>(&mut conn)
    .await
    {
        Ok(counts) => counts,
        Err(e) => {
            error!("❌ Admin: Failed to fetch user counts: {}", e);
            return Ok(Json(AdminApiResponse::server_error()));
        }
    };

    // Tier distribution removed - tier_level column deleted in migration #023
    // Returns empty vector for backwards compatibility
    let tier_distribution: Vec<TierStats> = Vec::new();

    // Generate time series data for registrations (mock data for now)
    let mut new_registrations = Vec::new();
    for i in 0..period_days {
        let date = Utc::now() - Duration::days(period_days - i - 1);
        new_registrations.push(TimeSeriesPoint {
            timestamp: date,
            value: (5.0 + (i as f64 % 10.0)), // Mock data
            label: date.format("%Y-%m-%d").to_string(),
        });
    }

    // Generate user activity data (mock)
    let mut user_activity = Vec::new();
    for i in 0..period_days {
        let date = Utc::now() - Duration::days(period_days - i - 1);
        user_activity.push(TimeSeriesPoint {
            timestamp: date,
            value: user_counts.active_users as f64 * (0.8 + ((i as f64 % 5.0) / 10.0)),
            label: date.format("%Y-%m-%d").to_string(),
        });
    }

    #[derive(QueryableByName)]
    struct CohortRow {
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
        cohort_period: Option<String>,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Integer>)]
        cohort_size: Option<i32>,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Double>)]
        retention_0m: Option<f64>,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Double>)]
        retention_1m: Option<f64>,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Double>)]
        retention_2m: Option<f64>,
    }

    // Implement cohort analysis (monthly cohorts with 3-month retention)
    let retention_cohorts = match diesel::sql_query(
        r#"
        WITH monthly_cohorts AS (
            SELECT
                DATE_TRUNC('month', created_at) as cohort_month,
                wallet_address
            FROM wallet_users
            WHERE created_at >= NOW() - INTERVAL '6 months'
        ),
        cohort_sizes AS (
            SELECT cohort_month, COUNT(*) as cohort_size
            FROM monthly_cohorts
            GROUP BY cohort_month
        ),
        cohort_retention AS (
            SELECT
                mc.cohort_month,
                cs.cohort_size,
                COUNT(DISTINCT CASE WHEN wu.last_auth_at >= mc.cohort_month THEN wu.wallet_address END) as retained_0m,
                COUNT(DISTINCT CASE WHEN wu.last_auth_at >= mc.cohort_month + INTERVAL '1 month' THEN wu.wallet_address END) as retained_1m,
                COUNT(DISTINCT CASE WHEN wu.last_auth_at >= mc.cohort_month + INTERVAL '2 months' THEN wu.wallet_address END) as retained_2m
            FROM monthly_cohorts mc
            JOIN cohort_sizes cs ON mc.cohort_month = cs.cohort_month
            LEFT JOIN wallet_users wu ON mc.wallet_address = wu.wallet_address
            GROUP BY mc.cohort_month, cs.cohort_size
        )
        SELECT
            TO_CHAR(cohort_month, 'YYYY-MM') as cohort_period,
            cohort_size::int as cohort_size,
            (retained_0m::float / cohort_size * 100) as retention_0m,
            (retained_1m::float / NULLIF(cohort_size, 0) * 100) as retention_1m,
            (retained_2m::float / NULLIF(cohort_size, 0) * 100) as retention_2m
        FROM cohort_retention
        ORDER BY cohort_month DESC
        "#
    )
    .load::<CohortRow>(&mut conn)
    .await
    {
        Ok(results) => results
            .into_iter()
            .map(|row| CohortData {
                cohort_period: row.cohort_period.unwrap_or_else(|| "Unknown".to_string()),
                cohort_size: row.cohort_size.unwrap_or(0),
                retention_periods: vec![
                    row.retention_0m.unwrap_or(100.0),
                    row.retention_1m.unwrap_or(0.0),
                    row.retention_2m.unwrap_or(0.0),
                ],
            })
            .collect(),
        Err(_) => Vec::new(),
    };

    let response = UserAnalyticsResponse {
        total_users: user_counts.total_users as i32,
        active_users: user_counts.active_users as i32,
        new_registrations,
        user_activity,
        tier_distribution,
        retention_cohorts,
        geographic_distribution: Vec::new(), // Geographic data not available (no IP/location tracking)
    };

    let metadata = AdminMetadata::crud_operation("get_user_analytics", Some("admin".to_string()));

    info!("✅ Admin: Successfully retrieved user analytics");
    Ok(Json(AdminApiResponse::success_with_meta(
        response,
        "User analytics retrieved successfully",
        metadata,
    )))
}

/**
 * Get permission analytics
 * GET /admin/analytics/permissions
 */
pub async fn get_permission_analytics_handler(
    Query(_query): Query<AnalyticsQuery>,
    State(app_state): State<AppState>,
) -> Result<Json<AdminApiResponse<PermissionAnalyticsResponse>>, StatusCode> {
    info!("🔐 Admin: Getting permission analytics");

    let mut conn = match app_state.db_pool.get().await {
        Ok(conn) => conn,
        Err(e) => {
            error!("❌ Admin: Failed to get database connection: {}", e);
            return Ok(Json(AdminApiResponse::server_error()));
        }
    };

    #[derive(QueryableByName)]
    struct GroupStatsRow {
        #[diesel(sql_type = diesel::sql_types::Text)]
        group_name: String,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::BigInt>)]
        member_count: Option<i64>,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::BigInt>)]
        active_members: Option<i64>,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Numeric>)]
        revenue: Option<bigdecimal::BigDecimal>,
    }

    // Get permission group stats with revenue
    let group_stats = match diesel::sql_query(
        r#"
        SELECT
            pg.name as group_name,
            COUNT(wgm.id)::bigint as member_count,
            COUNT(wgm.id) FILTER (WHERE wgm.is_active = true)::bigint as active_members,
            COALESCE(SUM(CASE WHEN wgm.is_active THEN pg.price ELSE 0 END), 0.0) as revenue
         FROM permission_groups pg
         LEFT JOIN wallet_group_memberships wgm ON pg.id = wgm.group_id
         WHERE pg.group_type = 'subscription'
         GROUP BY pg.id, pg.name
         ORDER BY member_count DESC
        "#
    )
    .load::<GroupStatsRow>(&mut conn)
    .await
    {
        Ok(stats) => stats
            .into_iter()
            .map(|stat| GroupMembershipStats {
                group_name: stat.group_name,
                member_count: stat.member_count.unwrap_or(0) as i32,
                active_members: stat.active_members.unwrap_or(0) as i32,
                revenue_contribution: stat.revenue.map(|r| r.to_string().parse::<f64>().unwrap_or(0.0)).unwrap_or(0.0),
            })
            .collect(),
        Err(_) => Vec::new(),
    };

    // Mock permission usage data
    let permission_usage = vec![
        PermissionUsageStats {
            permission: "epsx:analytics:view".to_string(),
            users_count: 150,
            active_count: 120,
            usage_frequency: 85.5,
        },
        PermissionUsageStats {
            permission: "epsx:export:data".to_string(),
            users_count: 75,
            active_count: 60,
            usage_frequency: 45.2,
        },
    ];

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
                timestamp: row.trend_date.unwrap_or_else(|| Utc::now()),
                value: row.permission_count.unwrap_or(0) as f64,
                label: row.trend_date
                    .unwrap_or_else(|| Utc::now())
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
                expires_at: row.expires_at.unwrap_or_else(|| Utc::now()),
                days_until_expiry: row.days_until_expiry.unwrap_or(0),
            })
            .collect(),
        Err(_) => Vec::new(),
    };

    let response = PermissionAnalyticsResponse {
        total_permissions: permission_usage.iter().map(|p| p.users_count).sum(),
        active_permissions: permission_usage.iter().map(|p| p.active_count).sum(),
        permission_usage,
        group_membership: group_stats,
        permission_trends,
        expiring_permissions,
    };

    let metadata = AdminMetadata::crud_operation("get_permission_analytics", Some("admin".to_string()));

    info!("✅ Admin: Successfully retrieved permission analytics");
    Ok(Json(AdminApiResponse::success_with_meta(
        response,
        "Permission analytics retrieved successfully",
        metadata,
    )))
}

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
        "SELECT COALESCE(SUM(pg.price), 0.0) as revenue FROM wallet_group_memberships wgm
         INNER JOIN permission_groups pg ON wgm.group_id = pg.id
         WHERE wgm.is_active = true AND pg.group_type = 'subscription'"
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
        ), 0.0) as revenue FROM wallet_group_memberships wgm
         INNER JOIN permission_groups pg ON wgm.group_id = pg.id
         WHERE wgm.is_active = true AND pg.group_type = 'subscription'
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

    // Get revenue by tier/permission group
    let revenue_by_tier = match diesel::sql_query(
        "SELECT pg.name as tier_name,
                COALESCE(SUM(pg.price), 0.0) as revenue,
                COUNT(*)::bigint as subscriber_count,
                CASE
                    WHEN COUNT(*) > 0 THEN COALESCE(SUM(pg.price), 0.0) / COUNT(*)::float
                    ELSE 0.0
                END as average_revenue_per_user
         FROM wallet_group_memberships wgm
         INNER JOIN permission_groups pg ON wgm.group_id = pg.id
         WHERE wgm.is_active = true AND pg.group_type = 'subscription'
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
        "SELECT COUNT(*)::bigint as count FROM wallet_group_memberships wgm
         INNER JOIN permission_groups pg ON wgm.group_id = pg.id
         WHERE wgm.is_active = true AND pg.group_type = 'subscription'"
    )
    .get_result::<CountResult>(&mut conn)
    .await
    {
        Ok(result) => result.count.unwrap_or(0) as i32,
        Err(_) => 0
    };

    let new_subscriptions = match diesel::sql_query(
        "SELECT COUNT(*)::bigint as count FROM wallet_group_memberships wgm
         INNER JOIN permission_groups pg ON wgm.group_id = pg.id
         WHERE wgm.is_active = true AND pg.group_type = 'subscription'
         AND wgm.created_at >= NOW() - INTERVAL '30 days'"
    )
    .get_result::<CountResult>(&mut conn)
    .await
    {
        Ok(result) => result.count.unwrap_or(0) as i32,
        Err(_) => 0
    };

    let cancelled_subscriptions = match diesel::sql_query(
        "SELECT COUNT(*)::bigint as count FROM wallet_group_memberships wgm
         INNER JOIN permission_groups pg ON wgm.group_id = pg.id
         WHERE wgm.is_active = false AND pg.group_type = 'subscription'
         AND wgm.updated_at >= NOW() - INTERVAL '30 days'"
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
            DATE_TRUNC('day', wgm.created_at) as trend_date,
            COALESCE(SUM(pg.price), 0.0) as daily_revenue
        FROM wallet_group_memberships wgm
        INNER JOIN permission_groups pg ON wgm.group_id = pg.id
        WHERE wgm.created_at >= NOW() - INTERVAL '30 days'
          AND pg.group_type = 'subscription'
        GROUP BY DATE_TRUNC('day', wgm.created_at)
        ORDER BY trend_date ASC
        "#
    )
    .load::<RevenueTrendRow>(&mut conn)
    .await
    {
        Ok(results) => results
            .into_iter()
            .map(|row| TimeSeriesPoint {
                timestamp: row.trend_date.unwrap_or_else(|| Utc::now()),
                value: row.daily_revenue.map(|r| r.to_string().parse::<f64>().unwrap_or(0.0)).unwrap_or(0.0),
                label: row.trend_date
                    .unwrap_or_else(|| Utc::now())
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