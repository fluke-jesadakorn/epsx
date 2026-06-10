//! Diesel models for notifications
use chrono::{DateTime, Utc};
use diesel::{Queryable, Selectable, Insertable};
use uuid::Uuid;
use serde::{Deserialize, Serialize};

/// Diesel Queryable model for notifications table
#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = crate::schemas::notifications::wallet_notifications)]
pub struct WalletNotificationDb {
    pub id: Uuid,
    pub recipient_wallet_address: Option<String>,
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
    pub created_by: Option<String>,
    pub image_url: Option<String>,
    pub action_url: Option<String>,
    pub data_payload: Option<serde_json::Value>,
    pub tags: Option<Vec<String>>,
    pub notes: Option<String>,
    pub version: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Diesel Insertable model for creating new notifications
#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = crate::schemas::notifications::wallet_notifications)]
pub struct NewWalletNotificationDb {
    pub id: Uuid,
    pub recipient_wallet_address: Option<String>,
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
    pub created_by: Option<String>,
    pub image_url: Option<String>,
    pub action_url: Option<String>,
    pub data_payload: Option<serde_json::Value>,
    pub tags: Option<Vec<String>>,
    pub notes: Option<String>,
    pub version: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/*
/// Diesel Queryable model for notification_subscriptions table
#[derive(Debug, Clone, Queryable, Selectable)]
#[diesel(table_name = crate::schemas::notifications::notification_subscriptions)]
pub struct NotificationSubscriptionDb {
    pub id: Uuid,
    pub wallet_address: String,
    pub instance_id: String,
    pub connection_id: String,
    pub connected_at: DateTime<Utc>,
    pub last_ping_at: DateTime<Utc>,
    pub disconnected_at: Option<DateTime<Utc>>,
    pub user_agent: Option<String>,
    pub ip_address: Option<String>, // Using wrapper for INET
    pub redis_channel: Option<String>,
}

/// Diesel Insertable model for creating new notification subscriptions
#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = crate::schemas::notifications::notification_subscriptions)]
pub struct NewNotificationSubscriptionDb {
    pub wallet_address: String,
    pub instance_id: String,
    pub connection_id: String,
    pub connected_at: DateTime<Utc>,
    pub last_ping_at: DateTime<Utc>,
    pub user_agent: Option<String>,
    // IP address handling might require custom types or raw SQL insert
    pub redis_channel: Option<String>,
}
*/
