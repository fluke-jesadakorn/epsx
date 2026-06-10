use crate::prelude::*;
use crate::application::shared::Command;

/// Command to record a delivery attempt for a notification
#[derive(Debug, Clone)]
pub struct RecordDeliveryAttemptCommand {
    /// Notification ID
    pub notification_id: String,

    /// Delivery channel (wallet, web_push, in_app, websocket)
    pub channel: String,

    /// Delivery success status
    pub success: bool,

    /// Optional error message if failed
    pub error_message: Option<String>,

    /// Optional response details from delivery service
    pub response_details: Option<String>,
}

impl Command for RecordDeliveryAttemptCommand {
    type Response = RecordDeliveryAttemptResponse;
}

/// Response returned after recording delivery attempt
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordDeliveryAttemptResponse {
    pub notification_id: String,
    pub channel: String,
    pub success: bool,
    pub attempt_count: u32,
    pub recorded_at: DateTime<Utc>,
}
