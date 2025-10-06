use crate::prelude::*;
use crate::application::shared::Command;

/// Command to update an existing plan
#[derive(Debug, Clone)]
pub struct UpdatePlanCommand {
    pub plan_id: i32,
    pub name: Option<String>,
    pub description: Option<String>,
    pub price: Option<f64>,
    pub currency: Option<String>,
    pub billing_cycle: Option<String>,
    pub target_audience: Option<String>,
    pub is_active: Option<bool>,
    pub is_promoted: Option<bool>,
    pub display_order: Option<i32>,
    pub metadata: Option<serde_json::Value>,
}

impl Command for UpdatePlanCommand {
    type Response = UpdatePlanResponse;
}

/// Response for update plan command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdatePlanResponse {
    pub plan_id: i32,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}
