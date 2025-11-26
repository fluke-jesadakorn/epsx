use crate::prelude::*;
use crate::application::shared::Query;

/// Query to get groups assigned to a wallet
#[derive(Debug, Clone)]
pub struct GetWalletGroupsQuery {
    pub wallet_address: String,
}

impl Query for GetWalletGroupsQuery {
    type Response = GetWalletGroupsResponse;
}

/// Response for get wallet groups query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetWalletGroupsResponse {
    pub wallet_address: String,
    pub groups: Vec<WalletGroupInfo>,
    pub total: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletGroupInfo {
    pub group_id: String,
    pub group_name: String,
    pub group_slug: String,
    pub permissions: Vec<String>,
    pub assigned_at: chrono::DateTime<chrono::Utc>,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
    pub is_active: bool,
}
