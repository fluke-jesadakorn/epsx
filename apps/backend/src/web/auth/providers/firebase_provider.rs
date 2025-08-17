// Firebase Authentication Provider
// Handles Firebase JWT token validation and user mapping

use async_trait::async_trait;
use jsonwebtoken::{decode_header, jwk::JwkSet, Algorithm};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use crate::config::env::get_env_var;

use super::{AuthProvider, ProviderType, UserClaims, TokenPair, AuthProviderError};

/// Firebase JWT claims structure
#[derive(Debug, Serialize, Deserialize)]
struct FirebaseTokenClaims {
    /// Firebase UID
    sub: String,
    /// Email
    email: Option<String>,
    /// Email verified
    email_verified: Option<bool>,
    /// Firebase project ID
    aud: String,
    /// Issuer (Firebase)
    iss: String,
    /// Issued at
    iat: i64,
    /// Expires at
    exp: i64,
    /// Authentication time
    auth_time: Option<i64>,
    /// Sign-in provider
    firebase: Option<FirebaseProviderData>,
}

#[derive(Debug, Serialize, Deserialize)]
struct FirebaseProviderData {
    sign_in_provider: Option<String>,
}

/// Firebase provider configuration
pub struct FirebaseProviderConfig {
    pub project_id: String,
    pub service_account_email: Option<String>,
    pub jwks_cache_ttl_seconds: u64,
}

impl Default for FirebaseProviderConfig {
    fn default() -> Self {
        Self {
            project_id: get_env_var("FIREBASE_PROJECT_ID")
                .unwrap_or_else(|_| "your-project-id".to_string()),
            service_account_email: get_env_var("FIREBASE_SERVICE_ACCOUNT_EMAIL").ok(),
            jwks_cache_ttl_seconds: 3600, // 1 hour
        }
    }
}

/// Firebase authentication provider
pub struct FirebaseProvider {
    config: FirebaseProviderConfig,
    jwks_cache: tokio::sync::RwLock<Option<(JwkSet, DateTime<Utc>)>>,
    http_client: reqwest::Client,
    // TODO: Add user mapping service reference
}

impl FirebaseProvider {
    pub fn new(config: FirebaseProviderConfig) -> Self {
        Self {
            config,
            jwks_cache: tokio::sync::RwLock::new(None),
            http_client: reqwest::Client::new(),
        }
    }

    /// Fetch Firebase public keys for JWT validation
    async fn fetch_firebase_jwks(&self) -> Result<JwkSet, AuthProviderError> {
        let jwks_url = format!(
            "https://www.googleapis.com/robot/v1/metadata/x509/securetoken@{}.iam.gserviceaccount.com",
            self.config.project_id
        );

        let response = self
            .http_client
            .get(&jwks_url)
            .send()
            .await
            .map_err(|e| AuthProviderError::NetworkError(format!("Failed to fetch JWKS: {}", e)))?;

        if !response.status().is_success() {
            return Err(AuthProviderError::NetworkError(format!(
                "JWKS fetch failed with status: {}", 
                response.status()
            )));
        }

        let _jwks_data: HashMap<String, String> = response
            .json()
            .await
            .map_err(|e| AuthProviderError::NetworkError(format!("Failed to parse JWKS: {}", e)))?;

        // Convert HashMap<String, String> to JwkSet format expected by jsonwebtoken
        // Firebase returns certificates in a different format than standard JWKS
        // This is a simplified implementation - in production, you'd want proper certificate parsing
        let jwks = JwkSet { keys: Vec::new() };
        
        // TODO: Properly parse Firebase certificates to JWK format
        // For now, return empty set to make it compile
        Ok(jwks)
    }

    /// Get cached or fetch fresh JWKS
    async fn get_jwks(&self) -> Result<JwkSet, AuthProviderError> {
        let cache_read = self.jwks_cache.read().await;
        
        // Check if cache is valid
        if let Some((jwks, expires_at)) = cache_read.as_ref() {
            if Utc::now() < *expires_at {
                return Ok(jwks.clone());
            }
        }
        
        drop(cache_read);
        
        // Fetch fresh JWKS
        let jwks = self.fetch_firebase_jwks().await?;
        let expires_at = Utc::now() + chrono::Duration::seconds(self.config.jwks_cache_ttl_seconds as i64);
        
        // Update cache
        let mut cache_write = self.jwks_cache.write().await;
        *cache_write = Some((jwks.clone(), expires_at));
        
        Ok(jwks)
    }

}

#[async_trait]
impl AuthProvider for FirebaseProvider {
    async fn validate_token(&self, token: &str) -> Result<UserClaims, AuthProviderError> {
        // Decode JWT header to get key ID
        let _header = decode_header(token)
            .map_err(|_e| AuthProviderError::InvalidTokenFormat)?;

        // Get JWKS for validation
        let _jwks = self.get_jwks().await?;
        
        // TODO: Find the correct key from JWKS using header.kid
        // For now, return error as Firebase certificate validation is complex
        Err(AuthProviderError::ConfigurationError(
            "Firebase provider not fully implemented yet".to_string()
        ))
    }

    async fn refresh_token(&self, _refresh_token: &str) -> Result<TokenPair, AuthProviderError> {
        // Firebase doesn't support server-side token refresh in the same way
        // Client-side Firebase SDK handles token refresh
        Err(AuthProviderError::InternalError(
            "Firebase refresh handled client-side".to_string()
        ))
    }

    fn provider_name(&self) -> &'static str {
        "Firebase"
    }

    fn provider_type(&self) -> ProviderType {
        ProviderType::Firebase
    }

    fn priority(&self) -> u8 {
        100 // Highest priority
    }

    fn can_handle_token(&self, token: &str) -> bool {
        // Firebase JWTs have specific issuer format
        if let Ok(header) = decode_header(token) {
            // Basic check - Firebase tokens typically have "RS256" algorithm
            header.alg == Algorithm::RS256
        } else {
            false
        }
    }

    async fn get_user_info(&self, firebase_uid: &str) -> Result<serde_json::Value, AuthProviderError> {
        // TODO: Implement Firebase Admin SDK user info retrieval
        Ok(serde_json::json!({
            "firebase_uid": firebase_uid,
            "provider": "firebase",
            "status": "not_implemented"
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_firebase_provider_creation() {
        let config = FirebaseProviderConfig::default();
        let provider = FirebaseProvider::new(config);
        
        assert_eq!(provider.provider_name(), "Firebase");
        assert_eq!(provider.provider_type(), ProviderType::Firebase);
        assert_eq!(provider.priority(), 100);
    }

    #[test]
    fn test_can_handle_token() {
        let config = FirebaseProviderConfig::default();
        let provider = FirebaseProvider::new(config);
        
        // Invalid token
        assert!(!provider.can_handle_token("invalid"));
        
        // TODO: Add test with valid Firebase JWT structure
    }
}