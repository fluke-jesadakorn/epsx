use crate::prelude::*;
use crate::application::shared::Query;

/// Query to get a single subscription
#[derive(Debug, Clone)]
pub struct GetSubscriptionQuery {
    pub subscription_id: String,
}

impl Query for GetSubscriptionQuery {
    type Response = GetSubscriptionResponse;
}

/// Response for get subscription query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetSubscriptionResponse {
    pub id: String,
    pub wallet_address: String,
    pub plan_id: i32,
    pub status: String,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
    pub auto_renew: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}
