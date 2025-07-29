// Audit logging domain entities for compliance and security tracking

use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

use crate::dom::values::UserId;

/// Unique identifier for audit log entries
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AuditLogId(String);

/// Audit log entry for tracking all IAM and security operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogEntry {
    /// Unique identifier for this audit entry
    id: AuditLogId,
    /// User who performed the action
    actor_id: UserId,
    /// Type of action performed
    action: AuditAction,
    /// Resource that was affected (role ID, policy ID, user ID, etc.)
    resource_type: ResourceType,
    /// Specific resource identifier
    resource_id: String,
    /// Result of the operation
    result: AuditResult,
    /// Additional context and metadata
    metadata: AuditMetadata,
    /// When the action occurred
    timestamp: DateTime<Utc>,
    /// Client IP address
    client_ip: Option<String>,
    /// User agent string
    user_agent: Option<String>,
    /// Session ID for correlation
    session_id: Option<String>,
}

/// Types of auditable actions in the system
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AuditAction {
    // Authentication actions
    Login,
    LoginFailed,
    Logout,
    PasswordReset,
    SessionExpired,
    
    // User management actions
    UserCreated,
    UserUpdated,
    UserDeleted,
    UserRoleChanged,
    UserLevelChanged,
    BulkUserUpdate,
    
    // IAM role actions
    RoleCreated,
    RoleUpdated,
    RoleDeleted,
    RoleAssigned,
    RoleUnassigned,
    
    // IAM policy actions
    PolicyCreated,
    PolicyUpdated,
    PolicyDeleted,
    PolicyAttached,
    PolicyDetached,
    
    // IAM group actions
    GroupCreated,
    GroupUpdated,
    GroupDeleted,
    GroupMemberAdded,
    GroupMemberRemoved,
    
    // Permission actions
    PermissionGranted,
    PermissionDenied,
    PermissionRevoked,
    PermissionEvaluated,
    PermissionOverrideSet,
    PermissionOverrideRemoved,
    
    // System actions
    ConfigurationChanged,
    SecurityPolicyUpdated,
    AuditLogAccessed,
    DataExported,
    BackupCreated,
    BackupRestored,
    SystemEvent,
    
    // Notification actions
    NotificationSent,
    NotificationFailed,
}

/// Types of resources that can be audited
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResourceType {
    User,
    Role,
    Policy,
    Group,
    Session,
    Permission,
    Configuration,
    AuditLog,
    Backup,
    Export,
    Notification,
    System,
}

/// Result of the audited operation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuditResult {
    Success,
    Failure,
    PartialSuccess,
    Denied,
    Error,
}

/// Additional metadata for audit entries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditMetadata {
    /// Previous values before change (for update operations)
    pub previous_values: Option<HashMap<String, String>>,
    /// New values after change (for update operations)
    pub new_values: Option<HashMap<String, String>>,
    /// Error message if operation failed
    pub error_message: Option<String>,
    /// Additional context-specific data
    pub additional_data: HashMap<String, String>,
    /// Number of affected resources (for bulk operations)
    pub affected_count: Option<u32>,
    /// Duration of the operation in milliseconds
    pub duration_ms: Option<u64>,
}

/// Audit query filters for searching logs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditQuery {
    /// Filter by actor (user who performed action)
    pub actor_id: Option<UserId>,
    /// Filter by action type
    pub action: Option<AuditAction>,
    /// Filter by resource type
    pub resource_type: Option<ResourceType>,
    /// Filter by specific resource ID
    pub resource_id: Option<String>,
    /// Filter by result
    pub result: Option<AuditResult>,
    /// Start time for date range
    pub from_time: Option<DateTime<Utc>>,
    /// End time for date range
    pub to_time: Option<DateTime<Utc>>,
    /// Filter by client IP
    pub client_ip: Option<String>,
    /// Session ID for correlation
    pub session_id: Option<String>,
    /// Maximum number of results
    pub limit: Option<u32>,
    /// Offset for pagination
    pub offset: Option<u32>,
}

/// Audit statistics for reporting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditStatistics {
    /// Total number of audit entries
    pub total_entries: u64,
    /// Number of failed operations
    pub failed_operations: u64,
    /// Number of successful operations
    pub successful_operations: u64,
    /// Number of unique actors
    pub unique_actors: u32,
    /// Most common actions
    pub top_actions: Vec<(AuditAction, u32)>,
    /// Most active users
    pub top_actors: Vec<(UserId, u32)>,
    /// Statistics time range
    pub from_time: DateTime<Utc>,
    pub to_time: DateTime<Utc>,
}

// Implementations

impl AuditLogId {
    pub fn new(id: String) -> Self {
        Self(id)
    }
    
    pub fn value(&self) -> &str {
        &self.0
    }
    
    pub fn generate() -> Self {
        Self(uuid::Uuid::new_v4().to_string())
    }
}

impl From<uuid::Uuid> for AuditLogId {
    fn from(uuid: uuid::Uuid) -> Self {
        Self(uuid.to_string())
    }
}

impl AuditLogEntry {
    pub fn new(
        actor_id: UserId,
        action: AuditAction,
        resource_type: ResourceType,
        resource_id: String,
        result: AuditResult,
    ) -> Self {
        Self {
            id: AuditLogId::generate(),
            actor_id,
            action,
            resource_type,
            resource_id,
            result,
            metadata: AuditMetadata::empty(),
            timestamp: Utc::now(),
            client_ip: None,
            user_agent: None,
            session_id: None,
        }
    }
    
    pub fn with_metadata(mut self, metadata: AuditMetadata) -> Self {
        self.metadata = metadata;
        self
    }
    
    pub fn with_client_info(mut self, client_ip: Option<String>, user_agent: Option<String>) -> Self {
        self.client_ip = client_ip;
        self.user_agent = user_agent;
        self
    }
    
    pub fn with_session_id(mut self, session_id: String) -> Self {
        self.session_id = Some(session_id);
        self
    }
    
    // Getters
    pub fn id(&self) -> &AuditLogId {
        &self.id
    }
    
    pub fn actor_id(&self) -> &UserId {
        &self.actor_id
    }
    
    pub fn action(&self) -> &AuditAction {
        &self.action
    }
    
    pub fn resource_type(&self) -> &ResourceType {
        &self.resource_type
    }
    
    pub fn resource_id(&self) -> &str {
        &self.resource_id
    }
    
    pub fn result(&self) -> &AuditResult {
        &self.result
    }
    
    pub fn metadata(&self) -> &AuditMetadata {
        &self.metadata
    }
    
    pub fn timestamp(&self) -> &DateTime<Utc> {
        &self.timestamp
    }
    
    pub fn client_ip(&self) -> Option<&str> {
        self.client_ip.as_deref()
    }
    
    pub fn user_agent(&self) -> Option<&str> {
        self.user_agent.as_deref()
    }
    
    pub fn session_id(&self) -> Option<&str> {
        self.session_id.as_deref()
    }
}

impl AuditMetadata {
    pub fn empty() -> Self {
        Self {
            previous_values: None,
            new_values: None,
            error_message: None,
            additional_data: HashMap::new(),
            affected_count: None,
            duration_ms: None,
        }
    }
    
    pub fn with_change(
        previous_values: HashMap<String, String>,
        new_values: HashMap<String, String>,
    ) -> Self {
        Self {
            previous_values: Some(previous_values),
            new_values: Some(new_values),
            error_message: None,
            additional_data: HashMap::new(),
            affected_count: None,
            duration_ms: None,
        }
    }
    
    pub fn with_error(error_message: String) -> Self {
        Self {
            previous_values: None,
            new_values: None,
            error_message: Some(error_message),
            additional_data: HashMap::new(),
            affected_count: None,
            duration_ms: None,
        }
    }
    
    pub fn with_bulk_count(affected_count: u32) -> Self {
        Self {
            previous_values: None,
            new_values: None,
            error_message: None,
            additional_data: HashMap::new(),
            affected_count: Some(affected_count),
            duration_ms: None,
        }
    }
    
    pub fn with_duration(mut self, duration_ms: u64) -> Self {
        self.duration_ms = Some(duration_ms);
        self
    }
    
    pub fn add_data(mut self, key: String, value: String) -> Self {
        self.additional_data.insert(key, value);
        self
    }
    
    pub fn with_additional_info(mut self, key: &str, value: String) -> Self {
        self.additional_data.insert(key.to_string(), value);
        self
    }
}

impl AuditQuery {
    pub fn new() -> Self {
        Self {
            actor_id: None,
            action: None,
            resource_type: None,
            resource_id: None,
            result: None,
            from_time: None,
            to_time: None,
            client_ip: None,
            session_id: None,
            limit: Some(100), // Default limit
            offset: Some(0),  // Default offset
        }
    }
    
    pub fn by_actor(mut self, actor_id: UserId) -> Self {
        self.actor_id = Some(actor_id);
        self
    }
    
    pub fn by_action(mut self, action: AuditAction) -> Self {
        self.action = Some(action);
        self
    }
    
    pub fn by_resource_type(mut self, resource_type: ResourceType) -> Self {
        self.resource_type = Some(resource_type);
        self
    }
    
    pub fn by_resource_id(mut self, resource_id: String) -> Self {
        self.resource_id = Some(resource_id);
        self
    }
    
    pub fn by_result(mut self, result: AuditResult) -> Self {
        self.result = Some(result);
        self
    }
    
    pub fn in_time_range(mut self, from: DateTime<Utc>, to: DateTime<Utc>) -> Self {
        self.from_time = Some(from);
        self.to_time = Some(to);
        self
    }
    
    pub fn with_pagination(mut self, limit: u32, offset: u32) -> Self {
        self.limit = Some(limit);
        self.offset = Some(offset);
        self
    }
}

impl Default for AuditQuery {
    fn default() -> Self {
        Self::new()
    }
}

// Display implementations
impl std::fmt::Display for AuditAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuditAction::Login => write!(f, "login"),
            AuditAction::LoginFailed => write!(f, "login_failed"),
            AuditAction::Logout => write!(f, "logout"),
            AuditAction::PasswordReset => write!(f, "password_reset"),
            AuditAction::SessionExpired => write!(f, "session_expired"),
            AuditAction::UserCreated => write!(f, "user_created"),
            AuditAction::UserUpdated => write!(f, "user_updated"),
            AuditAction::UserDeleted => write!(f, "user_deleted"),
            AuditAction::UserRoleChanged => write!(f, "user_role_changed"),
            AuditAction::UserLevelChanged => write!(f, "user_level_changed"),
            AuditAction::BulkUserUpdate => write!(f, "bulk_user_update"),
            AuditAction::RoleCreated => write!(f, "role_created"),
            AuditAction::RoleUpdated => write!(f, "role_updated"),
            AuditAction::RoleDeleted => write!(f, "role_deleted"),
            AuditAction::RoleAssigned => write!(f, "role_assigned"),
            AuditAction::RoleUnassigned => write!(f, "role_unassigned"),
            AuditAction::PolicyCreated => write!(f, "policy_created"),
            AuditAction::PolicyUpdated => write!(f, "policy_updated"),
            AuditAction::PolicyDeleted => write!(f, "policy_deleted"),
            AuditAction::PolicyAttached => write!(f, "policy_attached"),
            AuditAction::PolicyDetached => write!(f, "policy_detached"),
            AuditAction::GroupCreated => write!(f, "group_created"),
            AuditAction::GroupUpdated => write!(f, "group_updated"),
            AuditAction::GroupDeleted => write!(f, "group_deleted"),
            AuditAction::GroupMemberAdded => write!(f, "group_member_added"),
            AuditAction::GroupMemberRemoved => write!(f, "group_member_removed"),
            AuditAction::PermissionGranted => write!(f, "permission_granted"),
            AuditAction::PermissionDenied => write!(f, "permission_denied"),
            AuditAction::PermissionEvaluated => write!(f, "permission_evaluated"),
            AuditAction::PermissionOverrideSet => write!(f, "permission_override_set"),
            AuditAction::PermissionOverrideRemoved => write!(f, "permission_override_removed"),
            AuditAction::ConfigurationChanged => write!(f, "configuration_changed"),
            AuditAction::SecurityPolicyUpdated => write!(f, "security_policy_updated"),
            AuditAction::AuditLogAccessed => write!(f, "audit_log_accessed"),
            AuditAction::DataExported => write!(f, "data_exported"),
            AuditAction::BackupCreated => write!(f, "backup_created"),
            AuditAction::BackupRestored => write!(f, "backup_restored"),
            AuditAction::NotificationSent => write!(f, "notification_sent"),
            AuditAction::NotificationFailed => write!(f, "notification_failed"),
            AuditAction::PermissionRevoked => write!(f, "permission_revoked"),
            AuditAction::SystemEvent => write!(f, "system_event"),
        }
    }
}

impl std::fmt::Display for ResourceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ResourceType::User => write!(f, "user"),
            ResourceType::Role => write!(f, "role"),
            ResourceType::Policy => write!(f, "policy"),
            ResourceType::Group => write!(f, "group"),
            ResourceType::Session => write!(f, "session"),
            ResourceType::Permission => write!(f, "permission"),
            ResourceType::Configuration => write!(f, "configuration"),
            ResourceType::AuditLog => write!(f, "audit_log"),
            ResourceType::Backup => write!(f, "backup"),
            ResourceType::Export => write!(f, "export"),
            ResourceType::Notification => write!(f, "notification"),
            ResourceType::System => write!(f, "system"),
        }
    }
}

impl std::fmt::Display for AuditResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuditResult::Success => write!(f, "success"),
            AuditResult::Failure => write!(f, "failure"),
            AuditResult::PartialSuccess => write!(f, "partial_success"),
            AuditResult::Denied => write!(f, "denied"),
            AuditResult::Error => write!(f, "error"),
        }
    }
}

// Error type for audit operations
#[derive(Debug, thiserror::Error)]
pub enum AuditError {
    #[error("Audit log not found: {0}")]
    LogNotFound(String),
    
    #[error("Invalid audit query: {0}")]
    InvalidQuery(String),
    
    #[error("Audit storage error: {0}")]
    StorageError(String),
    
    #[error("Database error: {0}")]
    DatabaseError(String),
    
    #[error("Permission denied for audit operation")]
    PermissionDenied,
    
    #[error("Audit retention policy violation: {0}")]
    RetentionViolation(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn should_create_audit_log_entry() {
        let actor_id = UserId::new("user123".to_string());
        let entry = AuditLogEntry::new(
            actor_id.clone(),
            AuditAction::RoleCreated,
            ResourceType::Role,
            "role456".to_string(),
            AuditResult::Success,
        );
        
        assert_eq!(entry.actor_id(), &actor_id);
        assert_eq!(entry.action(), &AuditAction::RoleCreated);
        assert_eq!(entry.resource_type(), &ResourceType::Role);
        assert_eq!(entry.resource_id(), "role456");
        assert_eq!(entry.result(), &AuditResult::Success);
    }
    
    #[test]
    fn should_create_audit_query() {
        let actor_id = UserId::new("user123".to_string());
        let query = AuditQuery::new()
            .by_actor(actor_id.clone())
            .by_action(AuditAction::UserCreated)
            .with_pagination(50, 0);
        
        assert_eq!(query.actor_id.as_ref(), Some(&actor_id));
        assert_eq!(query.action.as_ref(), Some(&AuditAction::UserCreated));
        assert_eq!(query.limit, Some(50));
        assert_eq!(query.offset, Some(0));
    }
    
    #[test]
    fn should_create_metadata_with_changes() {
        let mut previous = HashMap::new();
        previous.insert("role".to_string(), "user".to_string());
        
        let mut new = HashMap::new();
        new.insert("role".to_string(), "admin".to_string());
        
        let metadata = AuditMetadata::with_change(previous.clone(), new.clone());
        
        assert_eq!(metadata.previous_values.as_ref(), Some(&previous));
        assert_eq!(metadata.new_values.as_ref(), Some(&new));
    }
}