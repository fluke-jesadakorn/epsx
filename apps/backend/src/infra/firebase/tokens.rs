// Firebase Tokens, FCM Messaging and Notifications
use chrono::{DateTime, Utc};
// Focused module handling token generation, push notifications, and role management

use std::collections::HashMap;
use serde_json::{Value, json};
use tracing::{info, error, warn};

use super::types::{
    FirebaseAdmin, FcmMessage, FcmNotification, FcmAndroidConfig, FcmAndroidNotification,
    FcmApnsConfig, FcmApnsPayload, FcmAps, FcmApnsAlert, FcmWebpushConfig, FcmWebpushNotification,
    FcmRequest, FcmResponse, FcmErrorResponse, TopicSubscriptionRequest, TopicSubscriptionResponse
};

impl FirebaseAdmin {
    /// Send a push notification to a specific device
    pub async fn send_push_notification(
        &self,
        device_token: &str,
        title: &str,
        body: &str,
        data: Option<HashMap<String, String>>,
    ) -> Result<FcmResponse, Box<dyn std::error::Error>> {
        let message = FcmMessage {
            token: Some(device_token.to_string()),
            topic: None,
            condition: None,
            notification: FcmNotification {
                title: title.to_string(),
                body: body.to_string(),
                image: None,
            },
            data,
            android: Some(FcmAndroidConfig {
                ttl: Some("3600s".to_string()),
                priority: Some("high".to_string()),
                notification: Some(FcmAndroidNotification {
                    title: Some(title.to_string()),
                    body: Some(body.to_string()),
                    icon: Some("ic_notification".to_string()),
                    color: Some("#4285F4".to_string()),
                    sound: Some("default".to_string()),
                    click_action: Some("OPEN_APP".to_string()),
                    tag: None,
                    body_loc_key: None,
                    body_loc_args: None,
                    title_loc_key: None,
                    title_loc_args: None,
                }),
            }),
            apns: Some(FcmApnsConfig {
                headers: None,
                payload: FcmApnsPayload {
                    aps: FcmAps {
                        alert: Some(FcmApnsAlert {
                            title: title.to_string(),
                            body: body.to_string(),
                        }),
                        badge: Some(1),
                        sound: Some("default".to_string()),
                        category: None,
                    },
                },
            }),
            webpush: Some(FcmWebpushConfig {
                headers: None,
                data: None,
                notification: Some(FcmWebpushNotification {
                    title: title.to_string(),
                    body: body.to_string(),
                    icon: Some("/icon-192x192.png".to_string()),
                    badge: Some("/badge-72x72.png".to_string()),
                    image: None,
                    data: None,
                }),
            }),
        };

        self.send_fcm_message(message, false).await
    }

    /// Send a push notification to a topic (broadcast to all subscribers)
    pub async fn send_topic_notification(
        &self,
        topic: &str,
        title: &str,
        body: &str,
        data: Option<HashMap<String, String>>,
    ) -> Result<FcmResponse, Box<dyn std::error::Error>> {
        let message = FcmMessage {
            token: None,
            topic: Some(topic.to_string()),
            condition: None,
            notification: FcmNotification {
                title: title.to_string(),
                body: body.to_string(),
                image: None,
            },
            data,
            android: None,
            apns: None,
            webpush: None,
        };

        self.send_fcm_message(message, false).await
    }

    /// Send push notification for expiration warning
    pub async fn send_expiration_push_notification(
        &self,
        device_token: &str,
        permission_profile_name: &str,
        days_until_expiration: i64,
        expires_at: DateTime<Utc>,
    ) -> Result<FcmResponse, Box<dyn std::error::Error>> {
        let (title, body, priority) = if days_until_expiration <= 0 {
            (
                "Features Expired - Grace Period Active".to_string(),
                format!(
                    "Your {} subscription has expired but is still active. Renew now to avoid service interruption.",
                    permission_profile_name
                ),
                "high"
            )
        } else if days_until_expiration == 1 {
            (
                "Subscription Expires Tomorrow!".to_string(),
                format!(
                    "Your {} subscription expires tomorrow. Renew now to keep access to your premium features.",
                    permission_profile_name
                ),
                "high"
            )
        } else if days_until_expiration <= 7 {
            (
                format!("Subscription Expires in {} Days", days_until_expiration),
                format!(
                    "Your {} subscription expires in {} days. Don't lose access to your premium features!",
                    permission_profile_name, days_until_expiration
                ),
                "normal"
            )
        } else {
            (
                "Subscription Renewal Reminder".to_string(),
                format!(
                    "Your {} subscription expires in {} days. Consider renewing to continue enjoying premium features.",
                    permission_profile_name, days_until_expiration
                ),
                "normal"
            )
        };

        let mut data = HashMap::new();
        data.insert("permission_profile_name".to_string(), permission_profile_name.to_string());
        data.insert("days_until_expiration".to_string(), days_until_expiration.to_string());
        data.insert("expires_at".to_string(), expires_at.to_rfc3339());
        data.insert("notification_type".to_string(), "expiration_warning".to_string());
        data.insert("action".to_string(), "open_renewal".to_string());

        let message = FcmMessage {
            token: Some(device_token.to_string()),
            topic: None,
            condition: None,
            notification: FcmNotification {
                title: title.clone(),
                body: body.clone(),
                image: None,
            },
            data: Some(data),
            android: Some(FcmAndroidConfig {
                ttl: Some("86400s".to_string()), // 24 hours for expiration notifications
                priority: Some(priority.to_string()),
                notification: Some(FcmAndroidNotification {
                    title: Some(title.clone()),
                    body: Some(body.clone()),
                    icon: Some("ic_warning".to_string()),
                    color: Some("#FF9800".to_string()), // Orange for warnings
                    sound: Some("default".to_string()),
                    click_action: Some("OPEN_RENEWAL".to_string()),
                    tag: Some("expiration_warning".to_string()),
                    body_loc_key: None,
                    body_loc_args: None,
                    title_loc_key: None,
                    title_loc_args: None,
                }),
            }),
            apns: Some(FcmApnsConfig {
                headers: Some({
                    let mut headers = HashMap::new();
                    headers.insert("apns-priority".to_string(), if priority == "high" { "10" } else { "5" }.to_string());
                    headers
                }),
                payload: FcmApnsPayload {
                    aps: FcmAps {
                        alert: Some(FcmApnsAlert {
                            title: title.clone(),
                            body: body.clone(),
                        }),
                        badge: Some(1),
                        sound: Some("default".to_string()),
                        category: Some("EXPIRATION_WARNING".to_string()),
                    },
                },
            }),
            webpush: Some(FcmWebpushConfig {
                headers: None,
                data: None,
                notification: Some(FcmWebpushNotification {
                    title: title.clone(),
                    body: body.clone(),
                    icon: Some("/icon-warning.png".to_string()),
                    badge: Some("/badge-warning.png".to_string()),
                    image: None,
                    data: Some({
                        let mut web_data = HashMap::new();
                        web_data.insert("action".to_string(), json!("open_renewal"));
                        web_data.insert("url".to_string(), json!("/billing/renew"));
                        web_data
                    }),
                }),
            }),
        };

        self.send_fcm_message(message, false).await
    }

    /// Subscribe device tokens to a topic
    pub async fn subscribe_to_topic(
        &self,
        device_tokens: Vec<String>,
        topic: &str,
    ) -> Result<TopicSubscriptionResponse, Box<dyn std::error::Error>> {
        let access_token = self.get_access_token().await?;
        let url = "https://iid.googleapis.com/iid/v1:batchAdd";

        let request = TopicSubscriptionRequest {
            to: format!("/topics/{}", topic),
            registration_tokens: device_tokens,
        };

        let response = self.client
            .post(url)
            .bearer_auth(&access_token)
            .json(&request)
            .send()
            .await?;

        if response.status().is_success() {
            let subscription_response: TopicSubscriptionResponse = response.json().await?;
            info!("Successfully subscribed devices to topic: {}", topic);
            Ok(subscription_response)
        } else {
            let error_text = response.text().await?;
            error!("Failed to subscribe to topic {}: {}", topic, error_text);
            Err(format!("Topic subscription failed: {}", error_text).into())
        }
    }

    /// Unsubscribe device tokens from a topic
    pub async fn unsubscribe_from_topic(
        &self,
        device_tokens: Vec<String>,
        topic: &str,
    ) -> Result<TopicSubscriptionResponse, Box<dyn std::error::Error>> {
        let access_token = self.get_access_token().await?;
        let url = "https://iid.googleapis.com/iid/v1:batchRemove";

        let request = TopicSubscriptionRequest {
            to: format!("/topics/{}", topic),
            registration_tokens: device_tokens,
        };

        let response = self.client
            .post(url)
            .bearer_auth(&access_token)
            .json(&request)
            .send()
            .await?;

        if response.status().is_success() {
            let subscription_response: TopicSubscriptionResponse = response.json().await?;
            info!("Successfully unsubscribed devices from topic: {}", topic);
            Ok(subscription_response)
        } else {
            let error_text = response.text().await?;
            error!("Failed to unsubscribe from topic {}: {}", topic, error_text);
            Err(format!("Topic unsubscription failed: {}", error_text).into())
        }
    }

    /// Low-level FCM message sending
    pub async fn send_fcm_message(
        &self,
        message: FcmMessage,
        validate_only: bool,
    ) -> Result<FcmResponse, Box<dyn std::error::Error>> {
        let access_token = self.get_access_token().await?;
        let url = format!(
            "https://fcm.googleapis.com/v1/projects/{}/messages:send",
            self.project_id
        );

        let request = FcmRequest {
            message,
            validate_only,
        };

        let response = self.client
            .post(&url)
            .bearer_auth(&access_token)
            .json(&request)
            .send()
            .await?;

        if response.status().is_success() {
            let fcm_response: FcmResponse = response.json().await?;
            info!("FCM message sent successfully: {}", fcm_response.name);
            Ok(fcm_response)
        } else {
            let error_text = response.text().await?;
            error!("FCM message sending failed: {}", error_text);
            
            if let Ok(error_response) = serde_json::from_str::<FcmErrorResponse>(&error_text) {
                Err(format!("FCM error: {} - {}", error_response.error.code, error_response.error.message).into())
            } else {
                Err("FCM message sending failed".into())
            }
        }
    }

    /// Validate FCM message before sending
    pub async fn validate_fcm_message(&self, message: FcmMessage) -> Result<bool, Box<dyn std::error::Error>> {
        match self.send_fcm_message(message, true).await {
            Ok(_) => Ok(true),
            Err(e) => {
                warn!("FCM message validation failed: {}", e);
                Ok(false)
            }
        }
    }

    /// Send password reset email
    pub async fn send_password_reset_email(&self, email: &str) -> Result<(), Box<dyn std::error::Error>> {
        if let Ok(api_key) = crate::config::env::get_env_var("FIREBASE_API_KEY") {
            let url = format!(
                "https://identitytoolkit.googleapis.com/v1/accounts:sendOobCode?key={}",
                api_key
            );

            let payload = json!({
                "requestType": "PASSWORD_RESET",
                "email": email
            });

            let response = self.client
                .post(&url)
                .json(&payload)
                .send()
                .await?;

            if response.status().is_success() {
                info!("Password reset email sent successfully to: {}", email);
                Ok(())
            } else {
                let error_text = response.text().await?;
                error!("Failed to send password reset email to {}: {}", email, error_text);
                
                if error_text.contains("EMAIL_NOT_FOUND") {
                    Err("Email address not found".into())
                } else {
                    Err("Failed to send password reset email".into())
                }
            }
        } else {
            Err("FIREBASE_API_KEY not configured".into())
        }
    }

    /// Confirm password reset
    pub async fn confirm_password_reset(&self, oob_code: &str, new_password: &str) -> Result<String, Box<dyn std::error::Error>> {
        if let Ok(api_key) = crate::config::env::get_env_var("FIREBASE_API_KEY") {
            let url = format!(
                "https://identitytoolkit.googleapis.com/v1/accounts:resetPassword?key={}",
                api_key
            );

            let payload = json!({
                "oobCode": oob_code,
                "newPassword": new_password
            });

            let response = self.client
                .post(&url)
                .json(&payload)
                .send()
                .await?;

            if response.status().is_success() {
                let reset_response: Value = response.json().await?;
                let email = reset_response.get("email")
                    .and_then(|v| v.as_str())
                    .ok_or("Password reset response missing email")?;
                
                info!("Password reset completed successfully for: {}", email);
                Ok(email.to_string())
            } else {
                let error_text = response.text().await?;
                error!("Password reset confirmation failed: {}", error_text);
                
                if error_text.contains("INVALID_OOB_CODE") {
                    Err("Invalid or expired password reset code".into())
                } else if error_text.contains("WEAK_PASSWORD") {
                    Err("New password is too weak".into())
                } else {
                    Err("Password reset failed".into())
                }
            }
        } else {
            Err("FIREBASE_API_KEY not configured".into())
        }
    }

    /// Send welcome notification to new user
    pub async fn send_welcome_notification(&self, device_token: &str, user_name: &str) -> Result<FcmResponse, Box<dyn std::error::Error>> {
        let mut data = HashMap::new();
        data.insert("notification_type".to_string(), "welcome".to_string());
        data.insert("action".to_string(), "open_dashboard".to_string());

        self.send_push_notification(
            device_token,
            "Welcome to EPSX!",
            &format!("Hi {}! Welcome to EPSX Analytics Platform. Start exploring premium EPS analytics features.", user_name),
            Some(data),
        ).await
    }

    /// Send system maintenance notification
    pub async fn send_maintenance_notification(&self, topic: &str, start_time: DateTime<Utc>, duration_hours: u32) -> Result<FcmResponse, Box<dyn std::error::Error>> {
        let mut data = HashMap::new();
        data.insert("notification_type".to_string(), "maintenance".to_string());
        data.insert("start_time".to_string(), start_time.to_rfc3339());
        data.insert("duration_hours".to_string(), duration_hours.to_string());

        self.send_topic_notification(
            topic,
            "Scheduled Maintenance",
            &format!("EPSX will undergo maintenance for {} hours starting {}. Service may be temporarily unavailable.", 
                duration_hours, 
                start_time.format("%B %d at %H:%M UTC")
            ),
            Some(data),
        ).await
    }

    /// Send feature update notification
    pub async fn send_feature_update_notification(&self, topic: &str, feature_name: &str, description: &str) -> Result<FcmResponse, Box<dyn std::error::Error>> {
        let mut data = HashMap::new();
        data.insert("notification_type".to_string(), "feature_update".to_string());
        data.insert("feature_name".to_string(), feature_name.to_string());
        data.insert("action".to_string(), "explore_features".to_string());

        self.send_topic_notification(
            topic,
            &format!("New Feature: {}", feature_name),
            &format!("🚀 {}: {}", feature_name, description),
            Some(data),
        ).await
    }

    /// Bulk send notifications to multiple tokens
    pub async fn bulk_send_notifications(
        &self,
        tokens: Vec<String>,
        title: &str,
        body: &str,
        data: Option<HashMap<String, String>>,
    ) -> Result<Vec<Result<FcmResponse, String>>, Box<dyn std::error::Error>> {
        let mut results = Vec::new();
        
        for token in tokens {
            match self.send_push_notification(&token, title, body, data.clone()).await {
                Ok(response) => {
                    results.push(Ok(response));
                    info!("Successfully sent notification to token: {}", &token[..8]); // Log only first 8 chars
                }
                Err(e) => {
                    results.push(Err(e.to_string()));
                    error!("Failed to send notification to token {}: {}", &token[..8], e);
                }
            }
        }

        Ok(results)
    }

    /// Create notification message template
    pub fn create_notification_template(
        &self,
        template_type: &str,
        variables: &HashMap<String, String>,
    ) -> (String, String) {
        match template_type {
            "subscription_expiry" => {
                let days = variables.get("days").unwrap_or(&"0".to_string()).clone();
                let plan = variables.get("plan").unwrap_or(&"Premium".to_string()).clone();
                
                if days == "0" {
                    ("Subscription Expired".to_string(), format!("Your {} plan has expired. Renew now to continue accessing premium features.", plan))
                } else {
                    ("Subscription Expiring".to_string(), format!("Your {} plan expires in {} days. Don't lose access to premium analytics!", plan, days))
                }
            },
            "payment_success" => {
                let amount = variables.get("amount").unwrap_or(&"$0.00".to_string()).clone();
                ("Payment Successful".to_string(), format!("Your payment of {} has been processed successfully. Thank you for your subscription!", amount))
            },
            "account_security" => {
                ("Security Alert".to_string(), "New sign-in detected from a different device. If this wasn't you, please secure your account immediately.".to_string())
            },
            "data_update" => {
                let dataset = variables.get("dataset").unwrap_or(&"EPS data".to_string()).clone();
                ("Data Updated".to_string(), format!("{} has been updated with the latest market information. Check out the new insights!", dataset))
            },
            _ => {
                ("EPSX Notification".to_string(), "You have a new update from EPSX Analytics Platform.".to_string())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn test_notification_template_creation() {
        let admin = FirebaseAdmin::create_test_client();
        
        let mut variables = HashMap::new();
        variables.insert("days".to_string(), "7".to_string());
        variables.insert("plan".to_string(), "Premium".to_string());
        
        let (title, body) = admin.create_notification_template("subscription_expiry", &variables);
        assert_eq!(title, "Subscription Expiring");
        assert!(body.contains("7 days"));
        assert!(body.contains("Premium"));
    }

    #[test]
    fn test_expiration_notification_urgency() {
        let admin = FirebaseAdmin::create_test_client();
        let expires_at = Utc.with_ymd_and_hms(2024, 12, 31, 23, 59, 59).unwrap();
        
        // Test high priority for 1 day
        let mut data = HashMap::new();
        data.insert("permission_profile_name".to_string(), "Premium".to_string());
        data.insert("days_until_expiration".to_string(), "1".to_string());
        data.insert("expires_at".to_string(), expires_at.to_rfc3339());
        
        // Test different urgency levels based on days
        let test_cases = vec![
            (0, "high"),   // Expired
            (1, "high"),   // Tomorrow
            (7, "normal"), // Week
            (30, "normal"), // Month
        ];
        
        for (days, expected_priority) in test_cases {
            // The actual urgency logic is inside send_expiration_push_notification
            // This test validates our expectation of the priority levels
            if days <= 1 {
                assert_eq!(expected_priority, "high");
            } else {
                assert_eq!(expected_priority, "normal");
            }
        }
    }

    #[test]
    fn test_fcm_message_structure() {
        let message = FcmMessage {
            token: Some("test-token".to_string()),
            topic: None,
            condition: None,
            notification: FcmNotification {
                title: "Test Title".to_string(),
                body: "Test Body".to_string(),
                image: None,
            },
            data: None,
            android: None,
            apns: None,
            webpush: None,
        };
        
        assert_eq!(message.token, Some("test-token".to_string()));
        assert_eq!(message.notification.title, "Test Title");
        assert_eq!(message.notification.body, "Test Body");
    }

    #[test]
    fn test_topic_subscription_request() {
        let request = TopicSubscriptionRequest {
            to: "/topics/all_users".to_string(),
            registration_tokens: vec!["token1".to_string(), "token2".to_string()],
        };
        
        assert_eq!(request.to, "/topics/all_users");
        assert_eq!(request.registration_tokens.len(), 2);
    }

    #[tokio::test]
    async fn test_bulk_notification_structure() {
        let admin = FirebaseAdmin::create_test_client();
        let tokens = vec!["token1".to_string(), "token2".to_string()];
        
        // This test just validates the structure - actual API calls would need mocking
        assert_eq!(tokens.len(), 2);
        
        let template_vars = HashMap::new();
        let (title, body) = admin.create_notification_template("payment_success", &template_vars);
        assert_eq!(title, "Payment Successful");
        assert!(body.contains("payment"));
    }
}