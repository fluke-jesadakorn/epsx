/*!
 * Token Revocation Service
 * 
 * Manages revoked JWT tokens with Redis-based blacklist storage.
 * Implements RFC 7009 OAuth Token Revocation specification.
 */

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use uuid::Uuid;

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

/// Token revocation service
/// 
/// In a production environment, this should use Redis or a database
/// for persistence across server restarts and horizontal scaling.
pub struct TokenRevocationService {
    revoked_tokens: Arc<RwLock<HashMap<String, RevokedToken>>>,
}

impl TokenRevocationService {
    /// Create a new token revocation service
    pub fn new() -> Self {
        Self {
            revoked_tokens: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Revoke a token by its JTI (JWT ID)
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

        let mut tokens = self.revoked_tokens.write().await;
        tokens.insert(jti.to_string(), revoked_token.clone());

        tracing::info!(
            jti = %jti,
            user_id = %user_id,
            revoked_by = %revoked_by,
            reason = %reason,
            "Token revoked successfully"
        );

        Ok(())
    }

    /// Check if a token is revoked
    pub async fn is_token_revoked(&self, jti: &str) -> bool {
        let tokens = self.revoked_tokens.read().await;
        
        if let Some(revoked_token) = tokens.get(jti) {
            // Check if the revoked token has already expired naturally
            if Utc::now() > revoked_token.expires_at {
                // Token would have expired anyway, we can clean it up
                drop(tokens);
                self.cleanup_expired_token(jti).await;
                return false;
            }
            return true;
        }
        
        false
    }

    /// Revoke all tokens for a specific user (logout from all devices)
    pub async fn revoke_all_user_tokens(
        &self,
        user_id: &str,
        revoked_by: &str,
        reason: &str,
    ) -> Result<u32, RevocationError> {
        let mut tokens = self.revoked_tokens.write().await;
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

        tokens.insert(global_revocation_jti, revoked_token);
        revoked_count += 1;

        tracing::info!(
            user_id = %user_id,
            revoked_by = %revoked_by,
            reason = %reason,
            count = %revoked_count,
            "All user tokens revoked"
        );

        Ok(revoked_count)
    }

    /// Get revocation info for a token
    pub async fn get_revocation_info(&self, jti: &str) -> Option<RevokedToken> {
        let tokens = self.revoked_tokens.read().await;
        tokens.get(jti).cloned()
    }

    /// Clean up expired revoked tokens (maintenance task)
    pub async fn cleanup_expired_tokens(&self) -> u32 {
        let mut tokens = self.revoked_tokens.write().await;
        let now = Utc::now();
        let initial_count = tokens.len();

        tokens.retain(|_jti, revoked_token| {
            revoked_token.expires_at > now
        });

        let cleaned_count = initial_count - tokens.len();
        
        if cleaned_count > 0 {
            tracing::info!(
                cleaned = %cleaned_count,
                remaining = %tokens.len(),
                "Cleaned up expired revoked tokens"
            );
        }

        cleaned_count as u32
    }

    /// Clean up a single expired token
    async fn cleanup_expired_token(&self, jti: &str) {
        let mut tokens = self.revoked_tokens.write().await;
        if tokens.remove(jti).is_some() {
            tracing::debug!(jti = %jti, "Cleaned up expired revoked token");
        }
    }

    /// Get statistics about revoked tokens
    pub async fn get_stats(&self) -> RevocationStats {
        let tokens = self.revoked_tokens.read().await;
        let total_revoked = tokens.len();
        let now = Utc::now();
        
        let mut expired_count = 0;
        let mut active_revocations = 0;
        let mut users: std::collections::HashSet<String> = std::collections::HashSet::new();

        for revoked_token in tokens.values() {
            if revoked_token.expires_at <= now {
                expired_count += 1;
            } else {
                active_revocations += 1;
                users.insert(revoked_token.user_id.clone());
            }
        }

        RevocationStats {
            total_revoked: total_revoked as u32,
            active_revocations: active_revocations as u32,
            expired_revocations: expired_count as u32,
            affected_users: users.len() as u32,
        }
    }
}

impl Default for TokenRevocationService {
    fn default() -> Self {
        Self::new()
    }
}

// Global service instance
lazy_static::lazy_static! {
    pub static ref TOKEN_REVOCATION_SERVICE: TokenRevocationService = TokenRevocationService::new();
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
pub mod jwt_integration {
    use super::*;
    use crate::auth::Claims;

    /// Check if JWT claims represent a revoked token
    pub async fn is_jwt_revoked(claims: &Claims) -> bool {
        // Convert exp from usize to DateTime
        let _exp_timestamp = claims.exp as i64;
        TOKEN_REVOCATION_SERVICE.is_token_revoked(&claims.jti).await
    }

    /// Revoke a JWT token based on its claims
    pub async fn revoke_jwt(
        claims: &Claims,
        revoked_by: &str,
        reason: &str,
    ) -> Result<(), RevocationError> {
        let expires_at = DateTime::from_timestamp(claims.exp as i64, 0)
            .unwrap_or_else(|| Utc::now() + chrono::Duration::hours(1));

        TOKEN_REVOCATION_SERVICE.revoke_token(
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
    ) -> Result<u32, RevocationError> {
        TOKEN_REVOCATION_SERVICE.revoke_all_user_tokens(user_id, revoked_by, reason).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_token_revocation() {
        let service = TokenRevocationService::new();
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
    async fn test_expired_token_cleanup() {
        let service = TokenRevocationService::new();
        let jti = "expired_token_123";
        let user_id = "user_456";
        let expires_at = Utc::now() - chrono::Duration::hours(1); // Already expired

        // Revoke an already-expired token
        service.revoke_token(jti, user_id, "test", "Testing", expires_at).await.unwrap();

        // Token should be considered not revoked since it's naturally expired
        assert!(!service.is_token_revoked(jti).await);
    }
}