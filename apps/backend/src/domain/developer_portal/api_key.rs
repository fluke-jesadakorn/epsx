//! API Key Domain Model
//!
//! Represents developer API keys for accessing backend resources.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// API Key status enum
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ApiKeyStatus {
    Active,
    Revoked,
    Expired,
}

impl From<&str> for ApiKeyStatus {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "active" => ApiKeyStatus::Active,
            "revoked" => ApiKeyStatus::Revoked,
            "expired" => ApiKeyStatus::Expired,
            _ => ApiKeyStatus::Active,
        }
    }
}

impl std::fmt::Display for ApiKeyStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ApiKeyStatus::Active => write!(f, "active"),
            ApiKeyStatus::Revoked => write!(f, "revoked"),
            ApiKeyStatus::Expired => write!(f, "expired"),
        }
    }
}

/// Access level for API modules
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum AccessLevel {
    Bronze,
    Silver,
    Gold,
    Platinum,
    Enterprise,
}

impl From<&str> for AccessLevel {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "bronze" => AccessLevel::Bronze,
            "silver" => AccessLevel::Silver,
            "gold" => AccessLevel::Gold,
            "platinum" => AccessLevel::Platinum,
            "enterprise" => AccessLevel::Enterprise,
            _ => AccessLevel::Bronze,
        }
    }
}

impl std::fmt::Display for AccessLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AccessLevel::Bronze => write!(f, "bronze"),
            AccessLevel::Silver => write!(f, "silver"),
            AccessLevel::Gold => write!(f, "gold"),
            AccessLevel::Platinum => write!(f, "platinum"),
            AccessLevel::Enterprise => write!(f, "enterprise"),
        }
    }
}

/// Module access configuration for an API key
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleAccess {
    pub module_id: Uuid,
    pub module_name: String,
    pub access_level: AccessLevel,
    pub custom_rate_limit: Option<i32>,
    pub custom_quotas: serde_json::Value,
}

/// Rate limits configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimits {
    pub per_minute: i32,
    pub per_day: i32,
}

impl Default for RateLimits {
    fn default() -> Self {
        Self {
            per_minute: 60,
            per_day: 10000,
        }
    }
}

/// Permission group information for an API key
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionGroupInfo {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub group_type: String,
}

/// API Key domain entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKey {
    pub id: Uuid,
    pub key_prefix: String,
    pub full_key: Option<String>, // Full key for user to copy (stored for owner access)
    pub client_name: String,
    pub client_description: Option<String>,
    pub client_contact_email: Option<String>,
    pub wallet_address: String,
    pub status: ApiKeyStatus,
    pub total_requests: i64,
    pub ip_restrictions: Vec<String>,
    pub rate_limits: RateLimits,
    #[serde(default)]
    pub allowed_modules: Vec<ModuleAccess>,
    #[serde(default)]
    pub permission_groups: Vec<PermissionGroupInfo>,
    /// Individual permission strings selected by the user
    #[serde(default)]
    pub selected_permissions: Vec<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub last_used_at: Option<DateTime<Utc>>,
    pub revoked_at: Option<DateTime<Utc>>,
    pub revoked_by: Option<String>,
    pub revocation_reason: Option<String>,
    pub created_at: DateTime<Utc>,
    pub created_by: String,
    pub updated_at: DateTime<Utc>,
}

/// Request to create a new API key
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateApiKeyRequest {
    pub client_name: String,
    pub client_description: Option<String>,
    pub client_contact_email: Option<String>,
    pub wallet_address: String,
    #[serde(default)]
    pub allowed_modules: Vec<ModuleAccessRequest>,
    /// Permission group IDs to assign to this API key
    #[serde(default)]
    pub group_ids: Vec<Uuid>,
    /// Individual permission strings to assign (must be within user's groups)
    #[serde(default)]
    pub permissions: Vec<String>,
    pub ip_restrictions: Option<Vec<String>>,
    pub rate_limit_per_minute: Option<i32>,
    pub rate_limit_per_day: Option<i32>,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_by: String,
}

/// Module access request for creating API key
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleAccessRequest {
    pub module_id: Uuid,
    pub access_level: String,
    pub custom_quotas: Option<serde_json::Value>,
}

/// Response for a newly created API key (includes full key, shown only once)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKeyCreatedResponse {
    pub api_key: ApiKey,
    pub full_key: String, // Only returned on creation
}

/// Request to update an API key
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateApiKeyRequest {
    pub client_name: Option<String>,
    pub client_description: Option<String>,
    pub client_contact_email: Option<String>,
    pub ip_restrictions: Option<Vec<String>>,
    pub rate_limit_per_minute: Option<i32>,
    pub rate_limit_per_day: Option<i32>,
    pub expires_at: Option<DateTime<Utc>>,
}

/// Request to revoke an API key
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevokeApiKeyRequest {
    pub reason: String,
    pub revoked_by: String,
}

/// API Key usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKeyUsageStats {
    pub api_key_id: Uuid,
    pub total_requests: i64,
    pub requests_today: i64,
    pub requests_this_hour: i64,
    pub average_response_time_ms: f64,
    pub error_rate: f64,
    pub top_endpoints: Vec<EndpointUsage>,
}

/// Endpoint usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointUsage {
    pub endpoint: String,
    pub method: String,
    pub request_count: i64,
    pub avg_response_time_ms: f64,
}
