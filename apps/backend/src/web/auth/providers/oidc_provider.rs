// OpenID Connect Provider
// Handles custom OIDC JWT tokens issued by our backend

use async_trait::async_trait;
use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use crate::config::env::get_env_var;

use super::{AuthProvider, ProviderType, UserClaims, TokenPair, AuthProviderError};
use crate::dom::values::{UserId, Email};

/// OIDC JWT claims structure for our backend-issued tokens
#[derive(Debug, Serialize, Deserialize)]
pub struct OIDCTokenClaims {
    /// Subject (user_id)
    pub sub: String,
    /// Issuer (our backend)
    pub iss: String,
    /// Audience (client_id)
    pub aud: String,
    /// Email
    pub email: String,
    /// User package tier
    pub package_tier: String,
    /// Structured permissions: "platform:resource:action"
    pub permissions: Vec<String>,
    /// Issued at
    pub iat: i64,
    /// Expires at
    pub exp: i64,
    /// JWT ID (for revocation)
    pub jti: Option<String>,
    /// Subscription tier
    pub subscription_tier: Option<String>,
}

/// OIDC provider configuration
pub struct OIDCProviderConfig {
    /// JWT signing secret (should be same as NEXTAUTH_SECRET)
    pub jwt_secret: String,
    /// Expected issuer URL
    pub issuer_url: String,
    /// Expected audience (client ID)
    pub expected_audience: String,
    /// Algorithm for JWT validation
    pub algorithm: Algorithm,
}

impl Default for OIDCProviderConfig {
    fn default() -> Self {
        Self {
            jwt_secret: get_env_var("NEXTAUTH_SECRET")
                .or_else(|_| get_env_var("AUTH_SECRET"))
                .unwrap_or_else(|_| "default-secret".to_string()),
            issuer_url: get_env_var("BACKEND_URL")
                .unwrap_or_else(|_| "http://localhost:8080".to_string()),
            expected_audience: get_env_var("OIDC_CLIENT_ID")
                .unwrap_or_else(|_| "frontend-client".to_string()),
            algorithm: Algorithm::HS256, // Using symmetric key for simplicity
        }
    }
}

/// OpenID Connect provider for backend-issued tokens
pub struct OIDCProvider {
    config: OIDCProviderConfig,
    decoding_key: DecodingKey,
    validation: Validation,
}

impl OIDCProvider {
    pub fn new(config: OIDCProviderConfig) -> Self {
        let decoding_key = DecodingKey::from_secret(config.jwt_secret.as_bytes());
        
        let mut validation = Validation::new(config.algorithm);
        validation.set_issuer(&[&config.issuer_url]);
        validation.set_audience(&[&config.expected_audience]);
        
        Self {
            config,
            decoding_key,
            validation,
        }
    }

    /// Parse JWT and extract claims
    async fn parse_token(&self, token: &str) -> Result<OIDCTokenClaims, AuthProviderError> {
        let token_data = decode::<OIDCTokenClaims>(
            token,
            &self.decoding_key,
            &self.validation,
        )
        .map_err(|e| {
            tracing::error!("JWT validation failed: {}", e);
            match e.kind() {
                jsonwebtoken::errors::ErrorKind::ExpiredSignature => AuthProviderError::TokenExpired,
                jsonwebtoken::errors::ErrorKind::InvalidToken => AuthProviderError::InvalidTokenFormat,
                jsonwebtoken::errors::ErrorKind::InvalidIssuer => AuthProviderError::TokenValidationFailed("Invalid issuer".to_string()),
                jsonwebtoken::errors::ErrorKind::InvalidAudience => AuthProviderError::TokenValidationFailed("Invalid audience".to_string()),
                _ => AuthProviderError::TokenValidationFailed(format!("JWT error: {}", e)),
            }
        })?;

        Ok(token_data.claims)
    }
}

#[async_trait]
impl AuthProvider for OIDCProvider {
    async fn validate_token(&self, token: &str) -> Result<UserClaims, AuthProviderError> {
        let claims = self.parse_token(token).await?;
        
        // Parse user_id from sub claim
        let user_id = UserId::from_string(claims.sub.clone());
        
        // Parse email
        let email = Email::new(claims.email.clone())
            .map_err(|e| AuthProviderError::TokenValidationFailed(format!("Invalid email: {}", e)))?;
        
        // Parse role
        // Convert timestamp to DateTime
        let expires_at = DateTime::from_timestamp(claims.exp, 0)
            .ok_or_else(|| AuthProviderError::TokenValidationFailed("Invalid expiry timestamp".to_string()))?;
        
        let user_claims = UserClaims::new(
            user_id,
            email,
            claims.permissions, // Use permissions directly instead of role
            claims.sub, // provider_user_id is same as backend user_id for OIDC
            ProviderType::OIDC,
            expires_at,
            claims.iat.try_into().unwrap(),
            claims.exp.try_into().unwrap(),
            claims.subscription_tier.clone(), // Use subscription_tier from claims
        );
        
        // Add extra claims if available
        let mut user_claims = user_claims;
        if let Some(subscription_tier) = claims.subscription_tier {
            user_claims = user_claims.with_claim(
                "subscription_tier".to_string(),
                serde_json::Value::String(subscription_tier),
            );
        }
        if let Some(jti) = claims.jti {
            user_claims = user_claims.with_claim(
                "jti".to_string(),
                serde_json::Value::String(jti),
            );
        }
        
        Ok(user_claims)
    }

    async fn refresh_token(&self, refresh_token: &str) -> Result<TokenPair, AuthProviderError> {
        use reqwest::Client;
        
        // Prepare token refresh request to OIDC provider
        let token_endpoint = format!("{}/token", self.config.issuer_url.trim_end_matches('/'));
        
        let params = [
            ("grant_type", "refresh_token"),
            ("refresh_token", refresh_token),
            ("client_id", &self.config.expected_audience),
            ("client_secret", "placeholder_secret"),
        ];
        
        let client = Client::new();
        let response = client
            .post(&token_endpoint)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .form(&params)
            .send()
            .await
            .map_err(|e| AuthProviderError::NetworkError(format!("Token refresh request failed: {}", e)))?;
        
        if !response.status().is_success() {
            let _error_text = response.text().await.unwrap_or_default();
            return Err(AuthProviderError::InvalidToken);
        }
        
        // Parse token response
        let token_response: serde_json::Value = response.json().await
            .map_err(|e| AuthProviderError::NetworkError(format!("Failed to parse token response: {}", e)))?;
        
        let access_token = token_response.get("access_token")
            .and_then(|t| t.as_str())
            .ok_or(AuthProviderError::InvalidToken)?
            .to_string();
        
        let new_refresh_token = token_response.get("refresh_token")
            .and_then(|t| t.as_str())
            .map(|t| t.to_string())
            .unwrap_or_else(|| refresh_token.to_string()); // Use original if not provided
        
        let expires_in = token_response.get("expires_in")
            .and_then(|e| e.as_i64())
            .unwrap_or(3600); // Default to 1 hour
        
        let expires_at = Utc::now() + chrono::Duration::seconds(expires_in);
        
        Ok(TokenPair {
            access_token,
            refresh_token: Some(new_refresh_token),
            expires_at,
            token_type: "Bearer".to_string(),
        })
    }

    fn provider_name(&self) -> &'static str {
        "OIDC"
    }

    fn provider_type(&self) -> ProviderType {
        ProviderType::OIDC
    }

    fn priority(&self) -> u8 {
        90 // High priority, but lower than Firebase
    }

    fn can_handle_token(&self, token: &str) -> bool {
        // Try to decode without validation to check format
        if let Ok(header) = jsonwebtoken::decode_header(token) {
            // Check if algorithm matches our expected algorithm
            header.alg == self.config.algorithm
        } else {
            false
        }
    }

    async fn get_user_info(&self, user_id: &str) -> Result<serde_json::Value, AuthProviderError> {
        // TODO: Query user information from database
        Ok(serde_json::json!({
            "user_id": user_id,
            "provider": "oidc",
            "issuer": self.config.issuer_url
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_oidc_provider_creation() {
        let config = OIDCProviderConfig::default();
        let provider = OIDCProvider::new(config);
        
        assert_eq!(provider.provider_name(), "OIDC");
        assert_eq!(provider.provider_type(), ProviderType::OIDC);
        assert_eq!(provider.priority(), 90);
    }

    #[test]
    fn test_can_handle_token() {
        let config = OIDCProviderConfig::default();
        let provider = OIDCProvider::new(config);
        
        // Invalid token
        assert!(!provider.can_handle_token("invalid"));
        
        // TODO: Add test with valid JWT structure
    }

    #[tokio::test]
    async fn test_token_validation_with_invalid_token() {
        let config = OIDCProviderConfig::default();
        let provider = OIDCProvider::new(config);
        
        let result = provider.validate_token("invalid.token.here").await;
        assert!(result.is_err());
        
        match result.unwrap_err() {
            AuthProviderError::InvalidTokenFormat => {},
            AuthProviderError::TokenValidationFailed(_) => {},
            _ => panic!("Expected token validation error"),
        }
    }
}