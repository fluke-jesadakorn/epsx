use serde_json::Value;
use std::collections::HashMap;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use jsonwebtoken::{encode, decode, Header, Algorithm, Validation, EncodingKey, DecodingKey};

#[derive(Debug, Clone)]
pub struct FirebaseAdmin {
    client: Client,
    api_key: String,
}

#[derive(Deserialize)]
struct FirebaseAuthResponse {
    #[serde(rename = "localId")]
    local_id: String,
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

impl FirebaseAdmin {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let api_key = std::env::var("FIREBASE_API_KEY")
            .map_err(|_| "FIREBASE_API_KEY environment variable not set")?;

        Ok(Self {
            client: Client::new(),
            api_key,
        })
    }

    /// Authenticate user with email and password using Firebase Auth REST API
    pub async fn authenticate_user(&self, email: &str, password: &str) -> Result<String, Box<dyn std::error::Error>> {
        let url = format!(
            "https://identitytoolkit.googleapis.com/v1/accounts:signInWithPassword?key={}",
            self.api_key
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
            let auth_response: FirebaseAuthResponse = response.json().await?;
            tracing::info!("Firebase authentication successful for user: {}", email);
            Ok(auth_response.local_id)
        } else {
            let error_text = response.text().await?;
            tracing::error!("Firebase authentication failed: {}", error_text);
            
            // Try to parse error response
            if let Ok(error_response) = serde_json::from_str::<FirebaseErrorResponse>(&error_text) {
                match error_response.error.message.as_str() {
                    "EMAIL_NOT_FOUND" => Err("Email not found".into()),
                    "INVALID_PASSWORD" => Err("Invalid password".into()),
                    "USER_DISABLED" => Err("User account has been disabled".into()),
                    _ => Err(format!("Authentication failed: {}", error_response.error.message).into()),
                }
            } else {
                Err("Authentication failed".into())
            }
        }
    }

    /// Verify Firebase ID token (for token-based authentication)
    pub async fn verify_token(&self, id_token: &str) -> Result<HashMap<String, Value>, Box<dyn std::error::Error>> {
        let url = format!(
            "https://identitytoolkit.googleapis.com/v1/accounts:lookup?key={}",
            self.api_key
        );

        let mut payload = HashMap::new();
        payload.insert("idToken", id_token);

        let response = self.client
            .post(&url)
            .json(&payload)
            .send()
            .await?;

        if response.status().is_success() {
            let json_response: Value = response.json().await?;
            if let Some(users) = json_response["users"].as_array() {
                if let Some(user) = users.first() {
                    let mut user_data = HashMap::new();
                    user_data.insert("uid".to_string(), user["localId"].clone());
                    user_data.insert("email".to_string(), user["email"].clone());
                    user_data.insert("email_verified".to_string(), user["emailVerified"].clone());
                    return Ok(user_data);
                }
            }
            Err("Invalid token or user not found".into())
        } else {
            let error_text = response.text().await?;
            tracing::error!("Token verification failed: {}", error_text);
            Err("Token verification failed".into())
        }
    }

    /// Create a new Firebase user (for registration)
    pub async fn create_user(&self, email: &str, password: &str) -> Result<String, Box<dyn std::error::Error>> {
        let url = format!(
            "https://identitytoolkit.googleapis.com/v1/accounts:signUp?key={}",
            self.api_key
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
            let auth_response: FirebaseAuthResponse = response.json().await?;
            tracing::info!("Firebase user created successfully: {}", email);
            Ok(auth_response.local_id)
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

    /// Generate a JWT token for a user
    pub fn generate_jwt_token(&self, firebase_uid: &str, email: &str) -> Result<String, Box<dyn std::error::Error>> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs() as usize;

        let claims = JWTClaims {
            sub: firebase_uid.to_string(),
            email: email.to_string(),
            iat: now,
            exp: now + 86400, // 24 hours
        };

        let jwt_secret = std::env::var("JWT_SECRET")
            .unwrap_or_else(|_| "default_secret_change_in_production".to_string());
        
        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(jwt_secret.as_ref())
        )?;

        Ok(token)
    }

    /// Verify a JWT token
    pub fn verify_jwt_token(&self, token: &str) -> Result<JWTClaims, Box<dyn std::error::Error>> {
        let jwt_secret = std::env::var("JWT_SECRET")
            .unwrap_or_else(|_| "default_secret_change_in_production".to_string());

        let validation = Validation::new(Algorithm::HS256);
        let token_data = decode::<JWTClaims>(
            token,
            &DecodingKey::from_secret(jwt_secret.as_ref()),
            &validation
        )?;

        Ok(token_data.claims)
    }
}