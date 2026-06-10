//! API Module Domain Model
//!
//! Represents modules that can be accessed via API keys.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Module status enum
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ModuleStatus {
    Active,
    Inactive,
    Deprecated,
    Maintenance,
}

impl From<&str> for ModuleStatus {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "active" => ModuleStatus::Active,
            "inactive" => ModuleStatus::Inactive,
            "deprecated" => ModuleStatus::Deprecated,
            "maintenance" => ModuleStatus::Maintenance,
            _ => ModuleStatus::Active,
        }
    }
}

impl std::fmt::Display for ModuleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ModuleStatus::Active => write!(f, "active"),
            ModuleStatus::Inactive => write!(f, "inactive"),
            ModuleStatus::Deprecated => write!(f, "deprecated"),
            ModuleStatus::Maintenance => write!(f, "maintenance"),
        }
    }
}

/// API Module endpoint definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleEndpoint {
    pub path: String,
    pub method: String,
    pub description: String,
    pub access_level_required: String,
}

/// Access level quotas for a module
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessLevelQuotas {
    pub requests_per_minute: i32,
    pub requests_per_day: i32,
}

/// API Module domain entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiModule {
    pub id: Uuid,
    pub name: String,
    pub display_name: String,
    pub description: Option<String>,
    pub category: String,
    pub status: ModuleStatus,
    pub base_path: String,
    pub default_rate_limit: i32,
    pub access_levels: serde_json::Value, // Map<AccessLevel, AccessLevelQuotas>
    pub endpoints: Vec<ModuleEndpoint>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Request to create a new API module
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateModuleRequest {
    pub name: String,
    pub display_name: String,
    pub description: Option<String>,
    pub category: String,
    pub base_path: String,
    pub default_rate_limit: Option<i32>,
    pub access_levels: Option<serde_json::Value>,
    pub endpoints: Option<Vec<ModuleEndpoint>>,
}

/// Request to update a module
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateModuleRequest {
    pub display_name: Option<String>,
    pub description: Option<String>,
    pub status: Option<String>,
    pub default_rate_limit: Option<i32>,
    pub access_levels: Option<serde_json::Value>,
    pub endpoints: Option<Vec<ModuleEndpoint>>,
}

/// Module list response with total count
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleListResponse {
    pub modules: Vec<ApiModule>,
    pub total: i64,
}

/// Developer portal statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeveloperPortalStats {
    pub total_api_keys: i64,
    pub active_api_keys: i64,
    pub revoked_api_keys: i64,
    pub expired_api_keys: i64,
    pub total_modules: i64,
    pub active_modules: i64,
    pub total_requests_today: i64,
    pub total_requests_this_month: i64,
    pub top_modules_by_usage: Vec<ModuleUsageStats>,
}

/// Module usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleUsageStats {
    pub module_id: Uuid,
    pub module_name: String,
    pub request_count: i64,
    pub unique_api_keys: i64,
}
