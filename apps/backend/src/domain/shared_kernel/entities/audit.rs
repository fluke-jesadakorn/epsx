use crate::domain::shared_kernel::value_objects::UserId;
use chrono::{ DateTime, Utc };
use serde::{ Deserialize, Serialize };

/// Audit log entry for tracking system events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogEntry {
  pub id: String,
  pub wallet_address: Option<UserId>,
  pub action: AuditAction,
  pub resource_type: ResourceType,
  pub resource_id: Option<String>,
  pub result: AuditResult,
  pub ip_address: Option<String>,
  pub user_agent: Option<String>,
  pub additional_data: Option<serde_json::Value>,
  pub timestamp: DateTime<Utc>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub category: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub action_raw: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub resource_type_raw: Option<String>,
}

/// Types of audit actions
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
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
#[serde(rename_all = "snake_case")]
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
#[serde(rename_all = "snake_case")]
pub enum AuditResult {
  Success,
  Failed,
  Denied,
}

impl AuditLogEntry {
  pub fn new(
    wallet_address: Option<UserId>,
    action: AuditAction,
    resource_type: ResourceType,
    result: AuditResult
  ) -> Self {
    Self {
      id: uuid::Uuid::new_v4().to_string(),
      wallet_address,
      action,
      resource_type,
      resource_id: None,
      result,
      ip_address: None,
      user_agent: None,
      additional_data: None,
      timestamp: Utc::now(),
      category: None,
      action_raw: None,
      resource_type_raw: None,
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

/// Actor type for unified audit log
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActorType {
    Admin,
    User,
    System,
    ApiKey,
}

impl std::fmt::Display for ActorType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Admin => write!(f, "admin"),
            Self::User => write!(f, "user"),
            Self::System => write!(f, "system"),
            Self::ApiKey => write!(f, "api_key"),
        }
    }
}

/// Effect/outcome of the audited action
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditEffect {
    Success,
    Failure,
    Denied,
}

impl std::fmt::Display for AuditEffect {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Success => write!(f, "success"),
            Self::Failure => write!(f, "failure"),
            Self::Denied => write!(f, "denied"),
        }
    }
}

/// Metadata for audit operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditMetadata {
  pub requester_id: Option<UserId>,
  pub ip_address: Option<String>,
  pub user_agent: Option<String>,
  pub sid: Option<String>,
  pub request_id: Option<String>,
  pub timestamp: DateTime<Utc>,
}

impl Default for AuditMetadata {
  fn default() -> Self {
    Self::new()
  }
}

impl AuditMetadata {
  pub fn new() -> Self {
    Self {
      requester_id: None,
      ip_address: None,
      user_agent: None,
      sid: None,
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
  pub wallet_address: Option<UserId>,
  pub action: Option<AuditAction>,
  pub resource_type: Option<ResourceType>,
  pub result: Option<AuditResult>,
  pub from_date: Option<DateTime<Utc>>,
  pub to_date: Option<DateTime<Utc>>,
  pub limit: Option<u32>,
  pub offset: Option<u32>,
  pub category: Option<String>,
  pub search: Option<String>,
}

impl Default for AuditQuery {
  fn default() -> Self {
    Self {
      wallet_address: None,
      action: None,
      resource_type: None,
      result: None,
      from_date: None,
      to_date: None,
      limit: Some(50),
      offset: Some(0),
      category: None,
      search: None,
    }
  }
}
