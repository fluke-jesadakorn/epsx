use diesel::prelude::*;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use serde_json::Value as JsonValue;

use crate::infra::db::diesel::schema::{
    security_events, 
    security_alert_rules, 
    attack_attempts, 
    ip_blacklist,
    alert_notifications
};

// Security Events models
#[derive(Queryable, Selectable, Insertable, AsChangeset, Debug, Clone, Serialize, Deserialize)]
#[diesel(table_name = security_events)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct DieselSecurityEvent {
    pub id: Uuid,
    pub event_type: String,
    pub severity: String,
    pub source: String,
    pub user_id: Option<String>,
    pub ip_address: Option<ipnetwork::IpNetwork>,
    pub user_agent: Option<String>,
    pub request_path: Option<String>,
    pub request_method: Option<String>,
    pub request_headers: Option<JsonValue>,
    pub response_status: Option<i32>,
    pub event_data: Option<JsonValue>,
    pub risk_score: Option<i32>,
    pub country_code: Option<String>,
    pub device_fingerprint: Option<String>,
    pub correlation_id: Option<Uuid>,
    pub alert_triggered: Option<bool>,
    pub blocked: Option<bool>,
    pub timestamp: DateTime<Utc>,
    pub processed: Option<bool>,
}

#[derive(Insertable, Debug, Clone)]
#[diesel(table_name = security_events)]
pub struct NewDieselSecurityEvent {
    pub id: Uuid,
    pub event_type: String,
    pub severity: String,
    pub source: String,
    pub user_id: Option<String>,
    pub ip_address: Option<ipnetwork::IpNetwork>,
    pub user_agent: Option<String>,
    pub request_path: Option<String>,
    pub request_method: Option<String>,
    pub request_headers: Option<JsonValue>,
    pub response_status: Option<i32>,
    pub event_data: Option<JsonValue>,
    pub risk_score: Option<i32>,
    pub country_code: Option<String>,
    pub device_fingerprint: Option<String>,
    pub correlation_id: Option<Uuid>,
    pub alert_triggered: Option<bool>,
    pub blocked: Option<bool>,
    pub timestamp: DateTime<Utc>,
    pub processed: Option<bool>,
}

// Security Alert Rules models
#[derive(Queryable, Selectable, Insertable, AsChangeset, Debug, Clone, Serialize, Deserialize)]
#[diesel(table_name = security_alert_rules)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct DieselSecurityAlertRule {
    pub id: Uuid,
    pub rule_name: String,
    pub description: Option<String>,
    pub event_pattern: JsonValue,
    pub severity: String,
    pub threshold_count: Option<i32>,
    pub time_window_seconds: Option<i32>,
    pub enabled: Option<bool>,
    pub notification_channels: Option<Vec<String>>,
    pub auto_block: Option<bool>,
    pub block_duration_seconds: Option<i32>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub last_triggered: Option<DateTime<Utc>>,
}

#[derive(Insertable, Debug, Clone)]
#[diesel(table_name = security_alert_rules)]
pub struct NewDieselSecurityAlertRule {
    pub id: Uuid,
    pub rule_name: String,
    pub description: Option<String>,
    pub event_pattern: JsonValue,
    pub severity: String,
    pub threshold_count: Option<i32>,
    pub time_window_seconds: Option<i32>,
    pub enabled: Option<bool>,
    pub notification_channels: Option<Vec<String>>,
    pub auto_block: Option<bool>,
    pub block_duration_seconds: Option<i32>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

// Attack Attempts models
#[derive(Queryable, Selectable, Insertable, AsChangeset, Debug, Clone, Serialize, Deserialize)]
#[diesel(table_name = attack_attempts)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct DieselAttackAttempt {
    pub id: Uuid,
    pub ip_address: ipnetwork::IpNetwork,
    pub attack_type: String,
    pub target_user: Option<String>,
    pub request_path: Option<String>,
    pub user_agent: Option<String>,
    pub severity: Option<String>,
    pub success: Option<bool>,
    pub blocked: Option<bool>,
    pub metadata: Option<JsonValue>,
    pub detection_method: Option<String>,
    pub risk_score: Option<i32>,
    pub geolocation: Option<JsonValue>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Insertable, Debug, Clone)]
#[diesel(table_name = attack_attempts)]
pub struct NewDieselAttackAttempt {
    pub id: Uuid,
    pub ip_address: ipnetwork::IpNetwork,
    pub attack_type: String,
    pub target_user: Option<String>,
    pub request_path: Option<String>,
    pub user_agent: Option<String>,
    pub severity: Option<String>,
    pub success: Option<bool>,
    pub blocked: Option<bool>,
    pub metadata: Option<JsonValue>,
    pub detection_method: Option<String>,
    pub risk_score: Option<i32>,
    pub geolocation: Option<JsonValue>,
    pub timestamp: DateTime<Utc>,
}

// IP Blacklist models
#[derive(Queryable, Selectable, Insertable, AsChangeset, Debug, Clone, Serialize, Deserialize)]
#[diesel(table_name = ip_blacklist)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct DieselIpBlacklist {
    pub id: Uuid,
    pub ip_address: ipnetwork::IpNetwork,
    pub reason: String,
    pub blocked_by: Option<String>,
    pub auto_generated: Option<bool>,
    pub expires_at: Option<DateTime<Utc>>,
    pub metadata: Option<JsonValue>,
    pub created_at: Option<DateTime<Utc>>,
    pub last_hit: Option<DateTime<Utc>>,
}

#[derive(Insertable, Debug, Clone)]
#[diesel(table_name = ip_blacklist)]
pub struct NewDieselIpBlacklist {
    pub id: Uuid,
    pub ip_address: ipnetwork::IpNetwork,
    pub reason: String,
    pub blocked_by: Option<String>,
    pub auto_generated: Option<bool>,
    pub expires_at: Option<DateTime<Utc>>,
    pub metadata: Option<JsonValue>,
    pub created_at: Option<DateTime<Utc>>,
}

// Alert Notifications models
#[derive(Queryable, Selectable, Insertable, AsChangeset, Debug, Clone, Serialize, Deserialize)]
#[diesel(table_name = alert_notifications)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct DieselAlertNotification {
    pub id: Uuid,
    pub rule_id: Uuid,
    pub event_id: Uuid,
    pub channel: String,
    pub recipient: String,
    pub message: String,
    pub sent_at: Option<DateTime<Utc>>,
    pub delivery_status: Option<String>,
    pub attempts: Option<i32>,
    pub error_message: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
}

#[derive(Insertable, Debug, Clone)]
#[diesel(table_name = alert_notifications)]
pub struct NewDieselAlertNotification {
    pub id: Uuid,
    pub rule_id: Uuid,
    pub event_id: Uuid,
    pub channel: String,
    pub recipient: String,
    pub message: String,
    pub delivery_status: Option<String>,
    pub attempts: Option<i32>,
    pub created_at: Option<DateTime<Utc>>,
}

// Query helper structs
#[derive(Debug, Clone, Default)]
pub struct SecurityEventFilters {
    pub event_types: Option<Vec<String>>,
    pub severity: Option<Vec<String>>,
    pub source: Option<String>,
    pub user_id: Option<String>,
    pub ip_address: Option<ipnetwork::IpNetwork>,
    pub timestamp_after: Option<DateTime<Utc>>,
    pub timestamp_before: Option<DateTime<Utc>>,
    pub limit: Option<i32>,
    pub offset: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityStats {
    pub total_events: i64,
    pub high_severity_count: i64,
    pub blocked_attempts: i64,
    pub unique_attackers: i64,
    pub last_event_at: Option<DateTime<Utc>>,
}