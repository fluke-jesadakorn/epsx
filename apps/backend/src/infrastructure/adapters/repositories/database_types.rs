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
    pub user_id: Uuid,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct NewRefreshToken {
    pub token: String,
    pub user_id: Uuid,
    pub expires_at: DateTime<Utc>,
}

// User Dynamic Limits
#[derive(Debug, Clone)]
pub struct UserDynamicLimit {
    pub id: Uuid,
    pub user_id: Uuid,
    pub resource: String,
    pub limit_type: String,
    pub limit_value: i32,
    pub window_seconds: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct ResolvedUserLimits {
    pub user_id: Option<Uuid>,
    pub ranking_limit: i32,
    pub api_minute_limit: i32,
    pub daily_limit: i32,
    pub weekly_limit: i32,
    pub monthly_limit: i32,
    pub total_limit: i32,
    pub has_premium_features: bool,
    pub is_admin: bool,
}

impl ResolvedUserLimits {
    pub fn new(ranking_limit: i32, api_minute_limit: i32, has_premium_features: bool, is_admin: bool) -> Self {
        Self {
            user_id: None,
            ranking_limit,
            api_minute_limit,
            daily_limit: api_minute_limit * 24,
            weekly_limit: api_minute_limit * 24 * 7,
            monthly_limit: api_minute_limit * 24 * 30,
            total_limit: api_minute_limit * 24 * 365,
            has_premium_features,
            is_admin,
        }
    }
    
    pub fn default_free() -> Self {
        Self::new(3, 10, false, false)
    }
    
    pub fn default_premium() -> Self {
        Self::new(100, 100, true, false)
    }
    
    pub fn default_admin() -> Self {
        Self::new(1000, 1000, true, true)
    }
}

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
        _user_id: uuid::Uuid,
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
        _recipient_user_id: Option<Uuid>,
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
    pub user_id: String,
    pub email: String,
    pub email_verified: bool,
    pub is_active: bool,
    pub permissions: Vec<String>,
}

impl UserUpdateResponse {
    pub fn placeholder(user_id: String, email: String) -> Self {
        Self {
            user_id,
            email,
            email_verified: true,
            is_active: true,
            permissions: vec!["epsx:basic:access".to_string()],
        }
    }
}

#[derive(Debug, Clone)]
pub struct UserCreateResponse {
    pub user_id: String,
}

impl UserCreateResponse {
    pub fn new(user_id: String) -> Self {
        Self { user_id }
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
    pub user_id: uuid::Uuid,
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
    pub user_id: uuid::Uuid,
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