use crate::prelude::*;
use crate::application::shared::Query;

/// Query to get a single plan
#[derive(Debug, Clone)]
pub struct GetPlanQuery {
    pub plan_id: i32,
}

impl Query for GetPlanQuery {
    type Response = GetPlanResponse;
}

/// Response for get plan query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetPlanResponse {
    pub id: i32,
    pub name: String,
    pub description: String,
    pub plan_id: String,
    pub price: f64,
    pub currency: String,
    pub billing_cycle: String,
    pub target_audience: String,
    pub is_active: bool,
    pub is_promoted: bool,
    pub display_order: i32,
    pub features: serde_json::Value,
    pub metadata: serde_json::Value,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}
