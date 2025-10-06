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

    let db_pool = app_state.db_pool.as_ref();
    let period_days = match query.period.as_deref() {
        Some("7d") => 7,
        Some("30d") => 30,
        Some("90d") => 90,
        Some("1y") => 365,
        _ => 30,
    };

    let period_start = Utc::now() - Duration::days(period_days);

    // Get basic user metrics
    let user_metrics = match sqlx::query!(
        "SELECT 
            COUNT(*) as total_users,
            COUNT(*) FILTER (WHERE is_active = true) as active_users,
            COUNT(*) FILTER (WHERE created_at >= $1) as new_users_period
         FROM wallet_users",
        period_start
    )
    .fetch_one(db_pool)
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
    let retention_rate = if user_metrics.total_users.unwrap_or(0) > 0 {
        (user_metrics.active_users.unwrap_or(0) as f64 / user_metrics.total_users.unwrap_or(1) as f64) * 100.0
    } else {
        0.0
    };

    // Popular features (mock data)
    let popular_features = vec![
        FeatureUsage {
            feature_name: "Analytics Dashboard".to_string(),
            usage_count: user_metrics.active_users.unwrap_or(0) as i32,
            unique_users: user_metrics.active_users.unwrap_or(0) as i32,
            growth_rate: 15.2,
        },
        FeatureUsage {
            feature_name: "Permission Management".to_string(),
            usage_count: (user_metrics.active_users.unwrap_or(0) as f64 * 0.8) as i32,
            unique_users: (user_metrics.active_users.unwrap_or(0) as f64 * 0.8) as i32,
            growth_rate: 8.5,
        },
    ];

    // Growth metrics
    let growth_metrics = GrowthMetrics {
        daily_active_users: (user_metrics.active_users.unwrap_or(0) as f64 * 0.6) as i32,
        weekly_active_users: (user_metrics.active_users.unwrap_or(0) as f64 * 0.8) as i32,
        monthly_active_users: user_metrics.active_users.unwrap_or(0) as i32,
        user_growth_rate: if user_metrics.total_users.unwrap_or(0) > 0 {
            (user_metrics.new_users_period.unwrap_or(0) as f64 / user_metrics.total_users.unwrap_or(1) as f64) * 100.0
        } else {
            0.0
        },
        retention_7_day: retention_rate * 0.9,
        retention_30_day: retention_rate,
    };

    let user_distribution = UserDistribution {
        by_tier: tier_distribution,
        by_region: vec![], // TODO: Implement geographic data
        by_signup_date: vec![], // TODO: Implement signup trends
    };

    let response = PlatformOverviewResponse {
        total_users: user_metrics.total_users.unwrap_or(0) as i32,
        active_users: user_metrics.active_users.unwrap_or(0) as i32,
        new_users_period: user_metrics.new_users_period.unwrap_or(0) as i32,
        retention_rate,
        revenue_total: {
            // Calculate total revenue from all active subscription-based permission groups
            match sqlx::query_scalar!(
                "SELECT COALESCE(SUM(pg.price), 0.0) FROM wallet_group_memberships wgm 
                 INNER JOIN permission_groups pg ON wgm.group_id = pg.id 
                 WHERE wgm.is_active = true AND pg.group_type = 'subscription'"
            ).fetch_one(db_pool).await {
                Ok(revenue) => revenue.map(|r| r.to_string().parse::<f64>().unwrap_or(0.0)).unwrap_or(0.0),
                Err(_) => 0.0
            }
        },
        revenue_period: {
            // Calculate revenue from new subscriptions in the last 30 days
            match sqlx::query_scalar!(
                "SELECT COALESCE(SUM(pg.price), 0.0) FROM wallet_group_memberships wgm 
                 INNER JOIN permission_groups pg ON wgm.group_id = pg.id 
                 WHERE wgm.is_active = true AND pg.group_type = 'subscription' 
                 AND wgm.created_at >= NOW() - INTERVAL '30 days'"
            ).fetch_one(db_pool).await {
                Ok(revenue) => revenue.map(|r| r.to_string().parse::<f64>().unwrap_or(0.0)).unwrap_or(0.0),
                Err(_) => 0.0
            }
        },
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

    let db_pool = app_state.db_pool.as_ref();
    let period_days = match query.period.as_deref() {
        Some("7d") => 7,
        Some("30d") => 30,
        Some("90d") => 90,
        Some("1y") => 365,
        _ => 30,
    };

    // Get basic user counts
    let user_counts = match sqlx::query!(
        "SELECT 
            COUNT(*) as total_users,
            COUNT(*) FILTER (WHERE is_active = true) as active_users
         FROM wallet_users"
    )
    .fetch_one(db_pool)
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
            value: user_counts.active_users.unwrap_or(0) as f64 * (0.8 + ((i as f64 % 5.0) / 10.0)),
            label: date.format("%Y-%m-%d").to_string(),
        });
    }

    let response = UserAnalyticsResponse {
        total_users: user_counts.total_users.unwrap_or(0) as i32,
        active_users: user_counts.active_users.unwrap_or(0) as i32,
        new_registrations,
        user_activity,
        tier_distribution,
        retention_cohorts: Vec::new(), // TODO: Implement cohort analysis
        geographic_distribution: Vec::new(), // TODO: Implement geographic data
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

    let db_pool = app_state.db_pool.as_ref();

    // Get permission group stats
    let group_stats = match sqlx::query!(
        "SELECT 
            pg.name as group_name,
            COUNT(wgm.id) as member_count,
            COUNT(wgm.id) FILTER (WHERE wgm.is_active = true) as active_members
         FROM permission_groups pg
         LEFT JOIN wallet_group_memberships wgm ON pg.id = wgm.group_id
         GROUP BY pg.id, pg.name
         ORDER BY member_count DESC"
    )
    .fetch_all(db_pool)
    .await
    {
        Ok(stats) => stats
            .into_iter()
            .map(|stat| GroupMembershipStats {
                group_name: stat.group_name,
                member_count: stat.member_count.unwrap_or(0) as i32,
                active_members: stat.active_members.unwrap_or(0) as i32,
                revenue_contribution: 0.0, // TODO: Calculate revenue
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

    let response = PermissionAnalyticsResponse {
        total_permissions: permission_usage.iter().map(|p| p.users_count).sum(),
        active_permissions: permission_usage.iter().map(|p| p.active_count).sum(),
        permission_usage,
        group_membership: group_stats,
        permission_trends: Vec::new(), // TODO: Implement trends
        expiring_permissions: Vec::new(), // TODO: Implement expiry tracking
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

    let db_pool = app_state.db_pool.as_ref();

    // Calculate total revenue from active subscriptions and lifetime packages
    let total_revenue = match sqlx::query_scalar!(
        "SELECT COALESCE(SUM(pg.price), 0.0) FROM wallet_group_memberships wgm 
         INNER JOIN permission_groups pg ON wgm.group_id = pg.id 
         WHERE wgm.is_active = true AND pg.group_type = 'subscription'"
    ).fetch_one(db_pool).await {
        Ok(revenue) => revenue.map(|r| r.to_string().parse::<f64>().unwrap_or(0.0)).unwrap_or(0.0),
        Err(_) => 0.0
    };

    // Calculate monthly recurring revenue (only monthly/yearly subscriptions)
    let monthly_recurring_revenue = match sqlx::query_scalar!(
        "SELECT COALESCE(SUM(
            CASE 
                WHEN pg.billing_cycle = 'monthly' THEN pg.price
                WHEN pg.billing_cycle = 'yearly' THEN pg.price / 12.0
                ELSE 0.0
            END
        ), 0.0) FROM wallet_group_memberships wgm 
         INNER JOIN permission_groups pg ON wgm.group_id = pg.id 
         WHERE wgm.is_active = true AND pg.group_type = 'subscription' 
         AND pg.billing_cycle IN ('monthly', 'yearly')"
    ).fetch_one(db_pool).await {
        Ok(mrr) => mrr.map(|r| r.to_string().parse::<f64>().unwrap_or(0.0)).unwrap_or(0.0),
        Err(_) => 0.0
    };

    // Get revenue by tier/permission group
    let revenue_by_tier = match sqlx::query!(
        "SELECT pg.name as tier_name, 
                COALESCE(SUM(pg.price), 0.0) as revenue,
                COUNT(*) as subscriber_count,
                CASE 
                    WHEN COUNT(*) > 0 THEN COALESCE(SUM(pg.price), 0.0) / COUNT(*)::float
                    ELSE 0.0
                END as average_revenue_per_user
         FROM wallet_group_memberships wgm 
         INNER JOIN permission_groups pg ON wgm.group_id = pg.id 
         WHERE wgm.is_active = true AND pg.group_type = 'subscription'
         GROUP BY pg.id, pg.name
         ORDER BY revenue DESC"
    ).fetch_all(db_pool).await {
        Ok(results) => results.into_iter().map(|row| TierRevenue {
            tier_name: row.tier_name,
            revenue: row.revenue.map(|r| r.to_string().parse::<f64>().unwrap_or(0.0)).unwrap_or(0.0),
            subscriber_count: row.subscriber_count.unwrap_or(0) as i32,
            average_revenue_per_user: row.average_revenue_per_user.unwrap_or(0.0),
        }).collect(),
        Err(_) => Vec::new()
    };

    // Calculate subscription metrics
    let active_subscriptions = match sqlx::query_scalar!(
        "SELECT COUNT(*) FROM wallet_group_memberships wgm 
         INNER JOIN permission_groups pg ON wgm.group_id = pg.id 
         WHERE wgm.is_active = true AND pg.group_type = 'subscription'"
    ).fetch_one(db_pool).await {
        Ok(count) => count.unwrap_or(0) as i32,
        Err(_) => 0
    };

    let new_subscriptions = match sqlx::query_scalar!(
        "SELECT COUNT(*) FROM wallet_group_memberships wgm 
         INNER JOIN permission_groups pg ON wgm.group_id = pg.id 
         WHERE wgm.is_active = true AND pg.group_type = 'subscription' 
         AND wgm.created_at >= NOW() - INTERVAL '30 days'"
    ).fetch_one(db_pool).await {
        Ok(count) => count.unwrap_or(0) as i32,
        Err(_) => 0
    };

    let cancelled_subscriptions = match sqlx::query_scalar!(
        "SELECT COUNT(*) FROM wallet_group_memberships wgm 
         INNER JOIN permission_groups pg ON wgm.group_id = pg.id 
         WHERE wgm.is_active = false AND pg.group_type = 'subscription' 
         AND wgm.updated_at >= NOW() - INTERVAL '30 days'"
    ).fetch_one(db_pool).await {
        Ok(count) => count.unwrap_or(0) as i32,
        Err(_) => 0
    };

    let subscription_churn_rate = if active_subscriptions > 0 {
        (cancelled_subscriptions as f64 / active_subscriptions as f64) * 100.0
    } else {
        0.0
    };

    let response = RevenueAnalyticsResponse {
        total_revenue,
        monthly_recurring_revenue,
        revenue_by_tier,
        revenue_trends: Vec::new(), // TODO: Implement time series revenue trends
        subscription_metrics: SubscriptionMetrics {
            active_subscriptions,
            new_subscriptions,
            cancelled_subscriptions,
            subscription_churn_rate,
            upgrade_rate: 0.0, // TODO: Implement tier upgrade tracking
            downgrade_rate: 0.0, // TODO: Implement tier downgrade tracking
        },
        churn_analysis: ChurnAnalysis {
            monthly_churn_rate: subscription_churn_rate,
            churn_reasons: Vec::new(), // TODO: Implement churn reason tracking
            at_risk_users: 0, // TODO: Implement at-risk user identification
            prevented_churn: 0, // TODO: Implement prevented churn tracking
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