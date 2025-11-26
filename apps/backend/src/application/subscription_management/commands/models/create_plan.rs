use crate::prelude::*;
use crate::application::shared::Command;

/// Command to create a new plan
#[derive(Debug, Clone)]
pub struct CreatePlanCommand {
    pub name: String,
    pub description: String,
    pub permission_group_id: String,
    pub price: f64,
    pub currency: String,
    pub billing_cycle: String,
    pub target_audience: String,
    pub api_calls_limit: Option<i32>,
    pub rankings_limit: Option<i32>,
    pub analytics_enabled: bool,
    pub premium_support: bool,
    pub is_active: Option<bool>,
    pub is_promoted: Option<bool>,
    pub display_order: Option<i32>,
    pub metadata: Option<serde_json::Value>,
}

impl Command for CreatePlanCommand {
    type Response = CreatePlanResponse;
}

/// Response for create plan command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePlanResponse {
    pub plan_id: i32,
    pub name: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}
