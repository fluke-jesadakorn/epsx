/// Web3-first Notification Mappers for SSE Integration
/// Convert between domain notification structures and SSE notification types

use chrono::Utc;

use crate::domain::notification::aggregates::notification::Notification;
use crate::domain::notification::value_objects::user_preferences::NotificationType;
use crate::domain::notification::aggregates::notification::NotificationPriority;
use crate::web::notifications::{SSENotification, NotificationType as SSENotificationType, NotificationPriority as SSENotificationPriority};
// Email service import removed - Web3-first system uses direct wallet notifications

/// Mapper for converting between domain and SSE notification structures
pub struct NotificationMapper;

impl NotificationMapper {
    /// Convert domain notification to SSE notification for real-time delivery
    pub fn convert_domain_to_sse(notification: &Notification, wallet_address: &str) -> SSENotification {
        SSENotification {
            id: notification.id().to_string(),
            wallet_address: wallet_address.to_string(),
            notification_type: Self::map_domain_type_to_sse(notification.notification_type()),
            title: notification.content().title().to_string(),
            message: notification.content().body().to_string(),
            data: notification.metadata().data_payload().cloned(),
            priority: Self::map_domain_priority_to_sse(notification.priority()),
            timestamp: Utc::now(),
            expires_at: notification.schedule().expires_at(),
        }
    }

    /// Map domain notification type to SSE notification type
    fn map_domain_type_to_sse(domain_type: &NotificationType) -> SSENotificationType {
        match domain_type {
            NotificationType::System => SSENotificationType::System,
            NotificationType::Admin => SSENotificationType::System,
            NotificationType::Security => SSENotificationType::Security,
            NotificationType::Feature => SSENotificationType::General,
            NotificationType::Marketing => SSENotificationType::General,
            NotificationType::Info => SSENotificationType::General,
            NotificationType::Warning => SSENotificationType::General,
            NotificationType::Error => SSENotificationType::General,
            NotificationType::Success => SSENotificationType::General,
            NotificationType::General => SSENotificationType::General,
        }
    }

    /// Map domain notification priority to SSE priority
    fn map_domain_priority_to_sse(domain_priority: &NotificationPriority) -> SSENotificationPriority {
        match domain_priority {
            NotificationPriority::Low => SSENotificationPriority::Low,
            NotificationPriority::Normal => SSENotificationPriority::Normal,
            NotificationPriority::High => SSENotificationPriority::High,
            NotificationPriority::Urgent => SSENotificationPriority::Critical,
            NotificationPriority::Critical => SSENotificationPriority::Critical,
        }
    }

    // Email notification creation method removed - Web3-first system uses direct wallet notifications

    /// Create domain notification from legacy database record (stub implementation)
    pub fn create_ddd_notification_from_legacy(
        _legacy_data: serde_json::Value
    ) -> Result<Notification, String> {
        // Stub implementation - in a real scenario this would parse legacy data
        Err("Legacy notification mapping not implemented for SSE system".to_string())
    }

    /// Validate SSE notification data
    pub fn validate_sse_notification(notification: &SSENotification) -> Result<(), String> {
        if notification.title.is_empty() {
            return Err("Notification title cannot be empty".to_string());
        }
        
        if notification.message.is_empty() {
            return Err("Notification message cannot be empty".to_string());
        }
        
        if notification.wallet_address.is_empty() {
            return Err("Wallet address cannot be empty".to_string());
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sse_notification_validation() {
        let notification = SSENotification {
            id: "test-id".to_string(),
            wallet_address: "user-123".to_string(),
            notification_type: SSENotificationType::General,
            title: "Test Notification".to_string(),
            message: "Test message".to_string(),
            data: None,
            priority: SSENotificationPriority::Normal,
            timestamp: Utc::now(),
            expires_at: None,
        };

        assert!(NotificationMapper::validate_sse_notification(&notification).is_ok());
    }

    #[test]
    fn test_sse_notification_validation_empty_title() {
        let notification = SSENotification {
            id: "test-id".to_string(),
            wallet_address: "user-123".to_string(),
            notification_type: SSENotificationType::General,
            title: "".to_string(),
            message: "Test message".to_string(),
            data: None,
            priority: SSENotificationPriority::Normal,
            timestamp: Utc::now(),
            expires_at: None,
        };

        assert!(NotificationMapper::validate_sse_notification(&notification).is_err());
    }
}