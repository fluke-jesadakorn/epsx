// Database Types and Models
// Unified type definitions for database operations

use chrono::{DateTime, Utc};
use uuid::Uuid;
use std::sync::Arc;
use bigdecimal::BigDecimal;
use crate::infrastructure::database::diesel_connection_manager::TlsPool;

// Database Pool Types
pub type DbPool = &'static TlsPool;

// Session Types


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
        recipient_wallet_address: Option<Uuid>,
        fcm_topic_id: Option<String>,
        title: String,
        body: String,
        notification_type: crate::domain::notification::value_objects::user_preferences::NotificationType,
        priority: crate::domain::notification::aggregates::notification::NotificationPriority,
        channels: Vec<String>,
        scheduled_for: Option<chrono::DateTime<chrono::Utc>>,
        expires_at: Option<chrono::DateTime<chrono::Utc>>,
        action_url: Option<String>,
        image_url: Option<String>,
        data_payload: Option<serde_json::Value>,
    ) -> Result<crate::domain::notification::aggregates::notification::Notification, String> {
        use crate::domain::notification::value_objects::*;
        use crate::domain::notification::aggregates::notification::Notification;
        
        // Create content from title and body
        let content = NotificationContent::new(title, body)?;
        
        // Parse channel configuration
        let channel_configs: Vec<DeliveryChannelConfig> = channels.iter()
            .filter_map(|ch| {
                let channel_type = DeliveryChannelType::from_str(ch).ok()?;
                Some(DeliveryChannelConfig::new(channel_type))
            })
            .collect();
        
        let multi_channel = if channel_configs.is_empty() {
            // Default to in-app if no channels specified
            MultiChannelConfig::single_channel(DeliveryChannelConfig::new(DeliveryChannelType::InApp))
        } else {
            MultiChannelConfig::new(channel_configs)
        };
        
        // Create schedule info
        let schedule = if let Some(scheduled_at) = scheduled_for {
            
            if let Some(exp) = expires_at {
                ScheduleInfo::scheduled_with_expiry(scheduled_at, exp)?
            } else {
                ScheduleInfo::scheduled(scheduled_at)?
            }
        } else if let Some(exp) = expires_at {
            ScheduleInfo::with_expiry(exp)?
        } else {
            ScheduleInfo::immediate()
        };
        
        // Create the notification based on whether it's for a user or topic
        let notification = if let Some(wallet_id) = recipient_wallet_address {
            Notification::create_for_user(
                wallet_id.to_string(),
                content,
                notification_type,
                priority,
                multi_channel,
                schedule,
            )?
        } else if let Some(topic_name) = fcm_topic_id {
            // Use from_name which handles simple topic reconstruction
            let topic = NotificationTopic::from_name(topic_name)?;
            Notification::create_for_topic(
                topic,
                content,
                notification_type,
                priority,
                multi_channel,
                schedule,
                None, // created_by
            )?
        } else {
            return Err("Either recipient_wallet_address or fcm_topic_id must be provided".to_string());
        };
        
        // Apply optional metadata
        let mut notification = notification;
        if let Some(url) = action_url {
            notification.metadata_mut().set_action_url(url);
        }
        if let Some(url) = image_url {
            notification.metadata_mut().set_image_url(url);
        }
        if let Some(payload) = data_payload {
            notification.metadata_mut().set_data_payload(payload);
        }
        
        Ok(notification)
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



// Permission Plan Types - Updated to match database schema exactly
// Supports both SQLx (legacy) and Diesel (new) during migration
#[derive(Debug, Clone, diesel::Queryable, diesel::Selectable)]
#[diesel(table_name = crate::schemas::primary::plans)]
pub struct PermissionPlan {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub description: String,
    pub plan_type: String,
    pub plan_metadata: serde_json::Value,
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

    pub rate_limit_per_minute: i32,
    pub rate_limit_per_hour: i32,
    pub rate_limit_per_day: i32,
    pub burst_capacity: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Option<String>,
    pub last_modified_by: Option<String>,
    pub tier_level: i32,
    pub is_public: bool,
}

// Helper to extract permissions from plan_metadata
impl PermissionPlan {
    pub fn permissions(&self) -> serde_json::Value {
        self.plan_metadata
            .get("permissions")
            .cloned()
            .unwrap_or(serde_json::json!([]))
    }
}

// Diesel Insertable model for creating new permission plans
#[derive(Debug, Clone, diesel::Insertable)]
#[diesel(table_name = crate::schemas::primary::plans)]
pub struct NewPermissionPlan {
    pub name: String,
    pub slug: String,
    pub description: String,
    pub plan_type: String,
    pub plan_metadata: serde_json::Value,
    pub price: Option<bigdecimal::BigDecimal>,
    pub currency: Option<String>,
    pub billing_cycle: Option<String>,
    pub is_active: Option<bool>,
    pub is_promoted: Option<bool>,
    pub display_order: Option<i32>,
    pub created_by: Option<String>,
    pub rate_limit_per_minute: i32,
    pub rate_limit_per_hour: i32,
    pub rate_limit_per_day: i32,
    pub burst_capacity: i32,
    pub tier_level: i32,
}

// Helper method for backward compatibility
impl NewPermissionPlan {
    pub fn with_permissions(mut self, permissions: serde_json::Value) -> Self {
        if let Some(obj) = self.plan_metadata.as_object_mut() {
            obj.insert("permissions".to_string(), permissions);
        }
        self
    }
}

// Diesel AsChangeset model for updating permission plans
#[derive(Debug, Clone, diesel::AsChangeset)]
#[diesel(table_name = crate::schemas::primary::plans)]
pub struct UpdatePermissionPlan {
    pub name: Option<String>,
    pub description: Option<String>,
    pub plan_metadata: Option<serde_json::Value>,
    pub price: Option<bigdecimal::BigDecimal>,
    pub currency: Option<String>,
    pub billing_cycle: Option<String>,
    pub is_active: Option<bool>,
    pub last_modified_by: Option<String>,
}

// Helper method for backward compatibility
impl UpdatePermissionPlan {
    pub fn with_permissions(mut self, permissions: serde_json::Value) -> Self {
        if let Some(ref mut metadata) = self.plan_metadata {
            if let Some(obj) = metadata.as_object_mut() {
                obj.insert("permissions".to_string(), permissions);
            }
        } else {
            let mut obj = serde_json::Map::new();
            obj.insert("permissions".to_string(), permissions);
            self.plan_metadata = Some(serde_json::Value::Object(obj));
        }
        self
    }
}

// PermissionPlanRepository has been removed - use PermissionPlanRepositoryAdapter instead
// Supports both SQLx (legacy) and Diesel (new) during migration
#[derive(Debug, Clone, diesel::Queryable)]
pub struct WalletAssignment {
    pub id: Uuid,
    pub wallet_address: String,
    pub plan_id: Uuid,
    pub plan_name: String,
    pub plan_type: String,
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
#[diesel(table_name = crate::schemas::primary::wallet_users)]
pub struct WalletUserDb {
    pub wallet_address: String,
    pub is_active: bool,
    pub tier_level: String,
    pub wallet_metadata: serde_json::Value,
    pub permission_plans: Option<serde_json::Value>,
    pub disable_info: Option<serde_json::Value>,
    pub plan_expires_at: Option<DateTime<Utc>>,
    pub current_plan_id: Option<i32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_auth_at: Option<DateTime<Utc>>,
}

/// Diesel Insertable model for creating new wallet users
#[derive(Debug, Clone, diesel::Insertable)]
#[diesel(table_name = crate::schemas::primary::wallet_users)]
pub struct NewWalletUserDb {
    pub wallet_address: String,
    pub is_active: bool,
    pub tier_level: String,
    pub wallet_metadata: serde_json::Value,
}

/// Diesel AsChangeset model for updating wallet users
#[derive(Debug, Clone, diesel::AsChangeset)]
#[diesel(table_name = crate::schemas::primary::wallet_users)]
pub struct UpdateWalletUserDb {
    pub is_active: Option<bool>,
    pub tier_level: Option<String>,
    pub wallet_metadata: Option<serde_json::Value>,
    pub last_auth_at: Option<DateTime<Utc>>,
    pub current_plan_id: Option<Option<i32>>,
    pub plan_expires_at: Option<Option<DateTime<Utc>>>,
}



// ============================================================================
// Permission Plan Models (Diesel)
// ============================================================================

/// Diesel Queryable model for plans table
#[derive(Debug, Clone, diesel::Queryable, diesel::Selectable)]
#[diesel(table_name = crate::schemas::primary::plans)]
pub struct PermissionPlanDb {
    pub id: uuid::Uuid,
    pub name: String,
    pub slug: String,
    pub description: String,
    pub plan_type: String,
    pub plan_metadata: serde_json::Value,
    pub price: Option<BigDecimal>,
    pub currency: Option<String>,
    pub billing_cycle: Option<String>,
    pub is_active: bool,
    pub is_promoted: bool,
    pub display_order: Option<i32>,
    pub max_members: Option<i32>,
    pub auto_assign_enabled: Option<bool>,
    pub assignment_rules: Option<serde_json::Value>,

    pub rate_limit_per_minute: i32,
    pub rate_limit_per_hour: i32,
    pub rate_limit_per_day: i32,
    pub burst_capacity: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Option<String>,
    pub last_modified_by: Option<String>,
    pub tier_level: i32,
    pub is_public: bool,
}

/// Diesel Insertable model for creating/updating permission plans
#[derive(Debug, Clone, diesel::Insertable, diesel::AsChangeset)]
#[diesel(table_name = crate::schemas::primary::plans)]
pub struct NewPermissionPlanDb {
    pub id: uuid::Uuid,
    pub name: String,
    pub slug: String,
    pub description: String,
    pub plan_type: String,
    pub plan_metadata: serde_json::Value,
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
    pub rate_limit_per_minute: i32,
    pub rate_limit_per_hour: i32,
    pub rate_limit_per_day: i32,
    pub burst_capacity: i32,
    pub tier_level: i32,
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
