use crate::prelude::*;
use crate::application::shared::Query;

/// Query to get members of a permission plan
#[derive(Debug, Clone)]
pub struct GetPlanMembersQuery {
    pub plan_id: String,
}

impl Query for GetPlanMembersQuery {
    type Response = GetPlanMembersResponse;
}

/// Response for get plan members query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetPlanMembersResponse {
    pub plan_id: String,
    pub members: Vec<PlanMemberInfo>,
    pub total: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanMemberInfo {
    pub wallet_address: String,
    pub assigned_at: chrono::DateTime<chrono::Utc>,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
    pub is_active: bool,
}
