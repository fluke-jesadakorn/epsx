//! Diesel models for notifications
use chrono::{DateTime, Utc};
use diesel::{Queryable, Selectable, Insertable, AsChangeset};
use uuid::Uuid;
use serde::{Deserialize, Serialize};

/// Diesel Queryable model for wallet_notifications table
#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = crate::schemas::notifications::wallet_notifications)]
pub struct WalletNotificationDb {
    pub id: Uuid,
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
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub queued_at: Option<DateTime<Utc>>,
    pub delivery_attempts: Option<i32>,
    pub last_delivery_attempt_at: Option<DateTime<Utc>>,
    pub delivery_error: Option<String>,
    pub acknowledged_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// Diesel Insertable model for creating new wallet notifications
#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = crate::schemas::notifications::wallet_notifications)]
pub struct NewWalletNotificationDb {
    pub wallet_address: String,
    pub notification_type: String,
    pub title: String,
    pub message: String,
    pub data: Option<serde_json::Value>,
    pub priority: String,
    pub timestamp: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub action_url: Option<String>,
    pub image_url: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Diesel AsChangeset model for updating wallet notifications
#[derive(Debug, Clone, AsChangeset)]
#[diesel(table_name = crate::schemas::notifications::wallet_notifications)]
pub struct UpdateWalletNotificationDb {
    pub read_at: Option<DateTime<Utc>>,
    pub clicked_at: Option<DateTime<Utc>>,
    pub delivered_at: Option<DateTime<Utc>>,
    pub queued_at: Option<DateTime<Utc>>,
    pub delivery_attempts: Option<i32>,
    pub last_delivery_attempt_at: Option<DateTime<Utc>>,
    pub delivery_error: Option<String>,
    pub acknowledged_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub updated_at: DateTime<Utc>,
}

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
