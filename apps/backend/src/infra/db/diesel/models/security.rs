// ============================================================================
// SIMPLE SECURITY MODELS - REPLACING COMPLEX SECURITY SYSTEM
// ============================================================================
// This file replaces 500+ lines of complex security models with simple audit logging
// Works with the simple role system from auth/roles.rs

use diesel::prelude::*;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use serde_json::Value as JsonValue;

use crate::infra::db::diesel::schema::security_events;
use crate::infra::db::diesel::types::DieselIpAddr;

// ============================================================================
// SIMPLE SECURITY EVENT MODEL (MATCHING ACTUAL SCHEMA)
// ============================================================================

#[derive(Queryable, Selectable, Insertable, AsChangeset, Debug, Clone, Serialize, Deserialize)]
#[diesel(table_name = security_events)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct DieselSecurityEvent {
    pub id: Uuid,
    pub event_type: String,
    pub severity: String,
    pub timestamp: DateTime<Utc>,
    pub user_id: Option<String>,
    pub session_id: Option<String>,
    pub ip_address: DieselIpAddr,
    pub user_agent: Option<String>,
    pub path: Option<String>,
    pub method: Option<String>,
    pub details: JsonValue,
    pub source: String,
    pub resolved: bool,
    pub resolution_notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub risk_score: Option<i32>,
}

#[derive(Insertable, Debug, Clone)]
#[diesel(table_name = security_events)]
pub struct NewDieselSecurityEvent {
    pub id: Uuid,
    pub event_type: String,
    pub severity: String,
    pub timestamp: DateTime<Utc>,
    pub user_id: Option<String>,
    pub session_id: Option<String>,
    pub ip_address: DieselIpAddr,
    pub user_agent: Option<String>,
    pub path: Option<String>,
    pub method: Option<String>,
    pub details: JsonValue,
    pub source: String,
    pub resolved: bool,
    pub resolution_notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub risk_score: Option<i32>,
}

// ============================================================================
// SIMPLE SECURITY EVENT TYPES
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityEventType {
    Login,
    Logout,
    PermissionDenied,
    FeatureAccess,
    RoleChange,
    SuspiciousActivity,
}

impl std::fmt::Display for SecurityEventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SecurityEventType::Login => write!(f, "login"),
            SecurityEventType::Logout => write!(f, "logout"),
            SecurityEventType::PermissionDenied => write!(f, "permission_denied"),
            SecurityEventType::FeatureAccess => write!(f, "feature_access"),
            SecurityEventType::RoleChange => write!(f, "role_change"),
            SecurityEventType::SuspiciousActivity => write!(f, "suspicious_activity"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecuritySeverity {
    Low,
    Medium,
    High,
    Critical,
}

impl std::fmt::Display for SecuritySeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SecuritySeverity::Low => write!(f, "low"),
            SecuritySeverity::Medium => write!(f, "medium"),
            SecuritySeverity::High => write!(f, "high"),
            SecuritySeverity::Critical => write!(f, "critical"),
        }
    }
}

// ============================================================================
// SIMPLE SECURITY HELPER FUNCTIONS
// ============================================================================

impl NewDieselSecurityEvent {
    pub fn new(
        event_type: SecurityEventType,
        severity: SecuritySeverity,
        source: &str,
        user_id: Option<&str>,
        ip_address: Option<std::net::IpAddr>,
        user_agent: Option<&str>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            event_type: event_type.to_string(),
            severity: severity.to_string(),
            source: source.to_string(),
            user_id: user_id.map(|s| s.to_string()),
            session_id: None,
            ip_address: ip_address.unwrap_or_else(|| "127.0.0.1".parse().unwrap()).into(),
            user_agent: user_agent.map(|s| s.to_string()),
            path: None,
            method: None,
            details: serde_json::json!({}),
            resolved: false,
            resolution_notes: None,
            timestamp: now,
            created_at: now,
            updated_at: now,
            risk_score: Some(50),
        }
    }
    
    pub fn login_event(user_id: &str, ip_address: Option<std::net::IpAddr>) -> Self {
        Self::new(
            SecurityEventType::Login,
            SecuritySeverity::Low,
            "auth",
            Some(user_id),
            ip_address,
            None,
        )
    }
    
    pub fn logout_event(user_id: &str, ip_address: Option<std::net::IpAddr>) -> Self {
        Self::new(
            SecurityEventType::Logout,
            SecuritySeverity::Low,
            "auth",
            Some(user_id),
            ip_address,
            None,
        )
    }
    
    pub fn permission_denied_event(user_id: &str, _feature: &str, ip_address: Option<std::net::IpAddr>) -> Self {
        Self::new(
            SecurityEventType::PermissionDenied,
            SecuritySeverity::Medium,
            "permission_check",
            Some(user_id),
            ip_address,
            None,
        )
    }
    
    pub fn feature_access_event(user_id: &str, _feature: &str, ip_address: Option<std::net::IpAddr>) -> Self {
        Self::new(
            SecurityEventType::FeatureAccess,
            SecuritySeverity::Low,
            "feature_access",
            Some(user_id),
            ip_address,
            None,
        )
    }
    
    pub fn role_change_event(user_id: &str, _new_role: &str, ip_address: Option<std::net::IpAddr>) -> Self {
        Self::new(
            SecurityEventType::RoleChange,
            SecuritySeverity::High,
            "role_management",
            Some(user_id),
            ip_address,
            None,
        )
    }
}

// ============================================================================
// STUB MODELS FOR REMOVED COMPLEX SECURITY FEATURES
// ============================================================================

// Stub models for attack attempts (matching actual schema)
#[derive(Queryable, Selectable, Debug, Clone, Serialize, Deserialize)]
#[diesel(table_name = crate::infra::db::diesel::schema::attack_attempts)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct DieselAttackAttempt {
    pub id: Uuid,
    pub ip_address: DieselIpAddr,
    pub user_id: Option<String>,
    pub attempt_type: String,
    pub timestamp: DateTime<Utc>,
    pub success: Option<bool>,
}

#[derive(Insertable, Debug, Clone)]
#[diesel(table_name = crate::infra::db::diesel::schema::attack_attempts)]
pub struct NewDieselAttackAttempt {
    pub id: Uuid,
    pub ip_address: DieselIpAddr,
    pub user_id: Option<String>,
    pub attempt_type: String,
    pub timestamp: DateTime<Utc>,
    pub success: Option<bool>,
}

// Stub models for IP blacklist (matching actual schema)
#[derive(Queryable, Selectable, Debug, Clone, Serialize, Deserialize)]
#[diesel(table_name = crate::infra::db::diesel::schema::ip_blacklist)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct DieselIpBlacklist {
    pub id: Uuid,
    pub ip_address: DieselIpAddr,
    pub reason: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Insertable, Debug, Clone)]
#[diesel(table_name = crate::infra::db::diesel::schema::ip_blacklist)]
pub struct NewDieselIpBlacklist {
    pub id: Uuid,
    pub ip_address: DieselIpAddr,
    pub reason: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
}

// Stub models for security alert rules (complex alerting removed)
#[derive(Queryable, Selectable, Debug, Clone, Serialize, Deserialize)]
#[diesel(table_name = crate::infra::db::diesel::schema::security_alert_rules)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct DieselSecurityAlertRule {
    pub id: Uuid,
    pub name: String,
    pub rule_type: String,
    pub condition: JsonValue,
    pub is_active: bool,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub last_triggered: Option<DateTime<Utc>>,
}

#[derive(Insertable, Debug, Clone)]
#[diesel(table_name = crate::infra::db::diesel::schema::security_alert_rules)]
pub struct NewDieselSecurityAlertRule {
    pub id: Uuid,
    pub name: String,
    pub rule_type: String,
    pub condition: JsonValue,
    pub is_active: bool,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub last_triggered: Option<DateTime<Utc>>,
}

// Stub models for alert notifications (matching actual schema)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DieselAlertNotification {
    pub id: Uuid,
    pub alert_id: Uuid,
    pub channel: String,
    pub sent_at: DateTime<Utc>,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub delivery_status: String,
}

#[derive(Insertable, Debug, Clone)]
#[diesel(table_name = crate::infra::db::diesel::schema::alert_notifications)]
pub struct NewDieselAlertNotification {
    pub id: Uuid,
    pub alert_id: Uuid,
    pub channel: String,
    pub sent_at: DateTime<Utc>,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub delivery_status: String,
}

// Stub for security stats
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityStats {
    pub total_events: i64,
    pub high_severity_count: i64,
    pub blocked_attempts: i64,
    pub unique_attackers: i64,
    pub last_event_at: Option<DateTime<Utc>>,
}

// ============================================================================
// DATABASE OPERATIONS (SIMPLE)
// ============================================================================

pub fn log_security_event(
    conn: &mut diesel::pg::PgConnection,
    event: NewDieselSecurityEvent,
) -> Result<DieselSecurityEvent, diesel::result::Error> {
    diesel::insert_into(security_events::table)
        .values(&event)
        .get_result(conn)
}

pub fn get_recent_security_events(
    conn: &mut diesel::pg::PgConnection,
    limit: i64,
) -> Result<Vec<DieselSecurityEvent>, diesel::result::Error> {
    security_events::table
        .order(security_events::timestamp.desc())
        .limit(limit)
        .load(conn)
}

pub fn get_user_security_events(
    conn: &mut diesel::pg::PgConnection,
    user_id: &str,
    limit: i64,
) -> Result<Vec<DieselSecurityEvent>, diesel::result::Error> {
    security_events::table
        .filter(security_events::user_id.eq(user_id))
        .order(security_events::timestamp.desc())
        .limit(limit)
        .load(conn)
}