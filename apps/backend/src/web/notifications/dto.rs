use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// Temporarily use strings instead of enum types

#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterFcmTokenRequest {
    pub token: String,
    pub platform: String,
    pub device_info: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterFcmTokenResponse {
    pub id: Uuid,
    pub message: String,
    pub subscribed_topics: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SendNotificationRequest {
    pub recipientuser_id: Option<Uuid>,
    pub fcm_topic_id: Option<String>,
    pub title: String,
    pub body: String,
    pub notification_type: String,
    pub priority: String,
    pub channels: Vec<String>,
    pub data_payload: Option<serde_json::Value>,
    pub image_url: Option<String>,
    pub action_url: Option<String>,
    pub scheduled_at: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SendNotificationResponse {
    pub id: Uuid,
    pub message: String,
    pub recipient_count: Option<u32>,
    pub delivery_ids: Vec<Uuid>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BroadcastRequest {
    pub topic: String,
    pub title: String,
    pub body: String,
    pub data: Option<serde_json::Value>,
    pub priority: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BroadcastResponse {
    pub message_id: String,
    pub topic: String,
    pub sent_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TrackNotificationRequest {
    pub notification_id: Uuid,
    pub action: String, // "received", "clicked", "dismissed"
    pub timestamp: DateTime<Utc>,
    pub context: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TrackNotificationResponse {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NotificationStatsResponse {
    pub total_sent: i64,
    pub total_delivered: i64,
    pub total_failed: i64,
    pub total_pending: i64,
    pub delivery_rate: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserNotificationsResponse {
    pub notifications: Vec<UserNotification>,
    pub total_count: i64,
    pub unread_count: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserNotification {
    pub id: Uuid,
    pub title: String,
    pub body: String,
    pub notification_type: String,
    pub priority: String,
    pub image_url: Option<String>,
    pub action_url: Option<String>,
    pub data_payload: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub read_at: Option<DateTime<Utc>>,
    pub clicked_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdatePreferencesRequest {
    pub fcm_enabled: Option<bool>,
    pub in_app_enabled: Option<bool>,
    pub email_enabled: Option<bool>,
    pub quiet_hours_start: Option<String>, // HH:MM format
    pub quiet_hours_end: Option<String>,   // HH:MM format
    pub timezone: Option<String>,
    pub blocked_topics: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NotificationPreferencesResponse {
    pub fcm_enabled: bool,
    pub in_app_enabled: bool,
    pub email_enabled: bool,
    pub quiet_hours_start: Option<String>,
    pub quiet_hours_end: Option<String>,
    pub timezone: String,
    pub blocked_topics: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TopicSubscriptionRequest {
    pub topics: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TopicSubscriptionResponse {
    pub subscribed: Vec<String>,
    pub failed: Vec<String>,
    pub message: String,
}