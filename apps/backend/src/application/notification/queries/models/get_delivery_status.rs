use crate::prelude::*;
use crate::application::shared::Query;

/// Query to get delivery status for a notification
#[derive(Debug, Clone)]
pub struct GetDeliveryStatusQuery {
    pub notification_id: String,
}

impl Query for GetDeliveryStatusQuery {
    type Response = GetDeliveryStatusResponse;
}

/// Response containing delivery status details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetDeliveryStatusResponse {
    pub notification_id: String,
    pub status: String,
    pub delivery_attempts: u32,
    pub last_attempt_at: Option<DateTime<Utc>>,
    pub delivered_at: Option<DateTime<Utc>>,
    pub channels: Vec<ChannelDeliveryStatus>,
    pub is_expired: bool,
    pub expires_at: Option<DateTime<Utc>>,
}

/// Delivery status per channel
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelDeliveryStatus {
    pub channel: String,
    pub delivered: bool,
    pub attempts: u32,
    pub last_attempt_at: Option<DateTime<Utc>>,
    pub last_error: Option<String>,
}
