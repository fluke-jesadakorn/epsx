use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

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
    pub total_groups: i32,
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
