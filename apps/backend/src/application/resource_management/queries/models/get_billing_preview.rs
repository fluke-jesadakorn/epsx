use crate::prelude::*;
use crate::application::shared::Query;
use std::collections::HashMap;

/// Query to get billing preview for a wallet
#[derive(Debug, Clone)]
pub struct GetBillingPreviewQuery {
    pub wallet_address: String,
}

impl Query for GetBillingPreviewQuery {
    type Response = GetBillingPreviewResponse;
}

/// Response containing billing preview
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetBillingPreviewResponse {
    pub wallet_address: String,
    pub plan_id: Option<i32>,
    pub billing_period_start: DateTime<Utc>,
    pub billing_period_end: DateTime<Utc>,
    pub base_cost: f64,
    pub overage_costs: HashMap<String, f64>,
    pub total_cost: f64,
    pub currency: String,
}
