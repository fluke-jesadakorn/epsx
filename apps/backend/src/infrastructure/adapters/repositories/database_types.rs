// Database Types and Models
// Unified type definitions for database operations

use chrono::{DateTime, Utc};
use uuid::Uuid;
use std::sync::Arc;
use sqlx::PgPool;

// Database Pool Types
pub type DbPool = PgPool;

// Session Types
#[derive(Debug, Clone)]
pub struct SessionRepository {
    _pool: Arc<PgPool>,
}

impl SessionRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { _pool: pool }
    }
    
    pub async fn save_session(&self, _session_data: &str) -> Result<(), String> {
        Ok(())
    }
    
    pub async fn save(&self, _session: &crate::domain::user_management::aggregates::session::Session) -> Result<(), String> {
        // TODO: Implement session storage
        Ok(())
    }
}

// User Repository Types
#[derive(Debug, Clone)]
pub struct UserRepository {
    _pool: Arc<PgPool>,
}

impl UserRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { _pool: pool }
    }
}

// Token Types
#[derive(Debug, Clone)]
pub struct RefreshTokenRepository {
    _pool: Arc<PgPool>,
}

impl RefreshTokenRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { _pool: pool }
    }
}

#[derive(Debug, Clone)]
pub struct RevokedTokenRepository {
    _pool: Arc<PgPool>,
}

impl RevokedTokenRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
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
#[derive(Debug, Clone)]
pub struct NotificationRepositoryAdapter {
    _pool: Arc<PgPool>,
}

impl NotificationRepositoryAdapter {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { _pool: pool }
    }
    
    pub async fn deliver_notification_to_topic(
        &self,
        _topic: &str,
        _title: &str,
        _body: &str,
        _data: Option<serde_json::Value>,
    ) -> Result<crate::domain::notification::aggregates::notification::DeliveryResult, crate::application::ApplicationError> {
        // TODO: Implement topic notification delivery
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
        // TODO: Implement user notification delivery
        Ok(vec![])
    }
}

#[derive(Debug, Clone)]
pub struct UserNotificationRepository {
    _pool: Arc<PgPool>,
}

impl UserNotificationRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { _pool: pool }
    }
}

#[derive(Debug, Clone)]
pub struct NotificationMapper;

impl NotificationMapper {
    pub fn new() -> Self {
        Self
    }
    
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

// User response types for API compatibility
#[derive(Debug, Clone)]
pub struct UserUpdateResponse {
    pub wallet_address: String,
    pub email: String,
    pub email_verified: bool,
    pub is_active: bool,
    pub permissions: Vec<String>,
}

impl UserUpdateResponse {
    pub fn placeholder(wallet_address: String, email: String) -> Self {
        Self {
            wallet_address,
            email,
            email_verified: true,
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

// Pool creation function
pub async fn create_pool() -> Result<Arc<PgPool>, Box<dyn std::error::Error + Send + Sync>> {
    let database_url = std::env::var("DATABASE_URL")
        .map_err(|_| "DATABASE_URL environment variable is required")?;
    let pool = PgPool::connect(&database_url).await?;
    Ok(Arc::new(pool))
}

// Database model types for mappers compatibility
#[derive(Debug, Clone)]
pub struct User {
    pub id: uuid::Uuid,
    pub email: String,
    // firebase_uid removed for Web3-first architecture
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub is_active: bool,
    pub email_verified: bool,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub last_login_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone)]
pub struct NewUser {
    pub email: String,
    // firebase_uid removed for Web3-first architecture
}

#[derive(Debug, Clone)]
pub struct UpdateUser {
    pub email: Option<String>,
}

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
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct PermissionGroup {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub description: String,
    pub group_type: String,
    pub permissions: serde_json::Value,
    pub group_metadata: serde_json::Value,
    pub price: Option<sqlx::types::BigDecimal>, // Handle nullable decimal
    pub currency: Option<String>,
    pub billing_cycle: Option<String>,
    pub is_active: Option<bool>,
    pub is_promoted: Option<bool>,
    pub display_order: Option<i32>,
    pub max_members: Option<i32>,
    pub auto_assign_enabled: Option<bool>,
    pub assignment_rules: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Option<String>,
    pub last_modified_by: Option<String>,
}

#[derive(Debug, Clone)]
pub struct NewPermissionGroup {
    pub name: String,
    pub slug: String,
    pub description: String,
    pub group_type: String,
    pub permissions: serde_json::Value,
    pub group_metadata: serde_json::Value,
    pub price: Option<sqlx::types::BigDecimal>,
    pub currency: Option<String>,
    pub billing_cycle: Option<String>,
    pub is_active: Option<bool>,
    pub display_order: Option<i32>,
    pub created_by: Option<String>,
}

#[derive(Debug, Clone)]
pub struct UpdatePermissionGroup {
    pub name: Option<String>,
    pub description: Option<String>,
    pub permissions: Option<serde_json::Value>,
    pub price: Option<sqlx::types::BigDecimal>,
    pub currency: Option<String>,
    pub billing_cycle: Option<String>,
    pub is_active: Option<bool>,
    pub last_modified_by: Option<String>,
}

#[derive(Debug, Clone)]
pub struct PermissionGroupRepository {
    pool: Arc<PgPool>,
}

impl PermissionGroupRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }

    /// Get all active subscription plans
    pub async fn get_subscription_plans(&self) -> Result<Vec<PermissionGroup>, sqlx::Error> {
        let plans = sqlx::query_as::<_, PermissionGroup>(
            r#"
            SELECT 
                id, name, slug, description, group_type, permissions, group_metadata,
                price, currency, billing_cycle, is_active, is_promoted, display_order,
                max_members, auto_assign_enabled, assignment_rules, created_at, updated_at,
                created_by, last_modified_by
            FROM permission_groups 
            WHERE group_type = 'subscription' AND COALESCE(is_active, true) = true 
            ORDER BY COALESCE(display_order, 0), COALESCE(price, 0)
            "#
        )
        .fetch_all(&*self.pool)
        .await?;
        
        Ok(plans)
    }

    /// Get plan by ID
    pub async fn get_plan_by_id(&self, plan_id: Uuid) -> Result<Option<PermissionGroup>, sqlx::Error> {
        let plan = sqlx::query_as::<_, PermissionGroup>(
            r#"
            SELECT 
                id, name, slug, description, group_type, permissions, group_metadata,
                price, currency, billing_cycle, is_active, is_promoted, display_order,
                max_members, auto_assign_enabled, assignment_rules, created_at, updated_at,
                created_by, last_modified_by
            FROM permission_groups 
            WHERE id = $1 AND group_type = 'subscription'
            "#
        )
        .bind(plan_id)
        .fetch_optional(&*self.pool)
        .await?;
        
        Ok(plan)
    }

    // Note: Create and update methods can be added later if needed
    // For now, we only need read operations to replace hardcoded plans
}