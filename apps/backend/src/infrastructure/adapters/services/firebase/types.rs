// Firebase type definitions

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Firebase user information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FirebaseUser {
    pub uid: String,
    pub email: Option<String>,
    pub display_name: Option<String>,
    pub photo_url: Option<String>,
    pub email_verified: bool,
    pub provider_id: String,
    pub custom_claims: std::collections::HashMap<String, serde_json::Value>,
}

impl FirebaseUser {
    pub fn new(uid: String, email: String) -> Self {
        Self {
            uid,
            email: Some(email),
            display_name: None,
            photo_url: None,
            email_verified: false,
            provider_id: "firebase".to_string(),
            custom_claims: std::collections::HashMap::new(),
        }
    }

    pub fn is_verified(&self) -> bool {
        self.email_verified
    }
}

/// Firebase ID token claims
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FirebaseTokenClaims {
    pub aud: String,
    pub auth_time: u64,
    pub email: Option<String>,
    pub email_verified: Option<bool>,
    pub exp: u64,
    pub firebase: FirebaseClaimsData,
    pub iat: u64,
    pub iss: String,
    pub sub: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FirebaseClaimsData {
    pub identities: serde_json::Value,
    pub sign_in_provider: String,
}

/// Firebase custom token request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomTokenRequest {
    pub uid: String,
    pub claims: Option<serde_json::Value>,
}

/// Firebase authentication result  
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FirebaseAuthResult {
    pub user: FirebaseUser,
    pub token: String,
    pub expires_at: DateTime<Utc>,
}