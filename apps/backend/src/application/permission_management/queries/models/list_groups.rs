use crate::prelude::*;
use crate::application::shared::Query;

/// Query to list permission groups
#[derive(Debug, Clone)]
pub struct ListPermissionGroupsQuery {
    pub group_type: Option<String>,
    pub is_active: Option<bool>,
    pub is_promoted: Option<bool>,
    pub search_term: Option<String>,
    pub page: Option<u32>,
    pub limit: Option<u32>,
}

impl Query for ListPermissionGroupsQuery {
    type Response = ListPermissionGroupsResponse;
}

/// Response for list permission groups query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListPermissionGroupsResponse {
    pub groups: Vec<PermissionGroupSummary>,
    pub total: i64,
    pub page: u32,
    pub limit: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionGroupSummary {
    pub id: String,
    pub name: String,
    pub slug: String,
    pub description: String,
    pub group_type: String,
    pub permissions: Vec<String>,
    pub price: f64,
    pub currency: String,
    pub is_active: bool,
    pub is_promoted: bool,
    pub member_count: i64,
}
