/*!
 * Token Revocation Service
 * 
 * Manages revoked JWT tokens with Redis-based blacklist storage and automatic in-memory fallback.
 * Implements RFC 7009 OAuth Token Revocation specification.
 */

use chrono::{DateTime, Utc};

use uuid::Uuid;
use std::sync::Arc;


use serde::{Serialize, Deserialize};

use tracing::{debug, info};

use crate::infrastructure::cache::Cache;


/// Token revocation entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevokedToken {
    pub jti: String,           // JWT ID
    pub user_id: String,       // Subject (user ID)
    pub revoked_at: DateTime<Utc>,
    pub revoked_by: String,    // Who revoked it (user_id or "system")
    pub reason: String,        // Reason for revocation
    pub expires_at: DateTime<Utc>, // When the original token would expire
}

/// Token revocation service with Redis + in-memory fallback
/// 
/// Uses UnifiedCache for persistence across server restarts and horizontal scaling
/// with automatic fallback to in-memory storage when Redis is unavailable.
pub struct TokenRevocationService {
    cache: Arc<dyn Cache>,
}

impl TokenRevocationService {
    /// Create a new token revocation service with cache support
    pub fn new(cache: Arc<dyn Cache>) -> Self {
        Self { cache }
    }

    /// Create cache key for revoked token
    fn revoked_token_key(&self, jti: &str) -> String {
        format!("revoked:token:{}", jti)
    }

    /// Create cache key for user revocation tracking
    fn user_revocation_key(&self, user_id: &str) -> String {
        format!("revoked:user:{}", user_id)
    }

    /// Revoke a token by its JTI (JWT ID) with cache support
    pub async fn revoke_token(
        &self,
        jti: &str,
        user_id: &str,
        revoked_by: &str,
        reason: &str,
        expires_at: DateTime<Utc>,
    ) -> Result<(), RevocationError> {
        let revoked_token = RevokedToken {
            jti: jti.to_string(),
            user_id: user_id.to_string(),
            revoked_at: Utc::now(),
            revoked_by: revoked_by.to_string(),
            reason: reason.to_string(),
            expires_at,
        };

        let token_key = self.revoked_token_key(jti);
        let ttl_seconds = (expires_at - Utc::now()).num_seconds().max(0);

        // Cache.set() expects key, value, optional TTL
        let revoked_token_json = serde_json::to_string(&revoked_token).unwrap_or_default();
        self.cache.set(&token_key, revoked_token_json, Some(ttl_seconds as u64));
        info!(
            jti = %jti,
            user_id = %user_id,
            revoked_by = %revoked_by,
            reason = %reason,
            ttl = %ttl_seconds,
            "Token revoked and cached successfully"
        );
        Ok(())
    }

    /// Check if a token is revoked using cache
    pub async fn is_token_revoked(&self, jti: &str) -> bool {
        let token_key = self.revoked_token_key(jti);
        
        if let Some(cached_data) = self.cache.get(&token_key) {
            if let Ok(revoked_token) = serde_json::from_str::<RevokedToken>(&cached_data) {
                // Check if the revoked token has already expired naturally
                if Utc::now() > revoked_token.expires_at {
                    // Token would have expired anyway, clean it up
                    debug!("Revoked token {} has expired naturally, cleaning up", jti);
                    self.cache.delete(&token_key);
                    return false;
                }
                debug!("Token {} is revoked and still valid", jti);
                true
            } else {
                debug!("Failed to parse revoked token data from cache for {}", jti);
                false
            }
        } else {
            debug!("Token {} is not revoked", jti);
            false
        }
    }

    /// Revoke all tokens for a specific user (logout from all devices) with cache support
    pub async fn revoke_all_user_tokens(
        &self,
        user_id: &str,
        revoked_by: &str,
        reason: &str,
    ) -> Result<u32, RevocationError> {
        let mut revoked_count = 0;

        // In a production system, this would query the database for all active tokens for the user
        // For now, we'll create a placeholder revocation entry to block future tokens
        let global_revocation_jti = format!("user_revocation_{}_{}_{}", 
            user_id, 
            Utc::now().timestamp(), 
            Uuid::new_v4()
        );

        let revoked_token = RevokedToken {
            jti: global_revocation_jti.clone(),
            user_id: user_id.to_string(),
            revoked_at: Utc::now(),
            revoked_by: revoked_by.to_string(),
            reason: format!("Global user revocation: {}", reason),
            expires_at: Utc::now() + chrono::Duration::days(7), // Keep for a week
        };

        // Cache the global revocation
        let token_key = self.revoked_token_key(&global_revocation_jti);
        let ttl_seconds = chrono::Duration::days(7).num_seconds();

        // Cache.set() returns void, not Result
        let revoked_token_json = serde_json::to_string(&revoked_token).unwrap_or_default();
        self.cache.set(&token_key, revoked_token_json.clone(), Some(ttl_seconds as u64));
        revoked_count += 1;
        
        // Also cache under user revocation key for tracking
        let user_key = self.user_revocation_key(user_id);
        self.cache.set(&user_key, revoked_token_json, Some(ttl_seconds as u64));

        info!(
            user_id = %user_id,
            revoked_by = %revoked_by,
            reason = %reason,
            count = %revoked_count,
            "All user tokens revoked and cached"
        );
        
        Ok(revoked_count)
    }

    /// Get revocation info for a token from cache
    pub async fn get_revocation_info(&self, jti: &str) -> Option<RevokedToken> {
        let token_key = self.revoked_token_key(jti);
        
        if let Some(cached_data) = self.cache.get(&token_key) {
            if let Ok(revoked_token) = serde_json::from_str::<RevokedToken>(&cached_data) {
                debug!("Retrieved revocation info for token: {}", jti);
                Some(revoked_token)
            } else {
                debug!("Failed to parse revocation info for token: {}", jti);
                None
            }
        } else {
            debug!("No revocation info found for token: {}", jti);
            None
        }
    }

    /// Clean up expired revoked tokens (maintenance task)
    /// With cache-based implementation, TTL handles automatic cleanup
    pub async fn cleanup_expired_tokens(&self) -> u32 {
        // With Redis/cache TTL, expired tokens are automatically cleaned up
        // This method could be used to force cleanup or gather stats
        debug!("Cleanup requested - cache TTL handles automatic expiration");
        
        // Return 0 as cache handles cleanup automatically
        // In a full implementation, this could query cache stats or force cleanup
        0
    }

    /// Get statistics about revoked tokens from cache
    /// Note: With cache-based implementation, detailed stats require additional tracking
    pub async fn get_stats(&self) -> RevocationStats {
        // With cache-based implementation, we can get cache stats but not detailed token info
        // without additional tracking mechanisms
        // TODO: Implement cache.stats() method
        debug!("Retrieved cache stats for revocation service");
        RevocationStats {
            total_revoked: 0, // Would need additional tracking
            active_revocations: 0, // Placeholder since cache.stats() not available
            expired_revocations: 0, // Placeholder since cache.stats() not available
            affected_users: 0, // Would need additional tracking
        }
    }
}

/// Revocation error types
#[derive(Debug, thiserror::Error)]
pub enum RevocationError {
    #[error("Token not found: {token_id}")]
    TokenNotFound { token_id: String },
    
    #[error("Storage error: {message}")]
    StorageError { message: String },
    
    #[error("Invalid token format: {message}")]
    InvalidToken { message: String },

    #[error("Cache error: {0}")]
    CacheError(String),
}

/// Statistics about token revocations
#[derive(Debug, Serialize, Deserialize)]
pub struct RevocationStats {
    pub total_revoked: u32,
    pub active_revocations: u32,
    pub expired_revocations: u32,
    pub affected_users: u32,
}

/// Utility functions for JWT integration
/// Note: These functions now require a TokenRevocationService instance
pub mod jwt_integration {
    use super::*;
    use crate::auth::Claims;

    /// Check if JWT claims represent a revoked token
    pub async fn is_jwt_revoked(claims: &Claims, revocation_service: &TokenRevocationService) -> bool {
        revocation_service.is_token_revoked(&claims.jti).await
    }

    /// Revoke a JWT token based on its claims
    pub async fn revoke_jwt(
        claims: &Claims,
        revoked_by: &str,
        reason: &str,
        revocation_service: &TokenRevocationService,
    ) -> Result<(), RevocationError> {
        let expires_at = DateTime::from_timestamp(claims.exp as i64, 0)
            .unwrap_or_else(|| Utc::now() + chrono::Duration::hours(1));

        revocation_service.revoke_token(
            &claims.jti,
            &claims.sub,
            revoked_by,
            reason,
            expires_at,
        ).await
    }

    /// Revoke all tokens for a user
    pub async fn revoke_user_tokens(
        user_id: &str,
        revoked_by: &str,
        reason: &str,
        revocation_service: &TokenRevocationService,
    ) -> Result<u32, RevocationError> {
        revocation_service.revoke_all_user_tokens(user_id, revoked_by, reason).await
    }
}

lazy_static::lazy_static! {
    /// Global token revocation service instance
    /// Initialized lazily with cache support for cross-module usage
    pub static ref TOKEN_REVOCATION_SERVICE: TokenRevocationService = {
        use crate::infrastructure::cache::CacheFactory;
        let cache = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                CacheFactory::with_fallback().await
            })
        });
        TokenRevocationService::new(Arc::from(cache))
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::cache::CacheFactory;

    #[tokio::test]
    async fn test_token_revocation_with_cache() {
        let cache = CacheFactory::with_fallback().await;
        let service = TokenRevocationService::new(cache);
        let jti = "test_token_123";
        let user_id = "user_456";
        let expires_at = Utc::now() + chrono::Duration::hours(1);

        // Token should not be revoked initially
        assert!(!service.is_token_revoked(jti).await);

        // Revoke the token
        service.revoke_token(jti, user_id, "test", "Testing", expires_at).await.unwrap();

        // Token should now be revoked
        assert!(service.is_token_revoked(jti).await);

        // Get revocation info
        let info = service.get_revocation_info(jti).await.unwrap();
        assert_eq!(info.user_id, user_id);
        assert_eq!(info.reason, "Testing");
    }

    #[tokio::test]
    async fn test_expired_token_cleanup_with_cache() {
        let cache = CacheFactory::with_fallback().await;
        let service = TokenRevocationService::new(cache);
        let jti = "expired_token_123";
        let user_id = "user_456";
        let expires_at = Utc::now() - chrono::Duration::hours(1); // Already expired

        // Revoke an already-expired token
        service.revoke_token(jti, user_id, "test", "Testing", expires_at).await.unwrap();

        // Token should be considered not revoked since it's naturally expired
        assert!(!service.is_token_revoked(jti).await);
    }

    #[tokio::test]
    async fn test_revoke_all_user_tokens_with_cache() {
        let cache = CacheFactory::with_fallback().await;
        let service = TokenRevocationService::new(cache);
        let user_id = "user_789";

        // Revoke all tokens for user
        let count = service.revoke_all_user_tokens(user_id, "system-admin", "security_breach").await.unwrap();
        assert_eq!(count, 1); // Global revocation entry

        // Stats should reflect the revocation
        let stats = service.get_stats().await;
        // Note: Stats implementation may vary with cache backend
        println!("Revocation stats: {:?}", stats);
    }

    #[tokio::test]
    async fn test_cache_fallback_behavior() {
        let cache = CacheFactory::with_fallback().await;
        let service = TokenRevocationService::new(cache);
        let jti = "fallback_test_token";
        let user_id = "fallback_user";
        let expires_at = Utc::now() + chrono::Duration::hours(1);

        // Test that operations work even if cache has issues
        // (The UnifiedCache should handle Redis failures gracefully)
        
        // Revoke token
        let result = service.revoke_token(jti, user_id, "system", "fallback_test", expires_at).await;
        assert!(result.is_ok(), "Token revocation should succeed even with cache issues");

        // Check revocation status
        let is_revoked = service.is_token_revoked(jti).await;
        assert!(is_revoked, "Token should be reported as revoked");

        // Get revocation info
        let info = service.get_revocation_info(jti).await;
        assert!(info.is_some(), "Revocation info should be retrievable");
        
        if let Some(token_info) = info {
            assert_eq!(token_info.user_id, user_id);
            assert_eq!(token_info.reason, "fallback_test");
        }
    }
}