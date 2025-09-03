use chrono::{DateTime, Utc, NaiveTime};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::infra::db::diesel::schema::{notifications, fcm_tokens, fcm_topics, notification_deliveries, user_notification_preferences};
use crate::infra::db::diesel::types::{NotificationPriority, NotificationType, DeliveryChannel, DeliveryStatus};

#[derive(Queryable, Selectable, Insertable, AsChangeset, Debug, Clone, Serialize, Deserialize)]
#[diesel(table_name = notifications)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct DieselNotification {
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

#[derive(Insertable, Debug)]
#[diesel(table_name = notifications)]
pub struct NewDieselNotification {
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

#[derive(Queryable, Selectable, Insertable, AsChangeset, Debug, Clone, Serialize, Deserialize)]
#[diesel(table_name = fcm_tokens)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct DieselFcmToken {
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

#[derive(Insertable, Debug)]
#[diesel(table_name = fcm_tokens)]
pub struct NewDieselFcmToken {
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

#[derive(Queryable, Selectable, Insertable, AsChangeset, Debug, Clone, Serialize, Deserialize)]
#[diesel(table_name = fcm_topics)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct DieselFcmTopic {
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

#[derive(Queryable, Selectable, Insertable, AsChangeset, Debug, Clone, Serialize, Deserialize)]
#[diesel(table_name = notification_deliveries)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct DieselNotificationDelivery {
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

#[derive(Insertable, Debug)]
#[diesel(table_name = notification_deliveries)]
pub struct NewDieselNotificationDelivery {
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

#[derive(Queryable, Selectable, Insertable, AsChangeset, Debug, Clone, Serialize, Deserialize)]
#[diesel(table_name = user_notification_preferences)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct DieselUserNotificationPreferences {
    pub user_id: Uuid,
    pub fcm_enabled: bool,
    pub in_app_enabled: bool,
    pub email_enabled: bool,
    pub quiet_hours_start: Option<NaiveTime>,
    pub quiet_hours_end: Option<NaiveTime>,
    pub timezone: String,
    pub blocked_topics: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}