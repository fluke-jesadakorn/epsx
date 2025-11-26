use crate::prelude::*;
use crate::domain::notification::{Notification, NotificationStatus};
use crate::domain::notification::value_objects::user_preferences::NotificationType;
use uuid::Uuid;

/// Search criteria for notifications
#[derive(Debug, Clone, Default)]
pub struct NotificationSearchCriteria {
    pub recipient_wallet_address: Option<Uuid>,
    pub topic: Option<String>,
    pub status: Option<NotificationStatus>,
    pub notification_type: Option<NotificationType>,
    pub priority: Option<String>,
    pub created_after: Option<DateTime<Utc>>,
    pub created_before: Option<DateTime<Utc>>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// Repository port for notification operations
#[async_trait]
pub trait NotificationRepositoryPort: Send + Sync {
    /// Find notification by ID
    async fn find_by_id(&self, notification_id: &str) -> AppResult<Option<Notification>>;

    /// List notifications with optional filtering
    async fn find_all(&self, criteria: NotificationSearchCriteria) -> AppResult<Vec<Notification>>;

    /// Save (create or update) a notification
    async fn save(&self, notification: &Notification) -> AppResult<()>;

    /// Delete a notification
    async fn delete(&self, notification_id: &str) -> AppResult<()>;

    /// Count notifications matching criteria
    async fn count(&self, criteria: NotificationSearchCriteria) -> AppResult<i64>;

    /// Check if notification ID exists
    async fn notification_exists(&self, notification_id: &str) -> AppResult<bool>;

    /// Find pending notifications (for processing)
    async fn find_pending(&self, limit: u32) -> AppResult<Vec<Notification>>;

    /// Find notifications by wallet address
    async fn find_by_wallet(&self, wallet_address: Uuid) -> AppResult<Vec<Notification>>;

    /// Find notifications by topic
    async fn find_by_topic(&self, topic: &str) -> AppResult<Vec<Notification>>;

    /// Find notifications by status
    async fn find_by_status(&self, status: NotificationStatus) -> AppResult<Vec<Notification>>;

    /// Find expired notifications
    async fn find_expired(&self) -> AppResult<Vec<Notification>>;
}
