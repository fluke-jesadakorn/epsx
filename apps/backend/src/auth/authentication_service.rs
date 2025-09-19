// Unified Authentication Service
// Consolidates: jwt.rs, admin_jwt.rs, user_jwt.rs, flow.rs, tokens.rs, key_manager.rs

use std::sync::Arc;
use jsonwebtoken::{encode, decode, Header, Validation, Algorithm};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc, Duration};
use uuid::Uuid;
use anyhow::Result;
use tracing::{debug, error, info};

use crate::infrastructure::cache::Cache;
use super::key_manager::KeyManager;

/// Unified JWT claims for all authentication scenarios
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AuthClaims {
    pub sub: String, // User ID
    pub email: Option<String>,
    pub name: Option<String>,
    pub role: Option<String>,
    pub permissions: Vec<String>,
    pub exp: i64,
    pub iat: i64,
    pub iss: String,
    pub aud: String,
    pub jti: String, // JWT ID for revocation
}

/// Authentication result
#[derive(Debug, Clone)]
pub struct AuthResult {
    pub user_id: String,
    pub email: Option<String>,
    pub name: Option<String>,
    pub role: Option<String>,
    pub permissions: Vec<String>,
    pub token: String,
    pub expires_at: DateTime<Utc>,
}

/// Unified Authentication Service
pub struct AuthenticationService {
    key_manager: Arc<KeyManager>,
    cache: Arc<dyn Cache>,
    token_expiry: Duration,
    issuer: String,
    audience: String,
}

impl AuthenticationService {
    /// Create new authentication service with RS256 key manager
    pub fn new(
        key_manager: Arc<KeyManager>,
        cache: Arc<dyn Cache>,
        token_expiry_hours: i64,
        issuer: String,
        audience: String,
    ) -> Self {
        Self {
            key_manager,
            cache,
            token_expiry: Duration::hours(token_expiry_hours),
            issuer,
            audience,
        }
    }
    
    /// Generate JWT token for authenticated user
    pub async fn create_token(
        &self,
        user_id: &str,
        email: Option<String>,
        name: Option<String>,
        role: Option<String>,
        permissions: Vec<String>,
    ) -> Result<AuthResult> {
        let now = Utc::now();
        let expires_at = now + self.token_expiry;
        let jti = Uuid::new_v4().to_string();
        
        let claims = AuthClaims {
            sub: user_id.to_string(),
            email: email.clone(),
            name: name.clone(),
            role: role.clone(),
            permissions: permissions.clone(),
            exp: expires_at.timestamp(),
            iat: now.timestamp(),
            iss: self.issuer.clone(),
            aud: self.audience.clone(),
            jti: jti.clone(),
        };
        
        let mut header = Header::new(Algorithm::RS256);
        header.kid = Some(self.key_manager.current_key().kid.clone());
        let token = encode(&header, &claims, &self.key_manager.current_key().encoding_key)?;
        
        // Cache token for quick validation
        let cache_key = format!("auth_token:{}", jti);
        self.cache.set(&cache_key, token.clone(), Some(self.token_expiry.num_seconds() as u64));
        
        info!("Created authentication token for user: {}", user_id);
        
        Ok(AuthResult {
            user_id: user_id.to_string(),
            email,
            name,
            role,
            permissions,
            token,
            expires_at,
        })
    }
    
    /// Validate JWT token and extract claims using RS256
    pub async fn validate_token(&self, token: &str) -> Result<AuthClaims> {
        // Decode header to get key ID
        let header = jsonwebtoken::decode_header(token)?;
        
        // Ensure algorithm is RS256
        if header.alg != Algorithm::RS256 {
            return Err(anyhow::anyhow!("Invalid algorithm, only RS256 supported"));
        }
        
        // Get the appropriate key for verification
        let key_id = header.kid.ok_or_else(|| anyhow::anyhow!("Missing key ID"))?;
        let key = self.key_manager.get_key(&key_id)
            .ok_or_else(|| anyhow::anyhow!("Unknown key ID"))?;
        
        let mut validation = Validation::new(Algorithm::RS256);
        validation.set_issuer(&[&self.issuer]);
        validation.set_audience(&[&self.audience]);
        
        let token_data = decode::<AuthClaims>(token, &key.decoding_key, &validation)?;
        
        let claims = token_data.claims;
        
        // Check if token is cached (not revoked)
        let cache_key = format!("auth_token:{}", claims.jti);
        match self.cache.get(&cache_key) {
            Some(_) => {
                debug!("Token validation successful for user: {}", claims.sub);
                Ok(claims)
            }
            None => {
                error!("Token not found in cache (may be revoked): {}", claims.jti);
                Err(anyhow::anyhow!("Token not found or revoked"))
            }
        }
    }
    
    /// Revoke token
    pub async fn revoke_token(&self, jti: &str) -> Result<()> {
        let cache_key = format!("auth_token:{}", jti);
        self.cache.delete(&cache_key);
        
        info!("Revoked authentication token: {}", jti);
        Ok(())
    }
    
    /// Extract user information from token without validation (for internal use)
    pub fn decode_token_unsafe(&self, token: &str) -> Result<AuthClaims> {
        // Decode header to get key ID
        let header = jsonwebtoken::decode_header(token)?;
        
        // Ensure algorithm is RS256
        if header.alg != Algorithm::RS256 {
            return Err(anyhow::anyhow!("Invalid algorithm, only RS256 supported"));
        }
        
        // Get the appropriate key for verification
        let key_id = header.kid.ok_or_else(|| anyhow::anyhow!("Missing key ID"))?;
        let key = self.key_manager.get_key(&key_id)
            .ok_or_else(|| anyhow::anyhow!("Unknown key ID"))?;
        
        let mut validation = Validation::new(Algorithm::RS256);
        // Skip expiration validation for unsafe decode
        validation.validate_exp = false;
        let token_data = decode::<AuthClaims>(token, &key.decoding_key, &validation)?;
        Ok(token_data.claims)
    }
    
    /// Health check
    pub async fn health_check(&self) -> bool {
        // Try to create and validate a test token
        match self.create_token("health_check", None, None, None, vec![]).await {
            Ok(auth_result) => {
                match self.validate_token(&auth_result.token).await {
                    Ok(_) => {
                        // Clean up test token
                        let _ = self.revoke_token(&auth_result.token).await;
                        true
                    }
                    Err(_) => false
                }
            }
            Err(_) => false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::cache::memory_cache::MemoryCache;
    
    #[tokio::test]
    async fn test_token_creation_and_validation() {
        let cache = Arc::new(MemoryCache::new());
        let auth_service = AuthenticationService::new(
            "test-secret-key",
            cache,
            1, // 1 hour expiry
            "epsx-test".to_string(),
            "epsx-frontend".to_string(),
        );
        
        // Create token
        let auth_result = auth_service.create_token(
            "test-user-123",
            Some("test@example.com".to_string()),
            Some("Test User".to_string()),
            Some("user".to_string()),
            vec!["epsx:basic:read".to_string()],
        ).await.unwrap();
        
        assert_eq!(auth_result.user_id, "test-user-123");
        assert_eq!(auth_result.email, Some("test@example.com".to_string()));
        assert!(!auth_result.token.is_empty());
        
        // Validate token
        let claims = auth_service.validate_token(&auth_result.token).await.unwrap();
        assert_eq!(claims.sub, "test-user-123");
        assert_eq!(claims.email, Some("test@example.com".to_string()));
        assert_eq!(claims.permissions, vec!["epsx:basic:read".to_string()]);
    }
    
    #[tokio::test]
    async fn test_token_revocation() {
        let cache = Arc::new(MemoryCache::new());
        let auth_service = AuthenticationService::new(
            "test-secret-key",
            cache,
            1,
            "epsx-test".to_string(),
            "epsx-frontend".to_string(),
        );
        
        // Create and validate token
        let auth_result = auth_service.create_token(
            "test-user-123",
            None,
            None,
            None,
            vec![],
        ).await.unwrap();
        
        let claims = auth_service.validate_token(&auth_result.token).await.unwrap();
        
        // Revoke token
        auth_service.revoke_token(&claims.jti).await.unwrap();
        
        // Token should no longer be valid
        let result = auth_service.validate_token(&auth_result.token).await;
        assert!(result.is_err());
    }
}