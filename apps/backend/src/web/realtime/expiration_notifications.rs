// Real-time expiration notification service
// Bridges domain-level expiration events with WebSocket real-time events

use std::sync::Arc;
use chrono::{Utc, Duration};
use tracing::{info, error};

use crate::dom::services::feature_expiration::{FeatureExpiration, FeatureExpirationService};
use crate::dom::values::UserId;
use super::websocket::ConnectionManager;
use super::events::{RealtimeEvent, EventMessage, ExpirationWarningLevel};

pub struct ExpirationNotificationService {
    connection_manager: Arc<ConnectionManager>,
    feature_expiration_service: Arc<dyn FeatureExpirationService>,
}

impl ExpirationNotificationService {
    pub fn new(
        connection_manager: Arc<ConnectionManager>,
        feature_expiration_service: Arc<dyn FeatureExpirationService>,
    ) -> Self {
        Self {
            connection_manager,
            feature_expiration_service,
        }
    }

    /// Check and notify about expiring features for a specific user
    pub async fn check_and_notify_user_expirations(&self, user_id: &UserId) -> Result<u32, ExpirationNotificationError> {
        let expiring_features = self.feature_expiration_service
            .get_expiring_features(user_id)
            .await
            .map_err(|e| ExpirationNotificationError::ServiceError(e.to_string()))?;

        let mut notifications_sent = 0;

        for feature in expiring_features {
            if let Err(e) = self.process_expiring_feature(&feature).await {
                error!("Failed to process expiring feature for user {}: {:?}", user_id, e);
            } else {
                notifications_sent += 1;
            }
        }

        info!("Sent {} expiration notifications to user {}", notifications_sent, user_id);
        Ok(notifications_sent)
    }

    /// Process a single expiring feature and send appropriate notifications
    async fn process_expiring_feature(&self, feature: &FeatureExpiration) -> Result<(), ExpirationNotificationError> {
        let now = Utc::now();
        let days_until_expiration = (feature.expires_at - now).num_days();
        let hours_until_expiration = (feature.expires_at - now).num_hours();

        if days_until_expiration < -(feature.grace_period_days as i64) {
            // Feature is expired beyond grace period - already deactivated
            return Ok(());
        }

        if days_until_expiration < 0 {
            // Feature is in grace period
            let grace_period_ends = feature.expires_at + Duration::days(feature.grace_period_days as i64);
            let hours_until_deactivation = (grace_period_ends - now).num_hours();

            if hours_until_deactivation <= 24 {
                // Grace period ending soon
                self.send_grace_period_ending_notification(feature, hours_until_deactivation as u32).await?;
            } else if days_until_expiration == -1 && hours_until_expiration.abs() < 1 {
                // Just entered grace period
                self.send_grace_period_started_notification(feature).await?;
            }
        } else if days_until_expiration == 0 {
            // Expires today
            if feature.grace_period_days > 0 {
                self.send_grace_period_started_notification(feature).await?;
            } else {
                self.send_feature_expired_notification(feature).await?;
            }
        } else {
            // Feature expiring in the future - send warning based on days remaining
            let warning_level = self.determine_warning_level(days_until_expiration);
            self.send_expiration_warning_notification(feature, days_until_expiration, warning_level).await?;
        }

        Ok(())
    }

    fn determine_warning_level(&self, days_until_expiration: i64) -> ExpirationWarningLevel {
        match days_until_expiration {
            0..=1 => ExpirationWarningLevel::Final,
            2..=3 => ExpirationWarningLevel::Critical,
            4..=7 => ExpirationWarningLevel::Urgent,
            8..=14 => ExpirationWarningLevel::Standard,
            _ => ExpirationWarningLevel::Early,
        }
    }

    async fn send_expiration_warning_notification(
        &self,
        feature: &FeatureExpiration,
        days_until_expiration: i64,
        warning_level: ExpirationWarningLevel,
    ) -> Result<(), ExpirationNotificationError> {
        let event = RealtimeEvent::feature_expiration_warning(
            feature.user_id.to_string(),
            feature.permission_profile_id.value().to_string(),
            feature.permission_profile_name.clone(),
            days_until_expiration,
            feature.expires_at,
            feature.features.clone(),
            warning_level,
        );

        let message = EventMessage::new(event, "expiration-service".to_string())
            .with_user_id(feature.user_id.to_string());

        self.connection_manager.send_to_user(&feature.user_id, message).await;
        
        info!(
            "Sent expiration warning to user {} for profile {} ({} days remaining)",
            feature.user_id, feature.permission_profile_name, days_until_expiration
        );

        Ok(())
    }

    async fn send_feature_expired_notification(&self, feature: &FeatureExpiration) -> Result<(), ExpirationNotificationError> {
        let grace_period_ends = if feature.grace_period_days > 0 {
            Some(feature.expires_at + Duration::days(feature.grace_period_days as i64))
        } else {
            None
        };

        let event = RealtimeEvent::feature_expired(
            feature.user_id.to_string(),
            feature.permission_profile_id.value().to_string(),
            feature.permission_profile_name.clone(),
            feature.expires_at,
            feature.features.clone(),
            feature.grace_period_days > 0,
            grace_period_ends,
        );

        let message = EventMessage::new(event, "expiration-service".to_string())
            .with_user_id(feature.user_id.to_string());

        self.connection_manager.send_to_user(&feature.user_id, message).await;
        
        info!(
            "Sent feature expired notification to user {} for profile {}",
            feature.user_id, feature.permission_profile_name
        );

        Ok(())
    }

    async fn send_grace_period_started_notification(&self, feature: &FeatureExpiration) -> Result<(), ExpirationNotificationError> {
        let grace_period_ends = feature.expires_at + Duration::days(feature.grace_period_days as i64);

        let event = RealtimeEvent::grace_period_started(
            feature.user_id.to_string(),
            feature.permission_profile_id.value().to_string(),
            feature.permission_profile_name.clone(),
            feature.grace_period_days,
            grace_period_ends,
            feature.features.clone(),
        );

        let message = EventMessage::new(event, "expiration-service".to_string())
            .with_user_id(feature.user_id.to_string());

        self.connection_manager.send_to_user(&feature.user_id, message).await;
        
        info!(
            "Sent grace period started notification to user {} for profile {} ({} days grace period)",
            feature.user_id, feature.permission_profile_name, feature.grace_period_days
        );

        Ok(())
    }

    async fn send_grace_period_ending_notification(
        &self,
        feature: &FeatureExpiration,
        hours_until_deactivation: u32,
    ) -> Result<(), ExpirationNotificationError> {
        let deactivation_at = feature.expires_at + Duration::days(feature.grace_period_days as i64);

        let event = RealtimeEvent::grace_period_ending(
            feature.user_id.to_string(),
            feature.permission_profile_id.value().to_string(),
            feature.permission_profile_name.clone(),
            hours_until_deactivation,
            deactivation_at,
            feature.features.clone(),
        );

        let message = EventMessage::new(event, "expiration-service".to_string())
            .with_user_id(feature.user_id.to_string());

        self.connection_manager.send_to_user(&feature.user_id, message).await;
        
        info!(
            "Sent grace period ending notification to user {} for profile {} ({} hours until deactivation)",
            feature.user_id, feature.permission_profile_name, hours_until_deactivation
        );

        Ok(())
    }

    /// Broadcast system-wide notification about expiration policy changes or maintenance
    pub async fn broadcast_expiration_system_notification(
        &self,
        title: String,
        message: String,
    ) -> Result<(), ExpirationNotificationError> {
        let event = RealtimeEvent::SystemNotification {
            title,
            message,
            level: crate::web::realtime::events::NotificationLevel::Info,
            target_user: None, // Broadcast to all users
            metadata: std::collections::HashMap::new(),
            timestamp: Utc::now(),
        };

        let event_message = EventMessage::new(event, "expiration-service".to_string());
        self.connection_manager.broadcast_event(event_message).await;

        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ExpirationNotificationError {
    #[error("Feature expiration service error: {0}")]
    ServiceError(String),
    
    #[error("WebSocket communication error: {0}")]
    WebSocketError(String),
    
    #[error("Serialization error: {0}")]
    SerializationError(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockall::mock;
    use crate::dom::entities::permission_profile::PermissionProfileId;

    mock! {
        FeatureExpirationServiceMock {}
        #[async_trait::async_trait]
        impl FeatureExpirationService for FeatureExpirationServiceMock {
            async fn check_feature_expirations(&self) -> Result<crate::dom::services::feature_expiration::ExpirationCheckResult, crate::dom::services::feature_expiration::ExpirationError>;
            async fn get_expiring_features(&self, _user_id: &UserId) -> Result<Vec<FeatureExpiration>, crate::dom::services::feature_expiration::ExpirationError>;
            async fn extend_feature_expiration(&self, _user_id: &UserId, permission__profile_id: &PermissionProfileId, extension_days: u32) -> Result<(), crate::dom::services::feature_expiration::ExpirationError>;
            async fn send_renewal_notification(&self, _user_id: &UserId, expiration: &FeatureExpiration, days_until_expiry: u32) -> Result<(), crate::dom::services::feature_expiration::ExpirationError>;
            async fn deactivate_expired_features(&self, _user_id: &UserId, permission__profile_id: &PermissionProfileId) -> Result<(), crate::dom::services::feature_expiration::ExpirationError>;
        }
    }

    #[tokio::test]
    async fn test_determine_warning_level() {
        let connection_manager = Arc::new(ConnectionManager::new());
        let mut mock_service = MockFeatureExpirationServiceMock::new();
        
        // Mock service doesn't need to be called for this test
        mock_service.expect_get_expiring_features()
            .returning(|_| Ok(Vec::new()));

        let service = ExpirationNotificationService::new(
            connection_manager,
            Arc::new(mock_service),
        );

        assert!(matches!(service.determine_warning_level(0), ExpirationWarningLevel::Final));
        assert!(matches!(service.determine_warning_level(1), ExpirationWarningLevel::Final));
        assert!(matches!(service.determine_warning_level(3), ExpirationWarningLevel::Critical));
        assert!(matches!(service.determine_warning_level(7), ExpirationWarningLevel::Urgent));
        assert!(matches!(service.determine_warning_level(14), ExpirationWarningLevel::Standard));
        assert!(matches!(service.determine_warning_level(30), ExpirationWarningLevel::Early));
    }

    #[tokio::test]
    async fn test_service_creation() {
        let connection_manager = Arc::new(ConnectionManager::new());
        let mock_service = MockFeatureExpirationServiceMock::new();
        
        let service = ExpirationNotificationService::new(
            connection_manager,
            Arc::new(mock_service),
        );

        // Test that service was created without panicking
        assert!(service.connection_manager.get_stats().await.total_connections == 0);
    }
}