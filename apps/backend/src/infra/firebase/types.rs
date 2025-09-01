// Firebase Shared Types and Data Structures
use chrono::{DateTime, Utc};
// Focused module containing all Firebase-related DTOs and shared types

use serde_json::Value;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};

// Core Firebase Admin Structure
#[derive(Debug, Clone)]
pub struct FirebaseAdmin {
    pub client: reqwest::Client,
    pub project_id: String,
    pub service_account_key: Option<String>,
    pub jwks_cache: HashMap<String, FirebasePublicKey>,
    pub jwks_cache_expiry: DateTime<Utc>,
}

// Firebase User Data Structures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FirebaseUser {
    pub uid: String,
    pub email: Option<String>,
    pub email_verified: bool,
    pub display_name: Option<String>,
    pub photo_url: Option<String>,
    pub phone_number: Option<String>,
    pub disabled: bool,
    pub custom_claims: HashMap<String, Value>,
    pub provider_data: Vec<UserProvider>,
    pub created_at: DateTime<Utc>,
    pub last_login_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProvider {
    pub uid: String,
    pub email: Option<String>,
    pub display_name: Option<String>,
    pub photo_url: Option<String>,
    pub provider_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FirebasePublicKey {
    pub kty: String,
    pub alg: String,
    pub r#use: String,
    pub kid: String,
    pub n: String,
    pub e: String,
}

// Firebase Admin API Response Structures
#[derive(Deserialize)]
pub struct GetUserResponse {
    pub users: Option<Vec<FirebaseUserRecord>>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FirebaseUserRecord {
    pub local_id: String,
    pub email: Option<String>,
    pub email_verified: Option<bool>,
    pub display_name: Option<String>,
    pub photo_url: Option<String>,
    pub phone_number: Option<String>,
    pub disabled: Option<bool>,
    pub custom_claims: Option<String>, // JSON string
    pub provider_user_info: Option<Vec<ProviderUserInfo>>,
    pub created_at: Option<String>,
    pub last_login_at: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderUserInfo {
    pub provider_id: String,
    pub federated_id: String,
    pub email: Option<String>,
    pub display_name: Option<String>,
    pub photo_url: Option<String>,
}

#[derive(Serialize)]
pub struct CreateUserRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    #[serde(rename = "emailVerified", skip_serializing_if = "Option::is_none")]
    pub email_verified: Option<bool>,
    #[serde(rename = "displayName", skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(rename = "photoUrl", skip_serializing_if = "Option::is_none")]
    pub photo_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disabled: Option<bool>,
}

#[derive(Serialize)]
pub struct UpdateUserRequest {
    #[serde(rename = "localId")]
    pub local_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    #[serde(rename = "emailVerified", skip_serializing_if = "Option::is_none")]
    pub email_verified: Option<bool>,
    #[serde(rename = "displayName", skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(rename = "photoUrl", skip_serializing_if = "Option::is_none")]
    pub photo_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disabled: Option<bool>,
    #[serde(rename = "customClaims", skip_serializing_if = "Option::is_none")]
    pub custom_claims: Option<String>, // JSON string
}

#[derive(Deserialize)]
pub struct FirebaseErrorResponse {
    pub error: FirebaseError,
}

#[derive(Deserialize)]
pub struct FirebaseError {
    pub message: String,
}

#[derive(Serialize, Deserialize)]
pub struct JWTClaims {
    pub sub: String, // Firebase UID
    pub email: String,
    pub iat: usize,
    pub exp: usize,
}

#[derive(Serialize)]
pub struct AuthRequest {
    pub email: String,
    pub password: String,
    #[serde(rename = "returnSecureToken")]
    pub return_secure_token: bool,
}

// Firebase Cloud Messaging (FCM) Data Structures

#[derive(Debug, Serialize)]
pub struct FcmMessage {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub topic: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub condition: Option<String>,
    pub notification: FcmNotification,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub android: Option<FcmAndroidConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub apns: Option<FcmApnsConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub webpush: Option<FcmWebpushConfig>,
}

#[derive(Debug, Serialize)]
pub struct FcmNotification {
    pub title: String,
    pub body: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct FcmAndroidConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ttl: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notification: Option<FcmAndroidNotification>,
}

#[derive(Debug, Serialize)]
pub struct FcmAndroidNotification {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sound: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tag: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub click_action: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body_loc_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body_loc_args: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title_loc_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title_loc_args: Option<Vec<String>>,
}

#[derive(Debug, Serialize)]
pub struct FcmApnsConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers: Option<HashMap<String, String>>,
    pub payload: FcmApnsPayload,
}

#[derive(Debug, Serialize)]
pub struct FcmApnsPayload {
    pub aps: FcmAps,
}

#[derive(Debug, Serialize)]
pub struct FcmAps {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alert: Option<FcmApnsAlert>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub badge: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sound: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct FcmApnsAlert {
    pub title: String,
    pub body: String,
}

#[derive(Debug, Serialize)]
pub struct FcmWebpushConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers: Option<HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notification: Option<FcmWebpushNotification>,
}

#[derive(Debug, Serialize)]
pub struct FcmWebpushNotification {
    pub title: String,
    pub body: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub badge: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<HashMap<String, Value>>,
}

#[derive(Debug, Serialize)]
pub struct FcmRequest {
    pub message: FcmMessage,
    #[serde(rename = "validate_only")]
    pub validate_only: bool,
}

#[derive(Debug, Deserialize)]
pub struct FcmResponse {
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct FcmErrorResponse {
    pub error: FcmError,
}

#[derive(Debug, Deserialize)]
pub struct FcmError {
    pub code: u32,
    pub message: String,
    pub status: String,
}

#[derive(Debug, Clone)]
pub struct DeviceToken {
    pub token: String,
    pub user_id: String,
    pub platform: DevicePlatform,
    pub app_version: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(non_camel_case_types)]
pub enum DevicePlatform {
    Android,
    iOS,
    Web,
}

#[derive(Debug, Serialize)]
pub struct TopicSubscriptionRequest {
    pub to: String, // Topic name with /topics/ prefix
    pub registration_tokens: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct TopicSubscriptionResponse {
    pub results: Vec<TopicSubscriptionResult>,
}

#[derive(Debug, Deserialize)]
pub struct TopicSubscriptionResult {
    pub error: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_firebase_user_serialization() {
        let user = FirebaseUser {
            uid: "test-uid".to_string(),
            email: Some("test@example.com".to_string()),
            email_verified: true,
            display_name: Some("Test User".to_string()),
            photo_url: None,
            phone_number: None,
            disabled: false,
            custom_claims: HashMap::new(),
            provider_data: Vec::new(),
            created_at: Utc::now(),
            last_login_at: None,
        };

        let json = serde_json::to_string(&user).unwrap();
        assert!(json.contains("test-uid"));
        assert!(json.contains("test@example.com"));
    }

    #[test]
    fn test_fcm_message_creation() {
        let message = FcmMessage {
            token: Some("device-token".to_string()),
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

        let json = serde_json::to_string(&message).unwrap();
        assert!(json.contains("Test Title"));
        assert!(json.contains("device-token"));
    }

    #[test]
    fn test_jwt_claims() {
        let claims = JWTClaims {
            sub: "firebase-uid".to_string(),
            email: "user@example.com".to_string(),
            iat: 1234567890,
            exp: 1234567890 + 3600,
        };

        let json = serde_json::to_string(&claims).unwrap();
        assert!(json.contains("firebase-uid"));
        assert!(json.contains("user@example.com"));
    }
}