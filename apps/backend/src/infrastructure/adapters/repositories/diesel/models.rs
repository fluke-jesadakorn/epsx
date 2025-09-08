// Database models for Diesel ORM
use serde::{Deserialize, Serialize};
use diesel::prelude::*;


/// Diesel-specific user model for queries
#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Identifiable, Selectable)]
#[diesel(table_name = crate::infrastructure::adapters::repositories::diesel::schema::users)]
pub struct DieselUser {
    pub id: uuid::Uuid,
    pub firebase_uid: String,
    pub email: String,
    pub display_name: Option<String>,
    pub name: Option<String>,
    pub avatar_url: Option<String>,
    pub package_tier: Option<String>,
    pub email_verified: Option<bool>,
    pub is_active: Option<bool>,
    pub last_login_at: Option<chrono::DateTime<chrono::Utc>>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub primary_platform_id: Option<uuid::Uuid>,
}

/// New user model for insertions
#[derive(Debug, Clone, Serialize, Deserialize, Insertable)]
#[diesel(table_name = crate::infrastructure::adapters::repositories::diesel::schema::users)]
pub struct NewDieselUser {
    pub id: uuid::Uuid,
    pub firebase_uid: String,
    pub email: String,
    pub display_name: Option<String>,
    pub name: Option<String>,
    pub avatar_url: Option<String>,
    pub package_tier: Option<String>,
    pub email_verified: Option<bool>,
    pub is_active: Option<bool>,
    pub primary_platform_id: Option<uuid::Uuid>,
}

/// Update user model for modifications
#[derive(Debug, Clone, Serialize, Deserialize, AsChangeset)]
#[diesel(table_name = crate::infrastructure::adapters::repositories::diesel::schema::users)]
pub struct UpdateDieselUser {
    pub firebase_uid: Option<String>,
    pub email: Option<String>,
    pub display_name: Option<String>,
    pub name: Option<String>,
    pub avatar_url: Option<String>,
    pub package_tier: Option<String>,
    pub email_verified: Option<bool>,
    pub is_active: Option<bool>,
    pub last_login_at: Option<chrono::DateTime<chrono::Utc>>,
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
    pub primary_platform_id: Option<uuid::Uuid>,
}

/// Session database model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: uuid::Uuid,
    pub user_id: uuid::Uuid,
    pub access_token: String,
    pub expires_at: chrono::DateTime<chrono::Utc>,
    pub provider: Option<String>,
    pub session_token: Option<String>,
    pub user_agent: Option<String>,
    pub ip_address: Option<crate::infrastructure::adapters::repositories::diesel::types::DieselIpAddr>,
    pub is_active: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Diesel-specific session model for queries
#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Identifiable, Selectable)]
#[diesel(table_name = crate::infrastructure::adapters::repositories::diesel::schema::sessions)]
pub struct DieselSession {
    pub id: uuid::Uuid,
    pub user_id: uuid::Uuid,
    pub access_token: String,
    pub expires_at: chrono::DateTime<chrono::Utc>,
    pub provider: Option<String>,
    pub session_token: Option<String>,
    pub user_agent: Option<String>,
    pub ip_address: Option<crate::infrastructure::adapters::repositories::diesel::types::DieselIpAddr>,
    pub is_active: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// New session model for insertions
#[derive(Debug, Clone, Serialize, Deserialize, Insertable)]
#[diesel(table_name = crate::infrastructure::adapters::repositories::diesel::schema::sessions)]
pub struct NewDieselSession {
    pub id: uuid::Uuid,
    pub user_id: uuid::Uuid,
    pub access_token: String,
    pub expires_at: chrono::DateTime<chrono::Utc>,
    pub provider: Option<String>,
    pub session_token: Option<String>,
    pub user_agent: Option<String>,
    pub ip_address: Option<crate::infrastructure::adapters::repositories::diesel::types::DieselIpAddr>,
    pub is_active: bool,
}

/// Update session model for modifications
#[derive(Debug, Clone, Serialize, Deserialize, AsChangeset)]
#[diesel(table_name = crate::infrastructure::adapters::repositories::diesel::schema::sessions)]
pub struct UpdateDieselSession {
    pub access_token: Option<String>,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
    pub provider: Option<String>,
    pub session_token: Option<String>,
    pub user_agent: Option<String>,
    pub ip_address: Option<crate::infrastructure::adapters::repositories::diesel::types::DieselIpAddr>,
    pub is_active: Option<bool>,
}

/// Notification database model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    pub id: String,
    pub user_id: String,
    pub notification_type: String,
    pub title: String,
    pub body: String,
    pub data: Option<serde_json::Value>,
    pub is_read: bool,
    pub created_at: chrono::NaiveDateTime,
}

/// Refresh token database model
#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Identifiable, Selectable)]
#[diesel(table_name = crate::infrastructure::adapters::repositories::diesel::schema::refresh_tokens)]
pub struct RefreshToken {
    pub id: uuid::Uuid,
    pub user_id: String,
    pub token_hash: String,
    pub family_id: uuid::Uuid,
    pub expires_at: chrono::DateTime<chrono::Utc>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub last_used_at: Option<chrono::DateTime<chrono::Utc>>,
    pub device_info: Option<serde_json::Value>,
    pub ip_address: Option<crate::infrastructure::adapters::repositories::diesel::types::DieselIpAddr>,
    pub user_agent: Option<String>,
    pub is_revoked: bool,
    pub revoked_at: Option<chrono::DateTime<chrono::Utc>>,
    pub revoked_reason: Option<String>,
}

/// New refresh token for insertions
#[derive(Debug, Clone, Serialize, Deserialize, Insertable)]
#[diesel(table_name = crate::infrastructure::adapters::repositories::diesel::schema::refresh_tokens)]
pub struct NewRefreshToken {
    pub id: uuid::Uuid,
    pub user_id: String,
    pub token_hash: String,
    pub family_id: uuid::Uuid,
    pub expires_at: chrono::DateTime<chrono::Utc>,
    pub device_info: Option<serde_json::Value>,
    pub ip_address: Option<crate::infrastructure::adapters::repositories::diesel::types::DieselIpAddr>,
    pub user_agent: Option<String>,
    pub is_revoked: bool,
}

/// Update refresh token for modifications
#[derive(Debug, Clone, Serialize, Deserialize, AsChangeset)]
#[diesel(table_name = crate::infrastructure::adapters::repositories::diesel::schema::refresh_tokens)]
pub struct UpdateRefreshToken {
    pub last_used_at: Option<chrono::DateTime<chrono::Utc>>,
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
    pub device_info: Option<serde_json::Value>,
    pub ip_address: Option<crate::infrastructure::adapters::repositories::diesel::types::DieselIpAddr>,
    pub user_agent: Option<String>,
    pub is_revoked: Option<bool>,
    pub revoked_at: Option<chrono::DateTime<chrono::Utc>>,
    pub revoked_reason: Option<String>,
}

/// Revoked token for tracking
#[derive(Debug, Clone, Serialize, Deserialize, Insertable)]
#[diesel(table_name = crate::infrastructure::adapters::repositories::diesel::schema::revoked_tokens)]
pub struct NewRevokedToken {
    pub id: uuid::Uuid,
    pub jti: String,
    pub user_id: String,
    pub token_type: String,
    pub expires_at: chrono::DateTime<chrono::Utc>,
    pub revoked_at: chrono::DateTime<chrono::Utc>,
    pub revoked_by: Option<String>,
    pub revoked_reason: String,
}

/// User dynamic limit model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DieselUserDynamicLimit {
    pub user_id: String,
    pub limit_type: String,
    pub limit_value: i64,
    pub expires_at: Option<chrono::NaiveDateTime>,
}

/// Resolved user limits for aggregation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolvedUserLimits {
    pub user_id: String,
    pub daily_limit: Option<i64>,
    pub weekly_limit: Option<i64>,
    pub monthly_limit: Option<i64>,
    pub total_limit: Option<i64>,
}

/// Source of a limit (tier, custom, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LimitSource {
    Tier,
    Custom,
    Temporary,
}