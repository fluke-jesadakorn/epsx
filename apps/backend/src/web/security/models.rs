// Security models - Minimal stubs for Diesel migration
// TODO: Implement with Diesel

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityAlert {
    pub id: Uuid,
    pub title: String,
    pub description: String,
    pub severity: SecuritySeverity,
    pub event_type: SecurityEventType,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityEvent {
    pub id: Uuid,
    pub event_type: SecurityEventType,
    pub severity: SecuritySeverity,
    pub description: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSecurityEventRequest {
    pub event_type: SecurityEventType,
    pub severity: SecuritySeverity,
    pub description: String,
    pub metadata: Option<serde_json::Value>,
    pub user_id: Option<String>,
    pub session_id: Option<String>,
    pub path: Option<String>,
    pub method: Option<String>,
    pub ip_address: String,
    pub user_agent: Option<String>,
    pub details: Option<serde_json::Value>,
    pub country_code: Option<String>,
    pub city: Option<String>,
    pub device_fingerprint: Option<String>,
    pub correlation_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityEventType {
    AuthenticationFailure,
    AuthorizationDenied,
    SuspiciousActivity,
    BruteForceAttempt,
    DataAccess,
    ConfigurationChange,
    SystemError,
    Other(String),
}

impl std::fmt::Display for SecurityEventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SecurityEventType::AuthenticationFailure => write!(f, "AuthenticationFailure"),
            SecurityEventType::AuthorizationDenied => write!(f, "AuthorizationDenied"),
            SecurityEventType::SuspiciousActivity => write!(f, "SuspiciousActivity"),
            SecurityEventType::BruteForceAttempt => write!(f, "BruteForceAttempt"),
            SecurityEventType::DataAccess => write!(f, "DataAccess"),
            SecurityEventType::ConfigurationChange => write!(f, "ConfigurationChange"),
            SecurityEventType::SystemError => write!(f, "SystemError"),
            SecurityEventType::Other(s) => write!(f, "Other({})", s),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum SecuritySeverity {
    Low,
    Medium,
    High,
    Critical,
}

impl std::fmt::Display for SecuritySeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SecuritySeverity::Low => write!(f, "Low"),
            SecuritySeverity::Medium => write!(f, "Medium"),
            SecuritySeverity::High => write!(f, "High"),
            SecuritySeverity::Critical => write!(f, "Critical"),
        }
    }
}

impl Default for SecuritySeverity {
    fn default() -> Self {
        SecuritySeverity::Low
    }
}

impl Default for SecurityEventType {
    fn default() -> Self {
        SecurityEventType::Other("Unknown".to_string())
    }
}

// Stub function to satisfy module requirements
pub fn stub_function() {
    tracing::warn!("Security models stubbed - implement with Diesel");
}