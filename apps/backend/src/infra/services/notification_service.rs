use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use serde::{Deserialize, Serialize};

use std::collections::HashMap;

use std::sync::Arc;

use tracing::warn;

use tracing::{info, debug};



use crate::app::ports::services::NotificationServiceError;

use crate::dom::ports::notification::{

    NotificationPort, DomainNotification, NotificationRecipient, 
    DomainNotificationType, DomainNotificationPriority, NotificationStatus, NotificationError
};
use crate::infra::db::diesel::{

    models::{DieselNotification},
};
use crate::infra::cache::Cache;


/// Enhanced notification with full feature support
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    pub id: String,
    pub user_id: String,
    pub user_firebase_uid: Option<String>,
    pub title: String,
    pub message: String,
    pub notification_type: NotificationType,
    pub priority: NotificationPriority,
    pub read: bool,
    pub delivery_status: NotificationDeliveryStatus,
    pub delivered_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub action_url: Option<String>,
    pub action_text: Option<String>,
    pub template_id: Option<String>,
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

/// User notification preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationPreferences {
    pub email_enabled: bool,
    pub push_enabled: bool,
    pub digest_mode: bool,
    pub digest_frequency: String,
    pub quiet_hours_start: Option<chrono::NaiveTime>,
    pub quiet_hours_end: Option<chrono::NaiveTime>,
    pub timezone: String,
    pub type_preferences: HashMap<String, TypePreference>,
    pub max_notifications_per_hour: i32,
    pub max_notifications_per_day: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypePreference {
    pub email: bool,
    pub push: bool,
}

/// Notification query parameters
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

/// Notification statistics for a user
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
    // Basic notification operations
    async fn send_notification(&self, notification: Notification) -> Result<String, NotificationServiceError>;
    async fn send_bulk_notifications(&self, notifications: Vec<Notification>) -> Result<Vec<String>, NotificationServiceError>;
    
    // Query operations  
    async fn get_user_notifications(&self, query: &NotificationQuery) -> Result<Vec<Notification>, NotificationServiceError>;
    async fn get_notification_by_id(&self, id: &str, user_id: &str) -> Result<Option<Notification>, NotificationServiceError>;
    async fn get_user_stats(&self, user_id: &str) -> Result<ServiceNotificationStats, NotificationServiceError>;
    async fn get_unread_count(&self, user_id: &str) -> Result<i64, NotificationServiceError>;
    
    // Update operations
    async fn mark_notification_read(&self, user_id: &str, notification_id: &str) -> Result<(), NotificationServiceError>;
    async fn mark_all_notifications_read(&self, user_id: &str) -> Result<i64, NotificationServiceError>;
    async fn update_delivery_status(&self, notification_id: &str, status: NotificationDeliveryStatus) -> Result<(), NotificationServiceError>;
    async fn delete_notification(&self, user_id: &str, notification_id: &str) -> Result<(), NotificationServiceError>;
    
    // Preferences
    async fn get_user_preferences(&self, user_id: &str) -> Result<Option<NotificationPreferences>, NotificationServiceError>;
    async fn update_user_preferences(&self, user_id: &str, preferences: &NotificationPreferences) -> Result<(), NotificationServiceError>;
    
    // Real-time delivery
    async fn deliver_real_time(&self, user_id: &str, notification: &Notification) -> Result<bool, NotificationServiceError>;
    
    // Template support
    async fn send_templated_notification(&self, template_id: &str, user_id: &str, context: HashMap<String, serde_json::Value>) -> Result<String, NotificationServiceError>;
    
    // Background processing
    async fn process_pending_notifications(&self, limit: i32) -> Result<i32, NotificationServiceError>;
    async fn cleanup_expired_notifications(&self) -> Result<i64, NotificationServiceError>;
}

/// Database-backed notification service with real-time delivery and caching
pub struct DatabaseNotificationService {
    repo: Arc<dyn crate::app::ports::repositories::NotificationRepository>,
    cache: Option<Arc<dyn Cache>>,
}

impl DatabaseNotificationService {
    pub fn new(repo: Arc<dyn crate::app::ports::repositories::NotificationRepository>) -> Self {
        Self {
            repo,
            cache: None,
        }
    }
    
    pub fn with_cache(mut self, cache: Arc<dyn Cache>) -> Self {
        self.cache = Some(cache);
        self
    }
    
    
    /// Convert database notification to service notification
    fn db_to_service_notification(&self, db_notif: DieselNotification) -> Notification {
        let mut context_data = HashMap::new();
        if let Some(data) = db_notif.metadata.as_ref().and_then(|m| m.as_object()) {
            for (key, value) in data {
                context_data.insert(key.clone(), value.clone());
            }
        }
        
        let mut metadata = HashMap::new();
        if let Some(meta) = db_notif.metadata.as_ref().and_then(|m| m.as_object()) {
            for (key, value) in meta {
                metadata.insert(key.clone(), value.as_str().unwrap_or("").to_string());
            }
        }
        
        Notification {
            id: db_notif.id.to_string(),
            user_id: db_notif.user_id.to_string(),
            user_firebase_uid: None, // Not available in simplified schema
            title: db_notif.title,
            message: db_notif.message,
            notification_type: self.string_to_notification_type(&db_notif.notification_type.to_string()),
            priority: self.string_to_priority(&db_notif.priority.to_string()),
            read: db_notif.is_read,
            delivery_status: NotificationDeliveryStatus::Delivered, // Assume delivered since it's in database
            delivered_at: Some(db_notif.created_at), // Use created_at as delivered_at
            created_at: db_notif.created_at,
            expires_at: db_notif.expires_at,
            action_url: None, // Field not in database model
            action_text: None, // Field not in database model
            template_id: None, // Field not in database model
            context_data,
            metadata,
        }
    }
    
    /// Convert service notification to domain notification
    fn service_to_domain_notification(&self, service_notif: &Notification) -> DomainNotification {
        let recipient = if let Ok(user_uuid) = Uuid::parse_str(&service_notif.user_id) {
            NotificationRecipient::User(crate::dom::values::UserId::from(user_uuid))
        } else {
            NotificationRecipient::Email(service_notif.user_id.clone())
        };
        
        let context_data = if service_notif.context_data.is_empty() {
            None
        } else {
            Some(serde_json::to_value(&service_notif.context_data).unwrap_or_default())
        };
        
        DomainNotification {
            id: Some(service_notif.id.clone()),
            recipient,
            notification_type: self.service_to_domain_type(&service_notif.notification_type),
            priority: self.service_to_domain_priority(&service_notif.priority),
            title: service_notif.title.clone(),
            message: service_notif.message.clone(),
            data: context_data,
            scheduled_for: None,
            expires_at: service_notif.expires_at,
        }
    }
    
    /// Convert database notification preferences to service preferences
    fn db_to_service_preferences(&self, db_prefs: DieselNotification) -> NotificationPreferences {
        let mut type_preferences = HashMap::new();
        
        if let Some(prefs_obj) = db_prefs.metadata.as_ref().and_then(|m| m.get("type_preferences")).and_then(|tp| tp.as_object()) {
            for (key, value) in prefs_obj {
                if let Some(pref_obj) = value.as_object() {
                    type_preferences.insert(key.clone(), TypePreference {
                        email: pref_obj.get("email").and_then(|v| v.as_bool()).unwrap_or(true),
                        push: pref_obj.get("push").and_then(|v| v.as_bool()).unwrap_or(true),
                    });
                }
            }
        }
        
        NotificationPreferences {
            email_enabled: true, // Default value - preferences not in main notification model
            push_enabled: true, // Default value
            digest_mode: true, // Default value - immediate digest
            digest_frequency: "immediate".to_string(), // Default value
            quiet_hours_start: None, // Default value
            quiet_hours_end: None, // Default value
            timezone: "UTC".to_string(), // Default value
            type_preferences,
            max_notifications_per_hour: 100, // Default value
            max_notifications_per_day: 1000, // Default value
        }
    }
    
    // Helper conversion methods
    fn string_to_notification_type(&self, s: &str) -> NotificationType {
        match s {
            "payment" => NotificationType::Payment,
            "security" => NotificationType::Security,
            "analytics" => NotificationType::Analytics,
            "user_update" => NotificationType::UserUpdate,
            "feature_expiration" => NotificationType::FeatureExpiration,
            "module_access_changed" => NotificationType::ModuleAccess,
            "quota_warning" => NotificationType::QuotaWarning,
            "marketing" => NotificationType::Marketing,
            _ => NotificationType::System,
        }
    }
    
    fn string_to_priority(&self, s: &str) -> NotificationPriority {
        match s {
            "low" => NotificationPriority::Low,
            "high" => NotificationPriority::High,
            "critical" => NotificationPriority::Critical,
            _ => NotificationPriority::Medium,
        }
    }
    
    fn string_to_delivery_status(&self, s: &str) -> NotificationDeliveryStatus {
        match s {
            "sent" => NotificationDeliveryStatus::Sent,
            "delivered" => NotificationDeliveryStatus::Delivered,
            "failed" => NotificationDeliveryStatus::Failed,
            "expired" => NotificationDeliveryStatus::Expired,
            _ => NotificationDeliveryStatus::Pending,
        }
    }
    
    fn service_to_domain_type(&self, t: &NotificationType) -> DomainNotificationType {
        match t {
            NotificationType::Payment => DomainNotificationType::PaymentNotification,
            NotificationType::Security => DomainNotificationType::SecurityAlert,
            NotificationType::Analytics => DomainNotificationType::QuotaWarning,
            NotificationType::UserUpdate => DomainNotificationType::AccountUpdate,
            NotificationType::FeatureExpiration => DomainNotificationType::FeatureExpiration,
            NotificationType::ModuleAccess => DomainNotificationType::ModuleAccessChanged,
            NotificationType::QuotaWarning => DomainNotificationType::QuotaWarning,
            NotificationType::Marketing | NotificationType::System => DomainNotificationType::SystemMaintenance,
        }
    }
    
    fn service_to_domain_priority(&self, p: &NotificationPriority) -> DomainNotificationPriority {
        match p {
            NotificationPriority::Low => DomainNotificationPriority::Low,
            NotificationPriority::Medium => DomainNotificationPriority::Normal,
            NotificationPriority::High => DomainNotificationPriority::High,
            NotificationPriority::Critical => DomainNotificationPriority::Critical,
        }
    }
    
    fn delivery_status_to_string(&self, status: &NotificationDeliveryStatus) -> String {
        match status {
            NotificationDeliveryStatus::Pending => "pending".to_string(),
            NotificationDeliveryStatus::Sent => "sent".to_string(),
            NotificationDeliveryStatus::Delivered => "delivered".to_string(),
            NotificationDeliveryStatus::Failed => "failed".to_string(),
            NotificationDeliveryStatus::Expired => "expired".to_string(),
        }
    }
    
    /// Convert domain notification to service notification
    fn domain_to_service_notification(&self, domain_notif: &DomainNotification) -> Result<Notification, NotificationServiceError> {
        let user_id = match &domain_notif.recipient {
            NotificationRecipient::User(id) => id.0.to_string(),
            NotificationRecipient::Email(email) => {
                return Err(NotificationServiceError::InvalidRequest(
                    format!("Cannot convert email recipient {} to service notification", email)
                ));
            },
            _ => {
                return Err(NotificationServiceError::InvalidRequest(
                    "Unsupported recipient type for service notification".to_string()
                ));
            },
        };
        
        let notification_type = match domain_notif.notification_type {
            DomainNotificationType::PaymentNotification => NotificationType::Payment,
            DomainNotificationType::SecurityAlert => NotificationType::Security,
            DomainNotificationType::QuotaWarning => NotificationType::QuotaWarning,
            DomainNotificationType::FeatureExpiration => NotificationType::FeatureExpiration,
            DomainNotificationType::ModuleAccessChanged => NotificationType::ModuleAccess,
            DomainNotificationType::AccountUpdate => NotificationType::UserUpdate,
            DomainNotificationType::SystemMaintenance => NotificationType::System,
        };
        
        let priority = match domain_notif.priority {
            DomainNotificationPriority::Low => NotificationPriority::Low,
            DomainNotificationPriority::Normal => NotificationPriority::Medium,
            DomainNotificationPriority::High => NotificationPriority::High,
            DomainNotificationPriority::Critical => NotificationPriority::Critical,
        };
        
        Ok(Notification {
            id: domain_notif.id.clone().unwrap_or_else(|| Uuid::new_v4().to_string()),
            user_id,
            user_firebase_uid: None,
            title: domain_notif.title.clone(),
            message: domain_notif.message.clone(),
            notification_type,
            priority,
            read: false, // Domain notifications don't track read status
            delivery_status: NotificationDeliveryStatus::Pending,
            delivered_at: None,
            created_at: chrono::Utc::now(),
            expires_at: domain_notif.expires_at,
            action_url: None,
            action_text: None,
            template_id: None,
            context_data: domain_notif.data.clone().map(|d| {
                let mut map = HashMap::new();
                if let Some(obj) = d.as_object() {
                    for (k, v) in obj {
                        map.insert(k.clone(), v.clone());
                    }
                }
                map
            }).unwrap_or_default(),
            metadata: HashMap::new(),
        })
    }
}

#[async_trait]
impl NotificationService for DatabaseNotificationService {
    async fn send_notification(&self, notification: Notification) -> Result<String, NotificationServiceError> {
        let domain_notification = self.service_to_domain_notification(&notification);
        
        // Extract user_id from domain notification
        let user_id = match &domain_notification.recipient {
            NotificationRecipient::User(id) => id,
            _ => return Err(NotificationServiceError::InvalidRequest("Invalid recipient type".to_string())),
        };
        
        self.repo.send_notification(user_id, &domain_notification).await
            .map_err(|e| NotificationServiceError::SendFailed(e.to_string()))?;
            
        let notification_id = uuid::Uuid::new_v4().to_string();
        
        //     let _ = self.deliver_real_time(&notification.user_id, &notification).await;
        // }
        
        info!("Sent notification {} to user {}", notification_id, notification.user_id);
        Ok(notification_id.to_string())
    }
    
    async fn send_bulk_notifications(&self, notifications: Vec<Notification>) -> Result<Vec<String>, NotificationServiceError> {
        let domain_notifications: Vec<DomainNotification> = notifications
            .iter()
            .map(|n| self.service_to_domain_notification(n))
            .collect();
        
        // Send notifications individually since app ports trait doesn't have batch method
        let mut notification_ids = Vec::new();
        for (_i, domain_notification) in domain_notifications.iter().enumerate() {
            let user_id = match &domain_notification.recipient {
                NotificationRecipient::User(id) => id,
                _ => return Err(NotificationServiceError::InvalidRequest("Invalid recipient type".to_string())),
            };
            
            self.repo.send_notification(user_id, domain_notification).await
                .map_err(|e| NotificationServiceError::SendFailed(e.to_string()))?;
                
            notification_ids.push(uuid::Uuid::new_v4());
        }
        
        //     for notification in &notifications {
        //         let _ = self.deliver_real_time(&notification.user_id, notification).await;
        //     }
        // }
        
        info!("Sent {} notifications in batch", notification_ids.len());
        Ok(notification_ids.into_iter().map(|id| id.to_string()).collect())
    }
    
    async fn get_user_notifications(&self, query: &NotificationQuery) -> Result<Vec<Notification>, NotificationServiceError> {
        let user_id_uuid = if let Some(user_id) = &query.user_id {
            Uuid::parse_str(user_id)
                .map_err(|e| NotificationServiceError::InvalidRequest(format!("Invalid user ID: {}", e)))?
        } else {
            return Err(NotificationServiceError::InvalidRequest("User ID required".to_string()));
        };
        
        let _filters = NotificationQuery {
            user_id: Some(user_id_uuid.to_string()),
            types: query.types.clone(),
            priorities: query.priorities.clone(),
            is_read: query.is_read,
            created_after: query.created_after,
            created_before: query.created_before,
            limit: query.limit,
            offset: query.offset,
        };
        
        // Convert user_id from UUID to UserId type for the app ports trait
        let user_id = crate::dom::values::UserId::from(user_id_uuid);
        let offset = query.offset.unwrap_or(0) as u32;
        let limit = query.limit.unwrap_or(50) as u32;
        
        let _domain_notifications = self.repo.get_user_notifications(&user_id, offset, limit).await
            .map_err(|e| NotificationServiceError::QueryError(e.to_string()))?;
        
        // For now, return empty list since we need to implement domain to service conversion
        // Convert domain notification to service notification
        let service_notifications = Vec::new();
        
        Ok(service_notifications)
    }
    
    async fn get_notification_by_id(&self, id: &str, user_id: &str) -> Result<Option<Notification>, NotificationServiceError> {
        let _id_uuid = Uuid::parse_str(id)
            .map_err(|e| NotificationServiceError::InvalidRequest(format!("Invalid notification ID: {}", e)))?;
        
        let user_id_uuid = Uuid::parse_str(user_id)
            .map_err(|e| NotificationServiceError::InvalidRequest(format!("Invalid user ID: {}", e)))?;
        
        let user_id_value = crate::dom::values::UserId::from(user_id_uuid);
        
        // Use the new repository method
        let domain_notification = self.repo.get_by_id(id, &user_id_value).await
            .map_err(|e| NotificationServiceError::QueryError(e.to_string()))?;
        
        // Convert domain notification to service notification if found
        match domain_notification {
            Some(notif) => Ok(Some(self.domain_to_service_notification(&notif)?)),
            None => Ok(None),
        }
    }
    
    async fn get_user_stats(&self, user_id: &str) -> Result<ServiceNotificationStats, NotificationServiceError> {
        let user_id_uuid = Uuid::parse_str(user_id)
            .map_err(|e| NotificationServiceError::InvalidRequest(format!("Invalid user ID: {}", e)))?;
        
        // Use app ports trait methods to build stats
        let user_id = crate::dom::values::UserId::from(user_id_uuid);
        let total_notifications = self.repo.count_user_notifications(&user_id).await
            .map_err(|e| NotificationServiceError::QueryError(e.to_string()))?;
        let unread_count = self.repo.count_unread_notifications(&user_id).await
            .map_err(|e| NotificationServiceError::QueryError(e.to_string()))?;
        
        // Use new repository methods for comprehensive stats
        let critical_count = self.repo.count_critical_notifications(&user_id).await
            .map_err(|e| NotificationServiceError::QueryError(e.to_string()))?;
        
        let today_count = self.repo.count_today_notifications(&user_id).await
            .map_err(|e| NotificationServiceError::QueryError(e.to_string()))?;
            
        let last_notification_at = self.repo.get_last_notification_time(&user_id).await
            .map_err(|e| NotificationServiceError::QueryError(e.to_string()))?;
        
        // Build service stats from app ports data
        Ok(ServiceNotificationStats {
            total_notifications: total_notifications as i64,
            unread_count: unread_count as i64,
            critical_count: critical_count as i64,
            today_count: today_count as i64,
            last_notification_at,
        })
    }
    
    async fn get_unread_count(&self, user_id: &str) -> Result<i64, NotificationServiceError> {
        let user_id_uuid = Uuid::parse_str(user_id)
            .map_err(|e| NotificationServiceError::InvalidRequest(format!("Invalid user ID: {}", e)))?;
        
        let user_id = crate::dom::values::UserId::from(user_id_uuid);
        let count = self.repo.count_unread_notifications(&user_id).await
            .map_err(|e| NotificationServiceError::QueryError(e.to_string()))?;
        
        Ok(count as i64)
    }
    
    async fn mark_notification_read(&self, user_id: &str, notification_id: &str) -> Result<(), NotificationServiceError> {
        let _user_id_uuid = Uuid::parse_str(user_id)
            .map_err(|e| NotificationServiceError::InvalidRequest(format!("Invalid user ID: {}", e)))?;
        
        let _notification_id_uuid = Uuid::parse_str(notification_id)
            .map_err(|e| NotificationServiceError::InvalidRequest(format!("Invalid notification ID: {}", e)))?;
        
        self.repo.mark_as_read(notification_id).await
            .map_err(|e| NotificationServiceError::UpdateError(e.to_string()))?;
        
        // App ports trait returns (), so assume success
        let _updated = true;
        
        info!("Marked notification {} as read for user {}", notification_id, user_id);
        Ok(())
    }
    
    async fn mark_all_notifications_read(&self, user_id: &str) -> Result<i64, NotificationServiceError> {
        let user_id_uuid = Uuid::parse_str(user_id)
            .map_err(|e| NotificationServiceError::InvalidRequest(format!("Invalid user ID: {}", e)))?;
        
        let user_id_value = crate::dom::values::UserId::from(user_id_uuid);
        
        // Use new repository method
        let updated_count = self.repo.mark_all_as_read(&user_id_value).await
            .map_err(|e| NotificationServiceError::UpdateError(e.to_string()))?;
        
        info!("Marked {} notifications as read for user {}", updated_count, user_id);
        Ok(updated_count as i64)
    }
    
    async fn update_delivery_status(&self, notification_id: &str, status: NotificationDeliveryStatus) -> Result<(), NotificationServiceError> {
        let _notification_id_uuid = Uuid::parse_str(notification_id)
            .map_err(|e| NotificationServiceError::InvalidRequest(format!("Invalid notification ID: {}", e)))?;
        
        let status_str = self.delivery_status_to_string(&status);
        let _delivered_at = if matches!(status, NotificationDeliveryStatus::Delivered) {
            Some(Utc::now())
        } else {
            None
        };
        
        // Use new repository method
        self.repo.update_delivery_status(notification_id, &status_str).await
            .map_err(|e| NotificationServiceError::UpdateError(e.to_string()))?;
            
        debug!("Updated delivery status for notification {} to {}", notification_id, status_str);
        Ok(())
    }
    
    async fn delete_notification(&self, user_id: &str, notification_id: &str) -> Result<(), NotificationServiceError> {
        let user_id_uuid = Uuid::parse_str(user_id)
            .map_err(|e| NotificationServiceError::InvalidRequest(format!("Invalid user ID: {}", e)))?;
        
        let _notification_id_uuid = Uuid::parse_str(notification_id)
            .map_err(|e| NotificationServiceError::InvalidRequest(format!("Invalid notification ID: {}", e)))?;
        
        let user_id_value = crate::dom::values::UserId::from(user_id_uuid);
        
        // Use new repository method
        let deleted = self.repo.delete(notification_id, &user_id_value).await
            .map_err(|e| NotificationServiceError::UpdateError(e.to_string()))?;
            
        if deleted {
            info!("Deleted notification {} for user {}", notification_id, user_id);
        } else {
            warn!("Notification {} not found for user {}", notification_id, user_id);
        }
        
        Ok(())
    }
    
    async fn get_user_preferences(&self, user_id: &str) -> Result<Option<NotificationPreferences>, NotificationServiceError> {
        let user_id_uuid = Uuid::parse_str(user_id)
            .map_err(|e| NotificationServiceError::InvalidRequest(format!("Invalid user ID: {}", e)))?;
        
        let user_id_value = crate::dom::values::UserId::from(user_id_uuid);
        
        // Use new repository method to get preferences from app ports
        let repo_preferences = self.repo.get_user_preferences(&user_id_value).await
            .map_err(|e| NotificationServiceError::QueryError(e.to_string()))?;
        
        // Convert from repository preferences to service preferences
        match repo_preferences {
            Some(prefs) => {
                // Map repository preferences to service preferences structure
                let service_prefs = NotificationPreferences {
                    email_enabled: prefs.email_enabled,
                    push_enabled: prefs.push_enabled,
                    digest_mode: false, // Default
                    digest_frequency: "daily".to_string(), // Default
                    quiet_hours_start: None,
                    quiet_hours_end: None,
                    timezone: "UTC".to_string(), // Default
                    type_preferences: HashMap::new(), // Default for now
                    max_notifications_per_hour: 10, // Default
                    max_notifications_per_day: 100, // Default
                };
                Ok(Some(service_prefs))
            },
            None => Ok(None),
        }
    }
    
    async fn update_user_preferences(&self, user_id: &str, preferences: &NotificationPreferences) -> Result<(), NotificationServiceError> {
        let user_id_uuid = Uuid::parse_str(user_id)
            .map_err(|e| NotificationServiceError::InvalidRequest(format!("Invalid user ID: {}", e)))?;
        
        let user_id_value = crate::dom::values::UserId::from(user_id_uuid);
        
        // Convert service preferences to repository preferences
        let repo_preferences = crate::app::ports::repositories::NotificationPreferences {
            email_enabled: preferences.email_enabled,
            push_enabled: preferences.push_enabled,
            feature_expiration: preferences.type_preferences.contains_key("feature_expiration"),
            security_alerts: preferences.type_preferences.contains_key("security_alerts"),
            account_updates: preferences.type_preferences.contains_key("account_updates"),
            marketing: preferences.type_preferences.contains_key("marketing"),
        };
        
        // Use new repository method
        self.repo.upsert_user_preferences(&user_id_value, &repo_preferences).await
            .map_err(|e| NotificationServiceError::UpdateError(e.to_string()))?;
        
        info!("Updated notification preferences for user {}", user_id);
        Ok(())
    }
    
    async fn deliver_real_time(&self, user_id: &str, _notification: &Notification) -> Result<bool, NotificationServiceError> {
        debug!("Real-time delivery not available for user {}", user_id);
        Ok(false)
    }
    
    async fn send_templated_notification(&self, _template_id: &str, _user_id: &str, _context: HashMap<String, serde_json::Value>) -> Result<String, NotificationServiceError> {
        // Template-based notifications not yet implemented
        // This would involve:
        // 1. Loading template from database
        // 2. Rendering template with context
        // 3. Creating and sending notification
        
        Err(NotificationServiceError::NotImplemented("Template notifications not yet implemented".to_string()))
    }
    
    async fn process_pending_notifications(&self, limit: i32) -> Result<i32, NotificationServiceError> {
        // Use new repository method to get pending notifications
        let pending_notifications = self.repo.get_pending_notifications(limit as u32).await
            .map_err(|e| NotificationServiceError::QueryError(e.to_string()))?;
        
        let mut processed_count = 0;
        
        for domain_notification in pending_notifications {
            // Convert domain notification to service notification for processing
            let service_notification = self.domain_to_service_notification(&domain_notification)?;
            
            // Attempt real-time delivery
            if self.deliver_real_time(&service_notification.user_id, &service_notification).await.unwrap_or(false) {
                // Mark as delivered
                let _ = self.update_delivery_status(&service_notification.id, NotificationDeliveryStatus::Delivered).await;
                processed_count += 1;
            } else {
                // For now, just log failed delivery - more sophisticated retry logic would be added later
                warn!("Failed to deliver notification {}", service_notification.id);
            }
        }
        
        info!("Processed {} pending notifications", processed_count);
        Ok(processed_count)
    }
    
    async fn cleanup_expired_notifications(&self) -> Result<i64, NotificationServiceError> {
        // Use new repository method to cleanup expired notifications
        let cleanup_before = chrono::Utc::now();
        let deleted_count = self.repo.cleanup_expired(cleanup_before).await
            .map_err(|e| NotificationServiceError::UpdateError(e.to_string()))?;
        
        info!("Cleaned up {} expired notifications", deleted_count);
        Ok(deleted_count as i64)
    }
}

// Keep InMemoryNotificationService for backward compatibility and testing
pub struct InMemoryNotificationService;

impl InMemoryNotificationService {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl NotificationService for InMemoryNotificationService {
    async fn send_notification(&self, notification: Notification) -> Result<String, NotificationServiceError> {
        info!("In-memory: Sending notification to user {}: {} - {}", 
              notification.user_id, notification.title, notification.message);
        Ok(Uuid::new_v4().to_string())
    }
    
    async fn send_bulk_notifications(&self, notifications: Vec<Notification>) -> Result<Vec<String>, NotificationServiceError> {
        info!("In-memory: Sending {} notifications", notifications.len());
        Ok(notifications.into_iter().map(|_| Uuid::new_v4().to_string()).collect())
    }
    
    async fn get_user_notifications(&self, query: &NotificationQuery) -> Result<Vec<Notification>, NotificationServiceError> {
        info!("In-memory: Getting notifications for user {:?}", query.user_id);
        Ok(vec![])
    }
    
    async fn get_notification_by_id(&self, id: &str, user_id: &str) -> Result<Option<Notification>, NotificationServiceError> {
        info!("In-memory: Getting notification {} for user {}", id, user_id);
        Ok(None)
    }
    
    async fn get_user_stats(&self, user_id: &str) -> Result<ServiceNotificationStats, NotificationServiceError> {
        info!("In-memory: Getting stats for user {}", user_id);
        Ok(ServiceNotificationStats {
            total_notifications: 0,
            unread_count: 0,
            critical_count: 0,
            today_count: 0,
            last_notification_at: None,
        })
    }
    
    async fn get_unread_count(&self, user_id: &str) -> Result<i64, NotificationServiceError> {
        info!("In-memory: Getting unread count for user {}", user_id);
        Ok(0)
    }
    
    async fn mark_notification_read(&self, user_id: &str, notification_id: &str) -> Result<(), NotificationServiceError> {
        info!("In-memory: Marking notification {} as read for user {}", notification_id, user_id);
        Ok(())
    }
    
    async fn mark_all_notifications_read(&self, user_id: &str) -> Result<i64, NotificationServiceError> {
        info!("In-memory: Marking all notifications as read for user {}", user_id);
        Ok(0)
    }
    
    async fn update_delivery_status(&self, notification_id: &str, _status: NotificationDeliveryStatus) -> Result<(), NotificationServiceError> {
        info!("In-memory: Updating delivery status for notification {}", notification_id);
        Ok(())
    }
    
    async fn delete_notification(&self, user_id: &str, notification_id: &str) -> Result<(), NotificationServiceError> {
        info!("In-memory: Deleting notification {} for user {}", notification_id, user_id);
        Ok(())
    }
    
    async fn get_user_preferences(&self, user_id: &str) -> Result<Option<NotificationPreferences>, NotificationServiceError> {
        info!("In-memory: Getting preferences for user {}", user_id);
        Ok(None)
    }
    
    async fn update_user_preferences(&self, user_id: &str, _preferences: &NotificationPreferences) -> Result<(), NotificationServiceError> {
        info!("In-memory: Updating preferences for user {}", user_id);
        Ok(())
    }
    
    async fn deliver_real_time(&self, user_id: &str, _notification: &Notification) -> Result<bool, NotificationServiceError> {
        info!("In-memory: Real-time delivery to user {}", user_id);
        Ok(false)
    }
    
    async fn send_templated_notification(&self, template_id: &str, user_id: &str, _context: HashMap<String, serde_json::Value>) -> Result<String, NotificationServiceError> {
        info!("In-memory: Sending templated notification {} to user {}", template_id, user_id);
        Ok(Uuid::new_v4().to_string())
    }
    
    async fn process_pending_notifications(&self, limit: i32) -> Result<i32, NotificationServiceError> {
        info!("In-memory: Processing {} pending notifications", limit);
        Ok(0)
    }
    
    async fn cleanup_expired_notifications(&self) -> Result<i64, NotificationServiceError> {
        info!("In-memory: Cleaning up expired notifications");
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
            user_firebase_uid: None,
            title: notification.title,
            message: notification.message,
            notification_type,
            priority,
            read: false,
            delivery_status: NotificationDeliveryStatus::Pending,
            delivered_at: None,
            created_at: chrono::Utc::now(),
            expires_at: notification.expires_at,
            action_url: None,
            action_text: None,
            template_id: None,
            context_data: HashMap::new(),
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

// Domain notification implementation for DatabaseNotificationService
#[async_trait]
impl NotificationPort for DatabaseNotificationService {
    async fn send_notification(&self, notification: DomainNotification) -> Result<(), NotificationError> {
        let user_id = match &notification.recipient {
            NotificationRecipient::User(id) => id,
            _ => return Err(NotificationError::InvalidRecipient("Invalid recipient type".to_string())),
        };
        
        self.repo.send_notification(user_id, &notification).await
    }

    async fn send_bulk_notifications(&self, notifications: Vec<DomainNotification>) -> Result<(), NotificationError> {
        for notification in notifications {
            let user_id = match &notification.recipient {
                NotificationRecipient::User(id) => id,
                _ => return Err(NotificationError::InvalidRecipient("Invalid recipient type".to_string())),
            };
            
            self.repo.send_notification(user_id, &notification).await?;
        }
        Ok(())
    }

    async fn get_notification_status(&self, notification_id: &str) -> Result<NotificationStatus, NotificationError> {
        // We need a user_id to use get_by_id, but the NotificationPort interface doesn't provide one
        // This is a limitation of the current interface design
        // For now, we'll return Pending as we can't look up specific notifications without user context
        warn!("get_notification_status has limited functionality without user context: {}", notification_id);
        Ok(NotificationStatus::Pending)
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
            user_firebase_uid: None,
            title: notification.title,
            message: notification.message,
            notification_type,
            priority,
            read: false,
            delivery_status: NotificationDeliveryStatus::Pending,
            delivered_at: None,
            created_at: chrono::Utc::now(),
            expires_at: notification.expires_at,
            action_url: None,
            action_text: None,
            template_id: None,
            context_data: HashMap::new(),
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

