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

// wave10(track-c): the two `notification_subscriptions` models
// (`NotificationSubscriptionDb`, `NewNotificationSubscriptionDb`)
// that used to live in this file as a `/* … */` block of
// commented-out code have been removed. The underlying table is
// dropped in
// `apps/backend/migrations/notifications/20260613000000_drop_notification_subscriptions/up.sql`.
// See the deliverable.md for the rg evidence (no live INSERT
// or SELECT in apps/backend/src/).
