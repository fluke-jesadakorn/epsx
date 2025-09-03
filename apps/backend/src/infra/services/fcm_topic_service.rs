use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::time::Duration;
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::core::errors::{AppError, ErrorKind, ErrorContext};
use crate::infra::firebase::FirebaseAdmin;
use crate::infra::services::fcm_service::{FcmService, FcmNotification};

/// FCM Topic Management Service
/// Handles topic subscription, unsubscription, and broadcasting
pub struct FcmTopicService {
    client: Client,
    firebase_admin: FirebaseAdmin,
    fcm_service: FcmService,
    project_id: String,
}

/// Topic subscription request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopicSubscriptionRequest {
    pub to: String, // Format: "/topics/{topic_name}"
    pub registration_tokens: Vec<String>,
}

/// Topic subscription response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopicSubscriptionResponse {
    pub results: Vec<TopicResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopicResult {
    pub error: Option<String>,
}

/// Built-in topics for the EPSX platform
#[derive(Debug, Clone)]
pub struct PlatformTopics;

impl PlatformTopics {
    pub const EPSX_ALL_USERS: &'static str = "epsx_all_users";
    pub const EPSX_ADMIN_USERS: &'static str = "epsx_admin_users"; 
    pub const EPSX_PREMIUM_USERS: &'static str = "epsx_premium_users";
    pub const EPSX_ANALYTICS_USERS: &'static str = "epsx_analytics_users";
    pub const EPSX_SECURITY_ALERTS: &'static str = "epsx_security_alerts";
    pub const EPSX_SYSTEM_UPDATES: &'static str = "epsx_system_updates";

    /// Get all platform topics
    pub fn all_topics() -> Vec<&'static str> {
        vec![
            Self::EPSX_ALL_USERS,
            Self::EPSX_ADMIN_USERS,
            Self::EPSX_PREMIUM_USERS,
            Self::EPSX_ANALYTICS_USERS,
            Self::EPSX_SECURITY_ALERTS,
            Self::EPSX_SYSTEM_UPDATES,
        ]
    }

    /// Get topics for a user based on permissions
    pub fn topics_for_permissions(permissions: &[String]) -> Vec<&'static str> {
        let mut topics = vec![Self::EPSX_ALL_USERS];

        for permission in permissions {
            if permission.starts_with("admin:") {
                topics.push(Self::EPSX_ADMIN_USERS);
                break;
            }
        }

        if permissions.iter().any(|p| p.contains("premium")) {
            topics.push(Self::EPSX_PREMIUM_USERS);
        }

        if permissions.iter().any(|p| p.contains("analytics")) {
            topics.push(Self::EPSX_ANALYTICS_USERS);
        }

        topics.into_iter().collect()
    }
}

impl FcmTopicService {
    pub fn new(firebase_admin: FirebaseAdmin, project_id: String) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .user_agent("EPSX/1.0 FCM-Topic-Client")
            .build()
            .expect("Failed to create HTTP client");

        let fcm_service = FcmService::new(std::sync::Arc::new(firebase_admin.clone()));

        Self {
            client,
            firebase_admin,
            fcm_service,
            project_id,
        }
    }

    /// Subscribe tokens to a topic
    pub async fn subscribe_to_topic(&self, topic_name: &str, tokens: Vec<String>) -> Result<TopicSubscriptionResponse, AppError> {
        if !FcmService::is_valid_topic(topic_name) {
            return Err(AppError {
                kind: ErrorKind::ValidationError,
                message: format!("Invalid topic name: {}", topic_name),
                context: ErrorContext::default(),
                correlation_id: Uuid::new_v4().to_string(),
                timestamp: chrono::Utc::now(),
                stack_trace: None,
            });
        }

        // Limit batch size to 1000 as per Firebase limits
        if tokens.len() > 1000 {
            return Err(AppError {
                kind: ErrorKind::ValidationError,
                message: "Cannot subscribe more than 1000 tokens in a single request".to_string(),
                context: ErrorContext::default(),
                correlation_id: Uuid::new_v4().to_string(),
                timestamp: chrono::Utc::now(),
                stack_trace: None,
            });
        }

        let access_token = self.get_access_token().await?;
        
        // Note: Firebase has deprecated the IID API endpoints for topic management
        // The HTTP v1 API doesn't provide direct replacements for bulk topic operations
        // For now, we attempt the legacy IID API with OAuth2 authentication
        let legacy_url = "https://iid.googleapis.com/iid/v1:batchAdd";
        
        let legacy_payload = TopicSubscriptionRequest {
            to: format!("/topics/{}", topic_name),
            registration_tokens: tokens.clone(),
        };

        warn!("Using deprecated IID API for topic subscription - Google may reject this request");
        info!("Attempting to subscribe {} tokens to topic: {}", tokens.len(), topic_name);

        let response = self.client
            .post(legacy_url)
            .header("Authorization", format!("Bearer {}", access_token))
            .header("Content-Type", "application/json")
            .header("User-Agent", "EPSX/1.0 FCM-Topic-Client")
            .json(&legacy_payload)
            .send()
            .await
            .map_err(|e| AppError {
                kind: ErrorKind::ExternalServiceError,
                message: format!("Failed to subscribe to topic: {}", e),
                context: ErrorContext::default(),
                correlation_id: Uuid::new_v4().to_string(),
                timestamp: chrono::Utc::now(),
                stack_trace: None,
            })?;

        if response.status().is_success() {
            let subscription_response: TopicSubscriptionResponse = response.json().await
                .map_err(|e| AppError {
                    kind: ErrorKind::ValidationError,
                    message: format!("Invalid topic subscription response: {}", e),
                    context: ErrorContext::default(),
                    correlation_id: Uuid::new_v4().to_string(),
                    timestamp: chrono::Utc::now(),
                    stack_trace: None,
                })?;

            info!("Successfully subscribed tokens to topic: {}", topic_name);
            Ok(subscription_response)
        } else {
            let error_text = response.text().await.unwrap_or_default();
            
            // Check if this is the known deprecated API error
            if error_text.contains("Authentication using server key is deprecated") {
                warn!("Google has deprecated the IID API and rejected OAuth2 authentication");
                warn!("Topic subscription for '{}' failed due to deprecated API: {}", topic_name, error_text);
                
                // Return a graceful failure response instead of an error
                // This allows the application to continue working without topic subscriptions
                Ok(TopicSubscriptionResponse {
                    results: vec![TopicResult { 
                        error: Some("IID API deprecated by Google".to_string()) 
                    }],
                })
            } else {
                error!("Failed to subscribe to topic {}: {}", topic_name, error_text);
                
                Err(AppError {
                    kind: ErrorKind::ExternalServiceError,
                    message: format!("Topic subscription failed: {}", error_text),
                    context: ErrorContext::default(),
                    correlation_id: Uuid::new_v4().to_string(),
                    timestamp: chrono::Utc::now(),
                    stack_trace: None,
                })
            }
        }
    }

    /// Unsubscribe tokens from a topic
    pub async fn unsubscribe_from_topic(&self, topic_name: &str, tokens: Vec<String>) -> Result<TopicSubscriptionResponse, AppError> {
        if !FcmService::is_valid_topic(topic_name) {
            return Err(AppError {
                kind: ErrorKind::ValidationError,
                message: format!("Invalid topic name: {}", topic_name),
                context: ErrorContext::default(),
                correlation_id: Uuid::new_v4().to_string(),
                timestamp: chrono::Utc::now(),
                stack_trace: None,
            });
        }

        // Limit batch size to 1000 as per Firebase limits
        if tokens.len() > 1000 {
            return Err(AppError {
                kind: ErrorKind::ValidationError,
                message: "Cannot unsubscribe more than 1000 tokens in a single request".to_string(),
                context: ErrorContext::default(),
                correlation_id: Uuid::new_v4().to_string(),
                timestamp: chrono::Utc::now(),
                stack_trace: None,
            });
        }

        let access_token = self.get_access_token().await?;
        
        // Note: Firebase has deprecated the IID API endpoints for topic management
        // The HTTP v1 API doesn't provide direct replacements for bulk topic operations
        // For now, we attempt the legacy IID API with OAuth2 authentication
        let legacy_url = "https://iid.googleapis.com/iid/v1:batchRemove";
        
        let legacy_payload = TopicSubscriptionRequest {
            to: format!("/topics/{}", topic_name),
            registration_tokens: tokens.clone(),
        };

        warn!("Using deprecated IID API for topic unsubscription - Google may reject this request");
        info!("Attempting to unsubscribe {} tokens from topic: {}", tokens.len(), topic_name);

        let response = self.client
            .post(legacy_url)
            .header("Authorization", format!("Bearer {}", access_token))
            .header("Content-Type", "application/json")
            .header("User-Agent", "EPSX/1.0 FCM-Topic-Client")
            .json(&legacy_payload)
            .send()
            .await
            .map_err(|e| AppError {
                kind: ErrorKind::ExternalServiceError,
                message: format!("Failed to unsubscribe from topic: {}", e),
                context: ErrorContext::default(),
                correlation_id: Uuid::new_v4().to_string(),
                timestamp: chrono::Utc::now(),
                stack_trace: None,
            })?;

        if response.status().is_success() {
            let subscription_response: TopicSubscriptionResponse = response.json().await
                .map_err(|e| AppError {
                    kind: ErrorKind::ValidationError,
                    message: format!("Invalid topic unsubscription response: {}", e),
                    context: ErrorContext::default(),
                    correlation_id: Uuid::new_v4().to_string(),
                    timestamp: chrono::Utc::now(),
                    stack_trace: None,
                })?;

            info!("Successfully unsubscribed tokens from topic: {}", topic_name);
            Ok(subscription_response)
        } else {
            let error_text = response.text().await.unwrap_or_default();
            
            // Check if this is the known deprecated API error
            if error_text.contains("Authentication using server key is deprecated") {
                warn!("Google has deprecated the IID API and rejected OAuth2 authentication");
                warn!("Topic unsubscription for '{}' failed due to deprecated API: {}", topic_name, error_text);
                
                // Return a graceful failure response instead of an error
                // This allows the application to continue working without topic subscriptions
                Ok(TopicSubscriptionResponse {
                    results: vec![TopicResult { 
                        error: Some("IID API deprecated by Google".to_string()) 
                    }],
                })
            } else {
                error!("Failed to unsubscribe from topic {}: {}", topic_name, error_text);
                
                Err(AppError {
                    kind: ErrorKind::ExternalServiceError,
                    message: format!("Topic unsubscription failed: {}", error_text),
                    context: ErrorContext::default(),
                    correlation_id: Uuid::new_v4().to_string(),
                    timestamp: chrono::Utc::now(),
                    stack_trace: None,
                })
            }
        }
    }

    /// Broadcast notification to a topic
    pub async fn broadcast_to_topic(&self, topic_name: &str, title: &str, body: &str, data: Option<serde_json::Value>) -> Result<String, AppError> {
        let notification = FcmNotification {
            title: Some(title.to_string()),
            body: Some(body.to_string()),
            image: None,
        };

        let response = self.fcm_service.send_to_topic(topic_name.to_string(), notification, data).await?;
        
        info!("Broadcasted message to topic {}: {}", topic_name, response.name);
        Ok(response.name)
    }

    /// Smart topic management - subscribe user to appropriate topics based on permissions
    pub async fn manage_user_topics(&self, user_id: &str, user_permissions: &[String], fcm_token: &str) -> Result<Vec<String>, AppError> {
        let appropriate_topics = PlatformTopics::topics_for_permissions(user_permissions);
        let mut subscribed_topics = Vec::new();

        for topic in appropriate_topics {
            match self.subscribe_to_topic(topic, vec![fcm_token.to_string()]).await {
                Ok(_) => {
                    subscribed_topics.push(topic.to_string());
                    info!("User {} subscribed to topic: {}", user_id, topic);
                }
                Err(e) => {
                    warn!("Failed to subscribe user {} to topic {}: {}", user_id, topic, e.message);
                }
            }
        }

        Ok(subscribed_topics)
    }

    /// Broadcast to all users (admin functionality)
    pub async fn broadcast_to_all_users(&self, title: &str, body: &str, data: Option<serde_json::Value>) -> Result<String, AppError> {
        self.broadcast_to_topic(PlatformTopics::EPSX_ALL_USERS, title, body, data).await
    }

    /// Broadcast to admin users only
    pub async fn broadcast_to_admins(&self, title: &str, body: &str, data: Option<serde_json::Value>) -> Result<String, AppError> {
        self.broadcast_to_topic(PlatformTopics::EPSX_ADMIN_USERS, title, body, data).await
    }

    /// Broadcast security alert
    pub async fn broadcast_security_alert(&self, title: &str, body: &str, severity: &str) -> Result<String, AppError> {
        let data = json!({
            "type": "security_alert",
            "severity": severity,
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "click_action": "/security/alerts"
        });

        self.broadcast_to_topic(PlatformTopics::EPSX_SECURITY_ALERTS, title, body, Some(data)).await
    }

    /// Broadcast system update notification
    pub async fn broadcast_system_update(&self, title: &str, body: &str, version: &str) -> Result<String, AppError> {
        let data = json!({
            "type": "system_update",
            "version": version,
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "click_action": "/updates"
        });

        self.broadcast_to_topic(PlatformTopics::EPSX_SYSTEM_UPDATES, title, body, Some(data)).await
    }

    /// Get OAuth 2.0 access token
    async fn get_access_token(&self) -> Result<String, AppError> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_platform_topics() {
        let topics = PlatformTopics::all_topics();
        assert!(topics.len() > 0);
        assert!(topics.contains(&PlatformTopics::EPSX_ALL_USERS));
        assert!(topics.contains(&PlatformTopics::EPSX_ADMIN_USERS));
    }

    #[test]
    fn test_topics_for_permissions() {
        // Admin user
        let admin_perms = vec!["admin:users:read".to_string()];
        let admin_topics = PlatformTopics::topics_for_permissions(&admin_perms);
        assert!(admin_topics.contains(&PlatformTopics::EPSX_ALL_USERS));
        assert!(admin_topics.contains(&PlatformTopics::EPSX_ADMIN_USERS));

        // Regular user
        let user_perms = vec!["epsx:dashboard:read".to_string()];
        let user_topics = PlatformTopics::topics_for_permissions(&user_perms);
        assert!(user_topics.contains(&PlatformTopics::EPSX_ALL_USERS));
        assert!(!user_topics.contains(&PlatformTopics::EPSX_ADMIN_USERS));

        // Premium user
        let premium_perms = vec!["epsx:premium:access".to_string()];
        let premium_topics = PlatformTopics::topics_for_permissions(&premium_perms);
        assert!(premium_topics.contains(&PlatformTopics::EPSX_ALL_USERS));
        assert!(premium_topics.contains(&PlatformTopics::EPSX_PREMIUM_USERS));
    }
}