use serde::{Deserialize, Serialize};
use async_trait::async_trait;
use std::sync::Arc;
use crate::application::ports::outbound::service_ports::NotificationServicePort;
use crate::infrastructure::firebase_admin::FirebaseAdmin;

#[derive(Debug, thiserror::Error)]
pub enum FcmServiceError {
    #[error("Firebase error: {0}")]
    FirebaseError(String),
    #[error("Network error: {0}")]
    NetworkError(String),
    #[error("Invalid token: {0}")]
    InvalidToken(String),
}

impl From<Box<dyn std::error::Error + Send + Sync>> for FcmServiceError {
    fn from(err: Box<dyn std::error::Error + Send + Sync>) -> Self {
        FcmServiceError::FirebaseError(err.to_string())
    }
}

/// FCM (Firebase Cloud Messaging) service for push notifications
pub struct FcmService {
    firebase_admin: Arc<FirebaseAdmin>,
}

impl FcmService {
    pub fn new(firebase_admin: Arc<FirebaseAdmin>) -> Self {
        Self {
            firebase_admin
        }
    }

    pub async fn send_notification(
        &self,
        _notification: &FcmNotification,
    ) -> Result<FcmResponse, Box<dyn std::error::Error + Send + Sync>> {
        // Placeholder implementation
        Ok(FcmResponse {
            success: true,
            message_id: Some("mock_id".to_string()),
            error: None,
            name: "projects/test/messages/mock_id".to_string(),
        })
    }
    
    pub async fn send_message(
        &self,
        _message: FcmMessage,
    ) -> Result<FcmResponse, crate::core::errors::AppError> {
        // Placeholder implementation
        Ok(FcmResponse {
            success: true,
            message_id: Some("mock_message_id".to_string()),
            error: None,
            name: "projects/test/messages/mock_message_id".to_string(),
        })
    }
    
    pub async fn send_to_topic(
        &self,
        _topic: String,
        _notification: FcmNotification,
        _data: Option<serde_json::Value>,
    ) -> Result<FcmResponse, crate::core::errors::AppError> {
        // Placeholder implementation
        Ok(FcmResponse {
            success: true,
            message_id: Some("mock_topic_message_id".to_string()),
            error: None,
            name: "projects/test/messages/mock_topic_message_id".to_string(),
        })
    }
    
    pub async fn send_multicast(
        &self,
        _tokens: Vec<String>,
        _notification: FcmNotification,
        _data: Option<serde_json::Value>,
    ) -> Result<DeliveryStats, crate::core::errors::AppError> {
        // Placeholder implementation
        Ok(DeliveryStats {
            sent: 1,
            failed: 0,
        })
    }
}

#[async_trait]
impl NotificationServicePort for FcmService {
    type Error = FcmServiceError;
    
    async fn send_push_notification(&self, device_token: &str, message: &str) -> Result<(), Self::Error> {
        // Create FCM notification from the device token and message
        let notification = FcmNotification {
            title: Some("Notification".to_string()),
            body: Some(message.to_string()),
            image: None,
        };
        
        // Use the existing send_notification method
        let response = self.send_notification(&notification).await
            .map_err(|e| FcmServiceError::FirebaseError(e.to_string()))?;
        
        // Check if the notification was sent successfully
        if response.success {
            Ok(())
        } else {
            Err(FcmServiceError::FirebaseError(
                response.error.unwrap_or_else(|| "Unknown FCM error".to_string())
            ))
        }
    }
}

/// FCM notification data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FcmNotification {
    pub title: Option<String>,
    pub body: Option<String>,
    pub image: Option<String>,
}

/// FCM message wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FcmMessage {
    pub target: FcmTarget,
    pub data: Option<serde_json::Value>,
    pub notification: Option<FcmNotification>,
    pub android: Option<FcmAndroidConfig>,
    pub webpush: Option<FcmWebpushConfig>,
}

/// FCM target specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FcmTarget {
    Token { token: String },
    Topic { topic: String },
    Condition { condition: String },
}

/// FCM response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FcmResponse {
    pub success: bool,
    pub message_id: Option<String>,
    pub error: Option<String>,
    pub name: String,
}

/// Delivery statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeliveryStats {
    pub sent: u64,
    pub failed: u64,
}

impl Default for DeliveryStats {
    fn default() -> Self {
        Self {
            sent: 0,
            failed: 0,
        }
    }
}

/// FCM topic service for managing topic subscriptions
pub struct FcmTopicService;

impl FcmTopicService {
    pub fn new() -> Self {
        Self
    }

    pub async fn subscribe_to_topic(
        &self,
        tokens: &[String],
        topic: &str,
    ) -> Result<TopicSubscriptionResult, Box<dyn std::error::Error + Send + Sync>> {
        // Placeholder implementation
        Ok(TopicSubscriptionResult {
            successful_tokens: tokens.to_vec(),
            failed_tokens: Vec::new(),
        })
    }

    pub async fn unsubscribe_from_topic(
        &self,
        tokens: &[String],
        topic: &str,
    ) -> Result<TopicSubscriptionResult, Box<dyn std::error::Error + Send + Sync>> {
        // Placeholder implementation
        Ok(TopicSubscriptionResult {
            successful_tokens: tokens.to_vec(),
            failed_tokens: Vec::new(),
        })
    }
}

/// Topic subscription result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopicSubscriptionResult {
    pub successful_tokens: Vec<String>,
    pub failed_tokens: Vec<String>,
}

/// FCM Android configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FcmAndroidConfig {
    pub priority: Option<String>,
    pub ttl: Option<String>,
    pub notification: Option<serde_json::Value>,
}

/// FCM Webpush configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FcmWebpushConfig {
    pub headers: Option<serde_json::Value>,
    pub data: Option<serde_json::Value>,
    pub notification: Option<serde_json::Value>,
    pub fcm_options: Option<FcmWebpushFcmOptions>,
}

/// FCM Webpush FCM options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FcmWebpushFcmOptions {
    pub link: Option<String>,
    pub analytics_label: Option<String>,
}