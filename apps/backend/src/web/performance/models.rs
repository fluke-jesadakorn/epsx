// Performance monitoring domain models

use serde::{Deserialize, Serialize};
// Diesel traits used for database operations
use chrono::{DateTime, Utc};
use uuid::Uuid;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetric {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub endpoint: String,
    pub method: String,
    pub duration_ms: i64,
    pub status_code: i32,
    pub cache_hit: Option<bool>,
    pub session_validation_ms: Option<i64>,
    pub db_query_ms: Option<i64>,
    pub middleware_stack_ms: Option<i64>,
    pub request_size_bytes: Option<i64>,
    pub response_size_bytes: Option<i64>,
    pub user_id: Option<Uuid>,
    pub client_ip: Option<String>,
    pub user_agent: Option<String>,
    pub error_message: Option<String>,
    pub trace_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachePerformanceMetric {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub cache_type: String,
    pub operation: String,
    pub key_pattern: Option<String>,
    pub hit: bool,
    pub duration_ms: i64,
    pub key_size_bytes: Option<i64>,
    pub value_size_bytes: Option<i64>,
    pub ttl_seconds: Option<i32>,
    pub evicted: Option<bool>,
    pub error_message: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceAlert {
    pub id: Uuid,
    pub alert_config_id: Uuid,
    pub triggered_at: DateTime<Utc>,
    pub resolved_at: Option<DateTime<Utc>>,
    pub metric_value: f64,
    pub threshold_value: f64,
    pub endpoint: Option<String>,
    pub time_window_start: DateTime<Utc>,
    pub time_window_end: DateTime<Utc>,
    pub alert_message: String,
    pub severity: AlertSeverity,
    pub acknowledged: bool,
    pub acknowledged_by: Option<Uuid>,
    pub acknowledged_at: Option<DateTime<Utc>>,
    pub notes: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceAlertConfig {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub metric_type: MetricType,
    pub threshold_value: f64,
    pub threshold_operator: ThresholdOperator,
    pub time_window_minutes: i32,
    pub endpoint_pattern: Option<String>,
    pub severity: AlertSeverity,
    pub enabled: bool,
    pub cooldown_minutes: i32,
    pub notification_channels: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceAggregate {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub time_bucket: TimeBucket,
    pub endpoint: Option<String>,
    pub method: Option<String>,
    pub total_requests: i64,
    pub error_requests: i64,
    pub avg_duration_ms: f64,
    pub p50_duration_ms: i64,
    pub p95_duration_ms: i64,
    pub p99_duration_ms: i64,
    pub min_duration_ms: i64,
    pub max_duration_ms: i64,
    pub cache_hits: i64,
    pub cache_misses: i64,
    pub avg_request_size_bytes: i64,
    pub avg_response_size_bytes: i64,
    pub throughput_rps: f64,
    pub error_rate_percent: f64,
    pub cache_hit_rate_percent: f64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemResourceMetric {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub cpu_usage_percent: Option<f64>,
    pub memory_usage_percent: Option<f64>,
    pub memory_usage_bytes: Option<i64>,
    pub disk_usage_percent: Option<f64>,
    pub disk_io_read_bytes: Option<i64>,
    pub disk_io_write_bytes: Option<i64>,
    pub network_rx_bytes: Option<i64>,
    pub network_tx_bytes: Option<i64>,
    pub active_connections: Option<i32>,
    pub db_connection_pool_active: Option<i32>,
    pub db_connection_pool_idle: Option<i32>,
    pub redis_connections: Option<i32>,
    pub goroutines_count: Option<i32>,
    pub gc_pause_ms: Option<f64>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceRecommendation {
    pub id: Uuid,
    pub created_at: DateTime<Utc>,
    pub recommendation_type: RecommendationType,
    pub priority: Priority,
    pub title: String,
    pub description: String,
    pub impact_score: i32,
    pub estimated_improvement: Option<String>,
    pub affected_endpoints: Vec<String>,
    pub implementation_effort: ImplementationEffort,
    pub auto_implementable: bool,
    pub metadata: Option<serde_json::Value>,
    pub status: RecommendationStatus,
    pub implemented_at: Option<DateTime<Utc>>,
    pub notes: Option<String>,
}

// Additional analytics types needed for stub
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SLACompliance {
    pub overall_compliance: f64,
    pub latency_compliance: f64,
    pub uptime_compliance: f64,
    pub error_rate_compliance: f64,
    pub sla_targets: SLATargets,
    pub current_metrics: CurrentSLAMetrics,
    pub breaches_today: i32,
    pub time_to_breach: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SLATargets {
    pub max_latency_ms: f64,
    pub min_uptime_percent: f64,
    pub max_error_rate_percent: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurrentSLAMetrics {
    pub current_latency_ms: f64,
    pub current_uptime_percent: f64,
    pub current_error_rate_percent: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PercentileMetrics {
    pub p50: f64,
    pub p95: f64,
    pub p99: f64,
    pub p999: f64,
    pub min: f64,
    pub max: f64,
    pub avg: f64,
    pub count: i64,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceInsights {
    pub summary: PerformanceSummary,
    pub slowest_endpoints: Vec<EndpointPerformance>,
    pub cache_analytics: CacheAnalytics,
    pub system_health: SystemHealthMetrics,
    pub anomalies: Vec<PerformanceAnomalyDetection>,
    pub bottlenecks: Vec<String>, // Simplified for stub
    pub recommendations: Vec<String>, // Simplified for stub
    pub sla_compliance: SLACompliance,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapacityPlanningMetrics {
    pub current_throughput: f64,
    pub projected_throughput: f64,
    pub growth_rate_percent: f64,
    pub current_utilization: f64,
    pub projected_utilization: f64,
    pub days_to_capacity: Option<i32>,
    pub recommended_scaling_date: Option<DateTime<Utc>>,
    pub confidence_score: f64,
}

// Enums for type safety
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum AlertSeverity {
    Low,
    Medium,
    High,
    Critical,
}

impl AlertSeverity {
    pub fn from_i32(value: i32) -> Result<Self, &'static str> {
        match value {
            0 => Ok(AlertSeverity::Low),
            1 => Ok(AlertSeverity::Medium),
            2 => Ok(AlertSeverity::High),
            3 => Ok(AlertSeverity::Critical),
            _ => Err("Invalid AlertSeverity value"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MetricType {
    Latency,
    ErrorRate,
    CacheHitRate,
    Throughput,
    ResourceUsage,
}

impl MetricType {
    pub fn from_i32(value: i32) -> Result<Self, &'static str> {
        match value {
            0 => Ok(MetricType::Latency),
            1 => Ok(MetricType::ErrorRate),
            2 => Ok(MetricType::CacheHitRate),
            3 => Ok(MetricType::Throughput),
            4 => Ok(MetricType::ResourceUsage),
            _ => Err("Invalid MetricType value"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ThresholdOperator {
    GreaterThan,
    LessThan,
    GreaterThanOrEqual,
    LessThanOrEqual,
    Equal,
}

impl ThresholdOperator {
    pub fn from_i32(value: i32) -> Result<Self, &'static str> {
        match value {
            0 => Ok(ThresholdOperator::GreaterThan),
            1 => Ok(ThresholdOperator::LessThan),
            2 => Ok(ThresholdOperator::GreaterThanOrEqual),
            3 => Ok(ThresholdOperator::LessThanOrEqual),
            4 => Ok(ThresholdOperator::Equal),
            _ => Err("Invalid ThresholdOperator value"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TimeBucket {
    Minute,
    Hour,
    Day,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecommendationType {
    CacheOptimization,
    QueryOptimization,
    Scaling,
    IndexCreation,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Priority {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ImplementationEffort {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum RecommendationStatus {
    Pending,
    Approved,
    Implemented,
    Rejected,
}

// DTO types for API responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceSummary {
    pub total_requests: i64,
    pub avg_response_time: f64,
    pub p95_response_time: f64,
    pub p99_response_time: f64,
    pub error_rate: f64,
    pub cache_hit_rate: f64,
    pub throughput_rps: f64,
    pub unique_endpoints: i64,
    pub active_alerts: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointPerformance {
    pub endpoint: String,
    pub method: String,
    pub request_count: i64,
    pub avg_duration: f64,
    pub p95_duration: f64,
    pub p99_duration: f64,
    pub max_duration: f64,
    pub error_count: i64,
    pub error_rate: f64,
    pub cache_hit_rate: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PerformanceTrend {
    pub timestamp: DateTime<Utc>,
    pub metric_value: f64,
    pub baseline_value: Option<f64>,
    pub trend_direction: TrendDirection,
    pub percentage_change: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TrendDirection {
    Up,
    Down,
    Stable,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AlertDashboard {
    pub active_alerts: Vec<PerformanceAlert>,
    pub alerts_by_severity: HashMap<AlertSeverity, i64>,
    pub recent_resolutions: Vec<PerformanceAlert>,
    pub alert_trends: Vec<PerformanceTrend>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheAnalytics {
    pub overall_hit_rate: f64,
    pub hit_rate_by_type: HashMap<String, f64>,
    pub avg_response_time: f64,
    pub cache_size_mb: f64,
    pub eviction_rate: f64,
    pub top_missed_keys: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemHealthMetrics {
    pub cpu_usage: f64,
    pub memory_usage: f64,
    pub disk_usage: f64,
    pub connection_pool_utilization: f64,
    pub goroutines_count: i32,
    pub gc_pause_ms: f64,
    pub health_score: f64, // 0-100
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceAnomalyDetection {
    pub endpoint: String,
    pub metric_type: MetricType,
    pub current_value: f64,
    pub expected_value: f64,
    pub deviation_score: f64,
    pub anomaly_type: AnomalyType,
    pub detected_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnomalyType {
    LatencySpike,
    ThroughputDrop,
    ErrorRateIncrease,
    CacheHitRateDecrease,
    ResourceExhaustion,
}

// Request/Response DTOs
#[derive(Debug, Deserialize)]
pub struct PerformanceQueryParams {
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub endpoint: Option<String>,
    pub method: Option<String>,
    pub time_bucket: Option<TimeBucket>,
    pub limit: Option<i32>,
    pub offset: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateAlertConfigRequest {
    pub name: String,
    pub description: Option<String>,
    pub metric_type: MetricType,
    pub threshold_value: f64,
    pub threshold_operator: ThresholdOperator,
    pub time_window_minutes: i32,
    pub endpoint_pattern: Option<String>,
    pub severity: AlertSeverity,
    pub cooldown_minutes: i32,
    pub notification_channels: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AcknowledgeAlertRequest {
    pub alert_id: Uuid,
    pub acknowledged_by: Uuid,
    pub notes: Option<String>,
}

// Implementation helpers
impl PerformanceMetric {
    pub fn new(
        endpoint: String,
        method: String,
        duration_ms: i64,
        status_code: i32,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            endpoint,
            method,
            duration_ms,
            status_code,
            cache_hit: None,
            session_validation_ms: None,
            db_query_ms: None,
            middleware_stack_ms: None,
            request_size_bytes: None,
            response_size_bytes: None,
            user_id: None,
            client_ip: None,
            user_agent: None,
            error_message: None,
            trace_id: None,
            created_at: Utc::now(),
        }
    }

    pub fn with_cache_hit(mut self, hit: bool) -> Self {
        self.cache_hit = Some(hit);
        self
    }

    pub fn with_timings(
        mut self,
        session_ms: Option<i64>,
        db_ms: Option<i64>,
        middleware_ms: Option<i64>,
    ) -> Self {
        self.session_validation_ms = session_ms;
        self.db_query_ms = db_ms;
        self.middleware_stack_ms = middleware_ms;
        self
    }

    pub fn with_user_context(
        mut self,
        user_id: Option<Uuid>,
        client_ip: Option<String>,
        user_agent: Option<String>,
    ) -> Self {
        self.user_id = user_id;
        self.client_ip = client_ip;
        self.user_agent = user_agent;
        self
    }

    pub fn with_error(mut self, error: String) -> Self {
        self.error_message = Some(error);
        self
    }

    pub fn with_trace_id(mut self, trace_id: Uuid) -> Self {
        self.trace_id = Some(trace_id);
        self
    }

    pub fn with_sizes(mut self, request_bytes: i64, response_bytes: i64) -> Self {
        self.request_size_bytes = Some(request_bytes);
        self.response_size_bytes = Some(response_bytes);
        self
    }
}

impl CachePerformanceMetric {
    pub fn new(
        cache_type: String,
        operation: String,
        hit: bool,
        duration_ms: i64,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            cache_type,
            operation,
            key_pattern: None,
            hit,
            duration_ms,
            key_size_bytes: None,
            value_size_bytes: None,
            ttl_seconds: None,
            evicted: None,
            error_message: None,
            created_at: Utc::now(),
        }
    }
}

impl ThresholdOperator {
    pub fn evaluate(&self, value: f64, threshold: f64) -> bool {
        match self {
            ThresholdOperator::GreaterThan => value > threshold,
            ThresholdOperator::LessThan => value < threshold,
            ThresholdOperator::GreaterThanOrEqual => value >= threshold,
            ThresholdOperator::LessThanOrEqual => value <= threshold,
            ThresholdOperator::Equal => (value - threshold).abs() < f64::EPSILON,
        }
    }
}

impl AlertSeverity {
    pub fn to_score(&self) -> i32 {
        match self {
            AlertSeverity::Low => 1,
            AlertSeverity::Medium => 2,
            AlertSeverity::High => 3,
            AlertSeverity::Critical => 4,
        }
    }
}

impl Priority {
    pub fn to_score(&self) -> i32 {
        match self {
            Priority::Low => 1,
            Priority::Medium => 2,
            Priority::High => 3,
            Priority::Critical => 4,
        }
    }
}