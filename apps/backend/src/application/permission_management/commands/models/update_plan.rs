use crate::prelude::*;
use crate::application::shared::Command;

/// Command to update an existing permission plan
#[derive(Debug, Clone)]
pub struct UpdatePermissionPlanCommand {
    pub plan_id: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub permissions: Option<Vec<String>>,
    pub price: Option<f64>,
    pub currency: Option<String>,
    pub billing_cycle: Option<String>,
    pub is_active: Option<bool>,
    pub is_promoted: Option<bool>,
    pub tier_level: Option<i32>,
    pub max_members: Option<Option<i32>>,
    pub auto_assign_enabled: Option<bool>,
    pub metadata: Option<serde_json::Value>,
}

impl Command for UpdatePermissionPlanCommand {
    type Response = UpdatePermissionPlanResponse;
}

/// Response for update permission plan command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdatePermissionPlanResponse {
    pub plan_id: String,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}
