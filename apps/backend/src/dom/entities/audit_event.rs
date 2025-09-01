use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    pub id: Uuid,
    pub event_type: String,
    pub firebase_uid: String,
    pub actor_firebase_uid: Option<String>,
    pub resource_type: String,
    pub resource_id: Option<String>,
    pub action: String,
    pub details: Option<serde_json::Value>,
    pub timestamp: DateTime<Utc>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub status: AuditEventStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditEventStatus {
    Success,
    Failed,
    Pending,
}

impl Default for AuditEventStatus {
    fn default() -> Self {
        AuditEventStatus::Success
    }
}

impl AuditEvent {
    pub fn new(
        event_type: String,
        firebase_uid: String,
        action: String,
        resource_type: String,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            event_type,
            firebase_uid,
            actor_firebase_uid: None,
            resource_type,
            resource_id: None,
            action,
            details: None,
            timestamp: Utc::now(),
            ip_address: None,
            user_agent: None,
            status: AuditEventStatus::Success,
        }
    }
}