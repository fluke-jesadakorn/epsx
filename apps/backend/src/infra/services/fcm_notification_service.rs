// ============================================================================
// FCM NOTIFICATION SERVICE
// ============================================================================
// Clean FCM-based notification service with push notification capabilities

use async_trait::async_trait;
use std::sync::Arc;
use std::collections::HashMap;
use tracing::{info, warn, error};

use crate::dom::ports::notification::{
    NotificationPort, DomainNotification, NotificationRecipient, 
    DomainNotificationPriority, NotificationStatus, NotificationError
};

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use crate::app::ports::services::NotificationServiceError;

use crate::infra::services::fcm_push_service::{FcmPushService, FcmMessage, FcmPriority};
use crate::infra::services::fcm_token_service::FcmTokenService;
use crate::dom::values::UserId;

// ============================================================================
// NOTIFICATION SERVICE TYPES 
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    pub id: String,
    pub user_id: String,
    pub title: String,
    pub message: String,
    pub notification_type: NotificationType,
    pub priority: NotificationPriority,
    pub read: bool,
    pub delivery_status: NotificationDeliveryStatus,
    pub delivered_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub context_data: HashMap<String, serde_json::Value>,
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
    FeatureExpiration,
    ModuleAccess,
    QuotaWarning,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum NotificationPriority {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum NotificationDeliveryStatus {
    Pending,
    Sent,
    Delivered,
    Failed,
    Expired,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationPreferences {
    pub email_enabled: bool,
    pub push_enabled: bool,
    pub timezone: String,
}

#[derive(Debug, Clone, Default)]
pub struct NotificationQuery {
    pub user_id: Option<String>,
    pub types: Option<Vec<NotificationType>>,
    pub priorities: Option<Vec<NotificationPriority>>,
    pub is_read: Option<bool>,
    pub created_after: Option<DateTime<Utc>>,
    pub created_before: Option<DateTime<Utc>>,
    pub limit: Option<i32>,
    pub offset: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceNotificationStats {
    pub total_notifications: i64,
    pub unread_count: i64,
    pub critical_count: i64,
    pub today_count: i64,
    pub last_notification_at: Option<DateTime<Utc>>,
}

#[async_trait]
pub trait NotificationService: Send + Sync {
    async fn send_notification(&self, notification: Notification) -> Result<String, NotificationServiceError>;
    async fn send_bulk_notifications(&self, notifications: Vec<Notification>) -> Result<Vec<String>, NotificationServiceError>;
    async fn get_user_notifications(&self, query: &NotificationQuery) -> Result<Vec<Notification>, NotificationServiceError>;
    async fn get_notification_by_id(&self, id: &str, user_id: &str) -> Result<Option<Notification>, NotificationServiceError>;
    async fn get_user_stats(&self, user_id: &str) -> Result<ServiceNotificationStats, NotificationServiceError>;
    async fn get_unread_count(&self, user_id: &str) -> Result<i64, NotificationServiceError>;
    async fn mark_notification_read(&self, user_id: &str, notification_id: &str) -> Result<(), NotificationServiceError>;
    async fn mark_all_notifications_read(&self, user_id: &str) -> Result<i64, NotificationServiceError>;
    async fn update_delivery_status(&self, notification_id: &str, status: NotificationDeliveryStatus) -> Result<(), NotificationServiceError>;
    async fn delete_notification(&self, user_id: &str, notification_id: &str) -> Result<(), NotificationServiceError>;
    async fn get_user_preferences(&self, user_id: &str) -> Result<Option<NotificationPreferences>, NotificationServiceError>;
    async fn update_user_preferences(&self, user_id: &str, preferences: &NotificationPreferences) -> Result<(), NotificationServiceError>;
    async fn deliver_real_time(&self, user_id: &str, notification: &Notification) -> Result<bool, NotificationServiceError>;
    async fn send_templated_notification(&self, template_id: &str, user_id: &str, context: HashMap<String, serde_json::Value>) -> Result<String, NotificationServiceError>;
    async fn process_pending_notifications(&self, limit: i32) -> Result<i32, NotificationServiceError>;
    async fn cleanup_expired_notifications(&self) -> Result<i64, NotificationServiceError>;
}

// ============================================================================
// FCM NOTIFICATION SERVICE
// ============================================================================

/// Clean FCM-based notification service
pub struct FcmNotificationService {
    fcm_push_service: Arc<dyn FcmPushService>,
    fcm_token_service: Arc<dyn FcmTokenService>,
    fcm_enabled: bool,
}

impl FcmNotificationService {
    pub fn new(
        fcm_push_service: Arc<dyn FcmPushService>,
        fcm_token_service: Arc<dyn FcmTokenService>,
    ) -> Self {
        Self {
            fcm_push_service,
            fcm_token_service,
            fcm_enabled: true,
        }
    }

    pub fn with_fcm_disabled(mut self) -> Self {
        self.fcm_enabled = false;
        self
    }

    /// Convert domain notification to FCM message
    fn domain_to_fcm_message(&self, notification: &DomainNotification) -> FcmMessage {
        let priority = match notification.priority {
            DomainNotificationPriority::Low => FcmPriority::Normal,
            DomainNotificationPriority::Normal => FcmPriority::Normal,
            DomainNotificationPriority::High => FcmPriority::High,
            DomainNotificationPriority::Critical => FcmPriority::High,
        };

        let mut message = FcmMessage::simple_notification(notification.title.clone(), notification.message.clone());
        message.priority = priority;
        
        // Add notification data
        if let Some(data) = &notification.data {
            if let Some(obj) = data.as_object() {
                for (key, value) in obj {
                    if message.data.is_none() {
                        message.data = Some(HashMap::new());
                    }
                    if let Some(ref mut data_map) = message.data {
                        data_map.insert(key.clone(), value.as_str().unwrap_or("").to_string());
                    }
                }
            }
        }

        message
    }

    /// Convert service notification to FCM message
    fn service_to_fcm_message(&self, notification: &Notification) -> FcmMessage {
        let priority = match notification.priority {
            NotificationPriority::Low => FcmPriority::Normal,
            NotificationPriority::Medium => FcmPriority::Normal,
            NotificationPriority::High => FcmPriority::High,
            NotificationPriority::Critical => FcmPriority::High,
        };

        let mut message = FcmMessage::simple_notification(notification.title.clone(), notification.message.clone());
        message.priority = priority;
        
        // Add context data
        for (key, value) in &notification.context_data {
            if message.data.is_none() {
                message.data = Some(HashMap::new());
            }
            if let Some(ref mut data) = message.data {
                data.insert(key.clone(), value.as_str().unwrap_or("").to_string());
            }
        }

        message
    }
}

#[async_trait]
impl NotificationService for FcmNotificationService {
    async fn send_notification(&self, notification: Notification) -> Result<String, NotificationServiceError> {
        info!("FCM: Sending notification to user {}", notification.user_id);

        if !self.fcm_enabled {
            info!("FCM disabled - notification logged only");
            return Ok(notification.id);
        }

        let user_id = UserId(Uuid::parse_str(&notification.user_id)
            .map_err(|e| NotificationServiceError::InvalidRequest(format!("Invalid user ID: {}", e)))?);
        
        let fcm_message = self.service_to_fcm_message(&notification);
        
        match self.fcm_push_service.send_to_user(&user_id, &fcm_message, None).await {
            Ok(result) => {
                info!("FCM notification sent to {}/{} devices for user {}", 
                    result.successful, result.total_sent, notification.user_id);
                Ok(notification.id)
            }
            Err(e) => {
                error!("Failed to send FCM notification to user {}: {}", notification.user_id, e);
                Err(NotificationServiceError::SendFailed(e.to_string()))
            }
        }
    }

    async fn send_bulk_notifications(&self, notifications: Vec<Notification>) -> Result<Vec<String>, NotificationServiceError> {
        info!("FCM: Sending {} notifications", notifications.len());
        
        let mut notification_ids = Vec::new();
        for notification in notifications {
            match NotificationService::send_notification(self, notification.clone()).await {
                Ok(id) => notification_ids.push(id),
                Err(e) => {
                    warn!("Failed to send notification {}: {}", notification.id, e);
                    // Continue with other notifications
                }
            }
        }
        
        Ok(notification_ids)
    }

    async fn get_user_notifications(&self, query: &NotificationQuery) -> Result<Vec<Notification>, NotificationServiceError> {
        info!("FCM: Getting notifications for user {:?}", query.user_id);
        // For FCM-only service, we don't store notifications locally
        Ok(vec![])
    }

    async fn get_notification_by_id(&self, id: &str, user_id: &str) -> Result<Option<Notification>, NotificationServiceError> {
        info!("FCM: Getting notification {} for user {}", id, user_id);
        // For FCM-only service, we don't store notifications locally
        Ok(None)
    }

    async fn get_user_stats(&self, user_id: &str) -> Result<ServiceNotificationStats, NotificationServiceError> {
        info!("FCM: Getting stats for user {}", user_id);
        // For FCM-only service, return basic stats
        Ok(ServiceNotificationStats {
            total_notifications: 0,
            unread_count: 0,
            critical_count: 0,
            today_count: 0,
            last_notification_at: None,
        })
    }

    async fn get_unread_count(&self, user_id: &str) -> Result<i64, NotificationServiceError> {
        info!("FCM: Getting unread count for user {}", user_id);
        Ok(0)
    }

    async fn mark_notification_read(&self, user_id: &str, notification_id: &str) -> Result<(), NotificationServiceError> {
        info!("FCM: Marking notification {} as read for user {}", notification_id, user_id);
        Ok(())
    }

    async fn mark_all_notifications_read(&self, user_id: &str) -> Result<i64, NotificationServiceError> {
        info!("FCM: Marking all notifications as read for user {}", user_id);
        Ok(0)
    }

    async fn update_delivery_status(&self, notification_id: &str, _status: NotificationDeliveryStatus) -> Result<(), NotificationServiceError> {
        info!("FCM: Updating delivery status for notification {}", notification_id);
        Ok(())
    }

    async fn delete_notification(&self, user_id: &str, notification_id: &str) -> Result<(), NotificationServiceError> {
        info!("FCM: Deleting notification {} for user {}", notification_id, user_id);
        Ok(())
    }

    async fn get_user_preferences(&self, user_id: &str) -> Result<Option<NotificationPreferences>, NotificationServiceError> {
        info!("FCM: Getting preferences for user {}", user_id);
        Ok(Some(NotificationPreferences {
            email_enabled: false,
            push_enabled: true,
            timezone: "UTC".to_string(),
        }))
    }

    async fn update_user_preferences(&self, user_id: &str, _preferences: &NotificationPreferences) -> Result<(), NotificationServiceError> {
        info!("FCM: Updating preferences for user {}", user_id);
        Ok(())
    }

    async fn deliver_real_time(&self, user_id: &str, notification: &Notification) -> Result<bool, NotificationServiceError> {
        info!("FCM: Real-time delivery to user {}", user_id);
        
        if !self.fcm_enabled {
            return Ok(false);
        }

        let user_id_parsed = UserId(Uuid::parse_str(user_id)
            .map_err(|e| NotificationServiceError::InvalidRequest(format!("Invalid user ID: {}", e)))?);
        
        let fcm_message = self.service_to_fcm_message(notification);
        
        match self.fcm_push_service.send_to_user(&user_id_parsed, &fcm_message, None).await {
            Ok(result) => Ok(result.successful > 0),
            Err(_) => Ok(false),
        }
    }

    async fn send_templated_notification(&self, template_id: &str, user_id: &str, _context: HashMap<String, serde_json::Value>) -> Result<String, NotificationServiceError> {
        info!("FCM: Sending templated notification {} to user {}", template_id, user_id);
        // Template functionality would be implemented here
        Ok(Uuid::new_v4().to_string())
    }

    async fn process_pending_notifications(&self, limit: i32) -> Result<i32, NotificationServiceError> {
        info!("FCM: Processing {} pending notifications", limit);
        Ok(0)
    }

    async fn cleanup_expired_notifications(&self) -> Result<i64, NotificationServiceError> {
        info!("FCM: Cleaning up expired notifications");
        Ok(0)
    }
}

#[async_trait]
impl NotificationPort for FcmNotificationService {
    async fn send_notification(&self, notification: DomainNotification) -> Result<(), NotificationError> {
        if !self.fcm_enabled {
            info!("FCM disabled - domain notification logged only");
            return Ok(());
        }

        let user_id = match &notification.recipient {
            NotificationRecipient::User(id) => id,
            _ => return Err(NotificationError::InvalidRecipient("Only user notifications supported".to_string())),
        };
        
        let fcm_message = self.domain_to_fcm_message(&notification);
        
        match self.fcm_push_service.send_to_user(user_id, &fcm_message, None).await {
            Ok(result) => {
                info!("FCM domain notification sent to {}/{} devices for user {}", 
                    result.successful, result.total_sent, user_id);
                Ok(())
            }
            Err(e) => {
                error!("Failed to send FCM domain notification to user {}: {}", user_id, e);
                Err(NotificationError::SendFailed(e.to_string()))
            }
        }
    }

    async fn send_bulk_notifications(&self, notifications: Vec<DomainNotification>) -> Result<(), NotificationError> {
        for notification in notifications {
            if let Err(e) = NotificationPort::send_notification(self, notification).await {
                warn!("Failed to send bulk domain notification: {}", e);
                // Continue with other notifications
            }
        }
        Ok(())
    }

    async fn get_notification_status(&self, notification_id: &str) -> Result<NotificationStatus, NotificationError> {
        info!("FCM: Getting status for notification {}", notification_id);
        Ok(NotificationStatus::Sent)
    }
}