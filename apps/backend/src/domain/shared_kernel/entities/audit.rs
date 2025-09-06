use crate::domain::shared_kernel::value_objects::UserId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Audit log entry for tracking system events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogEntry {
    pub id: String,
    pub user_id: Option<UserId>,
    pub action: AuditAction,
    pub resource_type: ResourceType,
    pub resource_id: Option<String>,
    pub result: AuditResult,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub additional_data: Option<serde_json::Value>,
    pub timestamp: DateTime<Utc>,
}

/// Types of audit actions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditAction {
    Create,
    Read,
    Update,
    Delete,
    Login,
    Logout,
    PermissionGranted,
    PermissionRevoked,
    PaymentInitiated,
    PaymentCompleted,
    Export,
}

/// Resource types being audited
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResourceType {
    User,
    Session,
    Payment,
    Notification,
    Analytics,
    Admin,
}

/// Result of the audited action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditResult {
    Success,
    Failed,
    Denied,
}

impl AuditLogEntry {
    pub fn new(
        user_id: Option<UserId>,
        action: AuditAction,
        resource_type: ResourceType,
        result: AuditResult,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            user_id,
            action,
            resource_type,
            resource_id: None,
            result,
            ip_address: None,
            user_agent: None,
            additional_data: None,
            timestamp: Utc::now(),
        }
    }

    pub fn with_resource_id(mut self, resource_id: String) -> Self {
        self.resource_id = Some(resource_id);
        self
    }

    pub fn with_ip_address(mut self, ip_address: String) -> Self {
        self.ip_address = Some(ip_address);
        self
    }

    pub fn with_user_agent(mut self, user_agent: String) -> Self {
        self.user_agent = Some(user_agent);
        self
    }

    pub fn with_additional_data(mut self, data: serde_json::Value) -> Self {
        self.additional_data = Some(data);
        self
    }
}

/// Metadata for audit operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditMetadata {
    pub requester_id: Option<UserId>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub session_id: Option<String>,
    pub request_id: Option<String>,
    pub timestamp: DateTime<Utc>,
}

impl AuditMetadata {
    pub fn new() -> Self {
        Self {
            requester_id: None,
            ip_address: None,
            user_agent: None,
            session_id: None,
            request_id: None,
            timestamp: Utc::now(),
        }
    }

    pub fn with_requester_id(mut self, requester_id: UserId) -> Self {
        self.requester_id = Some(requester_id);
        self
    }
}

/// Query parameters for audit log searches
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditQuery {
    pub user_id: Option<UserId>,
    pub action: Option<AuditAction>,
    pub resource_type: Option<ResourceType>,
    pub result: Option<AuditResult>,
    pub from_date: Option<DateTime<Utc>>,
    pub to_date: Option<DateTime<Utc>>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

impl Default for AuditQuery {
    fn default() -> Self {
        Self {
            user_id: None,
            action: None,
            resource_type: None,
            result: None,
            from_date: None,
            to_date: None,
            limit: Some(50),
            offset: Some(0),
        }
    }
}