use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};

use crate::web::auth::AppState;

/// Security monitoring handlers for admin dashboard
pub struct SecurityMonitoringHandlers;

#[derive(Debug, Deserialize, Serialize)]
pub struct SecurityEventsQuery {
    pub limit: Option<usize>,
    pub severity: Option<String>,
    pub event_type: Option<String>,
    pub resolved: Option<bool>,
    pub wallet_address: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SecurityEventsResponse {
    pub events: Vec<SecurityEventDto>,
    pub total_count: usize,
    pub filters_applied: SecurityEventsQuery,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct SecurityEventDto {
    pub id: String,
    pub wallet_address: String,
    pub event_type: String,
    pub severity: String,
    pub description: String,
    pub risk_score: f64,
    pub device_fingerprint: Option<String>,
    pub ip_address: String,
    pub user_agent: String,
    pub timestamp: DateTime<Utc>,
    pub resolved: bool,
    pub recommended_actions: Vec<String>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Serialize)]
pub struct SecurityMetricsResponse {
    pub metrics: SecurityMetricsDto,
    pub trends: SecurityTrends,
    pub alerts: Vec<SecurityAlert>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct SecurityMetricsDto {
    pub total_events: u64,
    pub active_threats: u64,
    pub resolved_threats: u64,
    pub avg_threat_score: f64,
    pub events_by_severity: HashMap<String, u64>,
    pub events_by_type: HashMap<String, u64>,
    pub threat_score_distribution: Vec<ThreatScoreRange>,
}

#[derive(Debug, Serialize)]
pub struct ThreatScoreRange {
    pub range: String,
    pub count: u64,
    pub percentage: f64,
}

#[derive(Debug, Serialize)]
pub struct SecurityTrends {
    pub hourly_events: Vec<HourlyEventCount>,
    pub severity_trends: Vec<SeverityTrend>,
    pub threat_score_trend: Vec<ThreatScoreTrend>,
}

#[derive(Debug, Serialize)]
pub struct HourlyEventCount {
    pub hour: DateTime<Utc>,
    pub count: u64,
    pub severity_breakdown: HashMap<String, u64>,
}

#[derive(Debug, Serialize)]
pub struct SeverityTrend {
    pub severity: String,
    pub trend: String, // "increasing", "decreasing", "stable"
    pub change_percentage: f64,
}

#[derive(Debug, Serialize)]
pub struct ThreatScoreTrend {
    pub timestamp: DateTime<Utc>,
    pub avg_score: f64,
    pub max_score: f64,
    pub min_score: f64,
}

#[derive(Debug, Serialize)]
pub struct SecurityAlert {
    pub id: String,
    pub alert_type: String,
    pub message: String,
    pub severity: String,
    pub timestamp: DateTime<Utc>,
    pub auto_resolved: bool,
    pub affected_users: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct UserThreatQuery {
    pub wallet_address: String,
}

#[derive(Debug, Serialize)]
pub struct UserThreatResponse {
    pub wallet_address: String,
    pub current_threat_score: f64,
    pub threat_level: String,
    pub is_under_threat: bool,
    pub recent_events: Vec<SecurityEventDto>,
    pub risk_factors: Vec<String>,
    pub recommendations: Vec<String>,
    pub last_assessed: DateTime<Utc>,
}

impl SecurityMonitoringHandlers {
    /// GET /admin/security/events - Get security events with filtering
    pub async fn get_security_events(
        State(_app_state): State<AppState>,
        Query(query): Query<SecurityEventsQuery>,
    ) -> Result<Json<SecurityEventsResponse>, StatusCode> {
        // In production, integrate with actual ThreatDetectionService
        let events = Self::mock_security_events(&query);
        
        let response = SecurityEventsResponse {
            total_count: events.len(),
            events,
            filters_applied: query,
            timestamp: Utc::now(),
        };

        Ok(Json(response))
    }

    /// GET /admin/security/metrics - Get security metrics and analytics
    pub async fn get_security_metrics(
        State(_app_state): State<AppState>,
    ) -> Result<Json<SecurityMetricsResponse>, StatusCode> {
        let metrics = Self::mock_security_metrics();
        let trends = Self::mock_security_trends();
        let alerts = Self::mock_security_alerts();

        let response = SecurityMetricsResponse {
            metrics,
            trends,
            alerts,
            timestamp: Utc::now(),
        };

        Ok(Json(response))
    }

    /// GET /admin/security/user-threat - Get specific user threat assessment
    pub async fn get_user_threat_assessment(
        State(_app_state): State<AppState>,
        Query(query): Query<UserThreatQuery>,
    ) -> Result<Json<UserThreatResponse>, StatusCode> {
        let response = UserThreatResponse {
            wallet_address: query.wallet_address.clone(),
            current_threat_score: 23.5,
            threat_level: "Low".to_string(),
            is_under_threat: false,
            recent_events: Self::mock_user_events(&query.wallet_address),
            risk_factors: vec![
                "Multiple device logins".to_string(),
                "Unusual time patterns".to_string(),
            ],
            recommendations: vec![
                "Enable MFA".to_string(),
                "Review recent activity".to_string(),
            ],
            last_assessed: Utc::now(),
        };

        Ok(Json(response))
    }

    // Mock data generators for testing - replace with real service integration
    fn mock_security_events(_query: &SecurityEventsQuery) -> Vec<SecurityEventDto> {
        vec![
            SecurityEventDto {
                id: "evt_001".to_string(),
                wallet_address: "user_123".to_string(),
                event_type: "SuspiciousLogin".to_string(),
                severity: "Medium".to_string(),
                description: "Login from unusual location".to_string(),
                risk_score: 65.0,
                device_fingerprint: Some("fp_abc123".to_string()),
                ip_address: "192.168.1.100".to_string(),
                user_agent: "Mozilla/5.0...".to_string(),
                timestamp: Utc::now(),
                resolved: false,
                recommended_actions: vec![
                    "Verify user identity".to_string(),
                    "Enable additional authentication".to_string(),
                ],
                metadata: HashMap::new(),
            },
            SecurityEventDto {
                id: "evt_002".to_string(),
                wallet_address: "user_456".to_string(),
                event_type: "TokenReuse".to_string(),
                severity: "High".to_string(),
                description: "Refresh token reuse detected".to_string(),
                risk_score: 85.0,
                device_fingerprint: Some("fp_def456".to_string()),
                ip_address: "10.0.0.50".to_string(),
                user_agent: "Mozilla/5.0...".to_string(),
                timestamp: Utc::now(),
                resolved: true,
                recommended_actions: vec![
                    "Revoke all user tokens".to_string(),
                    "Force re-authentication".to_string(),
                ],
                metadata: HashMap::new(),
            },
        ]
    }

    fn mock_security_metrics() -> SecurityMetricsDto {
        let mut events_by_severity = HashMap::new();
        events_by_severity.insert("Low".to_string(), 45);
        events_by_severity.insert("Medium".to_string(), 23);
        events_by_severity.insert("High".to_string(), 8);
        events_by_severity.insert("Critical".to_string(), 2);

        let mut events_by_type = HashMap::new();
        events_by_type.insert("SuspiciousLogin".to_string(), 32);
        events_by_type.insert("TokenReuse".to_string(), 15);
        events_by_type.insert("DeviceMismatch".to_string(), 18);
        events_by_type.insert("PermissionEscalation".to_string(), 3);

        SecurityMetricsDto {
            total_events: 78,
            active_threats: 12,
            resolved_threats: 66,
            avg_threat_score: 34.5,
            events_by_severity,
            events_by_type,
            threat_score_distribution: vec![
                ThreatScoreRange {
                    range: "0-25".to_string(),
                    count: 45,
                    percentage: 57.7,
                },
                ThreatScoreRange {
                    range: "26-50".to_string(),
                    count: 20,
                    percentage: 25.6,
                },
                ThreatScoreRange {
                    range: "51-75".to_string(),
                    count: 10,
                    percentage: 12.8,
                },
                ThreatScoreRange {
                    range: "76-100".to_string(),
                    count: 3,
                    percentage: 3.9,
                },
            ],
        }
    }

    fn mock_security_trends() -> SecurityTrends {
        SecurityTrends {
            hourly_events: vec![
                HourlyEventCount {
                    hour: Utc::now(),
                    count: 12,
                    severity_breakdown: {
                        let mut breakdown = HashMap::new();
                        breakdown.insert("Low".to_string(), 8);
                        breakdown.insert("Medium".to_string(), 3);
                        breakdown.insert("High".to_string(), 1);
                        breakdown
                    },
                },
            ],
            severity_trends: vec![
                SeverityTrend {
                    severity: "High".to_string(),
                    trend: "decreasing".to_string(),
                    change_percentage: -15.2,
                },
                SeverityTrend {
                    severity: "Medium".to_string(),
                    trend: "stable".to_string(),
                    change_percentage: 2.1,
                },
            ],
            threat_score_trend: vec![
                ThreatScoreTrend {
                    timestamp: Utc::now(),
                    avg_score: 34.5,
                    max_score: 89.0,
                    min_score: 5.0,
                },
            ],
        }
    }

    fn mock_security_alerts() -> Vec<SecurityAlert> {
        vec![
            SecurityAlert {
                id: "alert_001".to_string(),
                alert_type: "HighThreatScore".to_string(),
                message: "Multiple users with threat scores above 80".to_string(),
                severity: "High".to_string(),
                timestamp: Utc::now(),
                auto_resolved: false,
                affected_users: vec!["user_123".to_string(), "user_456".to_string()],
            },
        ]
    }

    fn mock_user_events(wallet_address: &str) -> Vec<SecurityEventDto> {
        vec![
            SecurityEventDto {
                id: "evt_user_001".to_string(),
                wallet_address: wallet_address.to_string(),
                event_type: "SuspiciousLogin".to_string(),
                severity: "Medium".to_string(),
                description: "Login from new device".to_string(),
                risk_score: 45.0,
                device_fingerprint: Some("fp_new_device".to_string()),
                ip_address: "203.0.113.1".to_string(),
                user_agent: "Mozilla/5.0...".to_string(),
                timestamp: Utc::now(),
                resolved: false,
                recommended_actions: vec!["Verify device ownership".to_string()],
                metadata: HashMap::new(),
            },
        ]
    }
}