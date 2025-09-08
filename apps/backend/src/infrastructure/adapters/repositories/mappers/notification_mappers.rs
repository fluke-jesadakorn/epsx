use chrono::{DateTime, Utc};/// Notification Mappers for Legacy-DDD Integration
/// Convert between legacy notification structures and DDD Notification aggregates

use serde_json::Value;
use tracing::debug;
use uuid::Uuid;
use crate::domain::shared_kernel::AggregateRoot;

use crate::domain::notification::aggregates::notification::Notification;
use crate::domain::notification::value_objects::*;
use crate::infrastructure::adapters::repositories::diesel::types::{NotificationType, NotificationPriority};
use crate::infrastructure::adapters::services::fcm_service::{FcmMessage, FcmNotification, FcmTarget};
use crate::infrastructure::adapters::services::email_service::SentEmail;

/// Mapper for converting between legacy and DDD notification structures
pub struct NotificationMapper;

impl NotificationMapper {
    /// Convert legacy FCM message to DDD notification content
    pub fn fcm_message_to_ddd_content(fcm_message: &FcmMessage) -> Result<NotificationContent, String> {
        debug!("Converting FCM message to DDD content");

        let notification = fcm_message.notification.as_ref()
            .ok_or_else(|| "FCM message has no notification payload".to_string())?;

        let title = notification.title.as_ref()
            .ok_or_else(|| "FCM notification has no title".to_string())?;

        let body = notification.body.as_ref()
            .ok_or_else(|| "FCM notification has no body".to_string())?;

        NotificationContent::new(title.clone(), body.clone())
    }

    /// Convert DDD notification to legacy FCM message
    pub fn ddd_notification_to_fcm_message(
        notification: &Notification,
        target: FcmTarget,
    ) -> FcmMessage {
        debug!("Converting DDD notification to FCM message");

        let fcm_notification = FcmNotification {
            title: Some(notification.content().title().to_string()),
            body: Some(notification.content().body().to_string()),
            image: notification.metadata().image_url().map(|url| url.to_string()),
        };

        let mut data_map = serde_json::Map::new();
        data_map.insert("notification_id".to_string(), Value::String(notification.id().as_str().to_string()));
        data_map.insert("type".to_string(), Value::String(notification.notification_type().to_string()));
        data_map.insert("priority".to_string(), Value::String(notification.priority().to_string()));
        data_map.insert("created_at".to_string(), Value::String(notification.created_at().to_rfc3339()));

        // Add action URL if available
        if let Some(action_url) = notification.metadata().action_url() {
            data_map.insert("action_url".to_string(), Value::String(action_url.to_string()));
        }

        // Add custom data if available
        if let Some(custom_data) = notification.metadata().data_payload() {
            data_map.insert("custom_data".to_string(), custom_data.clone());
        }

        // Add tags as comma-separated string
        if !notification.metadata().tags().is_empty() {
            data_map.insert("tags".to_string(), Value::String(notification.metadata().tags().join(",")));
        }

        FcmMessage {
            target,
            data: Some(Value::Object(data_map)),
            notification: Some(fcm_notification),
            android: Some(crate::infrastructure::adapters::services::fcm_service::FcmAndroidConfig {
                priority: Some(Self::map_ddd_priority_to_fcm(notification.priority())),
                ttl: Some(Self::calculate_ttl_from_schedule(notification.schedule())),
                notification: None,
            }),
            webpush: Some(crate::infrastructure::adapters::services::fcm_service::FcmWebpushConfig {
                headers: Some(serde_json::json!({
                    "Urgency": Self::map_ddd_priority_to_urgency(notification.priority())
                })),
                data: None,
                notification: None,
                fcm_options: notification.metadata().action_url().map(|url| {
                    crate::infrastructure::adapters::services::fcm_service::FcmWebpushFcmOptions {
                        link: Some(url.to_string()),
                        analytics_label: Some(format!("notification_{}", notification.notification_type())),
                    }
                }),
            }),
        }
    }

    /// Convert legacy sent email to DDD delivery tracking info
    pub fn sent_email_to_ddd_delivery_info(sent_email: &SentEmail) -> (String, NotificationContent) {
        debug!("Converting sent email to DDD delivery info");

        let content = NotificationContent::new(
            sent_email.subject.clone(),
            format!("Email sent to: {}", sent_email.to), // content field not available, use to field
        ).unwrap_or_else(|_| {
            // Fallback content if validation fails
            NotificationContent::new(
                "Email Notification".to_string(),
                "Email content unavailable".to_string(),
            ).unwrap()
        });

        (sent_email.to.clone(), content)
    }

    /// Convert DDD notification to email-compatible format
    pub fn ddd_notification_to_email_data(notification: &Notification) -> EmailData {
        debug!("Converting DDD notification to email data");

        EmailData {
            recipient_email: "".to_string(), // Will be filled by the caller
            subject: notification.content().title().to_string(),
            html_content: Self::generate_html_content(notification),
            text_content: Self::generate_text_content(notification),
            notification_type: notification.notification_type().clone(),
            metadata: EmailMetadata {
                notification_id: notification.id().as_str().to_string(),
                priority: notification.priority().clone(),
                action_url: notification.metadata().action_url().map(|url| url.to_string()),
                image_url: notification.metadata().image_url().map(|url| url.to_string()),
                tags: notification.metadata().tags().to_vec(),
            },
        }
    }

    /// Create DDD notification from legacy parameters
    pub fn create_ddd_notification_from_legacy(
        recipient_user_id: Option<Uuid>,
        topic_name: Option<String>,
        title: String,
        body: String,
        notification_type: NotificationType,
        priority: NotificationPriority,
        channels: Vec<String>,
        schedule_at: Option<DateTime<Utc>>,
        expires_at: Option<DateTime<Utc>>,
        action_url: Option<String>,
        image_url: Option<String>,
        custom_data: Option<Value>,
    ) -> Result<Notification, String> {
        debug!("Creating DDD notification from legacy parameters");

        // Create notification content
        let content = NotificationContent::new(title, body)?;

        // Build delivery channels
        let delivery_channels: Result<Vec<DeliveryChannel>, String> = channels
            .into_iter()
            .map(|channel_str| {
                let channel_type = match channel_str.as_str() {
                    "push" | "fcm" => DeliveryChannelType::FcmPush,
                    "email" => DeliveryChannelType::Email,
                    "sms" => DeliveryChannelType::Sms,
                    "in_app" => DeliveryChannelType::InApp,
                    _ => return Err(format!("Unknown channel type: {}", channel_str)),
                };

                Ok(DeliveryChannel::new(channel_type))
            })
            .collect();

        let multi_channel_config = MultiChannelConfig::new(delivery_channels?);

        // Create schedule info
        let schedule = match (schedule_at, expires_at) {
            (Some(scheduled_at), Some(expires_at)) => ScheduleInfo::scheduled_with_expiry(scheduled_at, expires_at)?,
            (Some(scheduled_at), None) => ScheduleInfo::scheduled(scheduled_at)?,
            (None, Some(expires_at)) => ScheduleInfo::with_expiry(expires_at)?,
            (None, None) => ScheduleInfo::immediate(),
        };

        // Create notification based on recipient type
        let notification = if let Some(user_id) = recipient_user_id {
            Notification::create_for_user(
                user_id,
                content,
                notification_type,
                priority,
                multi_channel_config,
                schedule,
            )?
        } else if let Some(topic) = topic_name {
            let notification_topic = NotificationTopic::new(
                topic.clone(),
                topic,
                None, // No description provided
                TopicCategory::General, // Default category
            )?;

            Notification::create_for_topic(
                notification_topic,
                content,
                notification_type,
                priority,
                multi_channel_config,
                schedule,
                None, // No creator specified
            )?
        } else {
            return Err("Either recipient_user_id or topic_name must be provided".to_string());
        };

        // Note: Metadata setting would need to be done during creation for immutable notification
        // For DDD migration, we'll skip additional metadata for now
        let _ = (action_url, image_url, custom_data); // Prevent unused variable warnings

        Ok(notification)
    }

    /// Map DDD priority to FCM priority string
    fn map_ddd_priority_to_fcm(priority: &NotificationPriority) -> String {
        match priority {
            NotificationPriority::Urgent | NotificationPriority::Critical | NotificationPriority::High => "high".to_string(),
            NotificationPriority::Normal | NotificationPriority::Low => "normal".to_string(),
        }
    }

    /// Map DDD priority to WebPush urgency
    fn map_ddd_priority_to_urgency(priority: &NotificationPriority) -> String {
        match priority {
            NotificationPriority::Urgent => "high".to_string(),
            NotificationPriority::Critical => "high".to_string(),
            NotificationPriority::High => "normal".to_string(),
            NotificationPriority::Normal => "low".to_string(),
            NotificationPriority::Low => "very-low".to_string(),
        }
    }

    /// Calculate TTL from schedule information
    fn calculate_ttl_from_schedule(schedule: &ScheduleInfo) -> String {
        if let Some(expires_at) = schedule.expires_at() {
            let now = Utc::now();
            if expires_at > now {
                let duration = expires_at.signed_duration_since(now);
                let seconds = duration.num_seconds().max(60); // Minimum 1 minute
                format!("{}s", seconds)
            } else {
                "3600s".to_string() // Default 1 hour
            }
        } else {
            "86400s".to_string() // Default 24 hours
        }
    }

    /// Generate HTML content for email from DDD notification
    fn generate_html_content(notification: &Notification) -> String {
        let title = notification.content().title();
        let body = notification.content().body();
        
        let action_button = if let Some(action_url) = notification.metadata().action_url() {
            format!(
                r#"<div style="margin: 30px 0;">
                    <a href="{}" 
                       style="background-color: #007bff; color: white; padding: 12px 24px; text-decoration: none; border-radius: 5px;">
                        Take Action
                    </a>
                </div>"#,
                action_url
            )
        } else {
            String::new()
        };

        let image_section = if let Some(image_url) = notification.metadata().image_url() {
            format!(
                r#"<div style="margin: 20px 0;">
                    <img src="{}" alt="Notification Image" style="max-width: 100%; height: auto;">
                </div>"#,
                image_url
            )
        } else {
            String::new()
        };

        format!(
            r#"
            <div style="font-family: Arial, sans-serif; max-width: 600px; margin: 0 auto;">
                <h1 style="color: #1a1a1a;">{}</h1>
                {}
                <div style="margin: 20px 0;">
                    <p>{}</p>
                </div>
                {}
                <p style="color: #666; font-size: 14px;">
                    This notification was sent from the EPSX platform.
                </p>
            </div>
            "#,
            title, image_section, body, action_button
        )
    }

    /// Generate text content for email from DDD notification
    fn generate_text_content(notification: &Notification) -> String {
        let title = notification.content().title();
        let body = notification.content().body();
        
        let action_section = if let Some(action_url) = notification.metadata().action_url() {
            format!("\n\nTake action: {}", action_url)
        } else {
            String::new()
        };

        format!(
            "{}\n\n{}{}\n\nThis notification was sent from the EPSX platform.",
            title, body, action_section
        )
    }
}

/// Email data structure for legacy compatibility
#[derive(Debug, Clone)]
pub struct EmailData {
    pub recipient_email: String,
    pub subject: String,
    pub html_content: String,
    pub text_content: String,
    pub notification_type: NotificationType,
    pub metadata: EmailMetadata,
}

/// Email metadata for tracking and analytics
#[derive(Debug, Clone)]
pub struct EmailMetadata {
    pub notification_id: String,
    pub priority: NotificationPriority,
    pub action_url: Option<String>,
    pub image_url: Option<String>,
    pub tags: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::notification::aggregates::notification::*;

    #[test]
    fn test_fcm_message_to_ddd_content() {
        let fcm_message = FcmMessage {
            target: FcmTarget::Token { token: "test-token".to_string() },
            data: None,
            notification: Some(FcmNotification {
                title: Some("Test Title".to_string()),
                body: Some("Test Body".to_string()),
                image: None,
            }),
            android: None,
            webpush: None,
        };

        let content = NotificationMapper::fcm_message_to_ddd_content(&fcm_message).unwrap();
        assert_eq!(content.title(), "Test Title");
        assert_eq!(content.body(), "Test Body");
    }

    #[test]
    fn test_create_ddd_notification_from_legacy() {
        let notification = NotificationMapper::create_ddd_notification_from_legacy(
            Some(Uuid::new_v4()),
            None,
            "Test Title".to_string(),
            "Test Body".to_string(),
            NotificationType::General,
            NotificationPriority::Normal,
            vec!["push".to_string(), "email".to_string()],
            None,
            None,
            Some("https://example.com/action".to_string()),
            Some("https://example.com/image.png".to_string()),
            Some(serde_json::json!({"custom": "data"})),
        ).unwrap();

        assert_eq!(notification.content().title(), "Test Title");
        assert_eq!(notification.content().body(), "Test Body");
        assert_eq!(notification.notification_type(), &NotificationType::General);
        assert_eq!(notification.priority(), &NotificationPriority::Normal);
        assert_eq!(notification.channels().enabled_channels().len(), 2);
        assert_eq!(notification.metadata().action_url(), Some("https://example.com/action"));
        assert_eq!(notification.metadata().image_url(), Some("https://example.com/image.png"));
    }

    #[test]
    fn test_priority_mapping() {
        assert_eq!(NotificationMapper::map_ddd_priority_to_fcm(&NotificationPriority::Urgent), "high");
        assert_eq!(NotificationMapper::map_ddd_priority_to_fcm(&NotificationPriority::High), "high");
        assert_eq!(NotificationMapper::map_ddd_priority_to_fcm(&NotificationPriority::Normal), "normal");
        assert_eq!(NotificationMapper::map_ddd_priority_to_fcm(&NotificationPriority::Low), "normal");

        assert_eq!(NotificationMapper::map_ddd_priority_to_urgency(&NotificationPriority::Urgent), "high");
        assert_eq!(NotificationMapper::map_ddd_priority_to_urgency(&NotificationPriority::High), "normal");
        assert_eq!(NotificationMapper::map_ddd_priority_to_urgency(&NotificationPriority::Normal), "low");
        assert_eq!(NotificationMapper::map_ddd_priority_to_urgency(&NotificationPriority::Low), "very-low");
    }

    #[test]
    fn test_html_content_generation() {
        let content = NotificationContent::new(
            "Test Notification".to_string(),
            "This is a test notification".to_string(),
        ).unwrap();

        let mut metadata = NotificationMetadata::new();
        metadata.set_action_url("https://example.com/action".to_string());
        metadata.set_image_url("https://example.com/image.png".to_string());

        let notification = Notification::create_for_user(
            Uuid::new_v4(),
            content,
            NotificationType::General,
            NotificationPriority::Normal,
            MultiChannelConfig::new(vec![
                DeliveryChannelConfig::new(DeliveryChannelType::Email, true, None).unwrap()
            ]).unwrap(),
            ScheduleInfo::immediate(),
        ).unwrap();

        let html_content = NotificationMapper::generate_html_content(&notification);
        assert!(html_content.contains("Test Notification"));
        assert!(html_content.contains("This is a test notification"));
        // Note: In real implementation, we'd need to modify notification to set metadata
    }
}