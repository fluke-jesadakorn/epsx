use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::collections::HashMap;

/// OIDC Authorization Request
#[derive(Debug, Deserialize)]
pub struct AuthorizationRequest {
    pub client_id: String,
    pub response_type: String,
    pub redirect_uri: String,
    pub scope: String,
    pub state: Option<String>,
    pub nonce: Option<String>,
    pub code_challenge: Option<String>,
    pub code_challenge_method: Option<String>,
}

/// OIDC Authorization Response (redirect parameters)
#[derive(Debug, Serialize)]
pub struct AuthorizationResponse {
    pub code: String,
    pub state: Option<String>,
}

/// OIDC Token Request
#[derive(Debug, Deserialize)]
pub struct TokenRequest {
    pub grant_type: String,
    pub code: Option<String>,
    pub redirect_uri: Option<String>,
    pub client_id: String,
    pub client_secret: Option<String>,
    pub refresh_token: Option<String>,
    pub code_verifier: Option<String>,
}

/// OIDC Token Response
#[derive(Debug, Serialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: u64,
    pub refresh_token: Option<String>,
    pub id_token: String,
    pub scope: String,
}

/// OIDC UserInfo Response
#[derive(Debug, Serialize)]
pub struct UserInfoResponse {
    pub sub: String, // Firebase UID
    pub email: Option<String>,
    pub email_verified: Option<bool>,
    pub name: Option<String>,
    pub picture: Option<String>,
    pub given_name: Option<String>,
    pub family_name: Option<String>,
    pub phone_number: Option<String>,
    pub phone_number_verified: Option<bool>,
    
    // Custom claims from Firebase
    #[serde(flatten)]
    pub custom_claims: HashMap<String, serde_json::Value>,
}

/// OIDC Discovery Document
#[derive(Debug, Serialize)]
pub struct OidcDiscoveryDocument {
    pub issuer: String,
    pub authorization_endpoint: String,
    pub token_endpoint: String,
    pub userinfo_endpoint: String,
    pub jwks_uri: String,
    
    // Standard OpenID Connect endpoints (RFC compliance)
    pub revocation_endpoint: String,        // RFC 7009
    pub introspection_endpoint: String,     // RFC 7662  
    pub end_session_endpoint: String,       // OpenID Connect Session Management
    
    pub scopes_supported: Vec<String>,
    pub response_types_supported: Vec<String>,
    pub grant_types_supported: Vec<String>,
    pub subject_types_supported: Vec<String>,
    pub id_token_signing_alg_values_supported: Vec<String>,
    pub claims_supported: Vec<String>,
    pub code_challenge_methods_supported: Vec<String>,
    
    // Additional RFC 7009/7662 support indicators
    pub revocation_endpoint_auth_methods_supported: Vec<String>,
    pub introspection_endpoint_auth_methods_supported: Vec<String>,
}

/// Authorization Code Storage
#[derive(Debug, Clone)]
pub struct AuthorizationCode {
    pub code: String,
    pub client_id: String,
    pub firebase_uid: String,
    pub redirect_uri: String,
    pub scopes: Vec<String>,
    pub state: Option<String>,
    pub nonce: Option<String>,
    pub code_challenge: Option<String>,
    pub code_challenge_method: Option<String>,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

/// OIDC Error Response
#[derive(Debug, Serialize)]
pub struct OidcErrorResponse {
    pub error: String,
    pub error_description: Option<String>,
    pub error_uri: Option<String>,
}

/// JWT Header for ID tokens
#[derive(Debug, Serialize)]
pub struct JwtHeader {
    pub alg: String,
    pub typ: String,
    pub kid: String,
}

/// Modern ID Token Claims - Clean Admin Module System
#[derive(Debug, Serialize)]
pub struct IdTokenClaims {
    // Standard OIDC claims
    pub iss: String, // Issuer
    pub sub: String, // Subject (Firebase UID)
    pub aud: String, // Audience (client_id)
    pub exp: u64,    // Expiration time
    pub iat: u64,    // Issued at time
    pub auth_time: Option<u64>, // Authentication time
    pub nonce: Option<String>,
    
    // Standard profile claims
    pub email: Option<String>,
    pub email_verified: Option<bool>,
    pub name: Option<String>,
    pub picture: Option<String>,
    pub given_name: Option<String>,
    pub family_name: Option<String>,
    pub phone_number: Option<String>,
    pub phone_number_verified: Option<bool>,
    
    // Modern IAM Claims - Admin Module System Only
    pub admin: bool,                        // Is user an admin (has any admin modules)
    pub access_level: String,               // Effective access level: admin/write/read/none
    pub admin_modules: Vec<String>,         // Assigned admin modules
    pub permissions: Vec<String>,           // Computed permissions from modules
    
    // Subscription data
    pub subscription_tier: Option<String>,   // User's subscription level
    pub subscription_status: Option<String>, // active/inactive/trial
    
    // Firebase custom claims (for any additional data)
    #[serde(flatten)]
    pub custom_claims: HashMap<String, serde_json::Value>,
}

/// OAuth Client Configuration
#[derive(Debug, Clone)]
pub struct OAuthClient {
    pub client_id: String,
    pub client_secret_hash: String,
    pub client_name: String,
    pub redirect_uris: Vec<String>,
    pub grant_types: Vec<String>,
    pub scopes: Vec<String>,
    pub is_active: bool,
}

/// PKCE Code Challenge Methods
#[derive(Debug, Clone, PartialEq)]
pub enum CodeChallengeMethod {
    Plain,
    S256,
}

impl std::str::FromStr for CodeChallengeMethod {
    type Err = String;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "plain" => Ok(CodeChallengeMethod::Plain),
            "S256" => Ok(CodeChallengeMethod::S256),
            _ => Err(format!("Unsupported code challenge method: {}", s)),
        }
    }
}

impl std::fmt::Display for CodeChallengeMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CodeChallengeMethod::Plain => write!(f, "plain"),
            CodeChallengeMethod::S256 => write!(f, "S256"),
        }
    }
}