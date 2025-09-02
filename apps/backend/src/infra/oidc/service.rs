// OIDC Service - Production OpenID Connect Implementation
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};
use jsonwebtoken::{encode, decode, EncodingKey, DecodingKey, Header, Algorithm, Validation};
use rsa::{RsaPrivateKey, RsaPublicKey};
use rsa::pkcs8::{EncodePrivateKey, EncodePublicKey, DecodePrivateKey};
use rsa::traits::PublicKeyParts;
use uuid::Uuid;
use tracing::{info, warn};

use crate::infra::firebase::FirebaseUser;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OIDCClaims {
    // Standard OIDC claims
    pub iss: String,        // Issuer
    pub sub: String,        // Subject (user ID)
    pub aud: String,        // Audience
    pub exp: u64,          // Expiration time
    pub iat: u64,          // Issued at
    pub auth_time: u64,    // Authentication time
    pub nonce: Option<String>, // Nonce for replay protection
    
    // Custom claims
    pub email: Option<String>,
    pub email_verified: Option<bool>,
    pub name: Option<String>,
    pub permissions: Option<Vec<String>>,
    pub role: Option<String>,
    pub token_use: String, // "access" or "id" or "refresh"
    
    // JTI for token revocation
    pub jti: String,       // JWT ID for token tracking
}

#[derive(Debug, Clone)]
pub struct OIDCTokens {
    pub access_token: String,
    pub id_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_in: u64,
    pub scope: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefreshTokenClaims {
    pub sub: String,       // Subject (user ID)
    pub iss: String,       // Issuer  
    pub aud: String,       // Audience
    pub exp: u64,          // Expiration time
    pub iat: u64,          // Issued at
    pub jti: String,       // JWT ID
    pub token_use: String, // "refresh"
}

pub struct OIDCService {
    private_key: RsaPrivateKey,
    public_key: RsaPublicKey,
    issuer: String,
    audience: String,
}

impl OIDCService {
    /// Create new OIDC service with RSA key generation
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let issuer = crate::config::env::get_env_var("OIDC_ISSUER")
            .unwrap_or_else(|_| "https://api.epsx.io".to_string());
        let audience = crate::config::env::get_env_var("OIDC_AUDIENCE") 
            .unwrap_or_else(|_| "epsx-platform".to_string());

        // Try to load existing RSA keys, or generate new ones
        let (private_key, public_key) = Self::load_or_generate_keys().await?;

        info!("OIDC Service initialized with issuer: {} and audience: {}", issuer, audience);

        Ok(Self {
            private_key,
            public_key,
            issuer,
            audience,
        })
    }

    /// Load existing RSA keys from environment or generate new ones
    async fn load_or_generate_keys() -> Result<(RsaPrivateKey, RsaPublicKey), Box<dyn std::error::Error>> {
        // Try to load from environment variables first
        if let Ok(private_key_pem) = crate::config::env::get_env_var("OIDC_PRIVATE_KEY") {
            info!("Loading OIDC RSA keys from environment");
            let private_key = RsaPrivateKey::from_pkcs8_pem(&private_key_pem)?;
            let public_key = RsaPublicKey::from(&private_key);
            return Ok((private_key, public_key));
        }

        // Generate new RSA keys for development/testing
        warn!("Generating new RSA keys for OIDC - use OIDC_PRIVATE_KEY env var in production");
        let mut rng = rand::thread_rng();
        let private_key = RsaPrivateKey::new(&mut rng, 2048)?;
        let public_key = RsaPublicKey::from(&private_key);

        // Log the generated public key for debugging (never log private key)
        let public_key_pem = public_key.to_public_key_pem(rsa::pkcs8::LineEnding::LF)?;
        info!("Generated RSA public key (first 100 chars): {}", &public_key_pem[..100.min(public_key_pem.len())]);

        Ok((private_key, public_key))
    }

    /// Generate complete OIDC token set for authenticated Firebase user
    pub async fn generate_tokens(&self, firebase_user: &FirebaseUser, nonce: Option<String>) -> Result<OIDCTokens, Box<dyn std::error::Error>> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let user_permissions = self.extract_user_permissions(firebase_user);
        let user_role = self.extract_user_role(firebase_user);

        // Generate access token (1 hour expiration)
        let access_claims = OIDCClaims {
            iss: self.issuer.clone(),
            sub: firebase_user.uid.clone(),
            aud: self.audience.clone(),
            exp: now + 3600, // 1 hour
            iat: now,
            auth_time: now,
            nonce: nonce.clone(),
            email: firebase_user.email.clone(),
            email_verified: Some(firebase_user.email_verified),
            name: firebase_user.display_name.clone(),
            permissions: Some(user_permissions.clone()),
            role: Some(user_role.clone()),
            token_use: "access".to_string(),
            jti: Uuid::new_v4().to_string(),
        };

        // Generate ID token (same structure but for identity)
        let id_claims = OIDCClaims {
            token_use: "id".to_string(),
            jti: Uuid::new_v4().to_string(),
            ..access_claims.clone()
        };

        // Generate refresh token (7 days expiration)
        let refresh_claims = RefreshTokenClaims {
            sub: firebase_user.uid.clone(),
            iss: self.issuer.clone(),
            aud: self.audience.clone(),
            exp: now + (7 * 24 * 3600), // 7 days
            iat: now,
            jti: Uuid::new_v4().to_string(),
            token_use: "refresh".to_string(),
        };

        // Create signing key
        let private_key_pem = self.private_key.to_pkcs8_pem(rsa::pkcs8::LineEnding::LF)?;
        let encoding_key = EncodingKey::from_rsa_pem(private_key_pem.as_bytes())?;
        
        // Create RS256 header
        let header = Header::new(Algorithm::RS256);

        // Encode tokens
        let access_token = encode(&header, &access_claims, &encoding_key)?;
        let id_token = encode(&header, &id_claims, &encoding_key)?;
        let refresh_token = encode(&header, &refresh_claims, &encoding_key)?;

        info!("Generated OIDC tokens for user: {} with {} permissions", 
              firebase_user.uid, user_permissions.len());

        Ok(OIDCTokens {
            access_token,
            id_token,
            refresh_token,
            token_type: "Bearer".to_string(),
            expires_in: 3600,
            scope: "openid profile email".to_string(),
        })
    }

    /// Validate Bearer token and extract claims
    pub async fn validate_bearer_token(&self, token: &str) -> Result<OIDCClaims, Box<dyn std::error::Error>> {
        let public_key_pem = self.public_key.to_public_key_pem(rsa::pkcs8::LineEnding::LF)?;
        let decoding_key = DecodingKey::from_rsa_pem(public_key_pem.as_bytes())?;

        let mut validation = Validation::new(Algorithm::RS256);
        validation.set_issuer(&[&self.issuer]);
        validation.set_audience(&[&self.audience]);
        validation.validate_exp = true;

        let token_data = decode::<OIDCClaims>(token, &decoding_key, &validation)?;
        
        // Additional validation
        if token_data.claims.token_use != "access" {
            return Err("Invalid token use - expected access token".into());
        }

        Ok(token_data.claims)
    }

    /// Refresh tokens using refresh token
    pub async fn refresh_tokens(&self, refresh_token: &str) -> Result<OIDCTokens, Box<dyn std::error::Error>> {
        let public_key_pem = self.public_key.to_public_key_pem(rsa::pkcs8::LineEnding::LF)?;
        let decoding_key = DecodingKey::from_rsa_pem(public_key_pem.as_bytes())?;

        let mut validation = Validation::new(Algorithm::RS256);
        validation.set_issuer(&[&self.issuer]);
        validation.set_audience(&[&self.audience]);

        let token_data = decode::<RefreshTokenClaims>(refresh_token, &decoding_key, &validation)?;
        
        if token_data.claims.token_use != "refresh" {
            return Err("Invalid token use - expected refresh token".into());
        }

        // Get user from Firebase to regenerate tokens with fresh data
        // For now, create a minimal user structure - in production, fetch from Firebase
        let firebase_user = FirebaseUser {
            uid: token_data.claims.sub.clone(),
            email: None, // Would be fetched from Firebase
            email_verified: false,
            display_name: None,
            photo_url: None,
            phone_number: None,
            disabled: false,
            custom_claims: HashMap::new(),
            provider_data: vec![],
            created_at: chrono::Utc::now(),
            last_login_at: Some(chrono::Utc::now()),
        };

        self.generate_tokens(&firebase_user, None).await
    }

    /// Get OIDC discovery document
    pub fn get_discovery_document(&self) -> serde_json::Value {
        serde_json::json!({
            "issuer": self.issuer,
            "authorization_endpoint": format!("{}/auth", self.issuer),
            "token_endpoint": format!("{}/token", self.issuer),
            "userinfo_endpoint": format!("{}/userinfo", self.issuer),
            "jwks_uri": format!("{}/.well-known/jwks.json", self.issuer),
            "response_types_supported": ["code", "token", "id_token"],
            "subject_types_supported": ["public"],
            "id_token_signing_alg_values_supported": ["RS256"],
            "token_endpoint_auth_methods_supported": ["client_secret_basic", "client_secret_post"],
            "claims_supported": [
                "sub", "iss", "aud", "exp", "iat", "auth_time", 
                "email", "email_verified", "name", "permissions", "role"
            ],
            "scopes_supported": ["openid", "profile", "email"]
        })
    }

    /// Get JSON Web Key Set (JWKS)
    pub fn get_jwks(&self) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
        use base64::engine::general_purpose::URL_SAFE_NO_PAD;
        use base64::Engine;

        let modulus = self.public_key.n().to_bytes_be();
        let exponent = self.public_key.e().to_bytes_be();

        let n_b64 = URL_SAFE_NO_PAD.encode(&modulus);
        let e_b64 = URL_SAFE_NO_PAD.encode(&exponent);

        Ok(serde_json::json!({
            "keys": [{
                "kty": "RSA",
                "alg": "RS256",
                "use": "sig",
                "kid": "epsx-signing-key-001",
                "n": n_b64,
                "e": e_b64
            }]
        }))
    }

    /// Extract user permissions from Firebase custom claims
    fn extract_user_permissions(&self, firebase_user: &FirebaseUser) -> Vec<String> {
        let mut permissions = Vec::new();

        // Extract permissions from custom claims
        if let Some(perms) = firebase_user.custom_claims.get("permissions") {
            if let Some(perm_array) = perms.as_array() {
                for perm in perm_array {
                    if let Some(perm_str) = perm.as_str() {
                        permissions.push(perm_str.to_string());
                    }
                }
            }
        }

        // Add role-based permissions
        if let Some(admin) = firebase_user.custom_claims.get("admin") {
            if admin.as_bool().unwrap_or(false) {
                permissions.extend([
                    "admin:*:*".to_string(),
                    "epsx:*:*".to_string()
                ]);
            }
        }

        // Ensure at least basic permissions
        if permissions.is_empty() {
            permissions.push("epsx:analytics:view".to_string());
        }

        permissions
    }

    /// Extract user role from Firebase custom claims
    fn extract_user_role(&self, firebase_user: &FirebaseUser) -> String {
        firebase_user.custom_claims
            .get("role")
            .and_then(|v| v.as_str())
            .unwrap_or("user")
            .to_string()
    }

    /// Get issuer URL for OIDC tokens
    pub fn get_issuer(&self) -> &str {
        &self.issuer
    }

    /// Get audience for OIDC tokens  
    pub fn get_audience(&self) -> &str {
        &self.audience
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_oidc_service_creation() {
        let service = OIDCService::new().await;
        assert!(service.is_ok());
    }

    #[tokio::test]
    async fn test_token_generation() {
        let service = OIDCService::new().await.unwrap();
        
        let mut custom_claims = HashMap::new();
        custom_claims.insert("role".to_string(), serde_json::Value::String("user".to_string()));
        
        let firebase_user = FirebaseUser {
            uid: "test-user-001".to_string(),
            email: Some("test@example.com".to_string()),
            email_verified: true,
            display_name: Some("Test User".to_string()),
            photo_url: None,
            phone_number: None,
            disabled: false,
            custom_claims,
            provider_data: vec![],
            created_at: chrono::Utc::now(),
            last_login_at: Some(chrono::Utc::now()),
        };

        let tokens = service.generate_tokens(&firebase_user, None).await;
        assert!(tokens.is_ok());
        
        let tokens = tokens.unwrap();
        assert_eq!(tokens.token_type, "Bearer");
        assert_eq!(tokens.expires_in, 3600);
        assert!(!tokens.access_token.is_empty());
        assert!(!tokens.id_token.is_empty());
        assert!(!tokens.refresh_token.is_empty());
    }

    #[tokio::test]
    async fn test_token_validation() {
        let service = OIDCService::new().await.unwrap();
        
        let mut custom_claims = HashMap::new();
        custom_claims.insert("role".to_string(), serde_json::Value::String("admin".to_string()));
        custom_claims.insert("admin".to_string(), serde_json::Value::Bool(true));
        
        let firebase_user = FirebaseUser {
            uid: "admin-user-001".to_string(),
            email: Some("admin@example.com".to_string()),
            email_verified: true,
            display_name: Some("Admin User".to_string()),
            photo_url: None,
            phone_number: None,
            disabled: false,
            custom_claims,
            provider_data: vec![],
            created_at: chrono::Utc::now(),
            last_login_at: Some(chrono::Utc::now()),
        };

        let tokens = service.generate_tokens(&firebase_user, None).await.unwrap();
        let claims = service.validate_bearer_token(&tokens.access_token).await;
        
        assert!(claims.is_ok());
        let claims = claims.unwrap();
        assert_eq!(claims.sub, "admin-user-001");
        assert_eq!(claims.token_use, "access");
        assert!(claims.permissions.unwrap().contains(&"admin:*:*".to_string()));
    }

    #[test]
    fn test_discovery_document() {
        let service_result = tokio::runtime::Runtime::new().unwrap().block_on(OIDCService::new());
        let service = service_result.unwrap();
        
        let discovery = service.get_discovery_document();
        assert_eq!(discovery["issuer"], "https://api.epsx.io");
        assert!(discovery["response_types_supported"].as_array().unwrap().contains(&serde_json::Value::String("code".to_string())));
    }

    #[test]
    fn test_jwks_generation() {
        let service_result = tokio::runtime::Runtime::new().unwrap().block_on(OIDCService::new());
        let service = service_result.unwrap();
        
        let jwks = service.get_jwks();
        assert!(jwks.is_ok());
        
        let jwks = jwks.unwrap();
        let keys = jwks["keys"].as_array().unwrap();
        assert_eq!(keys.len(), 1);
        assert_eq!(keys[0]["kty"], "RSA");
        assert_eq!(keys[0]["alg"], "RS256");
    }
}