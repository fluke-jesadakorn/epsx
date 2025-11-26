use crate::prelude::*;
use crate::application::shared::Command;

/// Command to cancel a notification
#[derive(Debug, Clone)]
pub struct CancelNotificationCommand {
    /// Notification ID to cancel
    pub notification_id: String,

    /// Reason for cancellation
    pub reason: String,
}

impl Command for CancelNotificationCommand {
    type Response = CancelNotificationResponse;
}

/// Response returned after successfully cancelling a notification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CancelNotificationResponse {
    pub notification_id: String,
    pub status: String,
    pub reason: String,
    pub cancelled_at: DateTime<Utc>,
}
