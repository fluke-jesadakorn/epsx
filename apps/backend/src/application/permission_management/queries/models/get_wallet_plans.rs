use crate::prelude::*;
use crate::application::shared::Query;

/// Query to get plans assigned to a wallet
#[derive(Debug, Clone)]
pub struct GetWalletPlansQuery {
    pub wallet_address: String,
}

impl Query for GetWalletPlansQuery {
    type Response = GetWalletPlansResponse;
}

/// Response for get wallet plans query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetWalletPlansResponse {
    pub wallet_address: String,
    pub plans: Vec<WalletPlanInfo>,
    pub total: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletPlanInfo {
    pub plan_id: String,
    pub plan_name: String,
    pub plan_slug: String,
    pub permissions: Vec<String>,
    pub assigned_at: chrono::DateTime<chrono::Utc>,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
    pub is_active: bool,
}
