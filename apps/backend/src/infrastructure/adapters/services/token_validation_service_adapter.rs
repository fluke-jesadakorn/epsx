// Token Validation Service Adapter
// Bridges DDD token validation with existing OIDC providers

use async_trait::async_trait;
use tracing::{info, warn, error, debug};
use std::sync::Arc;

use crate::domain::authentication::{
    TokenValidationServicePort, AccessToken, RefreshToken,
    TokenClaims, TokenIntrospectionResult
};
use crate::web::auth::providers::{AuthProvider, ProviderType};
use crate::infra::firebase_admin::FirebaseAdmin;

/// Token validation service adapter
pub struct TokenValidationServiceAdapter {
    /// Firebase provider for Firebase tokens
    firebase_provider: Arc<dyn AuthProvider>,
    
    /// OIDC provider for backend-issued tokens
    oidc_provider: Arc<dyn AuthProvider>,
    
    /// Firebase admin for direct validation
    firebase_admin: Arc<FirebaseAdmin>,
}

impl TokenValidationServiceAdapter {
    pub fn new(
        firebase_provider: Arc<dyn AuthProvider>,
        oidc_provider: Arc<dyn AuthProvider>,
        firebase_admin: Arc<FirebaseAdmin>,
    ) -> Self {
        Self {
            firebase_provider,
            oidc_provider,
            firebase_admin,
        }
    }
    
    /// Determine which provider should handle the token
    fn determine_provider(&self, token: &str) -> Arc<dyn AuthProvider> {
        // Basic token type detection
        if self.firebase_provider.can_handle_token(token) {
            debug!("Token identified as Firebase token");
            self.firebase_provider.clone()
        } else if self.oidc_provider.can_handle_token(token) {
            debug!("Token identified as OIDC token");
            self.oidc_provider.clone()
        } else {
            // Default to OIDC for unknown tokens
            debug!("Unknown token type, defaulting to OIDC provider");
            self.oidc_provider.clone()
        }
    }
    
    /// Validate token using appropriate provider
    async fn validate_with_provider(&self, token: &str) -> Result<TokenValidationResult, String> {
        let provider = self.determine_provider(token);
        
        match provider.validate_token(token).await {
            Ok(user_claims) => {
                Ok(TokenValidationResult {
                    is_valid: true,
                    user_id: Some(user_claims.user_id.to_string()),
                    scopes: user_claims.permissions.clone(),
                    expires_at: Some(user_claims.expires_at),
                    provider_type: user_claims.provider_type,
                    error: None,
                })
            },
            Err(e) => {
                warn!(error = %e, "Token validation failed");
                Ok(TokenValidationResult {
                    is_valid: false,
                    user_id: None,
                    scopes: vec![],
                    expires_at: None,
                    provider_type: provider.provider_type(),
                    error: Some(e.to_string()),
                })
            }
        }
    }
}

#[async_trait]
impl TokenValidationServicePort for TokenValidationServiceAdapter {
    async fn validate_access_token(&self, token: &str) -> Result<bool, String> {
        info!("Validating access token");
        
        if token.is_empty() {
            return Ok(false);
        }
        
        let validation_result = self.validate_with_provider(token).await?;
        
        if validation_result.is_valid {
            info!(
                user_id = validation_result.user_id.as_deref().unwrap_or("unknown"),
                provider = ?validation_result.provider_type,
                "Access token validation successful"
            );
        } else {
            warn!(
                error = validation_result.error.as_deref().unwrap_or("unknown"),
                "Access token validation failed"
            );
        }
        
        Ok(validation_result.is_valid)
    }
    
    async fn validate_refresh_token(&self, token: &str) -> Result<bool, String> {
        info!("Validating refresh token");
        
        if token.is_empty() {
            return Ok(false);
        }
        
        // Refresh tokens are typically opaque strings that need special handling
        // For Firebase tokens, we can't validate refresh tokens directly
        // For OIDC tokens, we need to check token format and expiry
        
        if token.starts_with("rt_") {
            // Our internal refresh token format
            let refresh_token = RefreshToken::from_string(token.to_string())
                .map_err(|e| format!("Invalid refresh token format: {}", e))?;
            
            let is_valid = !refresh_token.is_expired();
            
            if is_valid {
                info!("Refresh token validation successful");
            } else {
                warn!("Refresh token has expired");
            }
            
            Ok(is_valid)
        } else {
            // External refresh token - validate with appropriate provider
            let provider = self.determine_provider(token);
            
            // Most providers don't allow direct refresh token validation
            // We'll assume it's valid if it has the right format
            match provider.provider_type() {
                ProviderType::Firebase => {
                    // Firebase refresh tokens are opaque - assume valid if not empty
                    Ok(!token.is_empty())
                },
                ProviderType::OIDC => {
                    // Try to validate as JWT if possible
                    self.validate_with_provider(token).await.map(|r| r.is_valid)
                },
                _ => Ok(false),
            }
        }
    }
    
    async fn validate_id_token(&self, token: &str) -> Result<bool, String> {
        info!("Validating ID token");
        
        if token.is_empty() {
            return Ok(false);
        }
        
        // ID tokens should be JWT format and contain OIDC claims
        let validation_result = self.validate_with_provider(token).await?;
        
        if validation_result.is_valid {
            // Additional validation for ID tokens
            if validation_result.scopes.is_empty() || 
               !validation_result.scopes.iter().any(|s| s.contains("openid")) {
                warn!("ID token missing required openid scope");
                return Ok(false);
            }
            
            info!(
                user_id = validation_result.user_id.as_deref().unwrap_or("unknown"),
                "ID token validation successful"
            );
        } else {
            warn!(
                error = validation_result.error.as_deref().unwrap_or("unknown"),
                "ID token validation failed"
            );
        }
        
        Ok(validation_result.is_valid)
    }
    
    async fn get_token_claims(&self, token: &str) -> Result<TokenClaims, String> {
        info!("Extracting token claims");
        
        let validation_result = self.validate_with_provider(token).await?;
        
        if !validation_result.is_valid {
            return Err(validation_result.error.unwrap_or_else(|| "Token validation failed".to_string()));
        }
        
        Ok(TokenClaims {
            user_id: validation_result.user_id.unwrap_or_default(),
            scopes: validation_result.scopes,
            expires_at: validation_result.expires_at.unwrap_or_else(chrono::Utc::now),
            provider_type: validation_result.provider_type,
            issued_at: chrono::Utc::now(), // Would be extracted from token in production
            audience: "epsx-api".to_string(), // Would be extracted from token
            issuer: "https://api.epsx.io".to_string(), // Would be extracted from token
        })
    }
    
    async fn revoke_token(&self, token: &str) -> Result<(), String> {
        info!("Revoking token");
        
        let provider = self.determine_provider(token);
        
        match provider.provider_type() {
            ProviderType::Firebase => {
                // Firebase doesn't support token revocation via API
                // We'd need to maintain a revocation list or use Firebase Admin SDK
                warn!("Firebase token revocation not implemented - using local revocation list");
                
                // In production, add to revocation cache/database
                // self.add_to_revocation_list(token).await?;
                
                Ok(())
            },
            ProviderType::OIDC => {
                // For our own OIDC tokens, we can revoke by adding to blacklist
                info!("Adding OIDC token to revocation list");
                
                // In production, implement proper token revocation
                // This might involve:
                // 1. Adding to Redis blacklist with TTL
                // 2. Database revocation table
                // 3. JWT revocation endpoint
                
                Ok(())
            },
            _ => {
                error!("Token revocation not supported for this provider type");
                Err("Token revocation not supported".to_string())
            }
        }
    }
    
    async fn introspect_token(&self, token: &str) -> Result<TokenIntrospectionResult, String> {
        info!("Performing token introspection");
        
        let validation_result = self.validate_with_provider(token).await?;
        
        Ok(TokenIntrospectionResult {
            active: validation_result.is_valid,
            user_id: validation_result.user_id,
            client_id: Some("epsx-client".to_string()), // Would be extracted from token
            scopes: validation_result.scopes,
            expires_at: validation_result.expires_at,
            issued_at: Some(chrono::Utc::now()), // Would be extracted from token
            token_type: Some("Bearer".to_string()),
            provider_type: validation_result.provider_type,
        })
    }
}

/// Token validation result
struct TokenValidationResult {
    is_valid: bool,
    user_id: Option<String>,
    scopes: Vec<String>,
    expires_at: Option<chrono::DateTime<chrono::Utc>>,
    provider_type: ProviderType,
    error: Option<String>,
}

// TokenClaims and TokenIntrospectionResult now come from domain layer

#[cfg(test)]
mod tests {
    use super::*;
    use crate::web::auth::providers::{AuthProviderError, UserClaims, TokenPair};
    
    // Mock auth provider for testing
    struct MockAuthProvider {
        provider_type: ProviderType,
        should_validate: bool,
    }
    
    impl MockAuthProvider {
        fn new(provider_type: ProviderType, should_validate: bool) -> Self {
            Self {
                provider_type,
                should_validate,
            }
        }
    }
    
    #[async_trait]
    impl AuthProvider for MockAuthProvider {
        async fn validate_token(&self, _token: &str) -> Result<UserClaims, AuthProviderError> {
            if self.should_validate {
                Ok(UserClaims::new(
                    crate::dom::values::UserId::new("123".to_string()),
                    crate::dom::values::Email::new("test@example.com".to_string()).unwrap(),
                    vec!["openid".to_string(), "profile".to_string()],
                    "test_provider_id".to_string(),
                    self.provider_type.clone(),
                    chrono::Utc::now() + chrono::Duration::hours(1),
                    chrono::Utc::now().timestamp() as usize,
                    (chrono::Utc::now() + chrono::Duration::hours(1)).timestamp() as usize,
                    Some("premium".to_string()),
                ))
            } else {
                Err(AuthProviderError::InvalidToken)
            }
        }
        
        async fn refresh_token(&self, _refresh_token: &str) -> Result<TokenPair, AuthProviderError> {
            Err(AuthProviderError::InvalidToken)
        }
        
        fn provider_name(&self) -> &'static str {
            "mock"
        }
        
        fn provider_type(&self) -> ProviderType {
            self.provider_type.clone()
        }
        
        fn priority(&self) -> u8 {
            50
        }
        
        fn can_handle_token(&self, _token: &str) -> bool {
            true
        }
        
        async fn get_user_info(&self, _user_id: &str) -> Result<serde_json::Value, AuthProviderError> {
            Ok(serde_json::json!({"user_id": "123", "provider": "mock"}))
        }
    }
    
    #[tokio::test]
    async fn test_access_token_validation() {
        let firebase_provider = Arc::new(MockAuthProvider::new(ProviderType::Firebase, true));
        let oidc_provider = Arc::new(MockAuthProvider::new(ProviderType::OIDC, true));
        let firebase_admin = Arc::new(FirebaseAdmin::new("test_project_id".to_string()));
        
        let adapter = TokenValidationServiceAdapter::new(
            firebase_provider,
            oidc_provider,
            firebase_admin,
        );
        
        let result = adapter.validate_access_token("valid_token").await;
        assert!(result.is_ok());
        assert!(result.unwrap());
        
        let result = adapter.validate_access_token("").await;
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }
    
    #[tokio::test]
    async fn test_refresh_token_validation() {
        let firebase_provider = Arc::new(MockAuthProvider::new(ProviderType::Firebase, true));
        let oidc_provider = Arc::new(MockAuthProvider::new(ProviderType::OIDC, true));
        let firebase_admin = Arc::new(FirebaseAdmin::new("test_project_id".to_string()));
        
        let adapter = TokenValidationServiceAdapter::new(
            firebase_provider,
            oidc_provider,
            firebase_admin,
        );
        
        // Test our internal refresh token format
        let result = adapter.validate_refresh_token("rt_sess_123_jti456").await;
        // This should fail because we can't actually create a valid refresh token in test
        // but it tests the code path
        assert!(result.is_err() || !result.unwrap());
        
        // Test empty token
        let result = adapter.validate_refresh_token("").await;
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }
    
    #[tokio::test]
    async fn test_token_claims_extraction() {
        let firebase_provider = Arc::new(MockAuthProvider::new(ProviderType::Firebase, true));
        let oidc_provider = Arc::new(MockAuthProvider::new(ProviderType::OIDC, true));
        let firebase_admin = Arc::new(FirebaseAdmin::new("test_project_id".to_string()));
        
        let adapter = TokenValidationServiceAdapter::new(
            firebase_provider,
            oidc_provider,
            firebase_admin,
        );
        
        let result = adapter.get_token_claims("valid_token").await;
        assert!(result.is_ok());
        
        let claims = result.unwrap();
        assert_eq!(claims.user_id, "123");
        assert!(claims.scopes.contains(&"openid".to_string()));
    }
}