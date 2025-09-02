// ============================================================================
// FCM TOKEN MANAGEMENT SERVICE
// ============================================================================
// Complete service for managing Firebase Cloud Messaging device tokens

use std::sync::Arc;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;
use tracing::{info, warn, debug};

use crate::dom::values::UserId;
use crate::infra::db::diesel::{DbPool, DbConnection, types::DevicePlatform};
use crate::infra::db::diesel::models::fcm::{
    DieselFcmToken, NewDieselFcmToken, create_fcm_token, get_user_active_tokens,
    deactivate_fcm_token, get_all_active_tokens
};
use crate::infra::firebase::FirebaseAdmin;
use crate::infra::cache::Cache;

// ============================================================================
// FCM TOKEN SERVICE TYPES
// ============================================================================

#[derive(Debug, Clone)]
pub struct FcmTokenInfo {
    pub id: String,
    pub user_id: String,
    pub token: String,
    pub platform: DevicePlatform,
    pub device_info: Option<serde_json::Value>,
    pub user_agent: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_used_at: DateTime<Utc>,
}

#[derive(Debug, thiserror::Error)]
pub enum FcmTokenError {
    #[error("Database error: {0}")]
    DatabaseError(String),
    #[error("Firebase error: {0}")]
    FirebaseError(String),
    #[error("Invalid token format")]
    InvalidToken,
    #[error("Token not found")]
    TokenNotFound,
    #[error("User has too many tokens (max: {max})")]
    TooManyTokens { max: u32 },
    #[error("Validation failed: {0}")]
    ValidationError(String),
    #[error("Service unavailable")]
    ServiceUnavailable,
}

// ============================================================================
// FCM TOKEN SERVICE TRAIT
// ============================================================================

#[async_trait]
pub trait FcmTokenService: Send + Sync {
    async fn register_token(
        &self,
        user_id: &UserId,
        token: String,
        platform: DevicePlatform,
        device_info: Option<serde_json::Value>,
        user_agent: Option<String>,
    ) -> Result<FcmTokenInfo, FcmTokenError>;

    async fn get_user_tokens(&self, user_id: &UserId) -> Result<Vec<FcmTokenInfo>, FcmTokenError>;
    async fn deactivate_token(&self, token: &str) -> Result<bool, FcmTokenError>;
    async fn get_broadcast_tokens(&self, limit: Option<u32>) -> Result<Vec<FcmTokenInfo>, FcmTokenError>;
}

// ============================================================================
// FCM TOKEN SERVICE IMPLEMENTATION
// ============================================================================

pub struct SimpleFcmTokenService {
    pool: Arc<DbPool>,
    firebase_admin: Arc<FirebaseAdmin>,
    cache: Option<Arc<dyn Cache>>,
}

impl SimpleFcmTokenService {
    pub fn new(pool: Arc<DbPool>, firebase_admin: Arc<FirebaseAdmin>) -> Self {
        Self {
            pool,
            firebase_admin,
            cache: None,
        }
    }

    pub fn with_cache(mut self, cache: Arc<dyn Cache>) -> Self {
        self.cache = Some(cache);
        self
    }

    fn validate_token_format(&self, token: &str) -> Result<(), FcmTokenError> {
        if token.is_empty() || token.len() > 1000 {
            return Err(FcmTokenError::InvalidToken);
        }
        Ok(())
    }
    
    async fn get_db_connection(&self) -> Result<DbConnection, FcmTokenError> {
        self.pool.get().await
            .map_err(|e| FcmTokenError::DatabaseError(format!("Connection error: {}", e)))
    }
    
    fn convert_diesel_to_info(&self, token: DieselFcmToken) -> FcmTokenInfo {
        FcmTokenInfo {
            id: token.id.to_string(),
            user_id: token.user_id.to_string(),
            token: token.token,
            platform: token.platform,
            device_info: token.device_info,
            user_agent: token.user_agent,
            is_active: token.is_active.unwrap_or(false),
            created_at: token.created_at.unwrap_or(Utc::now()),
            updated_at: token.updated_at.unwrap_or(Utc::now()),
            last_used_at: token.last_used_at.unwrap_or(Utc::now()),
        }
    }
}

#[async_trait]
impl FcmTokenService for SimpleFcmTokenService {
    async fn register_token(
        &self,
        user_id: &UserId,
        token: String,
        platform: DevicePlatform,
        device_info: Option<serde_json::Value>,
        user_agent: Option<String>,
    ) -> Result<FcmTokenInfo, FcmTokenError> {
        self.validate_token_format(&token)?;

        info!("Registering FCM token for user {} on platform {:?}", user_id.0, platform);
        
        let user_uuid = Uuid::parse_str(&user_id.0.to_string())
            .map_err(|e| FcmTokenError::ValidationError(format!("Invalid user ID: {}", e)))?;

        let new_token = NewDieselFcmToken::new(
            user_uuid,
            token,
            platform,
            device_info,
            user_agent
        );

        let mut conn = self.get_db_connection().await?;
        
        let diesel_token = create_fcm_token(&mut conn, new_token)
            .await
            .map_err(|e| FcmTokenError::DatabaseError(format!("Failed to create token: {}", e)))?;

        info!("Successfully registered FCM token for user {}", user_id.0);
        Ok(self.convert_diesel_to_info(diesel_token))
    }

    async fn get_user_tokens(&self, user_id: &UserId) -> Result<Vec<FcmTokenInfo>, FcmTokenError> {
        debug!("Getting FCM tokens for user {}", user_id.0);
        
        let user_uuid = Uuid::parse_str(&user_id.0.to_string())
            .map_err(|e| FcmTokenError::ValidationError(format!("Invalid user ID: {}", e)))?;

        let mut conn = self.get_db_connection().await?;
        
        let diesel_tokens = get_user_active_tokens(&mut conn, user_uuid)
            .await
            .map_err(|e| FcmTokenError::DatabaseError(format!("Failed to get user tokens: {}", e)))?;

        let tokens: Vec<FcmTokenInfo> = diesel_tokens.into_iter()
            .map(|token| self.convert_diesel_to_info(token))
            .collect();

        debug!("Found {} FCM tokens for user {}", tokens.len(), user_id.0);
        Ok(tokens)
    }

    async fn deactivate_token(&self, token: &str) -> Result<bool, FcmTokenError> {
        info!("Deactivating FCM token: {}...", &token[..std::cmp::min(token.len(), 20)]);
        
        let mut conn = self.get_db_connection().await?;
        
        let rows_affected = deactivate_fcm_token(&mut conn, token)
            .await
            .map_err(|e| FcmTokenError::DatabaseError(format!("Failed to deactivate token: {}", e)))?;

        let success = rows_affected > 0;
        if success {
            info!("Successfully deactivated FCM token");
        } else {
            warn!("FCM token not found for deactivation");
        }
        
        Ok(success)
    }

    async fn get_broadcast_tokens(&self, limit: Option<u32>) -> Result<Vec<FcmTokenInfo>, FcmTokenError> {
        debug!("Getting FCM tokens for broadcast (limit: {:?})", limit);
        
        let mut conn = self.get_db_connection().await?;
        let db_limit = limit.map(|l| l as i64);
        
        let diesel_tokens = get_all_active_tokens(&mut conn, db_limit)
            .await
            .map_err(|e| FcmTokenError::DatabaseError(format!("Failed to get broadcast tokens: {}", e)))?;

        let tokens: Vec<FcmTokenInfo> = diesel_tokens.into_iter()
            .map(|token| self.convert_diesel_to_info(token))
            .collect();

        debug!("Found {} FCM tokens for broadcast", tokens.len());
        Ok(tokens)
    }
}

// ============================================================================
// CONVERSION TRAITS
// ============================================================================

// Removed NotificationServiceError conversion as it doesn't have the required variants