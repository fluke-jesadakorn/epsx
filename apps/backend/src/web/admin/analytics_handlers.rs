use axum::{
    extract::{Query, State},
    response::Json,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::{
    web::auth::AppState,
    core::errors::{AppError, ErrorKind},
};

// Analytics DTOs
#[derive(Debug, Serialize, Deserialize)]
pub struct PermissionAnalyticsQuery {
    pub time_range: Option<String>, // 24h, 7d, 30d, 90d
    pub include_trends: Option<bool>,
    pub include_users: Option<bool>,
    pub include_performance: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct PermissionAnalyticsResponse {
    pub total_users: u64,
    pub total_permissions: u64,
    pub total_profiles: u64,
    pub active_temporary_permissions: u64,
    pub recent_assignments: u64,
    pub security_alerts: u64,
    pub usage_stats: UsageStats,
    pub performance_metrics: PerformanceMetrics,
    pub generated_at: DateTime<Utc>,
    pub time_range: String,
}

#[derive(Debug, Serialize)]
pub struct UsageStats {
    pub profile_usage: Vec<ProfileUsageStat>,
    pub permission_trends: Vec<PermissionTrend>,
    pub user_activity: Vec<UserActivityStat>,
    pub risk_distribution: RiskDistribution,
}

#[derive(Debug, Serialize)]
pub struct ProfileUsageStat {
    pub profile_name: String,
    pub profile_id: String,
    pub assignment_count: u64,
    pub percentage: f64,
    pub category: String,
    pub risk_level: String,
}

#[derive(Debug, Serialize)]
pub struct PermissionTrend {
    pub date: String,
    pub assignments: u64,
    pub revocations: u64,
    pub net_change: i64,
    pub category_breakdown: HashMap<String, u64>,
}

#[derive(Debug, Serialize)]
pub struct UserActivityStat {
    pub user_id: String,
    pub user_email: String,
    pub last_activity: DateTime<Utc>,
    pub permission_count: u64,
    pub risk_score: f64,
    pub status: String,
    pub department: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct RiskDistribution {
    pub low_risk: u64,
    pub medium_risk: u64,
    pub high_risk: u64,
    pub critical_risk: u64,
}

#[derive(Debug, Serialize)]
pub struct PerformanceMetrics {
    pub avg_response_time: f64,
    pub cache_hit_rate: f64,
    pub error_rate: f64,
    pub throughput: f64,
    pub concurrent_users: u64,
    pub database_query_time: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RecommendationAnalyticsQuery {
    pub user_id: Option<String>,
    pub category: Option<String>, // security, efficiency, compliance, role-based
    pub confidence_threshold: Option<f64>,
}

#[derive(Debug, Serialize)]
pub struct RecommendationAnalyticsResponse {
    pub recommendations: Vec<SmartRecommendation>,
    pub insights: RecommendationInsights,
    pub trends: Vec<RecommendationTrend>,
    pub generated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct SmartRecommendation {
    pub id: String,
    pub user_id: String,
    pub recommendation_type: String, // add, remove, upgrade, temporary
    pub permission: String,
    pub resource: String,
    pub action: String,
    pub confidence: f64,
    pub reasoning: String,
    pub category: String,
    pub impact: String,
    pub similar_users: Vec<String>,
    pub estimated_benefit: String,
    pub risks: Vec<String>,
    pub prerequisites: Vec<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct RecommendationInsights {
    pub over_privileged_users: u64,
    pub under_privileged_users: u64,
    pub obsolete_permissions: u64,
    pub security_risks: u64,
    pub efficiency_opportunities: u64,
}

#[derive(Debug, Serialize)]
pub struct RecommendationTrend {
    pub category: String,
    pub count: u64,
    pub change: i64,
    pub acceptance_rate: f64,
}

async fn verify_admin_access(app_state: &AppState, resource: &str, action: &str) -> Result<(), AppError> {
    let user_id = "admin_user"; // TODO: Extract from authenticated context
    
    match app_state.casbin_service.enforce(user_id, resource, action).await {
        Ok(true) => Ok(()),
        Ok(false) => Err(AppError::new(
            ErrorKind::AuthorizationError,
            format!("Access denied for {}/{}", resource, action),
        )),
        Err(e) => Err(AppError::new(
            ErrorKind::InternalServerError,
            format!("Failed to check permissions: {}", e),
        )),
    }
}

/// Get comprehensive permission analytics
pub async fn get_permission_analytics_handler(
    State(app_state): State<AppState>,
    Query(query): Query<PermissionAnalyticsQuery>,
) -> Result<Json<PermissionAnalyticsResponse>, AppError> {
    verify_admin_access(&app_state, "analytics", "read").await?;

    let time_range = query.time_range.unwrap_or_else(|| "7d".to_string());
    
    // Mock analytics data - in production would query actual database
    let analytics = PermissionAnalyticsResponse {
        total_users: 1247,
        total_permissions: 3842,
        total_profiles: 28,
        active_temporary_permissions: 156,
        recent_assignments: 89,
        security_alerts: 3,
        usage_stats: UsageStats {
            profile_usage: vec![
                ProfileUsageStat {
                    profile_name: "user-basic-001".to_string(),
                    profile_id: "profile-1".to_string(),
                    assignment_count: 892,
                    percentage: 71.5,
                    category: "User".to_string(),
                    risk_level: "low".to_string(),
                },
                ProfileUsageStat {
                    profile_name: "user-premium-002".to_string(),
                    profile_id: "profile-2".to_string(),
                    assignment_count: 245,
                    percentage: 19.6,
                    category: "User".to_string(),
                    risk_level: "medium".to_string(),
                },
                ProfileUsageStat {
                    profile_name: "moderator-standard-003".to_string(),
                    profile_id: "profile-3".to_string(),
                    assignment_count: 85,
                    percentage: 6.8,
                    category: "Moderator".to_string(),
                    risk_level: "medium".to_string(),
                },
                ProfileUsageStat {
                    profile_name: "admin-full-004".to_string(),
                    profile_id: "profile-4".to_string(),
                    assignment_count: 25,
                    percentage: 2.0,
                    category: "Admin".to_string(),
                    risk_level: "high".to_string(),
                },
            ],
            permission_trends: vec![
                PermissionTrend {
                    date: "2025-01-01".to_string(),
                    assignments: 45,
                    revocations: 12,
                    net_change: 33,
                    category_breakdown: {
                        let mut map = HashMap::new();
                        map.insert("User".to_string(), 30);
                        map.insert("Admin".to_string(), 15);
                        map
                    },
                },
                PermissionTrend {
                    date: "2025-01-02".to_string(),
                    assignments: 52,
                    revocations: 8,
                    net_change: 44,
                    category_breakdown: {
                        let mut map = HashMap::new();
                        map.insert("User".to_string(), 35);
                        map.insert("Admin".to_string(), 17);
                        map
                    },
                },
            ],
            user_activity: vec![
                UserActivityStat {
                    user_id: "user-1".to_string(),
                    user_email: "john@example.com".to_string(),
                    last_activity: Utc::now(),
                    permission_count: 15,
                    risk_score: 2.1,
                    status: "active".to_string(),
                    department: Some("Engineering".to_string()),
                },
                UserActivityStat {
                    user_id: "user-2".to_string(),
                    user_email: "jane@example.com".to_string(),
                    last_activity: Utc::now(),
                    permission_count: 28,
                    risk_score: 4.7,
                    status: "active".to_string(),
                    department: Some("Marketing".to_string()),
                },
            ],
            risk_distribution: RiskDistribution {
                low_risk: 892,
                medium_risk: 287,
                high_risk: 56,
                critical_risk: 12,
            },
        },
        performance_metrics: PerformanceMetrics {
            avg_response_time: 127.5,
            cache_hit_rate: 94.2,
            error_rate: 0.3,
            throughput: 245.7,
            concurrent_users: 127,
            database_query_time: 45.2,
        },
        generated_at: Utc::now(),
        time_range,
    };

    Ok(Json(analytics))
}

/// Get smart permission recommendations
pub async fn get_permission_recommendations_handler(
    State(app_state): State<AppState>,
    Query(query): Query<RecommendationAnalyticsQuery>,
) -> Result<Json<RecommendationAnalyticsResponse>, AppError> {
    verify_admin_access(&app_state, "analytics", "read").await?;

    // Mock AI-powered recommendations
    let recommendations = RecommendationAnalyticsResponse {
        recommendations: vec![
            SmartRecommendation {
                id: "rec-1".to_string(),
                user_id: query.user_id.clone().unwrap_or_else(|| "user-1".to_string()),
                recommendation_type: "add".to_string(),
                permission: "analytics:read".to_string(),
                resource: "dashboard".to_string(),
                action: "view".to_string(),
                confidence: 92.0,
                reasoning: "User frequently requests analytics reports and works in similar role as 85% of users who have this permission".to_string(),
                category: "efficiency".to_string(),
                impact: "medium".to_string(),
                similar_users: vec!["jane@example.com".to_string(), "bob@example.com".to_string()],
                estimated_benefit: "Reduce approval wait time by 75%, increase productivity".to_string(),
                risks: vec![],
                prerequisites: vec![],
                created_at: Utc::now(),
            },
            SmartRecommendation {
                id: "rec-2".to_string(),
                user_id: query.user_id.unwrap_or_else(|| "user-1".to_string()),
                recommendation_type: "remove".to_string(),
                permission: "admin:delete".to_string(),
                resource: "system".to_string(),
                action: "delete".to_string(),
                confidence: 88.0,
                reasoning: "Permission not used in 90+ days and exceeds role requirements".to_string(),
                category: "security".to_string(),
                impact: "high".to_string(),
                similar_users: vec![],
                estimated_benefit: "Reduce security risk exposure by 40%".to_string(),
                risks: vec!["High privilege level".to_string(), "Unused for extended period".to_string()],
                prerequisites: vec![],
                created_at: Utc::now(),
            },
        ],
        insights: RecommendationInsights {
            over_privileged_users: 12,
            under_privileged_users: 8,
            obsolete_permissions: 5,
            security_risks: 3,
            efficiency_opportunities: 23,
        },
        trends: vec![
            RecommendationTrend {
                category: "Security".to_string(),
                count: 15,
                change: -8,
                acceptance_rate: 78.5,
            },
            RecommendationTrend {
                category: "Efficiency".to_string(),
                count: 23,
                change: 12,
                acceptance_rate: 65.2,
            },
            RecommendationTrend {
                category: "Compliance".to_string(),
                count: 7,
                change: 2,
                acceptance_rate: 89.1,
            },
        ],
        generated_at: Utc::now(),
    };

    Ok(Json(recommendations))
}

/// Get system performance metrics
pub async fn get_performance_metrics_handler(
    State(app_state): State<AppState>,
) -> Result<Json<PerformanceMetrics>, AppError> {
    verify_admin_access(&app_state, "analytics", "read").await?;

    // Mock performance data - in production would collect from actual system
    let metrics = PerformanceMetrics {
        avg_response_time: 127.5,
        cache_hit_rate: 94.2,
        error_rate: 0.3,
        throughput: 245.7,
        concurrent_users: 127,
        database_query_time: 45.2,
    };

    Ok(Json(metrics))
}

/// Get security risk analysis
pub async fn get_security_risk_analysis_handler(
    State(app_state): State<AppState>,
) -> Result<Json<RiskAnalysisResponse>, AppError> {
    verify_admin_access(&app_state, "analytics", "read").await?;

    let analysis = RiskAnalysisResponse {
        risk_distribution: RiskDistribution {
            low_risk: 892,
            medium_risk: 287,
            high_risk: 56,
            critical_risk: 12,
        },
        high_risk_users: vec![
            HighRiskUser {
                user_id: "user-123".to_string(),
                email: "admin@example.com".to_string(),
                risk_score: 8.9,
                risk_factors: vec![
                    "Multiple admin permissions".to_string(),
                    "Recently assigned critical access".to_string(),
                ],
                last_activity: Utc::now(),
                recommended_actions: vec![
                    "Review admin permissions".to_string(),
                    "Enable additional MFA".to_string(),
                ],
            },
        ],
        security_trends: vec![
            SecurityTrend {
                date: "2025-01-07".to_string(),
                new_risks: 2,
                resolved_risks: 5,
                risk_score_change: -0.3,
            },
        ],
        compliance_status: ComplianceStatus {
            compliant_users: 95.2,
            policy_violations: 3,
            pending_reviews: 8,
        },
        generated_at: Utc::now(),
    };

    Ok(Json(analysis))
}

#[derive(Debug, Serialize)]
pub struct RiskAnalysisResponse {
    pub risk_distribution: RiskDistribution,
    pub high_risk_users: Vec<HighRiskUser>,
    pub security_trends: Vec<SecurityTrend>,
    pub compliance_status: ComplianceStatus,
    pub generated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct HighRiskUser {
    pub user_id: String,
    pub email: String,
    pub risk_score: f64,
    pub risk_factors: Vec<String>,
    pub last_activity: DateTime<Utc>,
    pub recommended_actions: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct SecurityTrend {
    pub date: String,
    pub new_risks: u64,
    pub resolved_risks: u64,
    pub risk_score_change: f64,
}

#[derive(Debug, Serialize)]
pub struct ComplianceStatus {
    pub compliant_users: f64, // percentage
    pub policy_violations: u64,
    pub pending_reviews: u64,
}