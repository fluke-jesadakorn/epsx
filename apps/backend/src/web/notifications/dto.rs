// Data Transfer Objects for notification API
use chrono::{DateTime, Utc};

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Response for notification listing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationListResponse {
    pub notifications: Vec<NotificationResponse>,
    pub pagination: PaginationResponse,
    pub unread_count: u64,
    pub total_count: u64,
    pub fetched_at: DateTime<Utc>,
}

/// Individual notification response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationResponse {
    pub id: String,
    pub user_id: String,
    pub title: String,
    pub message: String,
    pub notification_type: String,
    pub category: String,
    pub priority: String,
    pub status: String,
    pub channel: String,
    pub metadata: Option<Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
    pub read_at: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
    pub scheduled_for: Option<DateTime<Utc>>,
    pub sent_at: Option<DateTime<Utc>>,
    pub delivery_status: Option<String>,
    pub error_message: Option<String>,
    pub retry_count: i32,
}

/// Pagination metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationResponse {
    pub page: u64,
    pub per_page: u64,
    pub total_pages: u64,
    pub has_next: bool,
    pub has_prev: bool,
}

/// Request for marking notifications as read
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarkNotificationsReadRequest {
    pub notification_ids: Vec<String>,
    pub mark_all: Option<bool>,
}

/// Response for mark as read operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarkNotificationsReadResponse {
    pub user_id: String,
    pub marked_count: u64,
    pub notification_ids: Vec<String>,
    pub mark_all: bool,
    pub marked_at: DateTime<Utc>,
}

/// Request for updating notification preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationPreferencesRequest {
    pub email_enabled: Option<bool>,
    pub push_enabled: Option<bool>,
    pub in_app_enabled: Option<bool>,
    pub categories: Option<Vec<CategoryPreference>>,
    pub quiet_hours: Option<QuietHours>,
}

/// Category-specific preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryPreference {
    pub category: String,
    pub enabled: bool,
    pub channels: Vec<String>, // email, push, in_app
    pub min_priority: String,  // low, normal, high, critical
}

/// Quiet hours configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuietHours {
    pub enabled: bool,
    pub start_time: String, // "22:00"
    pub end_time: String,   // "08:00"
    pub timezone: String,   // "America/New_York"
    pub days: Vec<String>,  // ["monday", "tuesday", ...]
}

/// Request for registering FCM device token
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceTokenRequest {
    pub token: String,
    pub device_type: String, // "android", "ios", "web"
    pub device_id: Option<String>,
    pub app_version: Option<String>,
}

/// Response for device token registration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceTokenResponse {
    pub user_id: String,
    pub token_id: String,
    pub device_type: String,
    pub registered_at: DateTime<Utc>,
    pub status: String,
}

/// Request for creating admin notification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateNotificationRequest {
    pub title: String,
    pub message: String,
    pub notification_type: String,
    pub category: String,
    pub priority: String,
    pub target_users: Option<Vec<String>>, // None for broadcast
    pub channels: Vec<String>, // email, push, in_app
    pub metadata: Option<Value>,
    pub expires_at: Option<DateTime<Utc>>,
    pub scheduled_for: Option<DateTime<Utc>>,
    pub template_id: Option<String>,
    pub template_data: Option<Value>,
}

/// Response for notification creation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateNotificationResponse {
    pub notification_id: String,
    pub target_count: u64,
    pub created_at: DateTime<Utc>,
    pub scheduled_for: Option<DateTime<Utc>>,
    pub status: String,
}

/// Notification query parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationQuery {
    pub page: Option<u64>,
    pub per_page: Option<u64>,
    pub status: Option<String>, // read, unread, all
    pub category: Option<String>,
    pub priority: Option<String>,
    pub from_date: Option<DateTime<Utc>>,
    pub to_date: Option<DateTime<Utc>>,
    pub notification_type: Option<String>,
}

impl Default for NotificationQuery {
    fn default() -> Self {
        Self {
            page: Some(1),
            per_page: Some(20),
            status: Some("all".to_string()),
            category: None,
            priority: None,
            from_date: None,
            to_date: None,
            notification_type: None,
        }
    }
}

/// Unread count response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnreadCountResponse {
    pub user_id: String,
    pub unread_count: u64,
    pub by_category: Vec<CategoryCount>,
    pub by_priority: Vec<PriorityCount>,
    pub last_checked: DateTime<Utc>,
}

/// Category count breakdown
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryCount {
    pub category: String,
    pub count: u64,
}

/// Priority count breakdown
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriorityCount {
    pub priority: String,
    pub count: u64,
}

/// Notification preferences response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationPreferencesResponse {
    pub user_id: String,
    pub email_enabled: bool,
    pub push_enabled: bool,
    pub in_app_enabled: bool,
    pub categories: Vec<CategoryPreference>,
    pub quiet_hours: Option<QuietHours>,
    pub updated_at: DateTime<Utc>,
}

/// Error response for API endpoints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationErrorResponse {
    pub error: String,
    pub code: String,
    pub message: String,
    pub details: Option<Value>,
    pub timestamp: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_serialize_notification_response() {
        let notification = NotificationResponse {
            id: "notif_123".to_string(),
            user_id: "user_456".to_string(),
            title: "Test Notification".to_string(),
            message: "This is a test".to_string(),
            notification_type: "system".to_string(),
            category: "general".to_string(),
            priority: "normal".to_string(),
            status: "unread".to_string(),
            channel: "in_app".to_string(),
            metadata: None,
            created_at: Utc::now(),
            updated_at: None,
            read_at: None,
            expires_at: None,
            scheduled_for: None,
            sent_at: None,
            delivery_status: None,
            error_message: None,
            retry_count: 0,
        };

        let json = serde_json::to_string(&notification).unwrap();
        assert!(json.contains("notif_123"));
        assert!(json.contains("Test Notification"));
    }

    #[test]
    fn should_deserialize_mark_read_request() {
        let json = r#"{"notification_ids":["1","2","3"],"mark_all":false}"#;
        let request: MarkNotificationsReadRequest = serde_json::from_str(json).unwrap();
        
        assert_eq!(request.notification_ids.len(), 3);
        assert_eq!(request.mark_all, Some(false));
    }

    #[test]
    fn should_create_default_query() {
        let query = NotificationQuery::default();
        assert_eq!(query.page, Some(1));
        assert_eq!(query.per_page, Some(20));
        assert_eq!(query.status, Some("all".to_string()));
    }
}