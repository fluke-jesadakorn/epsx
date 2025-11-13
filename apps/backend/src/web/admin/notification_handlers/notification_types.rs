// Notification request/response types
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use crate::web::notifications::{NotificationType, NotificationPriority};

// ============================================================================
// REQUEST TYPES
// ============================================================================

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct SendNotificationRequest {
    pub recipient_wallet_address: Option<String>,
    pub recipient_group: Option<String>,
    pub broadcast: Option<bool>,
    pub notification_type: NotificationType,
    pub priority: NotificationPriority,
    pub title: String,
    pub message: String,
    pub data: Option<serde_json::Value>,
    pub action_url: Option<String>,
    pub image_url: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub schedule_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize)]
pub struct NotificationFilters {
    pub page: Option<u32>,
    pub limit: Option<u32>,
    #[serde(rename = "type")]
    pub notification_type: Option<String>,
    pub priority: Option<String>,
    pub status: Option<String>,
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
    pub wallet_address: Option<String>,
}

// ============================================================================
// RESPONSE TYPES
// ============================================================================

#[derive(Debug, Serialize)]
pub struct SendNotificationResponse {
    pub success: bool,
    pub data: SendNotificationData,
    pub message: String,
    pub api_version: String,
}

#[derive(Debug, Serialize)]
pub struct SendNotificationData {
    pub notification_id: String,
    pub recipients_count: usize,
    pub scheduled: bool,
    pub delivery_status: String,
}

#[derive(Debug, Serialize)]
pub struct NotificationsResponse {
    pub success: bool,
    pub data: NotificationsData,
    pub api_version: String,
    pub access_level: String,
}

#[derive(Debug, Serialize)]
pub struct NotificationsData {
    pub notifications: Vec<NotificationDto>,
    pub total_count: usize,
    pub unread_count: usize,
    pub page: u32,
    pub limit: u32,
    pub total_pages: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct NotificationDto {
    pub id: String,
    pub wallet_address: String,
    pub notification_type: String,
    pub title: String,
    pub message: String,
    pub data: Option<serde_json::Value>,
    pub priority: String,
    pub timestamp: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub read_at: Option<DateTime<Utc>>,
    pub clicked_at: Option<DateTime<Utc>>,
    pub delivered_at: Option<DateTime<Utc>>,
    pub action_url: Option<String>,
    pub image_url: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct NotificationStatsResponse {
    pub success: bool,
    pub data: NotificationStats,
    pub api_version: String,
    pub access_level: String,
}

#[derive(Debug, Serialize)]
pub struct NotificationStats {
    pub total_notifications: usize,
    pub sent_today: usize,
    pub sent_this_week: usize,
    pub sent_this_month: usize,
    pub delivery_rate: f64,
    pub read_rate: f64,
    pub click_rate: f64,
    pub by_type: serde_json::Value,
    pub by_priority: serde_json::Value,
    pub recent_activity: Vec<RecentActivity>,
}

#[derive(Debug, Serialize)]
pub struct RecentActivity {
    pub timestamp: DateTime<Utc>,
    pub action: String,
    pub count: usize,
}
