// Enhanced OIDC Service with Granular Permissions Support
// Clean implementation of OpenID Connect with granular permission claims

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
use crate::auth::granular_permissions::{GranularPermissionClaim, GranularPermissionSet};

/// Enhanced OIDC Claims with granular permissions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedOIDCClaims {
    // Standard OIDC claims
    pub iss: String,        // Issuer
    pub sub: String,        // Subject (user ID)
    pub aud: String,        // Audience
    pub exp: u64,          // Expiration time
    pub iat: u64,          // Issued at
    pub auth_time: u64,    // Authentication time
    pub nonce: Option<String>, // Nonce for replay protection
    
    // Basic user claims
    pub email: Option<String>,
    pub email_verified: Option<bool>,
    pub name: Option<String>,
    pub role: Option<String>,
    pub token_use: String, // "access" or "id" or "refresh"
    
    // Enhanced granular permissions
    pub granular_permissions: HashMap<String, GranularPermissionClaim>,
    pub permission_version: u32,
    pub permission_hash: String,
    pub next_validation: u64,
    
    // JTI for token revocation
    pub jti: String,       // JWT ID for token tracking
}

/// Enhanced OIDC Tokens with granular permission support
#[derive(Debug, Clone)]
pub struct EnhancedOIDCTokens {
    pub access_token: String,
    pub id_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_in: u64,
    pub scope: String,
    pub permission_version: u32,
}

/// Token validation result with granular permission info
#[derive(Debug, Clone)]
pub struct TokenValidationResult {
    pub claims: EnhancedOIDCClaims,
    pub valid_permissions: Vec<String>,
    pub expired_permissions: Vec<String>,
    pub needs_refresh: bool,
    pub updated_token: Option<String>,
}

/// Enhanced OIDC Service
pub struct EnhancedOIDCService {
    private_key: RsaPrivateKey,
    public_key: RsaPublicKey,
    issuer: String,
    audience: String,
}

impl EnhancedOIDCService {
    /// Create new enhanced OIDC service
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let issuer = crate::config::env::get_env_var("OIDC_ISSUER")
            .unwrap_or_else(|_| "https://api.epsx.io".to_string());
        let audience = crate::config::env::get_env_var("OIDC_AUDIENCE") 
            .unwrap_or_else(|_| "epsx-platform".to_string());

        let (private_key, public_key) = Self::load_or_generate_keys().await?;

        info!("Enhanced OIDC Service initialized with granular permissions support");

        Ok(Self {
            private_key,
            public_key,
            issuer,
            audience,
        })
    }

    /// Load or generate RSA keys
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

        Ok((private_key, public_key))
    }

    /// Generate enhanced OIDC token set with granular permissions
    pub async fn generate_tokens_with_permissions(
        &self,
        firebase_user: &FirebaseUser,
        permission_set: GranularPermissionSet,
        nonce: Option<String>,
    ) -> Result<EnhancedOIDCTokens, Box<dyn std::error::Error>> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let user_role = self.extract_user_role(firebase_user);

        // Generate access token with granular permissions (1 hour expiration)
        let access_claims = EnhancedOIDCClaims {
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
            role: Some(user_role.clone()),
            token_use: "access".to_string(),
            granular_permissions: permission_set.permissions.clone(),
            permission_version: permission_set.version,
            permission_hash: permission_set.hash.clone(),
            next_validation: now + 300, // 5 minutes
            jti: Uuid::new_v4().to_string(),
        };

        // Generate ID token (same structure but for identity)
        let id_claims = EnhancedOIDCClaims {
            token_use: "id".to_string(),
            jti: Uuid::new_v4().to_string(),
            ..access_claims.clone()
        };

        // Generate refresh token (7 days expiration, no granular permissions)
        let refresh_claims = crate::infra::oidc::service::RefreshTokenClaims {
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

        info!(
            "Generated enhanced OIDC tokens for user: {} with {} granular permissions", 
            firebase_user.uid, 
            permission_set.permissions.len()
        );

        Ok(EnhancedOIDCTokens {
            access_token,
            id_token,
            refresh_token,
            token_type: "Bearer".to_string(),
            expires_in: 3600,
            scope: "openid profile email".to_string(),
            permission_version: permission_set.version,
        })
    }

    /// Validate Bearer token with granular permission checking
    pub async fn validate_bearer_token_with_permissions(
        &self, 
        token: &str
    ) -> Result<TokenValidationResult, Box<dyn std::error::Error>> {
        let public_key_pem = self.public_key.to_public_key_pem(rsa::pkcs8::LineEnding::LF)?;
        let decoding_key = DecodingKey::from_rsa_pem(public_key_pem.as_bytes())?;

        let mut validation = Validation::new(Algorithm::RS256);
        validation.set_issuer(&[&self.issuer]);
        validation.set_audience(&[&self.audience]);
        validation.validate_exp = true;

        let token_data = decode::<EnhancedOIDCClaims>(token, &decoding_key, &validation)?;
        let claims = token_data.claims;
        
        // Additional validation
        if claims.token_use != "access" {
            return Err("Invalid token use - expected access token".into());
        }

        // Check for permission expiry and cleanup
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let mut valid_permissions = Vec::new();
        let mut expired_permissions = Vec::new();
        let mut needs_refresh = false;

        for (permission, claim) in &claims.granular_permissions {
            if claim.is_valid(now as i64) {
                valid_permissions.push(permission.clone());
            } else {
                expired_permissions.push(permission.clone());
                needs_refresh = true;
            }
        }

        // Check if validation interval has passed
        if now >= claims.next_validation {
            needs_refresh = true;
        }

        // Generate updated token if needed
        let updated_token = if needs_refresh {
            let mut updated_permission_set = GranularPermissionSet::from_permissions(claims.granular_permissions.clone());
            let removed = updated_permission_set.cleanup_expired(now as i64);
            
            if !removed.is_empty() {
                info!("Cleaned up {} expired permissions for user {}", removed.len(), claims.sub);
                
                // Generate new token with cleaned permissions
                let updated_claims = EnhancedOIDCClaims {
                    granular_permissions: updated_permission_set.permissions,
                    permission_version: updated_permission_set.version,
                    permission_hash: updated_permission_set.hash,
                    next_validation: now + 300, // 5 minutes
                    ..claims.clone()
                };

                let header = Header::new(Algorithm::RS256);
                let private_key_pem = self.private_key.to_pkcs8_pem(rsa::pkcs8::LineEnding::LF)?;
                let encoding_key = EncodingKey::from_rsa_pem(private_key_pem.as_bytes())?;
                
                Some(encode(&header, &updated_claims, &encoding_key)?)
            } else {
                None
            }
        } else {
            None
        };

        Ok(TokenValidationResult {
            claims,
            valid_permissions,
            expired_permissions,
            needs_refresh,
            updated_token,
        })
    }

    /// Refresh tokens using refresh token
    pub async fn refresh_tokens_with_permissions(
        &self,
        refresh_token: &str,
        permission_set: GranularPermissionSet,
    ) -> Result<EnhancedOIDCTokens, Box<dyn std::error::Error>> {
        let public_key_pem = self.public_key.to_public_key_pem(rsa::pkcs8::LineEnding::LF)?;
        let decoding_key = DecodingKey::from_rsa_pem(public_key_pem.as_bytes())?;

        let mut validation = Validation::new(Algorithm::RS256);
        validation.set_issuer(&[&self.issuer]);
        validation.set_audience(&[&self.audience]);

        let token_data = decode::<crate::infra::oidc::service::RefreshTokenClaims>(
            refresh_token, 
            &decoding_key, 
            &validation
        )?;

        if token_data.claims.token_use != "refresh" {
            return Err("Invalid token use - expected refresh token".into());
        }

        // Create a mock Firebase user for token generation (we only have the user ID)
        let firebase_user = FirebaseUser {
            uid: token_data.claims.sub.clone(),
            email: None,
            email_verified: false,
            display_name: None,
            photo_url: None,
            phone_number: None,
            disabled: false,
            custom_claims: HashMap::new(),
            provider_data: Vec::new(),
            created_at: chrono::Utc::now(),
            last_login_at: None,
        };

        self.generate_tokens_with_permissions(&firebase_user, permission_set, None).await
    }

    /// Extract user role from Firebase user
    fn extract_user_role(&self, firebase_user: &FirebaseUser) -> String {
        firebase_user
            .custom_claims
            .get("role")
            .and_then(|v| v.as_str())
            .unwrap_or("user")
            .to_string()
    }

    /// Generate new token with updated permissions (for instant admin control)
    pub async fn generate_fresh_token_with_permissions(
        &self,
        _user_id: &str,
        permission_set: GranularPermissionSet,
        current_claims: &EnhancedOIDCClaims,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let fresh_claims = EnhancedOIDCClaims {
            granular_permissions: permission_set.permissions,
            permission_version: permission_set.version,
            permission_hash: permission_set.hash,
            next_validation: now + 300, // 5 minutes
            jti: Uuid::new_v4().to_string(), // New JWT ID
            iat: now, // New issued at
            ..current_claims.clone()
        };

        let header = Header::new(Algorithm::RS256);
        let private_key_pem = self.private_key.to_pkcs8_pem(rsa::pkcs8::LineEnding::LF)?;
        let encoding_key = EncodingKey::from_rsa_pem(private_key_pem.as_bytes())?;

        Ok(encode(&header, &fresh_claims, &encoding_key)?)
    }

    /// Validate permission hash for instant revocation
    pub fn validate_permission_hash(
        &self,
        claims: &EnhancedOIDCClaims,
        expected_hash: &str,
    ) -> bool {
        claims.permission_hash == expected_hash
    }

    /// Get public key for token validation (for other services)
    pub fn get_public_key_pem(&self) -> Result<String, Box<dyn std::error::Error>> {
        Ok(self.public_key.to_public_key_pem(rsa::pkcs8::LineEnding::LF)?)
    }

    /// Get JWT kid (key ID) for token header
    pub fn get_key_id(&self) -> String {
        // Generate key ID from public key modulus (first 8 chars of hex)
        format!("{:x}", self.public_key.n().to_bytes_be()[0..4].iter().fold(0u32, |acc, &b| acc << 8 | b as u32))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::granular_permissions::{PermissionSource};
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_enhanced_oidc_token_generation() {
        let service = EnhancedOIDCService::new().await.unwrap();
        
        let firebase_user = FirebaseUser {
            uid: "test_user_123".to_string(),
            email: Some("test@epsx.io".to_string()),
            email_verified: true,
            display_name: Some("Test User".to_string()),
            photo_url: None,
            phone_number: None,
            disabled: false,
            custom_claims: HashMap::new(),
            provider_data: Vec::new(),
            created_at: chrono::Utc::now(),
            last_login_at: None,
        };

        let mut permission_set = GranularPermissionSet::new();
        permission_set.add_permission(
            "epsx:rankings:view:10".to_string(),
            GranularPermissionClaim::permanent(PermissionSource::Subscription, None)
        );

        let tokens = service.generate_tokens_with_permissions(
            &firebase_user,
            permission_set,
            None
        ).await.unwrap();

        assert!(!tokens.access_token.is_empty());
        assert!(!tokens.id_token.is_empty());
        assert!(!tokens.refresh_token.is_empty());
        assert_eq!(tokens.token_type, "Bearer");
        assert_eq!(tokens.permission_version, 2); // Started at 1, one permission added
    }

    #[tokio::test]
    async fn test_token_validation_with_permissions() {
        let service = EnhancedOIDCService::new().await.unwrap();
        
        let firebase_user = FirebaseUser {
            uid: "test_user_456".to_string(),
            email: Some("test@epsx.io".to_string()),
            email_verified: true,
            display_name: Some("Test User".to_string()),
            photo_url: None,
            phone_number: None,
            disabled: false,
            custom_claims: HashMap::new(),
            provider_data: Vec::new(),
            created_at: chrono::Utc::now(),
            last_login_at: None,
        };

        let mut permission_set = GranularPermissionSet::new();
        permission_set.add_permission(
            "epsx:analytics:premium".to_string(),
            GranularPermissionClaim::permanent(PermissionSource::ManualGrant, Some("admin_123".to_string()))
        );

        let tokens = service.generate_tokens_with_permissions(
            &firebase_user,
            permission_set,
            None
        ).await.unwrap();

        let validation_result = service.validate_bearer_token_with_permissions(
            &tokens.access_token
        ).await.unwrap();

        assert_eq!(validation_result.claims.sub, "test_user_456");
        assert_eq!(validation_result.valid_permissions.len(), 1);
        assert_eq!(validation_result.valid_permissions[0], "epsx:analytics:premium");
        assert!(!validation_result.needs_refresh);
        assert!(validation_result.updated_token.is_none());
    }
}