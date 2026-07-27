//! Web3-first Notification Mappers for SSE Integration
//! Convert between domain notification structures and SSE notification types

use chrono::{DateTime, Utc};

use crate::domain::notification::aggregates::notification::{
    DeliveryResult, DeliveryTracking, Notification, NotificationMetadata, NotificationPriority,
    NotificationStatus,
};
use crate::domain::notification::value_objects::user_preferences::NotificationType;
use crate::domain::notification::value_objects::{
    DeliveryChannel, DeliveryChannelType, MultiChannelConfig, NotificationContent, NotificationId,
    NotificationTopic, ScheduleInfo, ScheduleType,
};
use crate::web::notifications::{
    NotificationPriority as SSENotificationPriority, NotificationType as SSENotificationType,
    SSENotification,
};
// Email service import removed - Web3-first system uses direct wallet notifications

/// Mapper for converting between domain and SSE notification structures
pub struct NotificationMapper;

impl NotificationMapper {
    /// Convert domain notification to SSE notification for real-time delivery
    pub fn convert_domain_to_sse(
        notification: &Notification,
        wallet_address: &str,
    ) -> SSENotification {
        SSENotification {
            id: notification.id().to_string(),
            wallet_address: wallet_address.to_string(),
            notification_type: Self::map_domain_type_to_sse(notification.notification_type()),
            title: notification.content().title().to_string(),
            message: notification.content().body().to_string(),
            data: notification.metadata().data_payload().cloned(),
            priority: Self::map_domain_priority_to_sse(&notification.priority()),
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
    fn map_domain_priority_to_sse(
        domain_priority: &NotificationPriority,
    ) -> SSENotificationPriority {
        match domain_priority {
            NotificationPriority::Low => SSENotificationPriority::Low,
            NotificationPriority::Normal => SSENotificationPriority::Normal,
            NotificationPriority::High => SSENotificationPriority::High,
            NotificationPriority::Urgent => SSENotificationPriority::Critical,
            NotificationPriority::Critical => SSENotificationPriority::Critical,
        }
    }

    // Email notification creation method removed - Web3-first system uses direct wallet notifications

    /// Create a domain notification from a legacy `wallet_notifications` row.
    ///
    /// The legacy table has several names for the same concepts depending on
    /// which API wrote the row (`wallet_address`/`recipient_wallet_address`,
    /// `message`/`body`, and `notification_id`/`id`).  This adapter accepts
    /// those reviewed aliases, but never invents a value or silently falls
    /// back to a new UUID.  Rows that cannot be represented by the Rust
    /// aggregate are rejected so the caller can quarantine them during a
    /// backfill instead of creating a false notification.
    pub fn create_ddd_notification_from_legacy(
        legacy_data: serde_json::Value,
    ) -> Result<Notification, String> {
        let object = legacy_data
            .as_object()
            .ok_or_else(|| "legacy notification must be a JSON object".to_string())?;

        let id = required_string(object, &["id", "notification_id"], "id")?;
        let id = uuid::Uuid::parse_str(&id)
            .map(NotificationId::from_uuid)
            .map_err(|_| "legacy notification id must be a UUID".to_string())?;

        let wallet = optional_string(object, &["recipient_wallet_address", "wallet_address"])?
            .map(|wallet| wallet.trim().to_ascii_lowercase());
        let wallet = wallet
            .map(|wallet| {
                if valid_wallet_or_broadcast(&wallet) {
                    Ok(wallet)
                } else {
                    Err("legacy notification wallet is not a canonical EVM address".to_string())
                }
            })
            .transpose()?;
        let topic_name = optional_string(object, &["topic_name", "topic"])?;
        if wallet.is_none() && topic_name.is_none() {
            return Err("legacy notification needs a wallet or topic target".to_string());
        }
        if wallet.is_some() && topic_name.is_some() {
            return Err("legacy notification cannot mix wallet and topic targets".to_string());
        }
        let topic = match topic_name {
            Some(topic_name) => Some(
                NotificationTopic::from_name(topic_name)
                    .map_err(|error| format!("invalid legacy notification topic: {error}"))?,
            ),
            None if wallet.as_deref() == Some("all") => Some(
                NotificationTopic::broadcast_topic()
                    .map_err(|error| format!("invalid broadcast topic: {error}"))?,
            ),
            None => None,
        };

        let title = required_string(object, &["title"], "title")?;
        let body = required_string(object, &["message", "body"], "message")?;
        let content = NotificationContent::new(title, body)
            .map_err(|error| format!("invalid legacy notification content: {error}"))?;

        let notification_type = optional_string(object, &["notification_type", "type"])?
            .unwrap_or_else(|| "general".to_string());
        let notification_type = legacy_notification_type(&notification_type)?;
        let priority =
            optional_string(object, &["priority"])?.unwrap_or_else(|| "normal".to_string());
        let priority = NotificationPriority::from_str(&priority)?;

        let channels = legacy_channels(object)?;
        let scheduled_at = optional_datetime(object, &["scheduled_at", "schedule_at"])?;
        let expires_at = optional_datetime(object, &["expires_at"])?;
        if scheduled_at
            .zip(expires_at)
            .is_some_and(|(scheduled, expires)| expires <= scheduled)
        {
            return Err("legacy notification expiry must be after its schedule".to_string());
        }
        let schedule_type = scheduled_at
            .map(|_| ScheduleType::Scheduled)
            .unwrap_or(ScheduleType::Immediate);
        let schedule = ScheduleInfo::from_persistence(schedule_type, scheduled_at, expires_at)?;

        let created_at =
            optional_datetime(object, &["created_at", "timestamp"])?.unwrap_or_else(Utc::now);
        let updated_at = optional_datetime(object, &["updated_at"])?.unwrap_or(created_at);
        if updated_at < created_at {
            return Err("legacy notification updated_at precedes created_at".to_string());
        }

        let mut metadata = NotificationMetadata::with_creator(optional_string(
            object,
            &["created_by", "actor_subject"],
        )?);
        if let Some(data) = object
            .get("data")
            .or_else(|| object.get("data_payload"))
            .filter(|data| !data.is_null())
        {
            if !data.is_object() {
                return Err("legacy notification data must be an object".to_string());
            }
            metadata.set_data_payload(data.clone());
        }
        if let Some(action_url) = optional_string(object, &["action_url"])? {
            if !valid_legacy_url(&action_url) {
                return Err("legacy notification action_url is unsafe".to_string());
            }
            metadata.set_action_url(action_url);
        }
        if let Some(image_url) = optional_string(object, &["image_url"])? {
            if !valid_legacy_url(&image_url) {
                return Err("legacy notification image_url is unsafe".to_string());
            }
            metadata.set_image_url(image_url);
        }

        let status = legacy_status(object)?;
        let mut delivery_tracking = DeliveryTracking::new();
        if let Some(delivered_at) = optional_datetime(object, &["delivered_at", "sent_at"])? {
            delivery_tracking.record_attempt(
                "legacy",
                DeliveryResult::Success {
                    delivered_at,
                    message_id: optional_string(object, &["provider_message_id"])?,
                },
            );
        }

        let notification = if let Some(wallet) = wallet {
            if wallet == "all" {
                Notification::create_for_topic(
                    topic
                        .ok_or_else(|| "broadcast legacy notification needs a topic".to_string())?,
                    content,
                    notification_type,
                    priority,
                    channels,
                    schedule,
                    metadata.created_by().map(str::to_owned),
                )?
            } else {
                Notification::create_for_user(
                    wallet,
                    content,
                    notification_type,
                    priority,
                    channels,
                    schedule,
                )?
            }
        } else {
            Notification::create_for_topic(
                topic.ok_or_else(|| "legacy notification topic is missing".to_string())?,
                content,
                notification_type,
                priority,
                channels,
                schedule,
                metadata.created_by().map(str::to_owned),
            )?
        };

        // Creation helpers intentionally generate a new aggregate identity and
        // timestamps. Rehydrate the reviewed legacy identity/metadata/status
        // only after all input validation has succeeded.
        Ok(Notification::from_persistence(
            id,
            notification.recipient_wallet_address().map(str::to_owned),
            notification.topic().cloned(),
            notification.content().clone(),
            notification.notification_type().clone(),
            notification.priority(),
            notification.channels().clone(),
            notification.schedule().clone(),
            metadata,
            delivery_tracking,
            status,
            1,
            created_at,
            updated_at,
        ))
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

fn required_string(
    object: &serde_json::Map<String, serde_json::Value>,
    aliases: &[&str],
    label: &str,
) -> Result<String, String> {
    optional_string(object, aliases)?
        .ok_or_else(|| format!("legacy notification {label} is missing"))
}

fn optional_string(
    object: &serde_json::Map<String, serde_json::Value>,
    aliases: &[&str],
) -> Result<Option<String>, String> {
    let mut value = None;
    let mut seen = false;
    for alias in aliases {
        if let Some(raw) = object.get(*alias) {
            if seen {
                return Err(format!(
                    "legacy notification contains duplicate aliases for {alias}"
                ));
            }
            seen = true;
            if !raw.is_null() {
                value = Some(
                    raw.as_str()
                        .ok_or_else(|| format!("legacy notification {alias} must be a string"))?
                        .to_string(),
                );
            }
        }
    }
    Ok(value)
}

fn optional_datetime(
    object: &serde_json::Map<String, serde_json::Value>,
    aliases: &[&str],
) -> Result<Option<DateTime<Utc>>, String> {
    optional_string(object, aliases)?
        .map(|value| {
            DateTime::parse_from_rfc3339(&value)
                .map(|parsed| parsed.with_timezone(&Utc))
                .map_err(|_| format!("legacy notification timestamp is not RFC3339: {value}"))
        })
        .transpose()
}

fn valid_wallet_or_broadcast(value: &str) -> bool {
    value == "all"
        || (value.len() == 42
            && value.starts_with("0x")
            && value[2..].bytes().all(|byte| byte.is_ascii_hexdigit()))
}

fn legacy_notification_type(value: &str) -> Result<NotificationType, String> {
    match value.to_ascii_lowercase().as_str() {
        "wallet_management" | "wallet" => Ok(NotificationType::Feature),
        "payment" => Ok(NotificationType::Info),
        "announcement" => Ok(NotificationType::System),
        "advertisement" => Ok(NotificationType::Marketing),
        "chat" => Ok(NotificationType::General),
        value => NotificationType::from_str(value),
    }
}

fn legacy_channels(
    object: &serde_json::Map<String, serde_json::Value>,
) -> Result<MultiChannelConfig, String> {
    let values = if let Some(raw) = object.get("channels") {
        raw.as_array()
            .ok_or_else(|| "legacy notification channels must be an array".to_string())?
            .iter()
            .map(|value| {
                value
                    .as_str()
                    .ok_or_else(|| "legacy notification channel must be a string".to_string())
                    .and_then(DeliveryChannelType::from_str)
            })
            .collect::<Result<Vec<_>, _>>()?
    } else if let Some(channel) = optional_string(object, &["channel"])? {
        vec![DeliveryChannelType::from_str(&channel)?]
    } else {
        vec![DeliveryChannelType::InApp]
    };
    if values.is_empty() {
        return Err("legacy notification needs at least one channel".to_string());
    }
    Ok(MultiChannelConfig::new(
        values.into_iter().map(DeliveryChannel::new).collect(),
    ))
}

fn legacy_status(
    object: &serde_json::Map<String, serde_json::Value>,
) -> Result<NotificationStatus, String> {
    let value = optional_string(object, &["status"])?
        .unwrap_or_else(|| "created".to_string())
        .to_ascii_lowercase();
    match value.as_str() {
        "pending" => Ok(NotificationStatus::Queued),
        "sent" | "delivered" | "read" => Ok(NotificationStatus::Delivered),
        value => NotificationStatus::from_str(value),
    }
}

fn valid_legacy_url(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 2048
        && !value
            .chars()
            .any(|character| character.is_control() || character.is_whitespace())
        && !value.contains('\\')
        && (value.starts_with('/') && !value.starts_with("//") || value.starts_with("https://"))
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

    #[test]
    fn legacy_row_maps_to_a_rust_notification_without_inventing_identity() {
        let mapped = NotificationMapper::create_ddd_notification_from_legacy(serde_json::json!({
            "id": "00000000-0000-4000-8000-000000000001",
            "wallet_address": "0x1111111111111111111111111111111111111111",
            "title": "Payment received",
            "message": "Your payment was accepted.",
            "notification_type": "payment",
            "priority": "high",
            "channel": "in_app",
            "status": "pending",
            "created_at": "2026-01-01T00:00:00Z",
            "updated_at": "2026-01-01T00:01:00Z",
            "expires_at": "2026-01-02T00:00:00Z",
            "action_url": "/payments/1",
            "data": {"payment_id": "p-1"}
        }))
        .expect("reviewed legacy row should map");

        assert_eq!(mapped.id().as_str(), "00000000-0000-4000-8000-000000000001");
        assert_eq!(
            mapped.recipient_wallet_address(),
            Some("0x1111111111111111111111111111111111111111")
        );
        assert_eq!(mapped.notification_type(), &NotificationType::Info);
        assert_eq!(mapped.priority(), NotificationPriority::High);
        assert_eq!(
            mapped.schedule().expires_at().unwrap().to_rfc3339(),
            "2026-01-02T00:00:00+00:00"
        );
        assert_eq!(mapped.metadata().action_url(), Some("/payments/1"));
        assert_eq!(
            mapped.metadata().data_payload(),
            Some(&serde_json::json!({"payment_id": "p-1"}))
        );
    }

    #[test]
    fn legacy_mapper_rejects_malformed_identity_and_unsafe_urls() {
        let malformed = serde_json::json!({
            "id": "not-a-uuid",
            "wallet_address": "0x1111111111111111111111111111111111111111",
            "title": "Title",
            "message": "Body"
        });
        assert!(NotificationMapper::create_ddd_notification_from_legacy(malformed).is_err());

        let unsafe_url = serde_json::json!({
            "id": "00000000-0000-4000-8000-000000000002",
            "wallet_address": "0x1111111111111111111111111111111111111111",
            "title": "Title",
            "message": "Body",
            "action_url": "javascript:alert(1)"
        });
        assert!(NotificationMapper::create_ddd_notification_from_legacy(unsafe_url).is_err());
    }

    #[test]
    fn legacy_broadcast_row_gets_the_explicit_broadcast_topic() {
        let mapped = NotificationMapper::create_ddd_notification_from_legacy(serde_json::json!({
            "id": "00000000-0000-4000-8000-000000000003",
            "wallet_address": "all",
            "title": "Maintenance",
            "message": "The service will restart shortly.",
            "notification_type": "announcement",
            "priority": "normal"
        }))
        .expect("broadcast legacy row should map");

        assert_eq!(mapped.recipient_wallet_address(), None);
        assert_eq!(
            mapped.topic().map(NotificationTopic::name),
            Some("broadcast")
        );
    }
}
