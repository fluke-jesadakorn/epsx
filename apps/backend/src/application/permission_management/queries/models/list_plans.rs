use crate::prelude::*;
use crate::application::shared::Query;

/// Query to list permission plans
#[derive(Debug, Clone)]
pub struct ListPermissionPlansQuery {
    pub plan_type: Option<String>,
    pub is_active: Option<bool>,
    pub is_promoted: Option<bool>,
    pub search_term: Option<String>,
    pub page: Option<u32>,
    pub limit: Option<u32>,
}

impl Query for ListPermissionPlansQuery {
    type Response = ListPermissionPlansResponse;
}

/// Response for list permission plans query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListPermissionPlansResponse {
    pub plans: Vec<PermissionPlanSummary>,
    pub total: i64,
    pub page: u32,
    pub limit: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionPlanSummary {
    pub id: String,
    pub name: String,
    pub slug: String,
    pub description: String,
    pub plan_type: String,
    pub permissions: Vec<String>,
    pub price: f64,
    pub currency: String,
    pub is_active: bool,
    pub is_promoted: bool,
    pub member_count: i64,
}
