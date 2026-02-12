use crate::prelude::*;
use crate::application::shared::Query;

/// Query to get a single permission plan
#[derive(Debug, Clone)]
pub struct GetPermissionPlanQuery {
    pub plan_id: String,
}

impl Query for GetPermissionPlanQuery {
    type Response = GetPermissionPlanResponse;
}

/// Response for get permission plan query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetPermissionPlanResponse {
    pub id: String,
    pub name: String,
    pub slug: String,
    pub description: String,
    pub plan_type: String,
    pub permissions: Vec<String>,
    pub price: f64,
    pub currency: String,
    pub billing_cycle: String,
    pub is_active: bool,
    pub is_promoted: bool,
    pub tier_level: i32,
    pub max_members: Option<i32>,
    pub auto_assign_enabled: bool,
    pub metadata: serde_json::Value,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub member_count: i64,
}
