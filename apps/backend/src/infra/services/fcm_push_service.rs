// ============================================================================
// FCM PUSH NOTIFICATION SERVICE
// ============================================================================
// Comprehensive service for sending Firebase Cloud Messaging push notifications

use std::sync::Arc;
use std::collections::HashMap;
use async_trait::async_trait;
// use chrono::{DateTime, Utc}; // Unused for now
use uuid::Uuid;
use tracing::{info, warn, debug, error};
use serde::{Serialize, Deserialize};

use crate::dom::values::UserId;
use crate::infra::db::diesel::{DbPool, DbConnection, types::{DevicePlatform, NotificationPriority}};
use crate::infra::db::diesel::models::notification::{update_notification_fcm_status};
use crate::infra::firebase::{FirebaseAdmin, FcmMessage as FirebaseFcmMessage, FcmNotification, FcmAndroidConfig, FcmApnsConfig, FcmApnsPayload, FcmAps};
use crate::infra::cache::Cache;
use crate::infra::services::fcm_token_service::{FcmTokenService, FcmTokenInfo};

// ============================================================================
// FCM PUSH SERVICE TYPES
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FcmMessage {
    pub title: String,
    pub body: String,
    pub data: Option<HashMap<String, String>>,
    pub image_url: Option<String>,
    pub click_action: Option<String>,
    pub badge: Option<u32>,
    pub sound: Option<String>,
    pub priority: FcmPriority,
    pub ttl: Option<u32>, // Time to live in seconds
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FcmPriority {
    Normal,
    High,
}

impl From<NotificationPriority> for FcmPriority {
    fn from(priority: NotificationPriority) -> Self {
        match priority {
            NotificationPriority::Low | NotificationPriority::Medium => FcmPriority::Normal,
            NotificationPriority::High | NotificationPriority::Critical => FcmPriority::High,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FcmSendResult {
    pub message_id: Option<String>,
    pub success: bool,
    pub error: Option<String>,
    pub should_retry: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FcmBatchResult {
    pub total_sent: u32,
    pub successful: u32,
    pub failed: u32,
    pub results: Vec<FcmSendResult>,
}

#[derive(Debug, thiserror::Error)]
pub enum FcmPushError {
    #[error("Database error: {0}")]
    DatabaseError(String),
    #[error("Firebase error: {0}")]
    FirebaseError(String),
    #[error("Invalid message format: {0}")]
    InvalidMessage(String),
    #[error("Token not found")]
    TokenNotFound,
    #[error("Service unavailable")]
    ServiceUnavailable,
    #[error("Rate limit exceeded")]
    RateLimited,
}

// ============================================================================
// FCM PUSH SERVICE TRAIT
// ============================================================================

#[async_trait]
pub trait FcmPushService: Send + Sync {
    /// Send a push notification to a single device token
    async fn send_to_token(
        &self,
        token: &str,
        message: &FcmMessage,
        notification_id: Option<Uuid>,
    ) -> Result<FcmSendResult, FcmPushError>;

    /// Send a push notification to all devices for a specific user
    async fn send_to_user(
        &self,
        user_id: &UserId,
        message: &FcmMessage,
        notification_id: Option<Uuid>,
    ) -> Result<FcmBatchResult, FcmPushError>;

    /// Send a push notification to all registered devices (broadcast)
    async fn send_broadcast(
        &self,
        message: &FcmMessage,
        limit: Option<u32>,
        notification_id: Option<Uuid>,
    ) -> Result<FcmBatchResult, FcmPushError>;

    /// Send push notification to specific platform devices for a user
    async fn send_to_user_platform(
        &self,
        user_id: &UserId,
        platform: DevicePlatform,
        message: &FcmMessage,
        notification_id: Option<Uuid>,
    ) -> Result<FcmBatchResult, FcmPushError>;
}

// ============================================================================
// FCM PUSH SERVICE IMPLEMENTATION
// ============================================================================

pub struct ComprehensiveFcmPushService {
    pool: Arc<DbPool>,
    firebase_admin: Arc<FirebaseAdmin>,
    token_service: Arc<dyn FcmTokenService>,
    cache: Option<Arc<dyn Cache>>,
    max_retry_attempts: u32,
}

impl ComprehensiveFcmPushService {
    pub fn new(
        pool: Arc<DbPool>,
        firebase_admin: Arc<FirebaseAdmin>,
        token_service: Arc<dyn FcmTokenService>,
    ) -> Self {
        Self {
            pool,
            firebase_admin,
            token_service,
            cache: None,
            max_retry_attempts: 3,
        }
    }

    pub fn with_cache(mut self, cache: Arc<dyn Cache>) -> Self {
        self.cache = Some(cache);
        self
    }

    pub fn with_max_retries(mut self, max_retries: u32) -> Self {
        self.max_retry_attempts = max_retries;
        self
    }

    async fn get_db_connection(&self) -> Result<DbConnection, FcmPushError> {
        self.pool.get().await
            .map_err(|e| FcmPushError::DatabaseError(format!("Connection error: {}", e)))
    }

    async fn send_single_message(
        &self,
        token: &str,
        message: &FcmMessage,
    ) -> Result<FcmSendResult, FcmPushError> {
        info!("Sending FCM message to token: {}...", &token[..std::cmp::min(token.len(), 20)]);
        
        // Create proper Firebase FCM message
        let firebase_message = FirebaseFcmMessage {
            token: Some(token.to_string()),
            topic: None,
            condition: None,
            notification: FcmNotification {
                title: message.title.clone(),
                body: message.body.clone(),
                image: message.image_url.clone(),
            },
            data: message.data.clone(),
            android: if message.ttl.is_some() || matches!(message.priority, FcmPriority::High) {
                Some(FcmAndroidConfig {
                    ttl: message.ttl.map(|t| format!("{}s", t)),
                    priority: Some(match message.priority {
                        FcmPriority::High => "high".to_string(),
                        FcmPriority::Normal => "normal".to_string(),
                    }),
                    notification: None, // Use the main notification field
                })
            } else {
                None
            },
            apns: if message.badge.is_some() || message.sound.is_some() {
                Some(FcmApnsConfig {
                    headers: None,
                    payload: FcmApnsPayload {
                        aps: FcmAps {
                            alert: None,
                            badge: message.badge,
                            sound: message.sound.clone(),
                            category: None,
                        }
                    },
                })
            } else {
                None
            },
            webpush: None,
        };

        // Send via Firebase Admin
        match self.firebase_admin.send_fcm_message(firebase_message, false).await {
            Ok(response) => {
                info!("Successfully sent FCM message with ID: {}", response.name);
                Ok(FcmSendResult {
                    message_id: Some(response.name),
                    success: true,
                    error: None,
                    should_retry: false,
                })
            }
            Err(e) => {
                error!("Failed to send FCM message: {}", e);
                let error_str = e.to_string();
                let should_retry = self.should_retry_error(&error_str);
                
                Ok(FcmSendResult {
                    message_id: None,
                    success: false,
                    error: Some(error_str),
                    should_retry,
                })
            }
        }
    }

    fn should_retry_error(&self, error: &str) -> bool {
        // Retry on temporary errors
        error.contains("UNAVAILABLE") ||
        error.contains("INTERNAL") ||
        error.contains("DEADLINE_EXCEEDED") ||
        error.contains("timeout")
    }

    async fn update_notification_delivery_status(
        &self,
        notification_id: Uuid,
        result: &FcmSendResult,
        attempt_number: u32,
    ) -> Result<(), FcmPushError> {
        let mut conn = self.get_db_connection().await?;
        
        let success = result.success;
        let message_id = result.message_id.clone();
        let error_reason = result.error.clone();
        
        update_notification_fcm_status(
            &mut conn,
            notification_id,
            success,
            message_id,
            error_reason,
            attempt_number,
        )
        .await
        .map_err(|e| FcmPushError::DatabaseError(format!("Failed to update notification status: {}", e)))?;
        
        Ok(())
    }
}

#[async_trait]
impl FcmPushService for ComprehensiveFcmPushService {
    async fn send_to_token(
        &self,
        token: &str,
        message: &FcmMessage,
        notification_id: Option<Uuid>,
    ) -> Result<FcmSendResult, FcmPushError> {
        let mut attempt = 0;
        let mut last_result = None;

        while attempt < self.max_retry_attempts {
            attempt += 1;
            debug!("FCM send attempt {} of {} for token", attempt, self.max_retry_attempts);
            
            let result = self.send_single_message(token, message).await?;
            
            if result.success {
                // Update notification status if provided
                if let Some(notif_id) = notification_id {
                    self.update_notification_delivery_status(notif_id, &result, attempt).await?;
                }
                return Ok(result);
            }
            
            // If not successful and shouldn't retry, break
            if !result.should_retry {
                last_result = Some(result);
                break;
            }
            
            last_result = Some(result);
            
            // Wait before retry (exponential backoff)
            let delay = std::time::Duration::from_secs(2u64.pow(attempt - 1));
            tokio::time::sleep(delay).await;
        }

        let final_result = last_result.unwrap();
        
        // Update notification status with failure
        if let Some(notif_id) = notification_id {
            self.update_notification_delivery_status(notif_id, &final_result, attempt).await?;
        }
        
        Ok(final_result)
    }

    async fn send_to_user(
        &self,
        user_id: &UserId,
        message: &FcmMessage,
        notification_id: Option<Uuid>,
    ) -> Result<FcmBatchResult, FcmPushError> {
        info!("Sending FCM message to all devices for user {}", user_id.0);
        
        // Get all user tokens
        let tokens = self.token_service.get_user_tokens(user_id).await
            .map_err(|e| FcmPushError::FirebaseError(format!("Failed to get user tokens: {}", e)))?;

        if tokens.is_empty() {
            warn!("No FCM tokens found for user {}", user_id.0);
            return Ok(FcmBatchResult {
                total_sent: 0,
                successful: 0,
                failed: 0,
                results: Vec::new(),
            });
        }

        let mut results = Vec::new();
        let mut successful = 0;
        let mut failed = 0;

        for token_info in tokens {
            let result = self.send_to_token(&token_info.token, message, notification_id).await?;
            
            if result.success {
                successful += 1;
            } else {
                failed += 1;
                
                // Deactivate invalid tokens
                if let Some(error) = &result.error {
                    if error.contains("NOT_FOUND") || error.contains("INVALID_REGISTRATION_TOKEN") {
                        warn!("Deactivating invalid token for user {}", user_id.0);
                        let _ = self.token_service.deactivate_token(&token_info.token).await;
                    }
                }
            }
            
            results.push(result);
        }

        let total = successful + failed;
        info!("FCM batch send complete: {}/{} successful for user {}", successful, total, user_id.0);

        Ok(FcmBatchResult {
            total_sent: total,
            successful,
            failed,
            results,
        })
    }

    async fn send_broadcast(
        &self,
        message: &FcmMessage,
        limit: Option<u32>,
        notification_id: Option<Uuid>,
    ) -> Result<FcmBatchResult, FcmPushError> {
        info!("Sending FCM broadcast message (limit: {:?})", limit);
        
        // Get broadcast tokens
        let tokens = self.token_service.get_broadcast_tokens(limit).await
            .map_err(|e| FcmPushError::FirebaseError(format!("Failed to get broadcast tokens: {}", e)))?;

        if tokens.is_empty() {
            warn!("No FCM tokens available for broadcast");
            return Ok(FcmBatchResult {
                total_sent: 0,
                successful: 0,
                failed: 0,
                results: Vec::new(),
            });
        }

        let mut results = Vec::new();
        let mut successful = 0;
        let mut failed = 0;

        for token_info in tokens {
            let result = self.send_to_token(&token_info.token, message, notification_id).await?;
            
            if result.success {
                successful += 1;
            } else {
                failed += 1;
                
                // Deactivate invalid tokens
                if let Some(error) = &result.error {
                    if error.contains("NOT_FOUND") || error.contains("INVALID_REGISTRATION_TOKEN") {
                        warn!("Deactivating invalid broadcast token");
                        let _ = self.token_service.deactivate_token(&token_info.token).await;
                    }
                }
            }
            
            results.push(result);
        }

        let total = successful + failed;
        info!("FCM broadcast complete: {}/{} successful", successful, total);

        Ok(FcmBatchResult {
            total_sent: total,
            successful,
            failed,
            results,
        })
    }

    async fn send_to_user_platform(
        &self,
        user_id: &UserId,
        platform: DevicePlatform,
        message: &FcmMessage,
        notification_id: Option<Uuid>,
    ) -> Result<FcmBatchResult, FcmPushError> {
        info!("Sending FCM message to {} devices for user {}", 
            match platform {
                DevicePlatform::Web => "web",
                DevicePlatform::Android => "Android",
                DevicePlatform::Ios => "iOS",
            }, 
            user_id.0
        );
        
        // Get user tokens and filter by platform
        let all_tokens = self.token_service.get_user_tokens(user_id).await
            .map_err(|e| FcmPushError::FirebaseError(format!("Failed to get user tokens: {}", e)))?;

        let tokens: Vec<FcmTokenInfo> = all_tokens.into_iter()
            .filter(|token| token.platform == platform)
            .collect();

        if tokens.is_empty() {
            warn!("No {} FCM tokens found for user {}", 
                match platform {
                    DevicePlatform::Web => "web",
                    DevicePlatform::Android => "Android", 
                    DevicePlatform::Ios => "iOS",
                },
                user_id.0
            );
            return Ok(FcmBatchResult {
                total_sent: 0,
                successful: 0,
                failed: 0,
                results: Vec::new(),
            });
        }

        let mut results = Vec::new();
        let mut successful = 0;
        let mut failed = 0;

        for token_info in tokens {
            let result = self.send_to_token(&token_info.token, message, notification_id).await?;
            
            if result.success {
                successful += 1;
            } else {
                failed += 1;
                
                // Deactivate invalid tokens
                if let Some(error) = &result.error {
                    if error.contains("NOT_FOUND") || error.contains("INVALID_REGISTRATION_TOKEN") {
                        warn!("Deactivating invalid {} token for user {}", 
                            match platform {
                                DevicePlatform::Web => "web",
                                DevicePlatform::Android => "Android",
                                DevicePlatform::Ios => "iOS",
                            },
                            user_id.0
                        );
                        let _ = self.token_service.deactivate_token(&token_info.token).await;
                    }
                }
            }
            
            results.push(result);
        }

        let total = successful + failed;
        info!("FCM platform send complete: {}/{} successful for user {} on {}", 
            successful, total, user_id.0,
            match platform {
                DevicePlatform::Web => "web",
                DevicePlatform::Android => "Android",
                DevicePlatform::Ios => "iOS",
            }
        );

        Ok(FcmBatchResult {
            total_sent: total,
            successful,
            failed,
            results,
        })
    }
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

impl FcmMessage {
    /// Create a simple text notification
    pub fn simple_notification(title: String, body: String) -> Self {
        Self {
            title,
            body,
            data: None,
            image_url: None,
            click_action: None,
            badge: None,
            sound: Some("default".to_string()),
            priority: FcmPriority::Normal,
            ttl: Some(86400), // 24 hours
        }
    }

    /// Create a data-only message (silent notification)
    pub fn data_message(data: HashMap<String, String>) -> Self {
        Self {
            title: String::new(),
            body: String::new(),
            data: Some(data),
            image_url: None,
            click_action: None,
            badge: None,
            sound: None,
            priority: FcmPriority::Normal,
            ttl: Some(86400), // 24 hours
        }
    }

    /// Create a rich notification with image and action
    pub fn rich_notification(
        title: String,
        body: String,
        image_url: String,
        click_action: String,
        data: Option<HashMap<String, String>>,
    ) -> Self {
        Self {
            title,
            body,
            data,
            image_url: Some(image_url),
            click_action: Some(click_action),
            badge: None,
            sound: Some("default".to_string()),
            priority: FcmPriority::Normal,
            ttl: Some(86400), // 24 hours
        }
    }
}