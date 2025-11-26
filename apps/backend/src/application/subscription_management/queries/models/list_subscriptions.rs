use crate::prelude::*;
use crate::application::shared::Query;

/// Query to list subscriptions
#[derive(Debug, Clone)]
pub struct ListSubscriptionsQuery {
    pub wallet_address: Option<String>,
    pub plan_id: Option<i32>,
    pub is_active: Option<bool>,
    pub page: Option<u32>,
    pub limit: Option<u32>,
}

impl Query for ListSubscriptionsQuery {
    type Response = ListSubscriptionsResponse;
}

/// Response for list subscriptions query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListSubscriptionsResponse {
    pub subscriptions: Vec<SubscriptionSummary>,
    pub total: i64,
    pub page: u32,
    pub limit: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionSummary {
    pub id: String,
    pub wallet_address: String,
    pub plan_id: i32,
    pub plan_name: String,
    pub status: String,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
    pub is_active: bool,
}
