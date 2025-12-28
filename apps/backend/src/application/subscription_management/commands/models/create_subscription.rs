use crate::prelude::*;
use crate::application::shared::Command;

/// Command to create a new subscription
#[derive(Debug, Clone)]
pub struct CreateSubscriptionCommand {
    pub wallet_address: String,
    pub plan_id: i32,
    pub payment_method_id: Option<String>,
}

impl Command for CreateSubscriptionCommand {
    type Response = CreateSubscriptionResponse;
}

/// Response for create subscription command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSubscriptionResponse {
    pub subscription_id: String,
    pub wallet_address: String,
    pub plan_id: i32,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
}
