use crate::prelude::*;
use crate::application::shared::Command;

/// Command to update an existing permission group
#[derive(Debug, Clone)]
pub struct UpdatePermissionGroupCommand {
    pub group_id: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub permissions: Option<Vec<String>>,
    pub price: Option<f64>,
    pub currency: Option<String>,
    pub billing_cycle: Option<String>,
    pub is_active: Option<bool>,
    pub is_promoted: Option<bool>,
    pub display_order: Option<i32>,
    pub max_members: Option<Option<i32>>,
    pub auto_assign_enabled: Option<bool>,
    pub metadata: Option<serde_json::Value>,
}

impl Command for UpdatePermissionGroupCommand {
    type Response = UpdatePermissionGroupResponse;
}

/// Response for update permission group command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdatePermissionGroupResponse {
    pub group_id: String,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}
