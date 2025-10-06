use crate::prelude::*;
use crate::application::shared::Command;

/// Command to create a notification for a topic (broadcast)
#[derive(Debug, Clone)]
pub struct CreateTopicNotificationCommand {
    /// Topic identifier
    pub topic: String,

    /// Notification title
    pub title: String,

    /// Notification message body
    pub message: String,

    /// Notification type
    pub notification_type: String,

    /// Priority level (urgent, critical, high, normal, low)
    pub priority: String,

    /// Delivery channels (comma-separated: wallet,web_push,in_app,websocket)
    pub channels: Vec<String>,

    /// Schedule type (immediate, scheduled, delayed)
    pub schedule_type: Option<String>,

    /// Scheduled delivery time (ISO 8601)
    pub scheduled_at: Option<DateTime<Utc>>,

    /// Expiry time (ISO 8601)
    pub expires_at: Option<DateTime<Utc>>,

    /// Optional image URL
    pub image_url: Option<String>,

    /// Optional action URL
    pub action_url: Option<String>,

    /// Optional metadata tags
    pub tags: Option<Vec<String>>,
}

impl Command for CreateTopicNotificationCommand {
    type Response = CreateTopicNotificationResponse;
}

/// Response returned after successfully creating a topic notification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTopicNotificationResponse {
    pub notification_id: String,
    pub topic: String,
    pub status: String,
    pub scheduled_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}
