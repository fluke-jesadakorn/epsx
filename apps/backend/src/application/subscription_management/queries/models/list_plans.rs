use crate::prelude::*;
use crate::application::shared::Query;

/// Query to list plans
#[derive(Debug, Clone)]
pub struct ListPlansQuery {
    pub is_active: Option<bool>,
    pub is_promoted: Option<bool>,
    pub page: Option<u32>,
    pub limit: Option<u32>,
}

impl Query for ListPlansQuery {
    type Response = ListPlansResponse;
}

/// Response for list plans query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListPlansResponse {
    pub plans: Vec<PlanSummary>,
    pub total: i64,
    pub page: u32,
    pub limit: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanSummary {
    pub id: i32,
    pub name: String,
    pub description: String,
    pub price: f64,
    pub currency: String,
    pub billing_cycle: String,
    pub is_active: bool,
    pub is_promoted: bool,
}
