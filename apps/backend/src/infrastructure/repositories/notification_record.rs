use uuid::Uuid;
use chrono::{DateTime, Utc};

use crate::domain::notification::*;

/// Database record structure for notifications
/// Maps between domain model and database columns
#[derive(Debug, Clone)]
pub struct NotificationRecord {
    pub id: Uuid,
    pub recipient_wallet_id: Option<Uuid>,
    pub topic_name: Option<String>,
    pub title: String,
    pub body: String,
    pub urgency: String,
    pub notification_type: String,
    pub priority: String,
    pub channels: serde_json::Value,
    pub schedule_type: String,
    pub scheduled_at: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
    pub status: String,
    pub send_started_at: Option<DateTime<Utc>>,
    pub channel_status: serde_json::Value,
    pub total_attempts: i32,
    pub created_by_wallet_id: Option<Uuid>,
    pub image_url: Option<String>,
    pub action_url: Option<String>,
    pub data_payload: Option<serde_json::Value>,
    pub tags: Vec<String>,
    pub notes: Vec<String>,
    pub version: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl NotificationRecord {
    /// Convert domain notification to database record
    pub fn from_domain(notification: &Notification) -> Self {
        Self {
            id: Uuid::parse_str(&notification.id().as_str()).unwrap(),
            recipient_wallet_id: notification.recipientwallet_address(),
            topic_name: notification.topic().map(|t| t.name().to_string()),
            title: notification.content().title().to_string(),
            body: notification.content().body().to_string(),
            urgency: notification.content().urgency().as_str().to_string(),
            notification_type: notification.notification_type().as_str().to_string(),
            priority: notification.priority().as_str().to_string(),
            channels: serde_json::to_value(notification.channels()).unwrap_or_default(),
            schedule_type: notification.schedule().schedule_type().as_str().to_string(),
            scheduled_at: notification.schedule().scheduled_at(),
            expires_at: notification.schedule().expires_at(),
            status: notification.status().as_str().to_string(),
            send_started_at: notification.delivery_tracking().send_started_at(),
            channel_status: serde_json::to_value(notification.delivery_tracking()).unwrap_or_default(),
            total_attempts: notification.delivery_tracking().total_attempts() as i32,
            created_by_wallet_id: notification.metadata().created_by(),
            image_url: notification.metadata().image_url().map(String::from),
            action_url: notification.metadata().action_url().map(String::from),
            data_payload: notification.metadata().data_payload().cloned(),
            tags: notification.metadata().tags().to_vec(),
            notes: notification.metadata().notes().to_vec(),
            version: notification.version() as i64,
            created_at: notification.created_at(),
            updated_at: notification.updated_at(),
        }
    }

    /// Convert database record to domain notification
    pub fn to_domain(self) -> Result<Notification, String> {
        let id = NotificationId::from_string(self.id.to_string());

        let content = NotificationContent::with_urgency(
            self.title,
            self.body,
            ContentUrgency::from_str(&self.urgency)?,
        )?;

        let notification_type = NotificationType::from_str(&self.notification_type)?;
        let priority = NotificationPriority::from_str(&self.priority)?;

        let channels: MultiChannelConfig = serde_json::from_value(self.channels)
            .map_err(|e| format!("Failed to parse channels: {}", e))?;

        let schedule = ScheduleInfo::from_persistence(
            ScheduleType::from_str(&self.schedule_type)?,
            self.scheduled_at,
            self.expires_at,
        )?;

        let mut metadata = if let Some(creator) = self.created_by_wallet_id {
            NotificationMetadata::with_creator(Some(creator))
        } else {
            NotificationMetadata::new()
        };

        if let Some(img) = self.image_url {
            metadata.set_image_url(img);
        }
        if let Some(action) = self.action_url {
            metadata.set_action_url(action);
        }
        if let Some(data) = self.data_payload {
            metadata.set_data_payload(data);
        }
        for tag in self.tags {
            metadata.add_tag(tag);
        }
        for note in self.notes {
            metadata.add_note(note);
        }

        let delivery_tracking: DeliveryTracking = serde_json::from_value(self.channel_status)
            .map_err(|e| format!("Failed to parse delivery tracking: {}", e))?;

        let status = NotificationStatus::from_str(&self.status)?;

        let topic = if let Some(topic_name) = self.topic_name {
            Some(NotificationTopic::from_name(topic_name)?)
        } else {
            None
        };

        Ok(Notification::from_persistence(
            id,
            self.recipient_wallet_id,
            topic,
            content,
            notification_type,
            priority,
            channels,
            schedule,
            metadata,
            delivery_tracking,
            status,
            self.version as u64,
            self.created_at,
            self.updated_at,
        ))
    }
}
