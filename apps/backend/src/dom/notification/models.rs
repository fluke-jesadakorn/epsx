use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::infra::db::diesel::types::{NotificationPriority, NotificationType, DeliveryChannel, DeliveryStatus};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FcmTopic {
    pub id: Uuid,
    pub name: String,
    pub display_name: String,
    pub description: Option<String>,
    pub target_permissions: Option<serde_json::Value>,
    pub is_active: bool,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FcmToken {
    pub id: Uuid,
    pub user_id: Uuid,
    pub token: String,
    pub platform: String,
    pub device_info: Option<serde_json::Value>,
    pub topics: serde_json::Value,
    pub is_active: bool,
    pub last_used_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    pub id: Uuid,
    pub recipient_user_id: Option<Uuid>,
    pub fcm_topic_id: Option<Uuid>,
    pub title: String,
    pub body: String,
    pub notification_type: NotificationType,
    pub priority: NotificationPriority,
    pub channels: Vec<DeliveryChannel>,
    pub data_payload: Option<serde_json::Value>,
    pub image_url: Option<String>,
    pub action_url: Option<String>,
    pub scheduled_at: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationDelivery {
    pub id: Uuid,
    pub notification_id: Uuid,
    pub user_id: Uuid,
    pub channel: DeliveryChannel,
    pub status: DeliveryStatus,
    pub fcm_message_id: Option<String>,
    pub error_message: Option<String>,
    pub delivered_at: Option<DateTime<Utc>>,
    pub read_at: Option<DateTime<Utc>>,
    pub clicked_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserNotificationPreferences {
    pub user_id: Uuid,
    pub fcm_enabled: bool,
    pub in_app_enabled: bool,
    pub email_enabled: bool,
    pub quiet_hours_start: Option<chrono::NaiveTime>,
    pub quiet_hours_end: Option<chrono::NaiveTime>,
    pub timezone: String,
    pub blocked_topics: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// DTOs for API requests/responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateNotificationRequest {
    pub recipient_user_id: Option<Uuid>,
    pub fcm_topic_id: Option<Uuid>,
    pub title: String,
    pub body: String,
    pub notification_type: NotificationType,
    pub priority: NotificationPriority,
    pub channels: Vec<DeliveryChannel>,
    pub data_payload: Option<serde_json::Value>,
    pub image_url: Option<String>,
    pub action_url: Option<String>,
    pub scheduled_at: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateFcmTokenRequest {
    pub token: String,
    pub platform: String,
    pub device_info: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscribeToTopicRequest {
    pub topic_names: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationStats {
    pub total_sent: i64,
    pub total_delivered: i64,
    pub total_failed: i64,
    pub total_pending: i64,
}