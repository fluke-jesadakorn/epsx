use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;
use tracing::info;

use crate::app::ports::services::NotificationServiceError;
use crate::dom::ports::notification::{
    NotificationPort, DomainNotification, NotificationRecipient, 
    DomainNotificationType, DomainNotificationPriority, NotificationStatus, NotificationError
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    pub id: String,
    pub user_id: String,
    pub title: String,
    pub message: String,
    pub notification_type: NotificationType,
    pub priority: NotificationPriority,
    pub read: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum NotificationType {
    System,
    Payment,
    Analytics,
    Security,
    Marketing,
    UserUpdate,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum NotificationPriority {
    Low,
    Medium,
    High,
    Critical,
}

#[async_trait]
pub trait NotificationService: Send + Sync {
    async fn send_notification(&self, notification: Notification) -> Result<(), NotificationServiceError>;
    async fn get_user_notifications(&self, user_id: &str, limit: Option<usize>) -> Result<Vec<Notification>, NotificationServiceError>;
    async fn mark_notification_read(&self, user_id: &str, notification_id: &str) -> Result<(), NotificationServiceError>;
    async fn mark_all_notifications_read(&self, user_id: &str) -> Result<(), NotificationServiceError>;
    async fn delete_notification(&self, user_id: &str, notification_id: &str) -> Result<(), NotificationServiceError>;
    async fn get_notification_count(&self, user_id: &str) -> Result<i64, NotificationServiceError>;
}

// In-memory notification service for now (can be replaced with proper database implementation later)
pub struct InMemoryNotificationService {
    // For simplicity, we'll just log notifications for now
    // In a real implementation, this would use a proper storage backend
}

impl InMemoryNotificationService {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl NotificationService for InMemoryNotificationService {
    async fn send_notification(&self, notification: Notification) -> Result<(), NotificationServiceError> {
        // For now, just log the notification
        info!(
            "Sending notification to user {}: {} - {}",
            notification.user_id, notification.title, notification.message
        );
        Ok(())
    }

    async fn get_user_notifications(&self, user_id: &str, _limit: Option<usize>) -> Result<Vec<Notification>, NotificationServiceError> {
        // Return empty list for now
        info!("Getting notifications for user {}", user_id);
        Ok(vec![])
    }

    async fn mark_notification_read(&self, user_id: &str, notification_id: &str) -> Result<(), NotificationServiceError> {
        info!("Marking notification {} as read for user {}", notification_id, user_id);
        Ok(())
    }

    async fn mark_all_notifications_read(&self, user_id: &str) -> Result<(), NotificationServiceError> {
        info!("Marking all notifications as read for user {}", user_id);
        Ok(())
    }

    async fn delete_notification(&self, user_id: &str, notification_id: &str) -> Result<(), NotificationServiceError> {
        info!("Deleting notification {} for user {}", notification_id, user_id);
        Ok(())
    }

    async fn get_notification_count(&self, user_id: &str) -> Result<i64, NotificationServiceError> {
        info!("Getting notification count for user {}", user_id);
        Ok(0)
    }
}

// Domain notification implementation
#[async_trait]
impl NotificationPort for InMemoryNotificationService {
    async fn send_notification(&self, notification: DomainNotification) -> Result<(), NotificationError> {
        let user_id = match &notification.recipient {
            NotificationRecipient::User(id) => id.to_string(),
            NotificationRecipient::Email(email) => email.clone(),
            NotificationRecipient::AdminGroup => "admin_group".to_string(),
            NotificationRecipient::Broadcast => "broadcast".to_string(),
        };

        let notification_type = match notification.notification_type {
            DomainNotificationType::FeatureExpiration => NotificationType::System,
            DomainNotificationType::ModuleAccessChanged => NotificationType::UserUpdate,
            DomainNotificationType::QuotaWarning => NotificationType::System,
            DomainNotificationType::SecurityAlert => NotificationType::Security,
            DomainNotificationType::SystemMaintenance => NotificationType::System,
            DomainNotificationType::AccountUpdate => NotificationType::UserUpdate,
            DomainNotificationType::PaymentNotification => NotificationType::Payment,
        };

        let priority = match notification.priority {
            DomainNotificationPriority::Low => NotificationPriority::Low,
            DomainNotificationPriority::Normal => NotificationPriority::Medium,
            DomainNotificationPriority::High => NotificationPriority::High,
            DomainNotificationPriority::Critical => NotificationPriority::Critical,
        };

        let infra_notification = Notification {
            id: Uuid::new_v4().to_string(),
            user_id,
            title: notification.title,
            message: notification.message,
            notification_type,
            priority,
            read: false,
            created_at: chrono::Utc::now(),
            expires_at: notification.expires_at,
            metadata: HashMap::new(),
        };

        <Self as NotificationService>::send_notification(self, infra_notification)
            .await
            .map_err(|e| NotificationError::SendFailed(e.to_string()))?;

        Ok(())
    }

    async fn send_bulk_notifications(&self, notifications: Vec<DomainNotification>) -> Result<(), NotificationError> {
        for notification in notifications {
            <Self as NotificationPort>::send_notification(self, notification).await?;
        }
        Ok(())
    }

    async fn get_notification_status(&self, notification_id: &str) -> Result<NotificationStatus, NotificationError> {
        info!("Getting status for notification {}", notification_id);
        Ok(NotificationStatus::Sent)
    }
}

/// Adapter to bridge NotificationService to NotificationPort
pub struct NotificationPortAdapter {
    service: Arc<dyn NotificationService>,
}

impl NotificationPortAdapter {
    pub fn new(service: Arc<dyn NotificationService>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl NotificationPort for NotificationPortAdapter {
    async fn send_notification(&self, notification: DomainNotification) -> Result<(), NotificationError> {
        let user_id = match &notification.recipient {
            NotificationRecipient::User(id) => id.to_string(),
            NotificationRecipient::Email(email) => email.clone(),
            NotificationRecipient::AdminGroup => "admin_group".to_string(),
            NotificationRecipient::Broadcast => "broadcast".to_string(),
        };

        let notification_type = match notification.notification_type {
            DomainNotificationType::FeatureExpiration => NotificationType::System,
            DomainNotificationType::ModuleAccessChanged => NotificationType::UserUpdate,
            DomainNotificationType::QuotaWarning => NotificationType::System,
            DomainNotificationType::SecurityAlert => NotificationType::Security,
            DomainNotificationType::SystemMaintenance => NotificationType::System,
            DomainNotificationType::AccountUpdate => NotificationType::UserUpdate,
            DomainNotificationType::PaymentNotification => NotificationType::Payment,
        };

        let priority = match notification.priority {
            DomainNotificationPriority::Low => NotificationPriority::Low,
            DomainNotificationPriority::Normal => NotificationPriority::Medium,
            DomainNotificationPriority::High => NotificationPriority::High,
            DomainNotificationPriority::Critical => NotificationPriority::Critical,
        };

        let infra_notification = Notification {
            id: Uuid::new_v4().to_string(),
            user_id,
            title: notification.title,
            message: notification.message,
            notification_type,
            priority,
            read: false,
            created_at: chrono::Utc::now(),
            expires_at: notification.expires_at,
            metadata: HashMap::new(),
        };

        self.service.send_notification(infra_notification)
            .await
            .map_err(|e| NotificationError::SendFailed(e.to_string()))?;

        Ok(())
    }

    async fn send_bulk_notifications(&self, notifications: Vec<DomainNotification>) -> Result<(), NotificationError> {
        for notification in notifications {
            self.send_notification(notification).await?;
        }
        Ok(())
    }

    async fn get_notification_status(&self, notification_id: &str) -> Result<NotificationStatus, NotificationError> {
        info!("Getting status for notification {}", notification_id);
        Ok(NotificationStatus::Sent)
    }
}

