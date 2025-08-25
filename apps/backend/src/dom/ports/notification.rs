// Domain notification port - abstracts notification infrastructure
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use crate::dom::values::UserId;

/// Domain-level notification abstraction
#[async_trait]
pub trait NotificationPort: Send + Sync {
    /// Send a notification
    async fn send_notification(&self, notification: DomainNotification) -> Result<(), NotificationError>;
    
    /// Send bulk notifications
    async fn send_bulk_notifications(&self, notifications: Vec<DomainNotification>) -> Result<(), NotificationError>;
    
    /// Check notification status
    async fn get_notification_status(&self, notification_id: &str) -> Result<NotificationStatus, NotificationError>;
}

/// Domain notification entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainNotification {
    pub id: Option<String>,
    pub recipient: NotificationRecipient,
    pub notification_type: DomainNotificationType,
    pub priority: DomainNotificationPriority,
    pub title: String,
    pub message: String,
    pub data: Option<serde_json::Value>,
    pub scheduled_for: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
}

/// Notification recipient
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NotificationRecipient {
    User(UserId),
    Email(String),
    AdminGroup,
    Broadcast,
}

/// Domain notification types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DomainNotificationType {
    /// Feature access expiration warnings
    FeatureExpiration,
    /// Module access changes
    ModuleAccessChanged,
    /// Quota limit warnings
    QuotaWarning,
    /// Security alerts
    SecurityAlert,
    /// System maintenance notifications
    SystemMaintenance,
    /// Account updates
    AccountUpdate,
    /// Payment notifications
    PaymentNotification,
}

/// Domain notification priorities
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum DomainNotificationPriority {
    Low,
    Normal,
    High,
    Critical,
}

impl std::fmt::Display for DomainNotificationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            DomainNotificationType::FeatureExpiration => "feature_expiration",
            DomainNotificationType::ModuleAccessChanged => "module_access_changed",
            DomainNotificationType::QuotaWarning => "quota_warning",
            DomainNotificationType::SecurityAlert => "security_alert",
            DomainNotificationType::SystemMaintenance => "system_maintenance",
            DomainNotificationType::AccountUpdate => "account_update",
            DomainNotificationType::PaymentNotification => "payment_notification",
        };
        write!(f, "{}", s)
    }
}

impl std::fmt::Display for DomainNotificationPriority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            DomainNotificationPriority::Low => "low",
            DomainNotificationPriority::Normal => "normal",
            DomainNotificationPriority::High => "high",
            DomainNotificationPriority::Critical => "critical",
        };
        write!(f, "{}", s)
    }
}

/// Notification delivery status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NotificationStatus {
    Pending,
    Sent,
    Delivered,
    Failed(String),
    Expired,
}

/// Domain notification errors
#[derive(Debug, thiserror::Error)]
pub enum NotificationError {
    #[error("Failed to send notification: {0}")]
    SendFailed(String),
    
    #[error("Invalid recipient: {0}")]
    InvalidRecipient(String),
    
    #[error("Notification service unavailable")]
    ServiceUnavailable,
    
    #[error("Notification expired")]
    Expired,
    
    #[error("Rate limit exceeded")]
    RateLimitExceeded,
    
    #[error("Database error: {0}")]
    DatabaseError(String),
    
    #[error("Validation error: {0}")]
    ValidationError(String),
}

impl DomainNotification {
    pub fn new(
        recipient: NotificationRecipient,
        notification_type: DomainNotificationType,
        title: String,
        message: String,
    ) -> Self {
        Self {
            id: None,
            recipient,
            notification_type,
            priority: DomainNotificationPriority::Normal,
            title,
            message,
            data: None,
            scheduled_for: None,
            expires_at: None,
        }
    }
    
    pub fn with_priority(mut self, priority: DomainNotificationPriority) -> Self {
        self.priority = priority;
        self
    }
    
    pub fn with_data(mut self, data: serde_json::Value) -> Self {
        self.data = Some(data);
        self
    }
    
    pub fn with_expiration(mut self, expires_at: DateTime<Utc>) -> Self {
        self.expires_at = Some(expires_at);
        self
    }
    
    pub fn scheduled_for(mut self, scheduled_for: DateTime<Utc>) -> Self {
        self.scheduled_for = Some(scheduled_for);
        self
    }
}