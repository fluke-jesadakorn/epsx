use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::core::errors::{AppError, ErrorKind, ErrorContext};
use crate::infra::firebase::{FirebaseAdmin};

/// FCM HTTP v1 API Implementation
/// Reference: https://firebase.google.com/docs/cloud-messaging/http-server-ref
pub struct FcmService {
    client: Client,
    firebase_admin: FirebaseAdmin,
    project_id: String,
}

/// FCM Message according to HTTP v1 API specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FcmMessage {
    /// Token, topic, or condition for targeting
    #[serde(flatten)]
    pub target: FcmTarget,
    /// Message data
    pub data: Option<Value>,
    /// Notification payload
    pub notification: Option<FcmNotification>,
    /// Android specific options
    pub android: Option<FcmAndroidConfig>,
    /// Web push specific options
    pub webpush: Option<FcmWebpushConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum FcmTarget {
    Token { token: String },
    Topic { topic: String },
    Condition { condition: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FcmNotification {
    pub title: Option<String>,
    pub body: Option<String>,
    pub image: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FcmAndroidConfig {
    pub priority: Option<String>, // "normal" or "high"
    pub ttl: Option<String>,      // e.g. "3600s"
    pub notification: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FcmWebpushConfig {
    pub headers: Option<Value>,
    pub data: Option<Value>,
    pub notification: Option<Value>,
    pub fcm_options: Option<FcmWebpushFcmOptions>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FcmWebpushFcmOptions {
    pub link: Option<String>,
    pub analytics_label: Option<String>,
}

/// FCM Response from HTTP v1 API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FcmResponse {
    pub name: String, // projects/{project_id}/messages/{message_id}
}

/// FCM Error Response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FcmErrorResponse {
    pub error: FcmError,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FcmError {
    pub code: u16,
    pub message: String,
    pub status: String,
    pub details: Option<Vec<Value>>,
}

/// Delivery statistics for monitoring
#[derive(Debug, Clone)]
pub struct DeliveryStats {
    pub sent: u32,
    pub failed: u32,
    pub invalid_tokens: Vec<String>,
    pub retry_after: Option<Duration>,
}

impl FcmService {
    pub fn new(firebase_admin: std::sync::Arc<FirebaseAdmin>) -> Self {
        // Use default project ID for now
        let project_id = "epsx-449804".to_string();
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .pool_idle_timeout(Duration::from_secs(90))
            .user_agent("EPSX/1.0 FCM-Client")
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            firebase_admin: (*firebase_admin).clone(),
            project_id,
        }
    }

    /// Send a single FCM message using HTTP v1 API
    pub async fn send_message(&self, message: FcmMessage) -> Result<FcmResponse, AppError> {
        let access_token = self.get_access_token().await?;
        
        let url = format!(
            "https://fcm.googleapis.com/v1/projects/{}/messages:send",
            self.project_id
        );

        let payload = json!({
            "message": message
        });

        info!("Sending FCM message to: {:?}", message.target);
        
        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", access_token))
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await
            .map_err(|e| AppError {
                kind: ErrorKind::ExternalServiceError,
                message: format!("Failed to send FCM request: {}", e),
                context: ErrorContext::default(),
                correlation_id: Uuid::new_v4().to_string(),
                timestamp: chrono::Utc::now(),
                stack_trace: None,
            })?;

        let status = response.status();
        let response_text = response.text().await.map_err(|e| AppError {
            kind: ErrorKind::ExternalServiceError,
            message: format!("Failed to read FCM response: {}", e),
            context: ErrorContext::default(),
            correlation_id: Uuid::new_v4().to_string(),
            timestamp: chrono::Utc::now(),
            stack_trace: None,
        })?;

        if status.is_success() {
            let fcm_response: FcmResponse = serde_json::from_str(&response_text)
                .map_err(|e| AppError {
                    kind: ErrorKind::ValidationError,
                    message: format!("Invalid FCM response format: {}", e),
                    context: ErrorContext::default(),
                    correlation_id: Uuid::new_v4().to_string(),
                    timestamp: chrono::Utc::now(),
                    stack_trace: None,
                })?;

            info!("FCM message sent successfully: {}", fcm_response.name);
            Ok(fcm_response)
        } else {
            let error_response: FcmErrorResponse = serde_json::from_str(&response_text)
                .map_err(|e| AppError {
                    kind: ErrorKind::ValidationError,
                    message: format!("Invalid FCM error response format: {}", e),
                    context: ErrorContext::default(),
                    correlation_id: Uuid::new_v4().to_string(),
                    timestamp: chrono::Utc::now(),
                    stack_trace: None,
                })?;

            error!("FCM API error: {} - {}", error_response.error.code, error_response.error.message);
            
            Err(AppError {
                kind: ErrorKind::ExternalServiceError,
                message: format!("FCM API error: {}", error_response.error.message),
                context: ErrorContext::default(),
                correlation_id: Uuid::new_v4().to_string(),
                timestamp: chrono::Utc::now(),
                stack_trace: None,
            })
        }
    }

    /// Send messages to multiple tokens with exponential backoff
    pub async fn send_multicast(&self, tokens: Vec<String>, notification: FcmNotification, data: Option<Value>) -> Result<DeliveryStats, AppError> {
        let mut stats = DeliveryStats {
            sent: 0,
            failed: 0,
            invalid_tokens: Vec::new(),
            retry_after: None,
        };

        const MAX_TOKENS_PER_REQUEST: usize = 500; // FCM limit
        const MAX_RETRIES: u32 = 3;

        for chunk in tokens.chunks(MAX_TOKENS_PER_REQUEST) {
            for token in chunk {
                let message = FcmMessage {
                    target: FcmTarget::Token { token: token.clone() },
                    data: data.clone(),
                    notification: Some(notification.clone()),
                    android: Some(FcmAndroidConfig {
                        priority: Some("high".to_string()),
                        ttl: Some("3600s".to_string()),
                        notification: None,
                    }),
                    webpush: Some(FcmWebpushConfig {
                        headers: Some(json!({"Urgency": "high"})),
                        data: None,
                        notification: None,
                        fcm_options: None,
                    }),
                };

                let mut retry_count = 0;
                let mut last_error = None;

                while retry_count < MAX_RETRIES {
                    match self.send_message(message.clone()).await {
                        Ok(_) => {
                            stats.sent += 1;
                            break;
                        }
                        Err(e) => {
                            retry_count += 1;
                            last_error = Some(e);

                            if retry_count < MAX_RETRIES {
                                let delay = Duration::from_millis(100 * (2_u64.pow(retry_count)));
                                tokio::time::sleep(delay).await;
                            }
                        }
                    }
                }

                if retry_count >= MAX_RETRIES {
                    stats.failed += 1;
                    if let Some(error) = last_error {
                        warn!("Failed to send to token {} after {} retries: {}", token, MAX_RETRIES, error.message);
                        if error.message.contains("invalid") || error.message.contains("not-registered") {
                            stats.invalid_tokens.push(token.clone());
                        }
                    }
                }
            }

            // Respect rate limits with traffic ramping
            if chunk.len() > 100 {
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
        }

        info!("Multicast completed: {} sent, {} failed, {} invalid tokens", 
              stats.sent, stats.failed, stats.invalid_tokens.len());

        Ok(stats)
    }

    /// Send to a topic
    pub async fn send_to_topic(&self, topic: String, notification: FcmNotification, data: Option<Value>) -> Result<FcmResponse, AppError> {
        let message = FcmMessage {
            target: FcmTarget::Topic { topic },
            data,
            notification: Some(notification),
            android: Some(FcmAndroidConfig {
                priority: Some("normal".to_string()),
                ttl: Some("86400s".to_string()), // 24 hours
                notification: None,
            }),
            webpush: Some(FcmWebpushConfig {
                headers: Some(json!({"Urgency": "normal"})),
                data: None,
                notification: None,
                fcm_options: None,
            }),
        };

        self.send_message(message).await
    }

    /// Send with condition (advanced targeting)
    pub async fn send_to_condition(&self, condition: String, notification: FcmNotification, data: Option<Value>) -> Result<FcmResponse, AppError> {
        let message = FcmMessage {
            target: FcmTarget::Condition { condition },
            data,
            notification: Some(notification),
            android: Some(FcmAndroidConfig {
                priority: Some("normal".to_string()),
                ttl: Some("86400s".to_string()),
                notification: None,
            }),
            webpush: None,
        };

        self.send_message(message).await
    }

    /// Get OAuth 2.0 access token for FCM API
    async fn get_access_token(&self) -> Result<String, AppError> {
        // Use Firebase admin to get access token
        self.firebase_admin.get_access_token().await
            .map_err(|e| AppError {
                kind: ErrorKind::AuthenticationError,
                message: format!("Failed to get FCM access token: {}", e),
                context: ErrorContext::default(),
                correlation_id: Uuid::new_v4().to_string(),
                timestamp: chrono::Utc::now(),
                stack_trace: None,
            })
    }
}

/// Helper functions for common notification patterns
impl FcmService {
    /// Create a simple text notification
    pub fn create_simple_notification(title: &str, body: &str) -> FcmNotification {
        FcmNotification {
            title: Some(title.to_string()),
            body: Some(body.to_string()),
            image: None,
        }
    }

    /// Create a rich notification with image and action
    pub fn create_rich_notification(title: &str, body: &str, image_url: &str, click_action: &str) -> (FcmNotification, Value) {
        let notification = FcmNotification {
            title: Some(title.to_string()),
            body: Some(body.to_string()),
            image: Some(image_url.to_string()),
        };

        let data = json!({
            "click_action": click_action,
            "timestamp": SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs()
                .to_string()
        });

        (notification, data)
    }

    /// Validate FCM token format
    pub fn is_valid_token(token: &str) -> bool {
        !token.is_empty() && token.len() > 50 && token.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-' || c == ':')
    }

    /// Validate topic name format
    pub fn is_valid_topic(topic: &str) -> bool {
        !topic.is_empty() 
            && topic.len() <= 900
            && topic.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-')
            && !topic.starts_with("/topics/")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_validation() {
        assert!(!FcmService::is_valid_token(""));
        assert!(!FcmService::is_valid_token("short"));
        assert!(FcmService::is_valid_token("d1234567890abcdef1234567890abcdef12345678:APA91bHs_valid_token_format"));
    }

    #[test]
    fn test_topic_validation() {
        assert!(FcmService::is_valid_topic("epsx_users"));
        assert!(FcmService::is_valid_topic("admin-notifications"));
        assert!(!FcmService::is_valid_topic(""));
        assert!(!FcmService::is_valid_topic("/topics/invalid"));
        assert!(!FcmService::is_valid_topic("invalid topic with spaces"));
    }

    #[test]
    fn test_notification_creation() {
        let notification = FcmService::create_simple_notification("Test", "Body");
        assert_eq!(notification.title, Some("Test".to_string()));
        assert_eq!(notification.body, Some("Body".to_string()));
        assert_eq!(notification.image, None);

        let (rich_notif, data) = FcmService::create_rich_notification("Rich", "Body", "image.png", "/dashboard");
        assert_eq!(rich_notif.image, Some("image.png".to_string()));
        assert!(data["click_action"].as_str().unwrap_or("") == "/dashboard");
    }
}