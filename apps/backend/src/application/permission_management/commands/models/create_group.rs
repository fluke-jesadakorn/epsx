use crate::prelude::*;
use crate::application::shared::Command;

/// Command to create a new permission group
#[derive(Debug, Clone)]
pub struct CreatePermissionGroupCommand {
    pub name: String,
    pub slug: String,
    pub description: String,
    pub group_type: String,
    pub permissions: Vec<String>,
    pub price: Option<f64>,
    pub currency: Option<String>,
    pub billing_cycle: Option<String>,
    pub is_active: Option<bool>,
    pub is_promoted: Option<bool>,
    pub display_order: Option<i32>,
    pub max_members: Option<i32>,
    pub auto_assign_enabled: Option<bool>,
    pub metadata: Option<serde_json::Value>,
}

impl Command for CreatePermissionGroupCommand {
    type Response = CreatePermissionGroupResponse;
}

/// Response for create permission group command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePermissionGroupResponse {
    pub group_id: String,
    pub name: String,
    pub slug: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}
