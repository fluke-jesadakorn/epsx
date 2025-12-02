// Database Types and Models
// Unified type definitions for database operations

use chrono::{DateTime, Utc};
use uuid::Uuid;
use std::sync::Arc;
use diesel_async::{AsyncPgConnection, pooled_connection::deadpool::Pool};
use bigdecimal::BigDecimal;

// Database Pool Types
pub type DbPool = &'static Pool<AsyncPgConnection>;

// Session Types
#[derive(Clone)]
pub struct SessionRepository {
    _pool: Arc<DbPool>,
}

impl SessionRepository {
    pub fn new(pool: Arc<DbPool>) -> Self {
        Self { _pool: pool }
    }
    
    pub async fn save_session(&self, _session_data: &str) -> Result<(), String> {
        Ok(())
    }
    
    pub async fn save(&self, _session: &crate::domain::wallet_management::aggregates::session::Session) -> Result<(), String> {
        // Session storage placeholder
        // Future: Insert into wallet_sessions table with session data
        Ok(())
    }
}

// User Repository Types
#[derive(Clone)]
pub struct UserRepository {
    _pool: Arc<DbPool>,
}

impl UserRepository {
    pub fn new(pool: Arc<DbPool>) -> Self {
        Self { _pool: pool }
    }
}

// Token Types
#[derive(Clone)]
pub struct RefreshTokenRepository {
    _pool: Arc<DbPool>,
}

impl RefreshTokenRepository {
    pub fn new(pool: Arc<DbPool>) -> Self {
        Self { _pool: pool }
    }
}

#[derive(Clone)]
pub struct RevokedTokenRepository {
    _pool: Arc<DbPool>,
}

impl RevokedTokenRepository {
    pub fn new(pool: Arc<DbPool>) -> Self {
        Self { _pool: pool }
    }
}

#[derive(Debug, Clone)]
pub struct RefreshToken {
    pub id: Uuid,
    pub token: String,
    pub wallet_address: Uuid,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct NewRefreshToken {
    pub token: String,
    pub wallet_address: Uuid,
    pub expires_at: DateTime<Utc>,
}

// User limits types have been moved to domain/shared_kernel/value_objects/user_limits.rs
// for proper clean architecture separation

// Notification Types
#[derive(Clone)]
pub struct NotificationRepositoryAdapter {
    _pool: Arc<DbPool>,
}

impl NotificationRepositoryAdapter {
    pub fn new(pool: Arc<DbPool>) -> Self {
        Self { _pool: pool }
    }
    
    pub async fn deliver_notification_to_topic(
        &self,
        _topic: &str,
        _title: &str,
        _body: &str,
        _data: Option<serde_json::Value>,
    ) -> Result<crate::domain::notification::aggregates::notification::DeliveryResult, crate::application::ApplicationError> {
        // Topic notification delivery placeholder
        // Future: Integrate with FCM topic messaging or notification service
        Ok(crate::domain::notification::aggregates::notification::DeliveryResult::Success {
            message_id: Some("placeholder_message_id".to_string()),
            delivered_at: chrono::Utc::now(),
        })
    }

    pub async fn deliver_notification_to_user(
        &self,
        _notification: &crate::domain::notification::aggregates::notification::Notification,
        _wallet_address: uuid::Uuid,
        _fcm_token: Option<String>,
        _email: Option<String>,
    ) -> Result<Vec<crate::domain::notification::aggregates::notification::DeliveryResult>, crate::application::ApplicationError> {
        // User notification delivery placeholder
        // Future: Integrate with FCM/APNS for push notifications and email service
        Ok(vec![])
    }
}

#[derive(Clone)]
pub struct UserNotificationRepository {
    _pool: Arc<DbPool>,
}

impl UserNotificationRepository {
    pub fn new(pool: Arc<DbPool>) -> Self {
        Self { _pool: pool }
    }
}

#[derive(Debug, Clone)]
pub struct NotificationMapper;

impl Default for NotificationMapper {
    fn default() -> Self {
        Self::new()
    }
}

impl NotificationMapper {
    pub fn new() -> Self {
        Self
    }

    #[allow(clippy::too_many_arguments)]
    pub fn create_ddd_notification_from_legacy(
        _recipient_wallet_address: Option<Uuid>,
        _fcm_topic_id: Option<String>,
        _title: String,
        _body: String,
        _notification_type: crate::domain::notification::value_objects::user_preferences::NotificationType,
        _priority: crate::domain::notification::aggregates::notification::NotificationPriority,
        _channels: Vec<String>,
        _scheduled_for: Option<chrono::DateTime<chrono::Utc>>,
        _expires_at: Option<chrono::DateTime<chrono::Utc>>,
        _action_url: Option<String>,
        _image_url: Option<String>,
        _data_payload: Option<serde_json::Value>,
    ) -> Result<crate::domain::notification::aggregates::notification::Notification, String> {
        // TODO: Implement actual DDD notification creation 
        // For now, return an error since we don't have the full implementation
        Err("NotificationMapper::create_ddd_notification_from_legacy not yet implemented".to_string())
    }
}

// User response types for API compatibility (Web3-first: wallet-based)
#[derive(Debug, Clone)]
pub struct UserUpdateResponse {
    pub wallet_address: String,
    pub is_active: bool,
    pub permissions: Vec<String>,
}

impl UserUpdateResponse {
    pub fn placeholder(wallet_address: String) -> Self {
        Self {
            wallet_address,
            is_active: true,
            permissions: vec!["epsx:basic:access".to_string()],
        }
    }
}

#[derive(Debug, Clone)]
pub struct UserCreateResponse {
    pub wallet_address: String,
}

impl UserCreateResponse {
    pub fn new(wallet_address: String) -> Self {
        Self { wallet_address }
    }
}

// Pool creation is now in mod.rs - removed duplicate function

// Database model types for mappers compatibility
// Legacy User/NewUser/UpdateUser structs removed - Web3-first uses WalletUser only

#[derive(Debug, Clone)]
pub struct Session {
    pub id: uuid::Uuid,
    pub wallet_address: uuid::Uuid,
    pub access_token: String,
    pub expires_at: chrono::DateTime<chrono::Utc>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub session_token: Option<String>,
    pub user_agent: Option<String>,
    pub ip_address: Option<IpAddr>,
    pub is_active: bool,
}

#[derive(Debug, Clone)]
pub struct NewSession {
    pub id: uuid::Uuid,
    pub wallet_address: uuid::Uuid,
    pub access_token: String,
    pub expires_at: chrono::DateTime<chrono::Utc>,
    pub provider: Option<String>,
    pub session_token: Option<String>,
    pub user_agent: Option<String>,
    pub ip_address: Option<IpAddr>,
    pub is_active: bool,
}

#[derive(Debug, Clone)]
pub struct UpdateSession {
    pub access_token: Option<String>,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone)]
pub struct IpAddr(pub String);

// Permission Group Types - Updated to match database schema exactly
// Supports both SQLx (legacy) and Diesel (new) during migration
#[derive(Debug, Clone, diesel::Queryable, diesel::Selectable)]
#[diesel(table_name = crate::schema::permission_groups)]
pub struct PermissionGroup {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub description: String,
    pub group_type: String,
    pub group_metadata: serde_json::Value,
    pub price: Option<bigdecimal::BigDecimal>,
    pub currency: Option<String>,
    pub billing_cycle: Option<String>,
    // Note: DB schema has non-null bool, but we keep Option for backward compatibility during migration
    #[diesel(deserialize_as = bool)]
    pub is_active: Option<bool>,
    #[diesel(deserialize_as = bool)]
    pub is_promoted: Option<bool>,
    pub display_order: Option<i32>,
    pub max_members: Option<i32>,
    pub auto_assign_enabled: Option<bool>,
    pub assignment_rules: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Option<String>,
    pub last_modified_by: Option<String>,
}

// Helper to extract permissions from group_metadata
impl PermissionGroup {
    pub fn permissions(&self) -> serde_json::Value {
        self.group_metadata
            .get("permissions")
            .cloned()
            .unwrap_or(serde_json::json!([]))
    }
}

// Diesel Insertable model for creating new permission groups
#[derive(Debug, Clone, diesel::Insertable)]
#[diesel(table_name = crate::schema::permission_groups)]
pub struct NewPermissionGroup {
    pub name: String,
    pub slug: String,
    pub description: String,
    pub group_type: String,
    pub group_metadata: serde_json::Value,
    pub price: Option<bigdecimal::BigDecimal>,
    pub currency: Option<String>,
    pub billing_cycle: Option<String>,
    pub is_active: Option<bool>,
    pub is_promoted: Option<bool>,
    pub display_order: Option<i32>,
    pub created_by: Option<String>,
}

// Helper method for backward compatibility
impl NewPermissionGroup {
    pub fn with_permissions(mut self, permissions: serde_json::Value) -> Self {
        if let Some(obj) = self.group_metadata.as_object_mut() {
            obj.insert("permissions".to_string(), permissions);
        }
        self
    }
}

// Diesel AsChangeset model for updating permission groups
#[derive(Debug, Clone, diesel::AsChangeset)]
#[diesel(table_name = crate::schema::permission_groups)]
pub struct UpdatePermissionGroup {
    pub name: Option<String>,
    pub description: Option<String>,
    pub group_metadata: Option<serde_json::Value>,
    pub price: Option<bigdecimal::BigDecimal>,
    pub currency: Option<String>,
    pub billing_cycle: Option<String>,
    pub is_active: Option<bool>,
    pub last_modified_by: Option<String>,
}

// Helper method for backward compatibility
impl UpdatePermissionGroup {
    pub fn with_permissions(mut self, permissions: serde_json::Value) -> Self {
        if let Some(ref mut metadata) = self.group_metadata {
            if let Some(obj) = metadata.as_object_mut() {
                obj.insert("permissions".to_string(), permissions);
            }
        } else {
            let mut obj = serde_json::Map::new();
            obj.insert("permissions".to_string(), permissions);
            self.group_metadata = Some(serde_json::Value::Object(obj));
        }
        self
    }
}

// PermissionGroupRepository has been removed - use PermissionGroupRepositoryAdapter instead
// Supports both SQLx (legacy) and Diesel (new) during migration
#[derive(Debug, Clone, diesel::Queryable)]
pub struct WalletAssignment {
    pub id: Uuid,
    pub wallet_address: String,
    pub group_id: Uuid,
    pub group_name: String,
    pub group_type: String,
    pub assignment_source: String,
    pub assignment_reason: Option<String>,
    pub assigned_by: Option<String>,
    pub assigned_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub is_active: bool,
}
// Wallet Users Diesel Models
// Models for wallet_users table with Diesel support

/// Diesel Queryable model for wallet_users table
#[derive(Debug, Clone, diesel::Queryable, diesel::Selectable)]
#[diesel(table_name = crate::schema::wallet_users)]
pub struct WalletUserDb {
    pub wallet_address: String,
    pub is_active: bool,
    pub tier_level: String,
    pub wallet_metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_auth_at: Option<DateTime<Utc>>,
    pub permission_groups: Option<serde_json::Value>,
}

/// Diesel Insertable model for creating new wallet users
#[derive(Debug, Clone, diesel::Insertable)]
#[diesel(table_name = crate::schema::wallet_users)]
pub struct NewWalletUserDb {
    pub wallet_address: String,
    pub is_active: bool,
    pub tier_level: String,
    pub wallet_metadata: serde_json::Value,
}

/// Diesel AsChangeset model for updating wallet users
#[derive(Debug, Clone, diesel::AsChangeset)]
#[diesel(table_name = crate::schema::wallet_users)]
pub struct UpdateWalletUserDb {
    pub is_active: Option<bool>,
    pub tier_level: Option<String>,
    pub wallet_metadata: Option<serde_json::Value>,
    pub last_auth_at: Option<DateTime<Utc>>,
}

// Models for sessions table with Diesel support
// Note: Using CURRENT database schema (user_id, is_active) not future Web3 schema (wallet_address, is_revoked)

/// Diesel Queryable model for sessions table
/// Note: We use raw SQL with ip_address::TEXT casting to handle INET->String conversion
#[derive(Debug, Clone, diesel::QueryableByName)]
pub struct SessionDb {
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    pub id: uuid::Uuid,
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    pub user_id: uuid::Uuid,
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub access_token: String,
    #[diesel(sql_type = diesel::sql_types::Timestamptz)]
    pub expires_at: DateTime<Utc>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    pub provider: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    pub session_token: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    pub user_agent: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    pub ip_address: Option<String>,  // Cast from INET to TEXT in SQL queries
    #[diesel(sql_type = diesel::sql_types::Bool)]
    pub is_active: bool,
    #[diesel(sql_type = diesel::sql_types::Timestamptz)]
    pub created_at: DateTime<Utc>,
}

/// Model for creating new sessions (used with raw SQL, not Diesel DSL)
#[derive(Debug, Clone)]
pub struct NewSessionDb {
    pub id: uuid::Uuid,
    pub user_id: uuid::Uuid,
    pub access_token: String,
    pub expires_at: DateTime<Utc>,
    pub provider: Option<String>,
    pub session_token: Option<String>,
    pub user_agent: Option<String>,
    pub ip_address: Option<String>,  // Using String for IP, convert to/from IpAddr in adapter
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

// ============================================================================
// Permission Group Models (Diesel)
// ============================================================================

/// Diesel Queryable model for permission_groups table
#[derive(Debug, Clone, diesel::Queryable, diesel::Selectable)]
#[diesel(table_name = crate::schema::permission_groups)]
pub struct PermissionGroupDb {
    pub id: uuid::Uuid,
    pub name: String,
    pub slug: String,
    pub description: String,
    pub group_type: String,
    pub group_metadata: serde_json::Value,
    pub price: Option<BigDecimal>,
    pub currency: Option<String>,
    pub billing_cycle: Option<String>,
    pub is_active: bool,
    pub is_promoted: bool,
    pub display_order: Option<i32>,
    pub max_members: Option<i32>,
    pub auto_assign_enabled: Option<bool>,
    pub assignment_rules: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Option<String>,
    pub last_modified_by: Option<String>,
}

/// Diesel Insertable model for creating/updating permission groups
#[derive(Debug, Clone, diesel::Insertable, diesel::AsChangeset)]
#[diesel(table_name = crate::schema::permission_groups)]
pub struct NewPermissionGroupDb {
    pub id: uuid::Uuid,
    pub name: String,
    pub slug: String,
    pub description: String,
    pub group_type: String,
    pub group_metadata: serde_json::Value,
    pub price: Option<BigDecimal>,
    pub currency: Option<String>,
    pub billing_cycle: Option<String>,
    pub is_active: bool,
    pub is_promoted: bool,
    pub display_order: Option<i32>,
    pub max_members: Option<i32>,
    pub auto_assign_enabled: Option<bool>,
    pub assignment_rules: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Option<String>,
    pub last_modified_by: Option<String>,
}

/// Query result for permission data from JOIN query
#[derive(Debug, Clone, diesel::QueryableByName)]
pub struct PermissionRow {
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub platform: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub resource: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub action: String,
}
