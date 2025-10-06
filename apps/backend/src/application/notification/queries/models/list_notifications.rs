use crate::prelude::*;
use crate::application::shared::Query;

/// Query to list notifications with filters
#[derive(Debug, Clone)]
pub struct ListNotificationsQuery {
    /// Filter by wallet address (optional)
    pub wallet_address: Option<String>,

    /// Filter by topic name (optional)
    pub topic: Option<String>,

    /// Filter by status (optional)
    pub status: Option<String>,

    /// Filter by notification type (optional)
    pub notification_type: Option<String>,

    /// Filter by priority (optional)
    pub priority: Option<String>,

    /// Pagination offset
    pub offset: u32,

    /// Pagination limit
    pub limit: u32,
}

impl Query for ListNotificationsQuery {
    type Response = ListNotificationsResponse;
}

/// Response containing list of notifications
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListNotificationsResponse {
    pub notifications: Vec<NotificationSummaryDTO>,
    pub total: u32,
    pub offset: u32,
    pub limit: u32,
}

/// Summary DTO for notification listing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationSummaryDTO {
    pub notification_id: String,
    pub recipient_type: String,
    pub recipient_id: String,
    pub title: String,
    pub notification_type: String,
    pub priority: String,
    pub status: String,
    pub scheduled_at: Option<DateTime<Utc>>,
    pub delivery_attempts: u32,
    pub created_at: DateTime<Utc>,
}
