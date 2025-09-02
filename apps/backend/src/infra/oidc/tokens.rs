// OIDC Token Management
// Handles token storage, cleanup, and revocation

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc, Duration};
use tracing::{info, error};

#[derive(Debug, Clone)]
pub struct StoredToken {
    pub jti: String,           // JWT ID
    pub user_id: String,       // Subject (user ID)
    pub token_type: String,    // "access", "id", or "refresh"
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub revoked: bool,
    pub last_used: Option<DateTime<Utc>>,
}

/// In-memory token store (for development/testing)
/// In production, this would be backed by Redis or database
pub struct TokenStore {
    tokens: RwLock<HashMap<String, StoredToken>>,
}

impl TokenStore {
    pub fn new() -> Self {
        Self {
            tokens: RwLock::new(HashMap::new()),
        }
    }

    /// Store a token
    pub async fn store_token(&self, token: StoredToken) -> Result<(), Box<dyn std::error::Error>> {
        let mut tokens = self.tokens.write().await;
        tokens.insert(token.jti.clone(), token);
        Ok(())
    }

    /// Get a token by JTI
    pub async fn get_token(&self, jti: &str) -> Option<StoredToken> {
        let tokens = self.tokens.read().await;
        tokens.get(jti).cloned()
    }

    /// Check if a token is revoked
    pub async fn is_revoked(&self, jti: &str) -> bool {
        let tokens = self.tokens.read().await;
        tokens.get(jti).map_or(false, |token| token.revoked)
    }

    /// Revoke a token
    pub async fn revoke_token(&self, jti: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut tokens = self.tokens.write().await;
        if let Some(token) = tokens.get_mut(jti) {
            token.revoked = true;
            info!("Revoked token: {} for user: {}", jti, token.user_id);
        }
        Ok(())
    }

    /// Revoke all tokens for a user
    pub async fn revoke_user_tokens(&self, user_id: &str) -> Result<u32, Box<dyn std::error::Error>> {
        let mut tokens = self.tokens.write().await;
        let mut revoked_count = 0;

        for token in tokens.values_mut() {
            if token.user_id == user_id && !token.revoked {
                token.revoked = true;
                revoked_count += 1;
            }
        }

        info!("Revoked {} tokens for user: {}", revoked_count, user_id);
        Ok(revoked_count)
    }

    /// Update last used time for a token
    pub async fn update_last_used(&self, jti: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut tokens = self.tokens.write().await;
        if let Some(token) = tokens.get_mut(jti) {
            token.last_used = Some(Utc::now());
        }
        Ok(())
    }

    /// Clean up expired tokens
    pub async fn cleanup_expired(&self) -> Result<u32, Box<dyn std::error::Error>> {
        let mut tokens = self.tokens.write().await;
        let now = Utc::now();
        let initial_count = tokens.len();

        tokens.retain(|_, token| token.expires_at > now);
        
        let cleaned_count = initial_count - tokens.len();
        if cleaned_count > 0 {
            info!("Cleaned up {} expired tokens", cleaned_count);
        }

        Ok(cleaned_count as u32)
    }

    /// Get token statistics
    pub async fn get_stats(&self) -> TokenStats {
        let tokens = self.tokens.read().await;
        let now = Utc::now();

        let total = tokens.len();
        let revoked = tokens.values().filter(|t| t.revoked).count();
        let expired = tokens.values().filter(|t| t.expires_at <= now).count();
        let active = total - revoked - expired;

        let mut by_type = HashMap::new();
        for token in tokens.values() {
            *by_type.entry(token.token_type.clone()).or_insert(0) += 1;
        }

        let mut by_user = HashMap::new();
        for token in tokens.values() {
            *by_user.entry(token.user_id.clone()).or_insert(0) += 1;
        }

        TokenStats {
            total,
            active,
            revoked,
            expired,
            by_type,
            user_count: by_user.len(),
        }
    }
}

#[derive(Debug)]
pub struct TokenStats {
    pub total: usize,
    pub active: usize,
    pub revoked: usize,
    pub expired: usize,
    pub by_type: HashMap<String, usize>,
    pub user_count: usize,
}

/// Token cleanup service that runs periodically
pub struct TokenCleanupService {
    token_store: Arc<TokenStore>,
    cleanup_interval: Duration,
}

impl TokenCleanupService {
    pub fn new(token_store: Arc<TokenStore>) -> Self {
        Self {
            token_store,
            cleanup_interval: Duration::hours(1), // Clean up every hour
        }
    }

    /// Start the cleanup service
    pub async fn start(&self) {
        let token_store = Arc::clone(&self.token_store);
        let interval = self.cleanup_interval;

        tokio::spawn(async move {
            let mut cleanup_interval = tokio::time::interval(
                std::time::Duration::from_secs(interval.num_seconds() as u64)
            );

            loop {
                cleanup_interval.tick().await;
                
                match token_store.cleanup_expired().await {
                    Ok(cleaned) => {
                        if cleaned > 0 {
                            info!("Token cleanup completed: {} tokens removed", cleaned);
                        }
                    }
                    Err(e) => {
                        error!("Token cleanup failed: {}", e);
                    }
                }
            }
        });

        info!("Token cleanup service started with interval: {:?}", self.cleanup_interval);
    }
}

/// Utility functions for token management
pub mod utils {
    use super::*;
    use base64::Engine;

    /// Extract JWT ID (JTI) from token without validation
    pub fn extract_jti(token: &str) -> Result<String, Box<dyn std::error::Error>> {
        let parts: Vec<&str> = token.split('.').collect();
        if parts.len() != 3 {
            return Err("Invalid JWT format".into());
        }

        let payload = parts[1];
        let decoded = base64::engine::general_purpose::URL_SAFE_NO_PAD.decode(payload)?;
        let claims: serde_json::Value = serde_json::from_slice(&decoded)?;
        
        claims.get("jti")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| "Missing JTI claim".into())
    }

    /// Extract expiration time from token without validation
    pub fn extract_exp(token: &str) -> Result<DateTime<Utc>, Box<dyn std::error::Error>> {
        let parts: Vec<&str> = token.split('.').collect();
        if parts.len() != 3 {
            return Err("Invalid JWT format".into());
        }

        let payload = parts[1];
        let decoded = base64::engine::general_purpose::URL_SAFE_NO_PAD.decode(payload)?;
        let claims: serde_json::Value = serde_json::from_slice(&decoded)?;
        
        let exp_timestamp = claims.get("exp")
            .and_then(|v| v.as_u64())
            .ok_or("Missing exp claim")?;

        Ok(DateTime::from_timestamp(exp_timestamp as i64, 0)
            .unwrap_or_else(|| Utc::now()))
    }

    /// Check if token is expired based on current time
    pub fn is_expired(token: &str) -> bool {
        match extract_exp(token) {
            Ok(exp) => Utc::now() > exp,
            Err(_) => true, // Treat invalid tokens as expired
        }
    }

    /// Get time until token expiration
    pub fn time_until_expiry(token: &str) -> Option<Duration> {
        match extract_exp(token) {
            Ok(exp) => {
                let now = Utc::now();
                if exp > now {
                    Some(exp - now)
                } else {
                    None // Already expired
                }
            }
            Err(_) => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    #[tokio::test]
    async fn test_token_store_operations() {
        let store = TokenStore::new();
        
        let token = StoredToken {
            jti: "test-jti-001".to_string(),
            user_id: "user-001".to_string(),
            token_type: "access".to_string(),
            expires_at: Utc::now() + Duration::hours(1),
            created_at: Utc::now(),
            revoked: false,
            last_used: None,
        };

        // Store token
        assert!(store.store_token(token.clone()).await.is_ok());

        // Retrieve token
        let retrieved = store.get_token("test-jti-001").await;
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().user_id, "user-001");

        // Check not revoked
        assert!(!store.is_revoked("test-jti-001").await);

        // Revoke token
        assert!(store.revoke_token("test-jti-001").await.is_ok());
        assert!(store.is_revoked("test-jti-001").await);
    }

    #[tokio::test]
    async fn test_token_cleanup() {
        let store = TokenStore::new();
        
        // Add expired token
        let expired_token = StoredToken {
            jti: "expired-001".to_string(),
            user_id: "user-001".to_string(),
            token_type: "access".to_string(),
            expires_at: Utc::now() - Duration::hours(1), // Expired
            created_at: Utc::now() - Duration::hours(2),
            revoked: false,
            last_used: None,
        };

        // Add valid token
        let valid_token = StoredToken {
            jti: "valid-001".to_string(),
            user_id: "user-002".to_string(),
            token_type: "access".to_string(),
            expires_at: Utc::now() + Duration::hours(1), // Valid
            created_at: Utc::now(),
            revoked: false,
            last_used: None,
        };

        store.store_token(expired_token).await.unwrap();
        store.store_token(valid_token).await.unwrap();

        // Check initial count
        let stats = store.get_stats().await;
        assert_eq!(stats.total, 2);

        // Cleanup expired tokens
        let cleaned = store.cleanup_expired().await.unwrap();
        assert_eq!(cleaned, 1);

        // Check remaining tokens
        let stats = store.get_stats().await;
        assert_eq!(stats.total, 1);
        assert!(store.get_token("valid-001").await.is_some());
        assert!(store.get_token("expired-001").await.is_none());
    }

    #[tokio::test]
    async fn test_user_token_revocation() {
        let store = TokenStore::new();
        
        // Add multiple tokens for same user
        for i in 1..=3 {
            let token = StoredToken {
                jti: format!("token-{}", i),
                user_id: "user-001".to_string(),
                token_type: "access".to_string(),
                expires_at: Utc::now() + Duration::hours(1),
                created_at: Utc::now(),
                revoked: false,
                last_used: None,
            };
            store.store_token(token).await.unwrap();
        }

        // Add token for different user
        let other_token = StoredToken {
            jti: "other-token".to_string(),
            user_id: "user-002".to_string(),
            token_type: "access".to_string(),
            expires_at: Utc::now() + Duration::hours(1),
            created_at: Utc::now(),
            revoked: false,
            last_used: None,
        };
        store.store_token(other_token).await.unwrap();

        // Revoke all tokens for user-001
        let revoked_count = store.revoke_user_tokens("user-001").await.unwrap();
        assert_eq!(revoked_count, 3);

        // Check that user-001 tokens are revoked but user-002 token is not
        assert!(store.is_revoked("token-1").await);
        assert!(store.is_revoked("token-2").await);
        assert!(store.is_revoked("token-3").await);
        assert!(!store.is_revoked("other-token").await);
    }

    #[test]
    fn test_jti_extraction() {
        // This would need a valid JWT token to test properly
        // For now, test error cases
        assert!(utils::extract_jti("invalid-token").is_err());
        assert!(utils::extract_jti("invalid.token").is_err());
        assert!(utils::extract_jti("").is_err());
    }
}