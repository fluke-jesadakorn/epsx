use crate::prelude::*;
use crate::application::shared::Query;
use std::collections::HashMap;

/// Query to get resource usage for a wallet
#[derive(Debug, Clone)]
pub struct GetResourceUsageQuery {
    pub wallet_address: String,
    pub access_context: String,
}

impl Query for GetResourceUsageQuery {
    type Response = GetResourceUsageResponse;
}

/// Response containing resource usage details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetResourceUsageResponse {
    pub wallet_address: String,
    pub plan_id: Option<i32>,
    pub current_usage: HashMap<String, i64>,
    pub quota_limits: HashMap<String, i64>,
    pub usage_percentages: HashMap<String, f64>,
    pub billing_period_start: DateTime<Utc>,
    pub billing_period_end: DateTime<Utc>,
}
