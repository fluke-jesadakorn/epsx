use crate::prelude::*;
use crate::application::shared::Command;

/// Command to create a new permission plan
#[derive(Debug, Clone)]
pub struct CreatePermissionPlanCommand {
    pub name: String,
    pub slug: String,
    pub description: String,
    pub plan_type: String,
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

impl Command for CreatePermissionPlanCommand {
    type Response = CreatePermissionPlanResponse;
}

/// Response for create permission plan command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePermissionPlanResponse {
    pub plan_id: String,
    pub name: String,
    pub slug: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}
