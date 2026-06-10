use crate::prelude::*;
use crate::application::shared::Command;
use crate::domain::subscription_management::{PlanId, BillingCycle, PlanFeatures};

/// Command to update an existing plan
#[derive(Debug, Clone)]
pub struct UpdatePlanCommand {
    pub id: PlanId,
    pub name: Option<String>,
    pub description: Option<String>,
    pub price: Option<rust_decimal::Decimal>,
    pub currency: Option<String>,
    pub billing_cycle: Option<BillingCycle>,
    pub features: Option<PlanFeatures>,
    pub target_audience: Option<String>,
    pub permissions: Option<Vec<String>>,
    pub is_active: Option<bool>,
    pub is_promoted: Option<bool>,
    pub tier_level: Option<i32>,
    pub metadata: Option<serde_json::Value>,
}

impl Command for UpdatePlanCommand {
    type Response = UpdatePlanResponse;
}

/// Response for update plan command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdatePlanResponse {
    pub plan_id: String,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}
