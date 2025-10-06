use crate::prelude::*;
use crate::application::shared::Query;

/// Query to get members of a permission group
#[derive(Debug, Clone)]
pub struct GetGroupMembersQuery {
    pub group_id: String,
}

impl Query for GetGroupMembersQuery {
    type Response = GetGroupMembersResponse;
}

/// Response for get group members query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetGroupMembersResponse {
    pub group_id: String,
    pub members: Vec<GroupMemberInfo>,
    pub total: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupMemberInfo {
    pub wallet_address: String,
    pub assigned_at: chrono::DateTime<chrono::Utc>,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
    pub is_active: bool,
}
