use diesel::prelude::*;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use serde_json::Value as JsonValue;

use crate::infra::db::diesel::schema::{notifications, firebase_sessions};

// Notification models
#[derive(Queryable, Selectable, Insertable, AsChangeset, Debug, Clone, Serialize, Deserialize)]
#[diesel(table_name = notifications)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct DieselNotification {
    pub id: Uuid,
    pub user_id: Uuid,
    pub user_firebase_uid: Option<String>,
    pub title: String,
    pub message: String,
    pub notification_type: String,
    pub priority: String,
    pub is_read: bool,
    pub delivery_status: Option<String>,
    pub delivered_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub metadata: Option<JsonValue>,
}

#[derive(Insertable, Debug, Clone)]
#[diesel(table_name = notifications)]
pub struct NewDieselNotification {
    pub id: Uuid,
    pub user_id: Uuid,
    pub user_firebase_uid: Option<String>,
    pub title: String,
    pub message: String,
    pub notification_type: String,
    pub priority: String,
    pub is_read: bool,
    pub delivery_status: Option<String>,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub metadata: Option<JsonValue>,
}

#[derive(AsChangeset, Debug, Clone)]
#[diesel(table_name = notifications)]
pub struct UpdateDieselNotification {
    pub title: Option<String>,
    pub message: Option<String>,
    pub priority: Option<String>,
    pub is_read: Option<bool>,
    pub delivery_status: Option<String>,
    pub delivered_at: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
    pub metadata: Option<JsonValue>,
}

// Firebase session models
#[derive(Queryable, Selectable, Insertable, AsChangeset, Debug, Clone, Serialize, Deserialize)]
#[diesel(table_name = firebase_sessions)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct DieselFirebaseSession {
    pub id: Uuid,
    pub firebase_uid: String,
    pub session_token: String,
    pub firebase_token_id: Option<String>,
    pub expires_at: DateTime<Utc>,
    pub user_agent: Option<String>,
    pub ip_address: Option<ipnetwork::IpNetwork>,
    pub is_active: Option<bool>,
    pub created_at: Option<DateTime<Utc>>,
    pub last_accessed_at: Option<DateTime<Utc>>,
}

#[derive(Insertable, Debug, Clone)]
#[diesel(table_name = firebase_sessions)]
pub struct NewDieselFirebaseSession {
    pub id: Uuid,
    pub firebase_uid: String,
    pub session_token: String,
    pub firebase_token_id: Option<String>,
    pub expires_at: DateTime<Utc>,
    pub user_agent: Option<String>,
    pub ip_address: Option<ipnetwork::IpNetwork>,
    pub is_active: Option<bool>,
    pub created_at: Option<DateTime<Utc>>,
    pub last_accessed_at: Option<DateTime<Utc>>,
}

#[derive(AsChangeset, Debug, Clone)]
#[diesel(table_name = firebase_sessions)]
pub struct UpdateDieselFirebaseSession {
    pub firebase_token_id: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub user_agent: Option<String>,
    pub is_active: Option<bool>,
    pub last_accessed_at: Option<DateTime<Utc>>,
}

// Query helper structs
#[derive(Debug, Clone, Default)]
pub struct NotificationFilters {
    pub user_id: Option<Uuid>,
    pub user_firebase_uid: Option<String>,
    pub notification_types: Option<Vec<String>>,
    pub priorities: Option<Vec<String>>,
    pub is_read: Option<bool>,
    pub delivery_status: Option<Vec<String>>,
    pub created_after: Option<DateTime<Utc>>,
    pub created_before: Option<DateTime<Utc>>,
    pub limit: Option<i32>,
    pub offset: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationStats {
    pub total_notifications: i64,
    pub unread_count: i64,
    pub critical_count: i64,
    pub today_count: i64,
    pub last_notification_at: Option<DateTime<Utc>>,
}