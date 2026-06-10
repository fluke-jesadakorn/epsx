use crate::prelude::*;
use crate::application::shared::Command;

use crate::domain::subscription_management::value_objects::PlanFeatures;

/// Command to create a new plan
#[derive(Debug, Clone)]
pub struct CreatePlanCommand {
    pub name: String,
    pub description: String,
    pub price_amount: rust_decimal::Decimal,
    pub currency: String,
    pub billing_cycle: String,
    pub permissions: Vec<String>,
    pub features: PlanFeatures,
    pub target_audience: String,
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
    pub plan_id: String,
    pub name: String,
}
