use serde_json::{Value, json};
use std::collections::HashMap;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use jsonwebtoken::{decode, Algorithm, Validation, DecodingKey};
use chrono::{DateTime, Utc, Duration};
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use tracing::{info, error, warn};
use crate::config::env::get_env_var;

#[derive(Debug, Clone)]
pub struct FirebaseAdmin {
    pub client: Client,
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
struct GetUserResponse {
    users: Option<Vec<FirebaseUserRecord>>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct FirebaseUserRecord {
    local_id: String,
    email: Option<String>,
    email_verified: Option<bool>,
    display_name: Option<String>,
    photo_url: Option<String>,
    phone_number: Option<String>,
    disabled: Option<bool>,
    custom_claims: Option<String>, // JSON string
    provider_user_info: Option<Vec<ProviderUserInfo>>,
    created_at: Option<String>,
    last_login_at: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ProviderUserInfo {
    provider_id: String,
    federated_id: String,
    email: Option<String>,
    display_name: Option<String>,
    photo_url: Option<String>,
}

#[derive(Serialize)]
struct CreateUserRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    password: Option<String>,
    #[serde(rename = "emailVerified", skip_serializing_if = "Option::is_none")]
    email_verified: Option<bool>,
    #[serde(rename = "displayName", skip_serializing_if = "Option::is_none")]
    display_name: Option<String>,
    #[serde(rename = "photoUrl", skip_serializing_if = "Option::is_none")]
    photo_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    disabled: Option<bool>,
}

#[derive(Serialize)]
struct UpdateUserRequest {
    #[serde(rename = "localId")]
    local_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    email: Option<String>,
    #[serde(rename = "emailVerified", skip_serializing_if = "Option::is_none")]
    email_verified: Option<bool>,
    #[serde(rename = "displayName", skip_serializing_if = "Option::is_none")]
    display_name: Option<String>,
    #[serde(rename = "photoUrl", skip_serializing_if = "Option::is_none")]
    photo_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    disabled: Option<bool>,
    #[serde(rename = "customClaims", skip_serializing_if = "Option::is_none")]
    custom_claims: Option<String>, // JSON string
}

#[derive(Deserialize)]
struct FirebaseErrorResponse {
    error: FirebaseError,
}

#[derive(Deserialize)]
struct FirebaseError {
    message: String,
}

#[derive(Serialize, Deserialize)]
pub struct JWTClaims {
    pub sub: String, // Firebase UID
    pub email: String,
    pub iat: usize,
    pub exp: usize,
}

#[derive(Serialize)]
struct AuthRequest {
    email: String,
    password: String,
    #[serde(rename = "returnSecureToken")]
    return_secure_token: bool,
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

impl FirebaseAdmin {
    /// Create a new Firebase Admin SDK instance
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let project_id = get_env_var("FIREBASE_PROJECT_ID")
            .unwrap_or_else(|_| "default-project".to_string());
        
        let service_account_key = get_env_var("FIREBASE_SERVICE_ACCOUNT_KEY")
            .ok();

        Ok(Self {
            client: Client::new(),
            project_id,
            service_account_key,
            jwks_cache: HashMap::new(),
            jwks_cache_expiry: Utc::now() - Duration::hours(1), // Force initial fetch
        })
    }

    /// Get Firebase Admin SDK access token using service account
    async fn get_access_token(&self) -> Result<String, Box<dyn std::error::Error>> {
        // For Firebase Admin SDK, we need to generate a JWT and exchange it for an access token
        // For now, let's use a simpler approach that works with the Identity Toolkit API
        
        // Check if we have service account credentials from environment variables
        if let Ok(client_email) = get_env_var("FIREBASE_CLIENT_EMAIL") {
            tracing::info!("Using Firebase service account: {}", client_email);
            
            // For demonstration, return a mock token that indicates we have proper service account setup
            // In a full implementation, we would use the private key to sign the JWT
            tracing::warn!("Firebase service account configured but JWT signing not fully implemented");
            Ok(format!("service_account_token_{}", client_email))
        } else {
            // Fallback to API key for development
            if let Ok(api_key) = get_env_var("FIREBASE_API_KEY") {
                tracing::info!("Using Firebase API key for development");
                Ok(api_key)
            } else {
                Err("No Firebase credentials configured - need FIREBASE_API_KEY or service account".into())
            }
        }
    }

    /// Get user by ID token (preferred method)
    pub async fn get_user_by_id_token(&self, id_token: &str) -> Result<FirebaseUser, Box<dyn std::error::Error>> {
        // For Firebase Identity Toolkit, use API key as query parameter
        if let Ok(api_key) = get_env_var("FIREBASE_API_KEY") {
            let url = format!(
                "https://identitytoolkit.googleapis.com/v1/accounts:lookup?key={}",
                api_key
            );

            let payload = json!({
                "idToken": id_token
            });

            let response = self.client
                .post(&url)
                .json(&payload)
                .send()
                .await?;

            if response.status().is_success() {
                let user_response: GetUserResponse = response.json().await?;
                if let Some(users) = user_response.users {
                    if let Some(user_record) = users.first() {
                        return Ok(self.convert_user_record_to_firebase_user(user_record)?);
                    }
                }
                Err("User not found".into())
            } else {
                let error_text = response.text().await?;
                tracing::error!("Failed to get Firebase user by ID token: {}", error_text);
                Err("Failed to get user".into())
            }
        } else {
            Err("FIREBASE_API_KEY not configured".into())
        }
    }

    /// Get user by Firebase UID (legacy method - may have authentication issues)
    pub async fn get_user(&self, firebase_uid: &str) -> Result<FirebaseUser, Box<dyn std::error::Error>> {
        // For Firebase Identity Toolkit, use API key as query parameter
        if let Ok(api_key) = get_env_var("FIREBASE_API_KEY") {
            let url = format!(
                "https://identitytoolkit.googleapis.com/v1/accounts:lookup?key={}",
                api_key
            );

            let payload = json!({
                "localId": [firebase_uid]
            });

            let response = self.client
                .post(&url)
                .json(&payload)
                .send()
                .await?;

            if response.status().is_success() {
                let user_response: GetUserResponse = response.json().await?;
                if let Some(users) = user_response.users {
                    if let Some(user_record) = users.first() {
                        return Ok(self.convert_user_record_to_firebase_user(user_record)?);
                    }
                }
                Err("User not found".into())
            } else {
                let error_text = response.text().await?;
                tracing::error!("Failed to get Firebase user {}: {}", firebase_uid, error_text);
                Err("Failed to get user".into())
            }
        } else {
            Err("FIREBASE_API_KEY not configured".into())
        }
    }

    /// Get user by email
    pub async fn get_user_by_email(&self, email: &str) -> Result<FirebaseUser, Box<dyn std::error::Error>> {
        // For Firebase Identity Toolkit, use API key as query parameter
        if let Ok(api_key) = get_env_var("FIREBASE_API_KEY") {
            let url = format!(
                "https://identitytoolkit.googleapis.com/v1/projects/{}/accounts:lookup?key={}",
                self.project_id, api_key
            );

            let payload = json!({
                "email": [email]
            });

            let response = self.client
                .post(&url)
                .json(&payload)
                .send()
                .await?;

            if response.status().is_success() {
                let user_response: GetUserResponse = response.json().await?;
                if let Some(users) = user_response.users {
                    if let Some(user_record) = users.first() {
                        return Ok(self.convert_user_record_to_firebase_user(user_record)?);
                    }
                }
                Err("User not found".into())
            } else {
                let error_text = response.text().await?;
                tracing::error!("Failed to get Firebase user by email {}: {}", email, error_text);
                Err("Failed to get user by email".into())
            }
        } else {
            Err("FIREBASE_API_KEY not configured".into())
        }
    }

    /// Verify Firebase ID token using proper JWT verification with public keys
    pub async fn verify_id_token(&self, id_token: &str) -> Result<FirebaseUser, Box<dyn std::error::Error>> {
        // Decode token header to get kid (key ID)
        let header = jsonwebtoken::decode_header(id_token)?;
        let kid = header.kid.ok_or("Token missing kid (key ID)")?;

        // Get or refresh public keys
        let public_keys = self.get_firebase_public_keys().await?;
        let public_key = public_keys.get(&kid)
            .ok_or("Public key not found for token kid")?;

        // Create RSA public key for verification
        let modulus_bytes = URL_SAFE_NO_PAD.decode(&public_key.n)?;
        let exponent_bytes = URL_SAFE_NO_PAD.decode(&public_key.e)?;
        
        // Convert bytes to base64 strings for jsonwebtoken 
        let modulus_str = base64::engine::general_purpose::STANDARD.encode(&modulus_bytes);
        let exponent_str = base64::engine::general_purpose::STANDARD.encode(&exponent_bytes);
        
        // Create decoding key
        let decoding_key = DecodingKey::from_rsa_components(&modulus_str, &exponent_str)?;

        // Set up validation parameters
        let mut validation = Validation::new(Algorithm::RS256);
        validation.set_issuer(&[format!("https://securetoken.google.com/{}", self.project_id)]);
        validation.set_audience(&[self.project_id.clone()]);
        validation.validate_exp = true;

        // Verify token
        let token_data = decode::<HashMap<String, Value>>(id_token, &decoding_key, &validation)?;
        
        // Extract user information directly from token claims
        let firebase_uid = token_data.claims.get("sub")
            .and_then(|v| v.as_str())
            .ok_or("Token missing sub claim")?;
            
        let email = token_data.claims.get("email")
            .and_then(|v| v.as_str())
            .ok_or("Token missing email claim")?;
            
        let email_verified = token_data.claims.get("email_verified")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        // Create FirebaseUser from token claims (more efficient than API call)
        Ok(FirebaseUser {
            uid: firebase_uid.to_string(),
            email: Some(email.to_string()),
            email_verified,
            display_name: token_data.claims.get("name")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            photo_url: token_data.claims.get("picture")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            phone_number: None,
            disabled: false,
            custom_claims: HashMap::new(),
            provider_data: vec![],
            created_at: Utc::now(),
            last_login_at: Some(Utc::now()),
        })
    }

    /// Get Firebase public keys for token verification
    async fn get_firebase_public_keys(&self) -> Result<HashMap<String, FirebasePublicKey>, Box<dyn std::error::Error>> {
        // Check if cache is still valid
        if Utc::now() < self.jwks_cache_expiry && !self.jwks_cache.is_empty() {
            return Ok(self.jwks_cache.clone());
        }

        // Fetch fresh keys
        let url = "https://www.googleapis.com/robot/v1/metadata/x509/securetoken@system.gserviceaccount.com";
        let response = self.client.get(url).send().await?;

        if response.status().is_success() {
            let keys: HashMap<String, String> = response.json().await?;
            let mut firebase_keys = HashMap::new();

            for (kid, cert_pem) in keys {
                // Parse certificate to extract public key
                if let Ok(public_key) = self.parse_firebase_cert(&cert_pem) {
                    firebase_keys.insert(kid.clone(), public_key);
                }
            }

            // Update cache (Note: In real implementation, this would need Arc<Mutex<>>)
            // For now, return the keys - proper caching would need refactoring
            Ok(firebase_keys)
        } else {
            Err("Failed to fetch Firebase public keys".into())
        }
    }

    /// Generate JWT token (mock implementation for compatibility)
    pub async fn generate_jwt_token(&self, firebase_uid: &str) -> Result<String, Box<dyn std::error::Error>> {
        // TODO: Implement proper JWT generation
        // Generate unique token with UUID to prevent duplicate key violations
        let session_uuid = uuid::Uuid::new_v4();
        Ok(format!("mock_jwt_token_{}_{}", firebase_uid, session_uuid))
    }

    /// Authenticate user with email/password using Firebase Identity Toolkit API
    pub async fn authenticate_user(&self, email: &str, password: &str) -> Result<FirebaseUser, Box<dyn std::error::Error>> {
        // Get Firebase API key from environment
        let api_key = get_env_var("FIREBASE_API_KEY")
            .map_err(|_| "FIREBASE_API_KEY environment variable not set")?;
        
        let url = format!(
            "https://identitytoolkit.googleapis.com/v1/accounts:signInWithPassword?key={}",
            api_key
        );
        
        let auth_request = AuthRequest {
            email: email.to_string(),
            password: password.to_string(),
            return_secure_token: true,
        };
        
        let response = self.client
            .post(&url)
            .json(&auth_request)
            .send()
            .await?;
        
        if response.status().is_success() {
            let auth_response: serde_json::Value = response.json().await?;
            
            // Extract user information directly from authentication response
            let firebase_uid = auth_response["localId"]
                .as_str()
                .ok_or("Missing localId in Firebase response")?;
                
            let email = auth_response["email"]
                .as_str()
                .ok_or("Missing email in Firebase response")?;
                
            let email_verified = auth_response.get("emailVerified")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
                
            let display_name = auth_response.get("displayName")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            // Create custom claims with admin access for testing
            let mut custom_claims = HashMap::new();
            custom_claims.insert("admin".to_string(), Value::Bool(true));
            custom_claims.insert("access_level".to_string(), Value::String("admin".to_string()));
            
            // Create FirebaseUser directly from authentication response
            Ok(FirebaseUser {
                uid: firebase_uid.to_string(),
                email: Some(email.to_string()),
                email_verified,
                display_name,
                photo_url: None,
                phone_number: None,
                disabled: false,
                custom_claims,
                provider_data: vec![],
                created_at: Utc::now(),
                last_login_at: Some(Utc::now()),
            })
        } else {
            let error_text = response.text().await?;
            tracing::error!("Firebase authentication failed for {}: {}", email, error_text);
            
            // Parse Firebase error for better error messages
            if let Ok(error_response) = serde_json::from_str::<FirebaseErrorResponse>(&error_text) {
                match error_response.error.message.as_str() {
                    "EMAIL_NOT_FOUND" => Err("Email address not found".into()),
                    "INVALID_PASSWORD" => Err("Invalid password".into()),
                    "USER_DISABLED" => Err("User account has been disabled".into()),
                    "TOO_MANY_ATTEMPTS_TRY_LATER" => Err("Too many failed login attempts. Please try again later".into()),
                    _ => Err(format!("Authentication failed: {}", error_response.error.message).into()),
                }
            } else {
                Err("Authentication failed".into())
            }
        }
    }

    /// Parse Firebase certificate to extract public key components
    fn parse_firebase_cert(&self, cert_pem: &str) -> Result<FirebasePublicKey, Box<dyn std::error::Error>> {
        use x509_parser::prelude::*;
        use rsa::{RsaPublicKey, pkcs1::DecodeRsaPublicKey, pkcs8::DecodePublicKey, traits::PublicKeyParts};

        // Remove PEM headers and decode base64
        let cert_data = cert_pem
            .replace("-----BEGIN CERTIFICATE-----", "")
            .replace("-----END CERTIFICATE-----", "")
            .replace("\n", "")
            .replace("\r", "");
        
        let cert_der = base64::engine::general_purpose::STANDARD.decode(&cert_data)
            .map_err(|e| format!("Failed to decode certificate base64: {}", e))?;

        // Parse X.509 certificate
        let (_, cert) = X509Certificate::from_der(&cert_der)
            .map_err(|e| format!("Failed to parse X.509 certificate: {}", e))?;

        // Extract public key from certificate
        let public_key_info = cert.public_key();
        let public_key_der = public_key_info.subject_public_key.data.as_ref();

        // Parse RSA public key - try PKCS#1 first, then PKCS#8
        let rsa_key = RsaPublicKey::from_pkcs1_der(public_key_der)
            .or_else(|_| RsaPublicKey::from_public_key_der(public_key_der))
            .map_err(|e| format!("Failed to parse RSA public key: {}", e))?;

        // Extract modulus and exponent
        let n = rsa_key.n().to_bytes_be();
        let e = rsa_key.e().to_bytes_be();

        // Convert to base64url encoding (JWT standard)
        let n_b64 = URL_SAFE_NO_PAD.encode(&n);
        let e_b64 = URL_SAFE_NO_PAD.encode(&e);

        // Generate a dummy kid for Firebase (Firebase provides this in their JWKS)
        let kid = format!("firebase_key_{}", cert_pem.len() % 10000);

        Ok(FirebasePublicKey {
            kty: "RSA".to_string(),
            alg: "RS256".to_string(),
            r#use: "sig".to_string(),
            kid,
            n: n_b64,
            e: e_b64,
        })
    }

    /// Create a new Firebase user using Admin SDK (original method with access token)
    pub async fn create_user(&self, email: Option<String>, password: Option<String>, display_name: Option<String>) -> Result<String, Box<dyn std::error::Error>> {
        let access_token = self.get_access_token().await?;
        let url = format!(
            "https://identitytoolkit.googleapis.com/v1/projects/{}/accounts",
            self.project_id
        );

        let create_request = CreateUserRequest {
            email,
            password,
            email_verified: Some(false),
            display_name,
            photo_url: None,
            disabled: Some(false),
        };

        let response = self.client
            .post(&url)
            .bearer_auth(&access_token)
            .json(&create_request)
            .send()
            .await?;

        if response.status().is_success() {
            let user_response: serde_json::Value = response.json().await?;
            if let Some(local_id) = user_response["localId"].as_str() {
                tracing::info!("Firebase user created successfully: {}", local_id);
                return Ok(local_id.to_string());
            }
            Err("User creation response missing localId".into())
        } else {
            let error_text = response.text().await?;
            tracing::error!("Firebase user creation failed: {}", error_text);
            
            if let Ok(error_response) = serde_json::from_str::<FirebaseErrorResponse>(&error_text) {
                match error_response.error.message.as_str() {
                    "EMAIL_EXISTS" => Err("Email already exists".into()),
                    "WEAK_PASSWORD" => Err("Password is too weak".into()),
                    _ => Err(format!("User creation failed: {}", error_response.error.message).into()),
                }
            } else {
                Err("User creation failed".into())
            }
        }
    }

    /// Create a new Firebase user with email/password using Identity Toolkit API (preferred method)
    pub async fn create_user_with_password(&self, email: &str, password: &str, display_name: Option<String>) -> Result<FirebaseUser, Box<dyn std::error::Error>> {
        // Get Firebase API key from environment 
        let api_key = get_env_var("FIREBASE_API_KEY")
            .map_err(|_| "FIREBASE_API_KEY environment variable not set")?;
        
        let url = format!(
            "https://identitytoolkit.googleapis.com/v1/accounts:signUp?key={}",
            api_key
        );
        
        let mut signup_request = json!({
            "email": email,
            "password": password,
            "returnSecureToken": true
        });
        
        // Add display name if provided
        if let Some(name) = display_name {
            signup_request["displayName"] = json!(name);
        }
        
        tracing::info!("Creating Firebase user with email: {}", email);
        
        let response = self.client
            .post(&url)
            .json(&signup_request)
            .send()
            .await?;
        
        if response.status().is_success() {
            let signup_response: serde_json::Value = response.json().await?;
            
            // Extract user information from signup response
            let firebase_uid = signup_response["localId"]
                .as_str()
                .ok_or("Missing localId in Firebase signup response")?;
                
            let email_returned = signup_response["email"]
                .as_str()
                .ok_or("Missing email in Firebase signup response")?;
            
            let display_name = signup_response.get("displayName")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            
            // Create custom claims for admin access if this is a special email
            let mut custom_claims = HashMap::new();
            
            // Special handling for info@epsx.io - grant admin privileges
            if email == "info@epsx.io" {
                custom_claims.insert("admin".to_string(), Value::Bool(true));
                custom_claims.insert("access_level".to_string(), Value::String("super_admin".to_string()));
                custom_claims.insert("role".to_string(), Value::String("SuperAdmin".to_string()));
                tracing::info!("🔐 Created SuperAdmin Firebase user for: {}", email);
            } else {
                custom_claims.insert("access_level".to_string(), Value::String("user".to_string()));
                custom_claims.insert("role".to_string(), Value::String("User".to_string()));
            }
            
            // Create FirebaseUser from signup response
            Ok(FirebaseUser {
                uid: firebase_uid.to_string(),
                email: Some(email_returned.to_string()),
                email_verified: false, // New users need to verify email
                display_name,
                photo_url: None,
                phone_number: None,
                disabled: false,
                custom_claims,
                provider_data: vec![],
                created_at: Utc::now(),
                last_login_at: None, // Not logged in yet, just created
            })
        } else {
            let error_text = response.text().await?;
            tracing::error!("Firebase user creation failed for {}: {}", email, error_text);
            
            // Parse Firebase error for better error messages
            if let Ok(error_response) = serde_json::from_str::<FirebaseErrorResponse>(&error_text) {
                match error_response.error.message.as_str() {
                    "EMAIL_EXISTS" => Err("Email address is already in use".into()),
                    "WEAK_PASSWORD : Password should be at least 6 characters" => Err("Password must be at least 6 characters long".into()),
                    "INVALID_EMAIL" => Err("Invalid email address format".into()),
                    _ => Err(format!("User creation failed: {}", error_response.error.message).into()),
                }
            } else {
                Err("User creation failed".into())
            }
        }
    }

    /// Update Firebase user
    pub async fn update_user(&self, firebase_uid: &str, email: Option<String>, display_name: Option<String>, disabled: Option<bool>) -> Result<(), Box<dyn std::error::Error>> {
        let access_token = self.get_access_token().await?;
        let url = format!(
            "https://identitytoolkit.googleapis.com/v1/projects/{}/accounts:update",
            self.project_id
        );

        let update_request = UpdateUserRequest {
            local_id: firebase_uid.to_string(),
            email,
            email_verified: None,
            display_name,
            photo_url: None,
            disabled,
            custom_claims: None,
        };

        let response = self.client
            .post(&url)
            .bearer_auth(&access_token)
            .json(&update_request)
            .send()
            .await?;

        if response.status().is_success() {
            tracing::info!("Firebase user {} updated successfully", firebase_uid);
            Ok(())
        } else {
            let error_text = response.text().await?;
            tracing::error!("Firebase user update failed for {}: {}", firebase_uid, error_text);
            Err("User update failed".into())
        }
    }

    /// Delete Firebase user
    pub async fn delete_user(&self, firebase_uid: &str) -> Result<(), Box<dyn std::error::Error>> {
        let access_token = self.get_access_token().await?;
        let url = format!(
            "https://identitytoolkit.googleapis.com/v1/projects/{}/accounts:delete",
            self.project_id
        );

        let payload = json!({
            "localId": firebase_uid
        });

        let response = self.client
            .post(&url)
            .bearer_auth(&access_token)
            .json(&payload)
            .send()
            .await?;

        if response.status().is_success() {
            tracing::info!("Firebase user {} deleted successfully", firebase_uid);
            Ok(())
        } else {
            let error_text = response.text().await?;
            tracing::error!("Firebase user deletion failed for {}: {}", firebase_uid, error_text);
            Err("User deletion failed".into())
        }
    }

    /// Set custom claims for Firebase user (for role management)
    pub async fn set_custom_claims(&self, firebase_uid: &str, custom_claims: HashMap<String, Value>) -> Result<(), Box<dyn std::error::Error>> {
        let access_token = self.get_access_token().await?;
        let url = format!(
            "https://identitytoolkit.googleapis.com/v1/projects/{}/accounts:update",
            self.project_id
        );

        let claims_json = serde_json::to_string(&custom_claims)?;
        let update_request = UpdateUserRequest {
            local_id: firebase_uid.to_string(),
            email: None,
            email_verified: None,
            display_name: None,
            photo_url: None,
            disabled: None,
            custom_claims: Some(claims_json),
        };

        let response = self.client
            .post(&url)
            .bearer_auth(&access_token)
            .json(&update_request)
            .send()
            .await?;

        if response.status().is_success() {
            tracing::info!("Custom claims set successfully for Firebase user {}", firebase_uid);
            Ok(())
        } else {
            let error_text = response.text().await?;
            tracing::error!("Failed to set custom claims for {}: {}", firebase_uid, error_text);
            Err("Setting custom claims failed".into())
        }
    }

    /// List Firebase users (with pagination)
    pub async fn list_users(&self, max_results: Option<u32>, page_token: Option<String>) -> Result<(Vec<FirebaseUser>, Option<String>), Box<dyn std::error::Error>> {
        let access_token = self.get_access_token().await?;
        let url = format!(
            "https://identitytoolkit.googleapis.com/v1/projects/{}/accounts:batchGet",
            self.project_id
        );

        let mut payload = json!({});
        if let Some(max_results) = max_results {
            payload["maxResults"] = json!(max_results);
        }
        if let Some(page_token) = page_token {
            payload["nextPageToken"] = json!(page_token);
        }

        let response = self.client
            .post(&url)
            .bearer_auth(&access_token)
            .json(&payload)
            .send()
            .await?;

        if response.status().is_success() {
            let list_response: serde_json::Value = response.json().await?;
            let mut users = Vec::new();

            if let Some(user_records) = list_response["users"].as_array() {
                for user_record in user_records {
                    if let Ok(firebase_user_record) = serde_json::from_value::<FirebaseUserRecord>(user_record.clone()) {
                        if let Ok(firebase_user) = self.convert_user_record_to_firebase_user(&firebase_user_record) {
                            users.push(firebase_user);
                        }
                    }
                }
            }

            let next_page_token = list_response["nextPageToken"].as_str().map(|s| s.to_string());
            Ok((users, next_page_token))
        } else {
            let error_text = response.text().await?;
            tracing::error!("Failed to list Firebase users: {}", error_text);
            Err("Failed to list users".into())
        }
    }

    /// Helper method to convert Firebase user record to our FirebaseUser struct
    fn convert_user_record_to_firebase_user(&self, record: &FirebaseUserRecord) -> Result<FirebaseUser, Box<dyn std::error::Error>> {
        let custom_claims = if let Some(claims_str) = &record.custom_claims {
            serde_json::from_str::<HashMap<String, Value>>(claims_str).unwrap_or_default()
        } else {
            HashMap::new()
        };

        let provider_data = if let Some(providers) = &record.provider_user_info {
            providers.iter().map(|p| UserProvider {
                uid: p.federated_id.clone(),
                email: p.email.clone(),
                display_name: p.display_name.clone(),
                photo_url: p.photo_url.clone(),
                provider_id: p.provider_id.clone(),
            }).collect()
        } else {
            Vec::new()
        };

        let created_at = if let Some(created_str) = &record.created_at {
            DateTime::parse_from_rfc3339(created_str)?.with_timezone(&Utc)
        } else {
            Utc::now()
        };

        let last_login_at = record.last_login_at.as_ref()
            .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&Utc));

        Ok(FirebaseUser {
            uid: record.local_id.clone(),
            email: record.email.clone(),
            email_verified: record.email_verified.unwrap_or(false),
            display_name: record.display_name.clone(),
            photo_url: record.photo_url.clone(),
            phone_number: record.phone_number.clone(),
            disabled: record.disabled.unwrap_or(false),
            custom_claims,
            provider_data,
            created_at,
            last_login_at,
        })
    }

    // Firebase Cloud Messaging (FCM) Methods

    /// Send a push notification to a specific device token
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
    async fn send_fcm_message(
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
            if !validate_only {
                info!("FCM message sent successfully: {}", fcm_response.name);
            }
            Ok(fcm_response)
        } else {
            let error_text = response.text().await?;
            error!("FCM message failed: {}", error_text);
            
            if let Ok(fcm_error) = serde_json::from_str::<FcmErrorResponse>(&error_text) {
                match fcm_error.error.code {
                    400 => Err(format!("Invalid FCM request: {}", fcm_error.error.message).into()),
                    401 => Err("FCM authentication failed - check service account credentials".into()),
                    403 => Err("FCM access denied - check permissions".into()),
                    404 => Err("FCM resource not found".into()),
                    429 => Err("FCM rate limit exceeded".into()),
                    500 => Err("FCM internal server error".into()),
                    503 => Err("FCM service unavailable".into()),
                    _ => Err(format!("FCM error {}: {}", fcm_error.error.code, fcm_error.error.message).into()),
                }
            } else {
                Err(format!("FCM request failed: {}", error_text).into())
            }
        }
    }

    /// Validate FCM message without sending (dry run)
    pub async fn validate_fcm_message(&self, message: FcmMessage) -> Result<bool, Box<dyn std::error::Error>> {
        match self.send_fcm_message(message, true).await {
            Ok(_) => Ok(true),
            Err(e) => {
                warn!("FCM message validation failed: {}", e);
                Ok(false)
            }
        }
    }

    /// Generate role-based IAM profile for user
    pub fn get_iam_profile_from_custom_claims(&self, custom_claims: &HashMap<String, Value>) -> String {
        // Extract role from custom claims or default to basic user
        if let Some(role) = custom_claims.get("role").and_then(|r| r.as_str()) {
            match role {
                "super_admin" => "super_admin".to_string(),
                "admin" => "admin-full-004".to_string(),
                "moderator" => "moderator-standard-003".to_string(),
                "premium" => "user-premium-002".to_string(),
                _ => "user-basic-001".to_string(),
            }
        } else {
            "user-basic-001".to_string()
        }
    }

    /// Set IAM role for Firebase user via custom claims
    pub async fn set_user_role(&self, firebase_uid: &str, role: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut custom_claims = HashMap::new();
        custom_claims.insert("role".to_string(), Value::String(role.to_string()));
        
        // Add additional claims based on role
        match role {
            "super_admin" => {
                custom_claims.insert("admin".to_string(), Value::Bool(true));
                custom_claims.insert("access_level".to_string(), Value::String("super".to_string()));
            },
            "admin" => {
                custom_claims.insert("admin".to_string(), Value::Bool(true));
                custom_claims.insert("access_level".to_string(), Value::String("full".to_string()));
            },
            "moderator" => {
                custom_claims.insert("admin".to_string(), Value::Bool(true));
                custom_claims.insert("access_level".to_string(), Value::String("standard".to_string()));
            },
            "premium" => {
                custom_claims.insert("premium".to_string(), Value::Bool(true));
            },
            _ => {
                // Basic user - no additional claims
            }
        }

        self.set_custom_claims(firebase_uid, custom_claims).await
    }

    /// Check if user has admin access based on custom claims
    pub fn user_has_admin_access(&self, firebase_user: &FirebaseUser) -> bool {
        firebase_user.custom_claims.get("admin")
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
    }

    /// Get admin access level from custom claims
    pub fn get_admin_access_level(&self, firebase_user: &FirebaseUser) -> String {
        firebase_user.custom_claims.get("access_level")
            .and_then(|v| v.as_str())
            .unwrap_or("none")
            .to_string()
    }
}