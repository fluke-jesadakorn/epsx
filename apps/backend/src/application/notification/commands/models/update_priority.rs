use crate::prelude::*;
use crate::application::shared::Command;

/// Command to update notification priority
#[derive(Debug, Clone)]
pub struct UpdateNotificationPriorityCommand {
    /// Notification ID to update
    pub notification_id: String,

    /// New priority level (urgent, critical, high, normal, low)
    pub new_priority: String,
}

impl Command for UpdateNotificationPriorityCommand {
    type Response = UpdateNotificationPriorityResponse;
}

/// Response returned after successfully updating notification priority
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateNotificationPriorityResponse {
    pub notification_id: String,
    pub old_priority: String,
    pub new_priority: String,
    pub updated_at: DateTime<Utc>,
}
