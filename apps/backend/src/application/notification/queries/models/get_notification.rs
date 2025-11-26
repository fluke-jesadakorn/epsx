use crate::prelude::*;
use crate::application::shared::Query;

/// Query to get a single notification by ID
#[derive(Debug, Clone)]
pub struct GetNotificationQuery {
    pub notification_id: String,
}

impl Query for GetNotificationQuery {
    type Response = GetNotificationResponse;
}

/// Response containing notification details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetNotificationResponse {
    pub notification_id: String,
    pub recipient_type: String, // "user" or "topic"
    pub recipient_id: String,
    pub title: String,
    pub message: String,
    pub notification_type: String,
    pub priority: String,
    pub status: String,
    pub channels: Vec<String>,
    pub schedule_type: String,
    pub scheduled_at: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
    pub delivery_attempts: u32,
    pub last_delivery_attempt: Option<DateTime<Utc>>,
    pub delivered_at: Option<DateTime<Utc>>,
    pub image_url: Option<String>,
    pub action_url: Option<String>,
    pub tags: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
