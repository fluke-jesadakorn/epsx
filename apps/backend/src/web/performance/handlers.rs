// Performance monitoring API handlers

use axum::{
    extract::{Query, Path},
    http::StatusCode,
    response::Json,
    Extension,
};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc, Duration};
use uuid::Uuid;
use std::sync::Arc;
use tracing::{info, error};

use crate::web::performance::{
    models::*,
    PerformanceService,
};

/// Performance dashboard summary
pub async fn get_performance_dashboard(
    Query(params): Query<DashboardQueryParams>,
    Extension(service): Extension<Arc<PerformanceService>>,
) -> Result<Json<PerformanceDashboardResponse>, StatusCode> {
    info!("Getting performance dashboard");

    let end_time = Utc::now();
    let _start_time = end_time - Duration::hours(params.hours.unwrap_or(24) as i64);

    match service.analytics.generate_insights(params.hours.unwrap_or(24)).await {
        Ok(insights) => {
            let response = PerformanceDashboardResponse {
                summary: insights.summary.clone(),
                slowest_endpoints: insights.slowest_endpoints,
                cache_analytics: insights.cache_analytics,
                system_health: insights.system_health.clone(),
                sla_compliance: insights.sla_compliance,
                active_alerts_count: insights.anomalies.len() as i32,
                timestamp: Utc::now(),
            };
            Ok(Json(response))
        }
        Err(e) => {
            error!("Failed to get performance dashboard: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Get performance metrics for specific endpoint
pub async fn get_endpoint_performance(
    Path(endpoint): Path<String>,
    Query(params): Query<PerformanceQueryParams>,
    Extension(service): Extension<Arc<PerformanceService>>,
) -> Result<Json<EndpointPerformanceResponse>, StatusCode> {
    info!("Getting performance metrics for endpoint: {}", endpoint);

    let end_time = params.end_time.unwrap_or_else(|| Utc::now());
    let start_time = params.start_time.unwrap_or_else(|| end_time - Duration::hours(24));

    match service.repo.get_slowest_endpoints(start_time, end_time, 100).await {
        Ok(endpoints) => {
            if let Some(endpoint_perf) = endpoints.into_iter().find(|e| e.endpoint == endpoint) {
                let trends = service.repo.get_performance_trends(
                    Some(&endpoint),
                    MetricType::Latency,
                    start_time,
                    end_time,
                    params.time_bucket.unwrap_or(TimeBucket::Hour),
                ).await.unwrap_or_default();

                let response = EndpointPerformanceResponse {
                    endpoint_performance: endpoint_perf,
                    trends,
                    recommendations: vec![], // TODO: Get endpoint-specific recommendations
                };
                Ok(Json(response))
            } else {
                Err(StatusCode::NOT_FOUND)
            }
        }
        Err(e) => {
            error!("Failed to get endpoint performance: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Get performance percentiles
pub async fn get_performance_percentiles(
    Query(params): Query<PercentileQueryParams>,
    Extension(service): Extension<Arc<PerformanceService>>,
) -> Result<Json<PercentileResponse>, StatusCode> {
    info!("Getting performance percentiles");

    let end_time = params.end_time.unwrap_or_else(|| Utc::now());
    let start_time = params.start_time.unwrap_or_else(|| end_time - Duration::hours(24));

    match service.analytics.calculate_percentiles(
        params.endpoint.as_deref(),
        start_time,
        end_time,
    ).await {
        Ok(percentiles) => {
            let response = PercentileResponse {
                percentiles,
                timeframe: TimeframeInfo {
                    start_time,
                    end_time,
                    duration_hours: (end_time - start_time).num_hours(),
                },
            };
            Ok(Json(response))
        }
        Err(e) => {
            error!("Failed to calculate percentiles: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Get performance anomalies
pub async fn get_performance_anomalies(
    Query(params): Query<AnomalyQueryParams>,
    Extension(service): Extension<Arc<PerformanceService>>,
) -> Result<Json<AnomalyResponse>, StatusCode> {
    info!("Getting performance anomalies");

    let lookback_hours = params.lookback_hours.unwrap_or(24);
    let threshold_factor = params.threshold_factor.unwrap_or(2.0);

    match service.analytics.detect_anomalies(lookback_hours, threshold_factor).await {
        Ok(anomalies) => {
            let response = AnomalyResponse {
                anomalies,
                detection_config: AnomalyDetectionConfig {
                    lookback_hours,
                    threshold_factor,
                    detection_algorithms: vec![
                        "z_score".to_string(),
                        "iqr".to_string(),
                        "baseline_comparison".to_string(),
                    ],
                },
                summary: AnomalySummary {
                    total_anomalies: anomalies.len() as i32,
                    critical_anomalies: anomalies.iter().filter(|a| 
                        matches!(a.anomaly_type, AnomalyType::LatencySpike) && a.deviation_score > 5.0
                    ).count() as i32,
                    affected_endpoints: anomalies.iter()
                        .map(|a| a.endpoint.clone())
                        .collect::<std::collections::HashSet<_>>()
                        .len() as i32,
                },
            };
            Ok(Json(response))
        }
        Err(e) => {
            error!("Failed to detect anomalies: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Get cache analytics
pub async fn get_cache_analytics(
    Query(params): Query<CacheAnalyticsQueryParams>,
    Extension(service): Extension<Arc<PerformanceService>>,
) -> Result<Json<CacheAnalyticsResponse>, StatusCode> {
    info!("Getting cache analytics");

    let end_time = params.end_time.unwrap_or_else(|| Utc::now());
    let start_time = params.start_time.unwrap_or_else(|| end_time - Duration::hours(24));

    match service.repo.get_cache_analytics(start_time, end_time).await {
        Ok(analytics) => {
            let response = CacheAnalyticsResponse {
                analytics,
                recommendations: vec![], // TODO: Get cache-specific recommendations
                optimization_opportunities: vec![
                    "Increase TTL for stable data".to_string(),
                    "Implement cache warming".to_string(),
                    "Optimize key design".to_string(),
                ],
            };
            Ok(Json(response))
        }
        Err(e) => {
            error!("Failed to get cache analytics: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Get system health metrics
pub async fn get_system_health(
    Extension(service): Extension<Arc<PerformanceService>>,
) -> Result<Json<SystemHealthResponse>, StatusCode> {
    info!("Getting system health metrics");

    match service.repo.get_system_health().await {
        Ok(health) => {
            let response = SystemHealthResponse {
                health,
                status: if health.health_score > 80.0 {
                    "healthy".to_string()
                } else if health.health_score > 60.0 {
                    "warning".to_string()
                } else {
                    "critical".to_string()
                },
                recommendations: if health.health_score < 80.0 {
                    vec![
                        "Monitor resource usage closely".to_string(),
                        "Consider scaling if trends continue".to_string(),
                    ]
                } else {
                    vec![]
                },
            };
            Ok(Json(response))
        }
        Err(e) => {
            error!("Failed to get system health: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Get active alerts
pub async fn get_active_alerts(
    Extension(service): Extension<Arc<PerformanceService>>,
) -> Result<Json<AlertDashboardResponse>, StatusCode> {
    info!("Getting active alerts");

    match service.alerts.get_alert_dashboard().await {
        Ok(dashboard) => {
            let response = AlertDashboardResponse {
                dashboard,
                alert_configs: vec![], // TODO: Get alert configurations
            };
            Ok(Json(response))
        }
        Err(e) => {
            error!("Failed to get active alerts: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Create alert configuration
pub async fn create_alert_config(
    Extension(service): Extension<Arc<PerformanceService>>,
    Json(request): Json<CreateAlertConfigRequest>,
) -> Result<Json<CreateAlertResponse>, StatusCode> {
    info!("Creating alert configuration: {}", request.name);

    match service.alerts.create_alert_config(request).await {
        Ok(config_id) => {
            let response = CreateAlertResponse {
                config_id,
                message: "Alert configuration created successfully".to_string(),
            };
            Ok(Json(response))
        }
        Err(e) => {
            error!("Failed to create alert config: {}", e);
            Err(StatusCode::BAD_REQUEST)
        }
    }
}

/// Acknowledge alert
pub async fn acknowledge_alert(
    Path(alert_id): Path<Uuid>,
    Extension(service): Extension<Arc<PerformanceService>>,
    Json(request): Json<AcknowledgeAlertRequest>,
) -> Result<Json<AcknowledgeAlertResponse>, StatusCode> {
    info!("Acknowledging alert: {}", alert_id);

    match service.alerts.acknowledge_alert(
        alert_id,
        request.acknowledged_by,
        request.notes,
    ).await {
        Ok(_) => {
            let response = AcknowledgeAlertResponse {
                success: true,
                message: "Alert acknowledged successfully".to_string(),
            };
            Ok(Json(response))
        }
        Err(e) => {
            error!("Failed to acknowledge alert: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Get performance recommendations
pub async fn get_recommendations(
    Query(params): Query<RecommendationQueryParams>,
    Extension(service): Extension<Arc<PerformanceService>>,
) -> Result<Json<RecommendationResponse>, StatusCode> {
    info!("Getting performance recommendations");

    match service.recommendations.get_recommendations(
        params.status,
        params.priority,
        params.limit,
    ).await {
        Ok(recommendations) => {
            let stats = service.recommendations.get_implementation_stats().await
                .unwrap_or_default();

            let response = RecommendationResponse {
                recommendations,
                stats,
                auto_implementable: recommendations.iter()
                    .filter(|r| r.auto_implementable)
                    .cloned()
                    .collect(),
            };
            Ok(Json(response))
        }
        Err(e) => {
            error!("Failed to get recommendations: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Generate new recommendations
pub async fn generate_recommendations(
    Query(params): Query<GenerateRecommendationsParams>,
    Extension(service): Extension<Arc<PerformanceService>>,
) -> Result<Json<GenerateRecommendationsResponse>, StatusCode> {
    info!("Generating new performance recommendations");

    let analysis_window = params.analysis_window_hours.unwrap_or(24);

    match service.recommendations.generate_recommendations(analysis_window).await {
        Ok(recommendations) => {
            let response = GenerateRecommendationsResponse {
                generated_count: recommendations.len() as i32,
                recommendations: recommendations.into_iter().take(10).collect(), // Return top 10
                analysis_window_hours: analysis_window,
                generation_timestamp: Utc::now(),
            };
            Ok(Json(response))
        }
        Err(e) => {
            error!("Failed to generate recommendations: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Get capacity planning metrics
pub async fn get_capacity_planning(
    Query(params): Query<CapacityPlanningParams>,
    Extension(service): Extension<Arc<PerformanceService>>,
) -> Result<Json<CapacityPlanningResponse>, StatusCode> {
    info!("Getting capacity planning metrics");

    let projection_days = params.projection_days.unwrap_or(30);

    match service.analytics.calculate_capacity_metrics(projection_days).await {
        Ok(metrics) => {
            let response = CapacityPlanningResponse {
                metrics,
                recommendations: if metrics.days_to_capacity.unwrap_or(999) < 60 {
                    vec![
                        "Consider scaling resources within 30 days".to_string(),
                        "Monitor growth trends closely".to_string(),
                        "Plan for additional capacity".to_string(),
                    ]
                } else {
                    vec!["Current capacity sufficient for projection period".to_string()]
                },
                projection_confidence: metrics.confidence_score,
            };
            Ok(Json(response))
        }
        Err(e) => {
            error!("Failed to get capacity planning: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Get performance trends
pub async fn get_performance_trends(
    Query(params): Query<TrendQueryParams>,
    Extension(service): Extension<Arc<PerformanceService>>,
) -> Result<Json<TrendResponse>, StatusCode> {
    info!("Getting performance trends");

    let end_time = params.end_time.unwrap_or_else(|| Utc::now());
    let start_time = params.start_time.unwrap_or_else(|| end_time - Duration::hours(24));

    match service.repo.get_performance_trends(
        params.endpoint.as_deref(),
        params.metric_type.unwrap_or(MetricType::Latency),
        start_time,
        end_time,
        params.time_bucket.unwrap_or(TimeBucket::Hour),
    ).await {
        Ok(trends) => {
            let response = TrendResponse {
                trends,
                trend_analysis: TrendAnalysis {
                    overall_direction: if trends.len() > 1 {
                        let first = trends.first().unwrap().metric_value;
                        let last = trends.last().unwrap().metric_value;
                        if last > first * 1.1 {
                            TrendDirection::Up
                        } else if last < first * 0.9 {
                            TrendDirection::Down
                        } else {
                            TrendDirection::Stable
                        }
                    } else {
                        TrendDirection::Stable
                    },
                    volatility_score: 0.0, // TODO: Calculate volatility
                    confidence_score: if trends.len() > 10 { 85.0 } else { 60.0 },
                },
                metadata: TrendMetadata {
                    metric_type: params.metric_type.unwrap_or(MetricType::Latency),
                    time_bucket: params.time_bucket.unwrap_or(TimeBucket::Hour),
                    data_points: trends.len() as i32,
                },
            };
            Ok(Json(response))
        }
        Err(e) => {
            error!("Failed to get performance trends: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// Request/Response DTOs

#[derive(Debug, Deserialize)]
pub struct DashboardQueryParams {
    pub hours: Option<i32>,
}

#[derive(Debug, Serialize)]
pub struct PerformanceDashboardResponse {
    pub summary: PerformanceSummary,
    pub slowest_endpoints: Vec<EndpointPerformance>,
    pub cache_analytics: CacheAnalytics,
    pub system_health: SystemHealthMetrics,
    pub sla_compliance: crate::web::performance::analytics::SLACompliance,
    pub active_alerts_count: i32,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct EndpointPerformanceResponse {
    pub endpoint_performance: EndpointPerformance,
    pub trends: Vec<PerformanceTrend>,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct PercentileQueryParams {
    pub endpoint: Option<String>,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize)]
pub struct PercentileResponse {
    pub percentiles: crate::web::performance::analytics::PercentileMetrics,
    pub timeframe: TimeframeInfo,
}

#[derive(Debug, Serialize)]
pub struct TimeframeInfo {
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub duration_hours: i64,
}

#[derive(Debug, Deserialize)]
pub struct AnomalyQueryParams {
    pub lookback_hours: Option<i32>,
    pub threshold_factor: Option<f64>,
}

#[derive(Debug, Serialize)]
pub struct AnomalyResponse {
    pub anomalies: Vec<PerformanceAnomalyDetection>,
    pub detection_config: AnomalyDetectionConfig,
    pub summary: AnomalySummary,
}

#[derive(Debug, Serialize)]
pub struct AnomalyDetectionConfig {
    pub lookback_hours: i32,
    pub threshold_factor: f64,
    pub detection_algorithms: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct AnomalySummary {
    pub total_anomalies: i32,
    pub critical_anomalies: i32,
    pub affected_endpoints: i32,
}

#[derive(Debug, Deserialize)]
pub struct CacheAnalyticsQueryParams {
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize)]
pub struct CacheAnalyticsResponse {
    pub analytics: CacheAnalytics,
    pub recommendations: Vec<String>,
    pub optimization_opportunities: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct SystemHealthResponse {
    pub health: SystemHealthMetrics,
    pub status: String,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct AlertDashboardResponse {
    pub dashboard: AlertDashboard,
    pub alert_configs: Vec<PerformanceAlertConfig>,
}

#[derive(Debug, Serialize)]
pub struct CreateAlertResponse {
    pub config_id: Uuid,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct AcknowledgeAlertResponse {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub struct RecommendationQueryParams {
    pub status: Option<RecommendationStatus>,
    pub priority: Option<Priority>,
    pub limit: Option<i32>,
}

#[derive(Debug, Serialize)]
pub struct RecommendationResponse {
    pub recommendations: Vec<PerformanceRecommendation>,
    pub stats: crate::web::performance::recommendations::RecommendationStats,
    pub auto_implementable: Vec<PerformanceRecommendation>,
}

impl Default for crate::web::performance::recommendations::RecommendationStats {
    fn default() -> Self {
        Self {
            total_recommendations: 0,
            implemented_recommendations: 0,
            pending_recommendations: 0,
            approved_recommendations: 0,
            avg_impact_score: 0.0,
            auto_implementable_count: 0,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct GenerateRecommendationsParams {
    pub analysis_window_hours: Option<i32>,
}

#[derive(Debug, Serialize)]
pub struct GenerateRecommendationsResponse {
    pub generated_count: i32,
    pub recommendations: Vec<PerformanceRecommendation>,
    pub analysis_window_hours: i32,
    pub generation_timestamp: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CapacityPlanningParams {
    pub projection_days: Option<i32>,
}

#[derive(Debug, Serialize)]
pub struct CapacityPlanningResponse {
    pub metrics: crate::web::performance::analytics::CapacityPlanningMetrics,
    pub recommendations: Vec<String>,
    pub projection_confidence: f64,
}

#[derive(Debug, Deserialize)]
pub struct TrendQueryParams {
    pub endpoint: Option<String>,
    pub metric_type: Option<MetricType>,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub time_bucket: Option<TimeBucket>,
}

#[derive(Debug, Serialize)]
pub struct TrendResponse {
    pub trends: Vec<PerformanceTrend>,
    pub trend_analysis: TrendAnalysis,
    pub metadata: TrendMetadata,
}

#[derive(Debug, Serialize)]
pub struct TrendAnalysis {
    pub overall_direction: TrendDirection,
    pub volatility_score: f64,
    pub confidence_score: f64,
}

#[derive(Debug, Serialize)]
pub struct TrendMetadata {
    pub metric_type: MetricType,
    pub time_bucket: TimeBucket,
    pub data_points: i32,
}