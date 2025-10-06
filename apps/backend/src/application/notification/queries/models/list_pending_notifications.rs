use crate::prelude::*;
use crate::application::shared::Query;

/// Query to list pending notifications ready for delivery
#[derive(Debug, Clone)]
pub struct ListPendingNotificationsQuery {
    /// Maximum number of pending notifications to retrieve
    pub limit: u32,

    /// Include only notifications scheduled before this time
    pub before: Option<DateTime<Utc>>,
}

impl Query for ListPendingNotificationsQuery {
    type Response = ListPendingNotificationsResponse;
}

/// Response containing pending notifications
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListPendingNotificationsResponse {
    pub notifications: Vec<PendingNotificationDTO>,
    pub total: u32,
}

/// DTO for pending notification details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingNotificationDTO {
    pub notification_id: String,
    pub recipient_type: String,
    pub recipient_id: String,
    pub title: String,
    pub message: String,
    pub notification_type: String,
    pub priority: String,
    pub channels: Vec<String>,
    pub scheduled_at: DateTime<Utc>,
    pub delivery_attempts: u32,
    pub created_at: DateTime<Utc>,
}
