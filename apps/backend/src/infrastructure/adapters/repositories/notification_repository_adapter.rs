/// Bridges legacy notification persistence with DDD Notification bounded context

use async_trait::async_trait;
use std::sync::Arc;
use chrono::Utc;
use uuid::Uuid;
use tracing::{debug, info, warn};

use crate::domain::notification::aggregates::notification::{Notification, NotificationStatus, DeliveryResult};
use crate::domain::notification::value_objects::*;
use crate::domain::notification::value_objects::user_preferences::NotificationType;
use crate::domain::notification::aggregates::notification::NotificationPriority;
use crate::infrastructure::adapters::services::fcm_service::{FcmService, FcmNotification, FcmResponse, DeliveryStats};
use crate::infrastructure::adapters::services::email_service::SendGridEmailService;
use crate::application::ports::services::EmailServiceError;
use crate::application::shared::error::{ApplicationError, ApplicationResult};

/// Notification repository adapter that bridges DDD domain with legacy infrastructure
pub struct NotificationRepositoryAdapter {
    fcm_service: Arc<FcmService>,
    email_service: Arc<SendGridEmailService>,
}

unsafe impl Send for NotificationRepositoryAdapter {}
unsafe impl Sync for NotificationRepositoryAdapter {}

impl NotificationRepositoryAdapter {
    pub fn new(
        fcm_service: Arc<FcmService>,
        email_service: Arc<SendGridEmailService>,
    ) -> Self {
        debug!("Creating NotificationRepositoryAdapter");
        Self {
            fcm_service,
            email_service,
        }
    }

    /// Convert DDD Notification to legacy format for delivery
    pub async fn deliver_notification_to_user(
        &self,
        notification: &Notification,
        user_id: Uuid,
        fcm_token: Option<String>,
        email: Option<String>,
    ) -> ApplicationResult<Vec<DeliveryResult>> {
        debug!("Delivering notification {} to user {}", notification.id().as_str(), user_id);

        let mut delivery_results = Vec::new();

        // Get enabled delivery channels from notification
        let enabled_channels = notification.channels().enabled_channels();

        for channel in enabled_channels {
            match channel.channel_type() {
                DeliveryChannelType::FcmPush => {
                    if let Some(token) = &fcm_token {
                        match self.deliver_via_fcm(notification, token).await {
                            Ok(response) => {
                                delivery_results.push(DeliveryResult::Success {
                                    delivered_at: Utc::now(),
                                    message_id: Some(Self::extract_fcm_message_id(&response.name)),
                                });
                                info!("FCM delivery successful for notification {}", notification.id().as_str());
                            }
                            Err(e) => {
                                warn!("FCM delivery failed for notification {}: {}", notification.id().as_str(), e);
                                delivery_results.push(DeliveryResult::Failed {
                                    error_message: e.to_string(),
                                    retry_after: Some(Utc::now() + chrono::Duration::minutes(5)),
                                });
                            }
                        }
                    } else {
                        delivery_results.push(DeliveryResult::Failed {
                            error_message: "No FCM token available for user".to_string(),
                            retry_after: None,
                        });
                    }
                }
                DeliveryChannelType::Email => {
                    if let Some(user_email) = &email {
                        match self.deliver_via_email(notification, user_email).await {
                            Ok(()) => {
                                delivery_results.push(DeliveryResult::Success {
                                    delivered_at: Utc::now(),
                                    message_id: None, // Email service doesn't return message ID
                                });
                                info!("Email delivery successful for notification {}", notification.id().as_str());
                            }
                            Err(e) => {
                                warn!("Email delivery failed for notification {}: {}", notification.id().as_str(), e);
                                delivery_results.push(DeliveryResult::Failed {
                                    error_message: e.to_string(),
                                    retry_after: Some(Utc::now() + chrono::Duration::minutes(10)),
                                });
                            }
                        }
                    } else {
                        delivery_results.push(DeliveryResult::Failed {
                            error_message: "No email address available for user".to_string(),
                            retry_after: None,
                        });
                    }
                }
                DeliveryChannelType::Sms => {
                    // SMS not implemented yet
                    delivery_results.push(DeliveryResult::Failed {
                        error_message: "SMS delivery not implemented".to_string(),
                        retry_after: None,
                    });
                }
                DeliveryChannelType::InApp => {
                    // Slack not implemented yet
                    delivery_results.push(DeliveryResult::Failed {
                        error_message: "Slack delivery not implemented".to_string(),
                        retry_after: None,
                    });
                }
                _ => {
                    // Handle any other channel types
                    delivery_results.push(DeliveryResult::Failed {
                        error_message: "Unsupported delivery channel".to_string(),
                        retry_after: None,
                    });
                }
            }
        }

        Ok(delivery_results)
    }

    /// Deliver notification to topic (broadcast)
    pub async fn deliver_notification_to_topic(
        &self,
        notification: &Notification,
        topic_name: &str,
    ) -> ApplicationResult<DeliveryResult> {
        debug!("Delivering notification {} to topic {}", notification.id().as_str(), topic_name);

        // For now, only support FCM topic delivery
        match self.deliver_topic_via_fcm(notification, topic_name).await {
            Ok(response) => {
                info!("FCM topic delivery successful for notification {}", notification.id().as_str());
                Ok(DeliveryResult::Success {
                    delivered_at: Utc::now(),
                    message_id: Some(Self::extract_fcm_message_id(&response.name)),
                })
            }
            Err(e) => {
                warn!("FCM topic delivery failed for notification {}: {}", notification.id().as_str(), e);
                Ok(DeliveryResult::Failed {
                    error_message: e.to_string(),
                    retry_after: Some(Utc::now() + chrono::Duration::minutes(5)),
                })
            }
        }
    }

    /// Deliver multicast to multiple users
    pub async fn deliver_notification_multicast(
        &self,
        notification: &Notification,
        fcm_tokens: Vec<String>,
    ) -> ApplicationResult<DeliveryStats> {
        debug!("Delivering notification {} to {} tokens", notification.id().as_str(), fcm_tokens.len());

        let fcm_notification = self.convert_to_fcm_notification(notification);
        let data = self.build_fcm_data(notification);

        match self.fcm_service.send_multicast(fcm_tokens, fcm_notification, data).await {
            Ok(stats) => {
                info!("Multicast delivery completed: {} sent, {} failed", stats.sent, stats.failed);
                Ok(stats)
            }
            Err(e) => {
                warn!("Multicast delivery failed: {}", e);
                Err(ApplicationError::external_service("FCM", e.to_string()))
            }
        }
    }

    /// Convert DDD notification to FCM notification
    fn convert_to_fcm_notification(&self, notification: &Notification) -> FcmNotification {
        FcmNotification {
            title: Some(notification.content().title().to_string()),
            body: Some(notification.content().body().to_string()),
            image: notification.metadata().image_url().map(|url| url.to_string()),
        }
    }

    /// Build FCM data payload from DDD notification
    fn build_fcm_data(&self, notification: &Notification) -> Option<serde_json::Value> {
        let mut data = serde_json::Map::new();
        
        data.insert("notification_id".to_string(), serde_json::Value::String(notification.id().as_str().to_string()));
        data.insert("notification_type".to_string(), serde_json::Value::String(notification.notification_type().to_string()));
        data.insert("priority".to_string(), serde_json::Value::String(notification.priority().to_string()));

        // Add action URL if available
        if let Some(action_url) = notification.metadata().action_url() {
            data.insert("click_action".to_string(), serde_json::Value::String(action_url.to_string()));
        }

        // Add custom data payload if available
        if let Some(custom_data) = notification.metadata().data_payload() {
            data.insert("custom_data".to_string(), custom_data.clone());
        }

        Some(serde_json::Value::Object(data))
    }

    /// Deliver via FCM to individual token
    async fn deliver_via_fcm(
        &self,
        notification: &Notification,
        token: &str,
    ) -> Result<FcmResponse, crate::core::errors::AppError> {
        let fcm_notification = self.convert_to_fcm_notification(notification);
        let data = self.build_fcm_data(notification);

        let fcm_message = crate::infrastructure::adapters::services::fcm_service::FcmMessage {
            target: crate::infrastructure::adapters::services::fcm_service::FcmTarget::Token {
                token: token.to_string(),
            },
            data,
            notification: Some(fcm_notification),
            android: Some(crate::infrastructure::adapters::services::fcm_service::FcmAndroidConfig {
                priority: Some(self.map_priority_to_fcm(notification.priority())),
                ttl: Some("86400s".to_string()), // 24 hours
                notification: None,
            }),
            webpush: Some(crate::infrastructure::adapters::services::fcm_service::FcmWebpushConfig {
                headers: Some(serde_json::json!({"Urgency": self.map_priority_to_urgency(notification.priority())})),
                data: None,
                notification: None,
                fcm_options: notification.metadata().action_url().map(|url| {
                    crate::infrastructure::adapters::services::fcm_service::FcmWebpushFcmOptions {
                        link: Some(url.to_string()),
                        analytics_label: Some(format!("notification_{}", notification.notification_type())),
                    }
                }),
            }),
        };

        self.fcm_service.send_message(fcm_message).await
    }

    /// Deliver via FCM to topic
    async fn deliver_topic_via_fcm(
        &self,
        notification: &Notification,
        topic: &str,
    ) -> Result<FcmResponse, crate::core::errors::AppError> {
        let fcm_notification = self.convert_to_fcm_notification(notification);
        let data = self.build_fcm_data(notification);

        self.fcm_service.send_to_topic(topic.to_string(), fcm_notification, data).await
    }

    /// Deliver via email
    async fn deliver_via_email(
        &self,
        notification: &Notification,
        email: &str,
    ) -> Result<(), EmailServiceError> {
        match notification.notification_type() {
            NotificationType::Feature => {
                // Extract name from metadata or use default
                let name = "User"; // Would extract from notification metadata
                self.email_service.send_welcome_email(email, name).await
            }
            NotificationType::Security => {
                // Extract reset link from action URL
                let reset_link = notification.metadata().action_url().unwrap_or("https://epsx.io/reset");
                self.email_service.send_password_reset(email, reset_link).await
            }
            NotificationType::System => {
                // Would need to extract amount and currency from custom data
                let amount = rust_decimal::Decimal::new(1000, 2); // $10.00 - placeholder
                let currency = "USD";
                self.email_service.send_payment_confirmation(email, amount, currency).await
            }
            NotificationType::Admin => {
                // Would extract new role from custom data
                let new_role = "premium"; // placeholder
                self.email_service.send_role_upgrade_notification(email, new_role).await
            }
            _ => {
                // For other notification types, would need to implement generic email sending
                warn!("Email delivery not implemented for notification type: {}", notification.notification_type());
                Err(EmailServiceError::DeliveryFailed("Notification type not supported for email".to_string()))
            }
        }
    }

    /// Map DDD priority to FCM priority
    fn map_priority_to_fcm(&self, priority: &NotificationPriority) -> String {
        match priority {
            NotificationPriority::Urgent | NotificationPriority::Critical | NotificationPriority::High => "high".to_string(),
            NotificationPriority::Normal | NotificationPriority::Low => "normal".to_string(),
        }
    }

    /// Map DDD priority to WebPush urgency
    fn map_priority_to_urgency(&self, priority: &NotificationPriority) -> String {
        match priority {
            NotificationPriority::Urgent => "high".to_string(),
            NotificationPriority::Critical => "high".to_string(),
            NotificationPriority::High => "normal".to_string(),
            NotificationPriority::Normal => "low".to_string(),
            NotificationPriority::Low => "very-low".to_string(),
        }
    }

    /// Extract message ID from FCM response name
    fn extract_fcm_message_id(response_name: &str) -> String {
        // FCM response name format: projects/{project_id}/messages/{message_id}
        response_name.split('/').last().unwrap_or(response_name).to_string()
    }
}

/// For future implementation - would need database schema for notifications
#[async_trait]
trait NotificationRepository {
    async fn save(&self, notification: &Notification) -> ApplicationResult<()>;
    async fn find_by_id(&self, id: &NotificationId) -> ApplicationResult<Option<Notification>>;
    async fn find_byuser_id(&self, user_id: Uuid) -> ApplicationResult<Vec<Notification>>;
    async fn find_pending_for_processing(&self) -> ApplicationResult<Vec<Notification>>;
    async fn update_status(&self, id: &NotificationId, status: NotificationStatus) -> ApplicationResult<()>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::notification::value_objects::*;
    use crate::domain::notification::aggregates::notification::*;

    #[test]
    fn test_fcm_notification_conversion() {
        // Create test notification content
        let content = NotificationContent::new(
            "Test Notification".to_string(),
            "This is a test notification body".to_string(),
        ).unwrap();

        // Create test notification
        let notification = Notification::create_for_user(
            Uuid::new_v4(),
            content,
            NotificationType::General,
            NotificationPriority::Normal,
            MultiChannelConfig::new(vec![
                DeliveryChannelConfig::new(
                    DeliveryChannelType::Push,
                    true,
                    None,
                ).unwrap()
            ]).unwrap(),
            ScheduleInfo::immediate(),
        ).unwrap();

        // Create adapter with mock services
        let fcm_service = Arc::new(FcmService::new(
            Arc::new(crate::infrastructure::firebase::FirebaseAdmin::new_mock())
        ));
        let email_service = Arc::new(SendGridEmailService::new(
            Arc::new(crate::config::Config::default())
        ));
        let adapter = NotificationRepositoryAdapter::new(fcm_service, email_service);

        // Test conversion
        let fcm_notification = adapter.convert_to_fcm_notification(&notification);
        assert_eq!(fcm_notification.title, Some("Test Notification".to_string()));
        assert_eq!(fcm_notification.body, Some("This is a test notification body".to_string()));
        assert_eq!(fcm_notification.image, None);
    }

    #[test]
    fn test_priority_mapping() {
        let fcm_service = Arc::new(FcmService::new(
            Arc::new(crate::infrastructure::firebase::FirebaseAdmin::new_mock())
        ));
        let email_service = Arc::new(SendGridEmailService::new(
            Arc::new(crate::config::Config::default())
        ));
        let adapter = NotificationRepositoryAdapter::new(fcm_service, email_service);

        assert_eq!(adapter.map_priority_to_fcm(&NotificationPriority::Urgent), "high");
        assert_eq!(adapter.map_priority_to_fcm(&NotificationPriority::High), "high");
        assert_eq!(adapter.map_priority_to_fcm(&NotificationPriority::Normal), "normal");
        assert_eq!(adapter.map_priority_to_fcm(&NotificationPriority::Low), "normal");

        assert_eq!(adapter.map_priority_to_urgency(&NotificationPriority::Urgent), "high");
        assert_eq!(adapter.map_priority_to_urgency(&NotificationPriority::High), "normal");
        assert_eq!(adapter.map_priority_to_urgency(&NotificationPriority::Normal), "low");
        assert_eq!(adapter.map_priority_to_urgency(&NotificationPriority::Low), "very-low");
    }
}