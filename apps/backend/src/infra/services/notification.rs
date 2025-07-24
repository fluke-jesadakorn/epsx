use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::app::ports::services::NotificationServiceError;

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
    Trading,
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
    async fn mark_as_read(&self, notification_id: &str, user_id: &str) -> Result<(), NotificationServiceError>;
    async fn mark_all_as_read(&self, user_id: &str) -> Result<(), NotificationServiceError>;
    async fn delete_notification(&self, notification_id: &str, user_id: &str) -> Result<(), NotificationServiceError>;
    async fn get_unread_count(&self, user_id: &str) -> Result<usize, NotificationServiceError>;
}

/// In-memory notification service for development and testing
pub struct InMemoryNotificationService {
    notifications: std::sync::Arc<std::sync::Mutex<Vec<Notification>>>,
}

impl InMemoryNotificationService {
    pub fn new() -> Self {
        Self {
            notifications: std::sync::Arc::new(std::sync::Mutex::new(Vec::new())),
        }
    }

    pub fn clear(&self) {
        self.notifications.lock().unwrap().clear();
    }

    pub fn get_all_notifications(&self) -> Vec<Notification> {
        self.notifications.lock().unwrap().clone()
    }
}

#[async_trait]
impl NotificationService for InMemoryNotificationService {
    async fn send_notification(&self, notification: Notification) -> Result<(), NotificationServiceError> {
        let mut notifications = self.notifications.lock().unwrap();
        notifications.push(notification);
        Ok(())
    }

    async fn get_user_notifications(&self, user_id: &str, limit: Option<usize>) -> Result<Vec<Notification>, NotificationServiceError> {
        let notifications = self.notifications.lock().unwrap();
        let mut user_notifications: Vec<Notification> = notifications
            .iter()
            .filter(|n| n.user_id == user_id)
            .filter(|n| {
                // Filter out expired notifications
                if let Some(expires_at) = n.expires_at {
                    chrono::Utc::now() < expires_at
                } else {
                    true
                }
            })
            .cloned()
            .collect();

        // Sort by created_at descending (newest first)
        user_notifications.sort_by(|a, b| b.created_at.cmp(&a.created_at));

        if let Some(limit) = limit {
            user_notifications.truncate(limit);
        }

        Ok(user_notifications)
    }

    async fn mark_as_read(&self, notification_id: &str, user_id: &str) -> Result<(), NotificationServiceError> {
        let mut notifications = self.notifications.lock().unwrap();
        if let Some(notification) = notifications.iter_mut()
            .find(|n| n.id == notification_id && n.user_id == user_id) {
            notification.read = true;
            Ok(())
        } else {
            Err(NotificationServiceError::NotificationNotFound)
        }
    }

    async fn mark_all_as_read(&self, user_id: &str) -> Result<(), NotificationServiceError> {
        let mut notifications = self.notifications.lock().unwrap();
        for notification in notifications.iter_mut() {
            if notification.user_id == user_id {
                notification.read = true;
            }
        }
        Ok(())
    }

    async fn delete_notification(&self, notification_id: &str, user_id: &str) -> Result<(), NotificationServiceError> {
        let mut notifications = self.notifications.lock().unwrap();
        let original_len = notifications.len();
        notifications.retain(|n| !(n.id == notification_id && n.user_id == user_id));
        
        if notifications.len() < original_len {
            Ok(())
        } else {
            Err(NotificationServiceError::NotificationNotFound)
        }
    }

    async fn get_unread_count(&self, user_id: &str) -> Result<usize, NotificationServiceError> {
        let notifications = self.notifications.lock().unwrap();
        let count = notifications
            .iter()
            .filter(|n| n.user_id == user_id && !n.read)
            .filter(|n| {
                // Filter out expired notifications
                if let Some(expires_at) = n.expires_at {
                    chrono::Utc::now() < expires_at
                } else {
                    true
                }
            })
            .count();
        Ok(count)
    }
}

/// Database-backed notification service
pub struct DatabaseNotificationService {
    // This would contain database connection pool
    // For now, we'll use the in-memory service as a placeholder
    inner: InMemoryNotificationService,
}

impl DatabaseNotificationService {
    pub fn new() -> Self {
        Self {
            inner: InMemoryNotificationService::new(),
        }
    }
}

#[async_trait]
impl NotificationService for DatabaseNotificationService {
    async fn send_notification(&self, notification: Notification) -> Result<(), NotificationServiceError> {
        // TODO: Implement database storage
        // This would insert the notification into PostgreSQL
        self.inner.send_notification(notification).await
    }

    async fn get_user_notifications(&self, user_id: &str, limit: Option<usize>) -> Result<Vec<Notification>, NotificationServiceError> {
        // TODO: Implement database query
        self.inner.get_user_notifications(user_id, limit).await
    }

    async fn mark_as_read(&self, notification_id: &str, user_id: &str) -> Result<(), NotificationServiceError> {
        // TODO: Implement database update
        self.inner.mark_as_read(notification_id, user_id).await
    }

    async fn mark_all_as_read(&self, user_id: &str) -> Result<(), NotificationServiceError> {
        // TODO: Implement database update
        self.inner.mark_all_as_read(user_id).await
    }

    async fn delete_notification(&self, notification_id: &str, user_id: &str) -> Result<(), NotificationServiceError> {
        // TODO: Implement database delete
        self.inner.delete_notification(notification_id, user_id).await
    }

    async fn get_unread_count(&self, user_id: &str) -> Result<usize, NotificationServiceError> {
        // TODO: Implement database count query
        self.inner.get_unread_count(user_id).await
    }
}

/// Helper functions for creating common notifications
impl Notification {
    pub fn new_payment_notification(
        user_id: String,
        amount: rust_decimal::Decimal,
        currency: &str,
    ) -> Self {
        let mut metadata = HashMap::new();
        metadata.insert("amount".to_string(), amount.to_string());
        metadata.insert("currency".to_string(), currency.to_string());

        Self {
            id: uuid::Uuid::new_v4().to_string(),
            user_id,
            title: "Payment Confirmed".to_string(),
            message: format!("Your payment of {} {} has been confirmed.", amount, currency.to_uppercase()),
            notification_type: NotificationType::Payment,
            priority: NotificationPriority::High,
            read: false,
            created_at: chrono::Utc::now(),
            expires_at: Some(chrono::Utc::now() + chrono::Duration::days(30)),
            metadata,
        }
    }

    pub fn new_role_upgrade_notification(user_id: String, new_role: &str) -> Self {
        let mut metadata = HashMap::new();
        metadata.insert("new_role".to_string(), new_role.to_string());

        Self {
            id: uuid::Uuid::new_v4().to_string(),
            user_id,
            title: "Account Upgraded".to_string(),
            message: format!("Your account has been upgraded to {} tier.", new_role),
            notification_type: NotificationType::UserUpdate,
            priority: NotificationPriority::High,
            read: false,
            created_at: chrono::Utc::now(),
            expires_at: None, // Role upgrades don't expire
            metadata,
        }
    }

    pub fn new_security_notification(user_id: String, message: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            user_id,
            title: "Security Alert".to_string(),
            message,
            notification_type: NotificationType::Security,
            priority: NotificationPriority::Critical,
            read: false,
            created_at: chrono::Utc::now(),
            expires_at: Some(chrono::Utc::now() + chrono::Duration::days(7)),
            metadata: HashMap::new(),
        }
    }

    pub fn new_system_notification(user_id: String, title: String, message: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            user_id,
            title,
            message,
            notification_type: NotificationType::System,
            priority: NotificationPriority::Medium,
            read: false,
            created_at: chrono::Utc::now(),
            expires_at: Some(chrono::Utc::now() + chrono::Duration::days(14)),
            metadata: HashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_in_memory_notification_service() {
        let service = InMemoryNotificationService::new();
        let user_id = "user123";

        // Test sending notification
        let notification = Notification::new_payment_notification(
            user_id.to_string(),
            rust_decimal::Decimal::new(100, 0),
            "USD"
        );
        
        service.send_notification(notification.clone()).await.unwrap();

        // Test getting user notifications
        let notifications = service.get_user_notifications(user_id, None).await.unwrap();
        assert_eq!(notifications.len(), 1);
        assert_eq!(notifications[0].user_id, user_id);

        // Test unread count
        let unread_count = service.get_unread_count(user_id).await.unwrap();
        assert_eq!(unread_count, 1);

        // Test marking as read
        service.mark_as_read(&notification.id, user_id).await.unwrap();
        let unread_count = service.get_unread_count(user_id).await.unwrap();
        assert_eq!(unread_count, 0);

        // Test deleting notification
        service.delete_notification(&notification.id, user_id).await.unwrap();
        let notifications = service.get_user_notifications(user_id, None).await.unwrap();
        assert_eq!(notifications.len(), 0);
    }

    #[tokio::test]
    async fn test_notification_helpers() {
        let user_id = "user123".to_string();
        
        let payment_notif = Notification::new_payment_notification(
            user_id.clone(),
            rust_decimal::Decimal::new(10000, 2), // 100.00
            "usd"
        );
        
        assert_eq!(payment_notif.notification_type, NotificationType::Payment);
        assert_eq!(payment_notif.priority, NotificationPriority::High);
        assert!(payment_notif.message.contains("100"));
        assert!(payment_notif.message.contains("USD"));

        let role_notif = Notification::new_role_upgrade_notification(user_id.clone(), "premium");
        assert_eq!(role_notif.notification_type, NotificationType::UserUpdate);
        assert!(role_notif.message.contains("premium"));

        let security_notif = Notification::new_security_notification(
            user_id.clone(),
            "Suspicious login attempt detected".to_string()
        );
        assert_eq!(security_notif.notification_type, NotificationType::Security);
        assert_eq!(security_notif.priority, NotificationPriority::Critical);
    }
}