use crate::prelude::*;
use crate::application::shared::Command;

/// Command to cancel a subscription
#[derive(Debug, Clone)]
pub struct CancelSubscriptionCommand {
    pub subscription_id: String,
}

impl Command for CancelSubscriptionCommand {
    type Response = CancelSubscriptionResponse;
}

/// Response for cancel subscription command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CancelSubscriptionResponse {
    pub subscription_id: String,
    pub cancelled_at: chrono::DateTime<chrono::Utc>,
}
