// Security Alerts API Models
// Request/response models for alert management API

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use std::collections::HashMap;

// Removed imports from stubbed security modules
// TODO: Re-implement with proper Diesel models

/// Alert response for API endpoints
#[derive(Debug, Serialize)]
pub struct AlertResponse {
    pub id: Uuid,
    pub title: String,
    pub description: String,
    pub severity: AlertSeverity,
    pub status: AlertStatus,
    pub created_at: DateTime<Utc>,
}

/// Security alert (minimal version)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityAlert {
    pub id: Uuid,
    pub title: String,
    pub description: String,
    pub severity: AlertSeverity,
    pub created_at: DateTime<Utc>,
}

/// Alert rule (minimal version)
#[derive(Debug, Serialize)]
pub struct SecurityAlertRule {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub is_active: bool,
}

/// Alert enums
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum AlertSeverity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertStatus {
    Open,
    Acknowledged,
    Resolved,
    Closed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertCategory {
    Authentication,
    Authorization,
    BruteForce,
    Malware,
    DataLoss,
    SystemHealth,
    Other,
}

/// List rules query parameters
#[derive(Debug, Deserialize)]
pub struct ListRulesQuery {
    pub category: Option<AlertCategory>,
    pub severity: Option<AlertSeverity>,
    pub is_active: Option<bool>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// List rules response
#[derive(Debug, Serialize)]
pub struct ListRulesResponse {
    pub rules: Vec<SecurityAlertRule>,
    pub total: i64,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// List alerts response
#[derive(Debug, Serialize)]
pub struct ListAlertsResponse {
    pub alerts: Vec<SecurityAlert>,
    pub total: i64,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// Alert statistics query parameters
#[derive(Debug, Deserialize)]
pub struct AlertStatsQuery {
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
}

/// API Error response
#[derive(Debug, Serialize)]
pub struct ApiError {
    pub error: String,
    pub message: String,
    pub code: String,
}

/// Dashboard data for real-time alerts
#[derive(Debug, Serialize)]
pub struct AlertDashboardData {
    pub active_alerts: Vec<SecurityAlert>,
    pub critical_alerts: Vec<SecurityAlert>,
    pub recent_events: Vec<RecentSecurityEvent>,
    pub alert_trends: Vec<AlertTrendData>,
    pub top_threats: Vec<ThreatSummary>,
    pub system_health: AlertSystemHealth,
    pub generated_at: DateTime<Utc>,
}

/// Recent security event for dashboard
#[derive(Debug, Serialize)]
pub struct RecentSecurityEvent {
    pub id: Uuid,
    pub event_type: String,
    pub severity: String,
    pub timestamp: DateTime<Utc>,
    pub source_ip: String,
    pub user_id: Option<String>,
    pub description: String,
    pub alert_generated: bool,
}

/// Alert trend data for charts
#[derive(Debug, Serialize)]
pub struct AlertTrendData {
    pub timestamp: DateTime<Utc>,
    pub alert_count: i64,
    pub critical_count: i64,
    pub high_count: i64,
    pub medium_count: i64,
    pub low_count: i64,
}

/// Threat summary for dashboard
#[derive(Debug, Serialize)]
pub struct ThreatSummary {
    pub threat_type: String,
    pub count: i64,
    pub severity: String,
    pub affected_ips: Vec<String>,
    pub last_seen: DateTime<Utc>,
    pub trend: String, // "INCREASING", "STABLE", "DECREASING"
}

/// Alert system health status
#[derive(Debug, Serialize)]
pub struct AlertSystemHealth {
    pub alert_engine_status: String,
    pub webhook_system_status: String,
    pub database_connection: String,
    pub cache_status: String,
    pub active_rules: i64,
    pub processed_alerts_last_hour: i64,
    pub failed_notifications: i64,
    pub average_processing_time_ms: f64,
}

/// Real-time alert update for WebSocket
#[derive(Debug, Serialize)]
pub struct AlertUpdate {
    pub update_type: AlertUpdateType,
    pub alert: Option<SecurityAlert>,
    pub correlation_info: Option<CorrelationInfo>,
    pub affected_systems: Vec<String>,
    pub recommended_actions: Vec<String>,
    pub timestamp: DateTime<Utc>,
}

/// Types of alert updates
#[derive(Debug, Serialize)]
pub enum AlertUpdateType {
    NewAlert,
    AlertAcknowledged,
    AlertResolved,
    AlertEscalated,
    AlertCorrelated,
    SystemStatus,
}

/// Correlation information for related alerts
#[derive(Debug, Serialize)]
pub struct CorrelationInfo {
    pub correlation_id: String,
    pub related_alert_ids: Vec<Uuid>,
    pub correlation_type: String,
    pub confidence_score: f64,
    pub pattern_description: String,
}

/// Alert rule template for easy creation
#[derive(Debug, Serialize)]
pub struct AlertRuleTemplate {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: AlertCategory,
    pub severity: AlertSeverity,
    pub template_conditions: serde_json::Value,
    pub example_config: serde_json::Value,
    pub use_cases: Vec<String>,
    pub tags: Vec<String>,
}

/// Notification channel configuration
#[derive(Debug, Serialize, Deserialize)]
pub struct NotificationChannelConfig {
    pub id: Uuid,
    pub name: String,
    pub channel_type: String,
    pub config: serde_json::Value,
    pub is_active: bool,
    pub alert_filters: AlertFilters,
    pub rate_limits: RateLimits,
}

/// Alert filters for notification channels
#[derive(Debug, Serialize, Deserialize)]
pub struct AlertFilters {
    pub severity_filter: Option<Vec<AlertSeverity>>,
    pub category_filter: Option<Vec<AlertCategory>>,
    pub time_restrictions: Option<TimeRestrictions>,
    pub ip_whitelist: Option<Vec<String>>,
    pub ip_blacklist: Option<Vec<String>>,
}

/// Time restrictions for notifications
#[derive(Debug, Serialize, Deserialize)]
pub struct TimeRestrictions {
    pub business_hours_only: bool,
    pub timezone: String,
    pub excluded_hours: Vec<i32>, // Hours to exclude (0-23)
    pub excluded_days: Vec<i32>,  // Days to exclude (0-6, Sunday=0)
}

/// Rate limiting configuration
#[derive(Debug, Serialize, Deserialize)]
pub struct RateLimits {
    pub max_per_minute: Option<i32>,
    pub max_per_hour: Option<i32>,
    pub max_per_day: Option<i32>,
    pub burst_allowance: Option<i32>,
}

/// Alert suppression request
#[derive(Debug, Deserialize)]
pub struct CreateSuppressionRequest {
    pub name: String,
    pub alert_rule_id: Option<Uuid>,
    pub conditions: serde_json::Value,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub is_recurring: bool,
    pub recurrence_pattern: Option<serde_json::Value>,
    pub reason: String,
}

/// Alert escalation rule request
#[derive(Debug, Deserialize)]
pub struct CreateEscalationRuleRequest {
    pub name: String,
    pub severity: AlertSeverity,
    pub category: Option<AlertCategory>,
    pub unacknowledged_minutes: i32,
    pub escalate_to_severity: Option<AlertSeverity>,
    pub notify_channels: Vec<Uuid>,
    pub assign_to: Option<String>,
}

/// Advanced alert search request
#[derive(Debug, Deserialize)]
pub struct AdvancedAlertSearch {
    pub query: Option<String>,        // Free text search
    pub filters: SearchFilters,
    pub sort_by: Option<String>,      // Field to sort by
    pub sort_direction: Option<String>, // "ASC" or "DESC"
    pub facets: Option<Vec<String>>,  // Fields to get facet counts for
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// Search filters for advanced search
#[derive(Debug, Deserialize)]
pub struct SearchFilters {
    pub severity: Option<Vec<AlertSeverity>>,
    pub category: Option<Vec<AlertCategory>>,
    pub status: Option<Vec<AlertStatus>>,
    pub date_range: Option<DateRange>,
    pub ip_addresses: Option<Vec<String>>,
    pub user_ids: Option<Vec<String>>,
    pub rule_ids: Option<Vec<Uuid>>,
    pub correlation_ids: Option<Vec<String>>,
    pub has_comments: Option<bool>,
}

/// Date range filter
#[derive(Debug, Deserialize)]
pub struct DateRange {
    pub from: DateTime<Utc>,
    pub to: DateTime<Utc>,
}

/// Advanced search response
#[derive(Debug, Serialize)]
pub struct AdvancedSearchResponse {
    pub alerts: Vec<SecurityAlert>,
    pub total: i64,
    pub facets: HashMap<String, Vec<FacetCount>>,
    pub search_time_ms: u64,
    pub suggestions: Vec<String>,
}

/// Facet count for search results
#[derive(Debug, Serialize)]
pub struct FacetCount {
    pub value: String,
    pub count: i64,
}

/// Alert export request
#[derive(Debug, Deserialize)]
pub struct AlertExportRequest {
    pub format: ExportFormat,
    pub filters: SearchFilters,
    pub fields: Option<Vec<String>>,
    pub include_comments: Option<bool>,
    pub include_related_events: Option<bool>,
}

/// Export formats
#[derive(Debug, Deserialize)]
pub enum ExportFormat {
    Json,
    Csv,
    Excel,
    Pdf,
}

/// Alert metrics for monitoring
#[derive(Debug, Serialize)]
pub struct AlertMetrics {
    pub total_alerts_24h: i64,
    pub alerts_by_hour: Vec<HourlyCount>,
    pub mean_time_to_acknowledge: f64,
    pub mean_time_to_resolve: f64,
    pub escalation_rate: f64,
    pub false_positive_rate: f64,
    pub top_alert_sources: Vec<AlertSource>,
    pub rule_performance: Vec<RulePerformance>,
}

/// Hourly alert count
#[derive(Debug, Serialize)]
pub struct HourlyCount {
    pub hour: DateTime<Utc>,
    pub count: i64,
}

/// Alert source statistics
#[derive(Debug, Serialize)]
pub struct AlertSource {
    pub source: String,
    pub count: i64,
    pub percentage: f64,
}

/// Rule performance metrics
#[derive(Debug, Serialize)]
pub struct RulePerformance {
    pub rule_id: Uuid,
    pub rule_name: String,
    pub alerts_generated: i64,
    pub false_positives: i64,
    pub accuracy: f64,
    pub avg_processing_time_ms: f64,
}

/// WebSocket message types for real-time updates
#[derive(Debug, Serialize, Deserialize)]
pub struct WebSocketMessage {
    pub message_type: WebSocketMessageType,
    pub data: serde_json::Value,
    pub timestamp: DateTime<Utc>,
}

/// WebSocket message types
#[derive(Debug, Serialize, Deserialize)]
pub enum WebSocketMessageType {
    AlertUpdate,
    SystemStatus,
    RuleUpdate,
    WebhookStatus,
    ClientHeartbeat,
    ServerHeartbeat,
    Error,
}

/// System status for WebSocket updates
#[derive(Debug, Serialize)]
pub struct SystemStatusUpdate {
    pub component: String,
    pub status: String,
    pub details: Option<serde_json::Value>,
    pub last_updated: DateTime<Utc>,
}

/// Client subscription preferences for WebSocket
#[derive(Debug, Serialize, Deserialize)]
pub struct SubscriptionPreferences {
    pub alert_updates: bool,
    pub system_status: bool,
    pub rule_updates: bool,
    pub webhook_status: bool,
    pub severity_filter: Option<Vec<AlertSeverity>>,
    pub category_filter: Option<Vec<AlertCategory>>,
}

/// Alert investigation context
#[derive(Debug, Serialize)]
pub struct InvestigationContext {
    pub alert: SecurityAlert,
    pub related_alerts: Vec<SecurityAlert>,
    pub security_events: Vec<RecentSecurityEvent>,
    pub affected_assets: Vec<AffectedAsset>,
    pub threat_intelligence: Option<ThreatIntelligenceInfo>,
    pub recommended_actions: Vec<RecommendedAction>,
    pub similar_incidents: Vec<SimilarIncident>,
}

/// Affected asset information
#[derive(Debug, Serialize)]
pub struct AffectedAsset {
    pub asset_type: String, // "IP", "USER", "SESSION", "ENDPOINT"
    pub asset_id: String,
    pub asset_name: Option<String>,
    pub risk_score: f64,
    pub last_seen: DateTime<Utc>,
    pub associated_events: i64,
}

/// Threat intelligence information
#[derive(Debug, Serialize)]
pub struct ThreatIntelligenceInfo {
    pub threat_type: String,
    pub confidence: f64,
    pub sources: Vec<String>,
    pub indicators: Vec<ThreatIndicator>,
    pub attribution: Option<String>,
    pub first_seen: DateTime<Utc>,
    pub last_seen: DateTime<Utc>,
}

/// Threat indicator
#[derive(Debug, Serialize)]
pub struct ThreatIndicator {
    pub indicator_type: String, // "IP", "DOMAIN", "HASH", "URL"
    pub value: String,
    pub confidence: f64,
    pub tags: Vec<String>,
}

/// Recommended action
#[derive(Debug, Serialize)]
pub struct RecommendedAction {
    pub action: String,
    pub description: String,
    pub priority: String,
    pub estimated_impact: String,
    pub automation_available: bool,
}

/// Similar incident
#[derive(Debug, Serialize)]
pub struct SimilarIncident {
    pub incident_id: Uuid,
    pub alert_id: Uuid,
    pub similarity_score: f64,
    pub occurred_at: DateTime<Utc>,
    pub resolution: Option<String>,
    pub lessons_learned: Option<String>,
}

/// Alert playbook response
#[derive(Debug, Serialize)]
pub struct AlertPlaybook {
    pub playbook_id: String,
    pub name: String,
    pub description: String,
    pub applicable_alert_types: Vec<String>,
    pub steps: Vec<PlaybookStep>,
    pub estimated_time_minutes: i32,
    pub required_permissions: Vec<String>,
}

/// Playbook step
#[derive(Debug, Serialize)]
pub struct PlaybookStep {
    pub step_number: i32,
    pub title: String,
    pub description: String,
    pub action_type: String, // "INVESTIGATION", "CONTAINMENT", "ERADICATION", "RECOVERY"
    pub automated: bool,
    pub tools_required: Vec<String>,
    pub expected_outcome: String,
}