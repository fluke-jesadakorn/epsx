//! Database models for notifications (sqlx-friendly).
//!
//! BIG-BANG: migrated from diesel to plain sqlx structs.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Row model for wallet_notifications table.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
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

/// Insert model for wallet_notifications.
#[derive(Debug, Clone, Deserialize)]
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
