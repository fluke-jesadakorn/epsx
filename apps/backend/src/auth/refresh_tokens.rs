/*!
 * Refresh Token Service with Rotation
 * 
 * Implements secure refresh token rotation following OAuth 2.0 best practices.
 * Each refresh token use generates a new token and revokes the old one.
 */

use std::collections::HashMap;
use chrono::{DateTime, Utc, Duration};
use uuid::Uuid;
use std::sync::Arc;
use tokio::sync::RwLock;

use serde::{Serialize, Deserialize};

use sha2::{Sha256, Digest};

use base64::{Engine as _, engine::general_purpose};


/// Refresh token data stored in the service
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefreshTokenData {
    pub token_id: String,           // Unique token identifier
    pub user_id: String,            // User this token belongs to
    pub client_id: String,          // Client that owns this token
    pub scope: String,              // Granted scopes
    pub created_at: DateTime<Utc>,  // When token was created
    pub expires_at: DateTime<Utc>,  // When token expires
    pub used_at: Option<DateTime<Utc>>, // When token was last used (for rotation)
    pub device_info: Option<String>, // Device information for tracking
    pub revoked: bool,              // Whether token has been revoked
    pub parent_token_id: Option<String>, // Previous token in rotation chain
    pub rotation_count: u32,        // Number of times this token has been rotated
}

/// Refresh token service managing token lifecycle and rotation
pub struct RefreshTokenService {
    tokens: Arc<RwLock<HashMap<String, RefreshTokenData>>>,
    max_rotation_count: u32,
    token_lifetime: Duration,
}

impl RefreshTokenService {
    /// Create a new refresh token service
    pub fn new() -> Self {
        Self {
            tokens: Arc::new(RwLock::new(HashMap::new())),
            max_rotation_count: 10, // Prevent infinite rotation chains
            token_lifetime: Duration::days(30), // 30 days for refresh tokens
        }
    }

    /// Generate a new refresh token
    pub async fn create_refresh_token(
        &self,
        user_id: &str,
        client_id: &str,
        scope: &str,
        device_info: Option<String>,
    ) -> Result<String, RefreshTokenError> {
        let token_id = Uuid::new_v4().to_string();
        let token_value = self.generate_secure_token(&token_id, user_id);
        
        let token_data = RefreshTokenData {
            token_id: token_id.clone(),
            user_id: user_id.to_string(),
            client_id: client_id.to_string(),
            scope: scope.to_string(),
            created_at: Utc::now(),
            expires_at: Utc::now() + self.token_lifetime,
            used_at: None,
            device_info,
            revoked: false,
            parent_token_id: None,
            rotation_count: 0,
        };

        let mut tokens = self.tokens.write().await;
        let expires_at = token_data.expires_at;
        tokens.insert(token_value.clone(), token_data);
        
        tracing::info!(
            token_id = %token_id,
            user_id = %user_id,
            client_id = %client_id,
            expires_at = %expires_at,
            "Refresh token created"
        );

        Ok(token_value)
    }

    /// Rotate a refresh token (use current token to generate new one)
    pub async fn rotate_refresh_token(
        &self,
        current_token: &str,
        device_info: Option<String>,
    ) -> Result<RefreshTokenRotation, RefreshTokenError> {
        let mut tokens = self.tokens.write().await;
        
        // Get current token data
        let current_data = tokens.get_mut(current_token)
            .ok_or_else(|| RefreshTokenError::TokenNotFound {
                token: "hidden".to_string()
            })?;

        // Check if token is already revoked
        if current_data.revoked {
            return Err(RefreshTokenError::TokenRevoked {
                token_id: current_data.token_id.clone()
            });
        }

        // Check if token has expired
        if Utc::now() > current_data.expires_at {
            return Err(RefreshTokenError::TokenExpired {
                token_id: current_data.token_id.clone(),
                expired_at: current_data.expires_at,
            });
        }

        // Check rotation limit
        if current_data.rotation_count >= self.max_rotation_count {
            tracing::warn!(
                token_id = %current_data.token_id,
                rotation_count = %current_data.rotation_count,
                max_count = %self.max_rotation_count,
                "Refresh token rotation limit exceeded"
            );
            
            return Err(RefreshTokenError::RotationLimitExceeded {
                token_id: current_data.token_id.clone(),
                count: current_data.rotation_count,
            });
        }

        // Extract values we need before mutating current_data
        let old_token_id = current_data.token_id.clone();
        let user_id = current_data.user_id.clone();
        let client_id = current_data.client_id.clone();
        let scope = current_data.scope.clone();
        let device_info_clone = current_data.device_info.clone();
        let rotation_count = current_data.rotation_count;

        // Mark current token as used and revoked (part of rotation)
        current_data.used_at = Some(Utc::now());
        current_data.revoked = true;

        // Create new token
        let new_token_id = Uuid::new_v4().to_string();
        let new_token_value = self.generate_secure_token(&new_token_id, &user_id);
        
        let new_token_data = RefreshTokenData {
            token_id: new_token_id.clone(),
            user_id: user_id.clone(),
            client_id: client_id.clone(),
            scope: scope.clone(),
            created_at: Utc::now(),
            expires_at: Utc::now() + self.token_lifetime,
            used_at: None,
            device_info: device_info.or_else(|| device_info_clone),
            revoked: false,
            parent_token_id: Some(old_token_id.clone()),
            rotation_count: rotation_count + 1,
        };

        // Store new token
        let new_token_data_clone = new_token_data.clone();
        tokens.insert(new_token_value.clone(), new_token_data_clone);

        tracing::info!(
            old_token_id = %old_token_id,
            new_token_id = %new_token_id,
            user_id = %user_id,
            rotation_count = %new_token_data.rotation_count,
            "Refresh token rotated successfully"
        );

        Ok(RefreshTokenRotation {
            new_token: new_token_value,
            new_token_data,
            old_token_id,
        })
    }

    /// Validate a refresh token
    pub async fn validate_refresh_token(&self, token: &str) -> Result<RefreshTokenData, RefreshTokenError> {
        let tokens = self.tokens.read().await;
        
        let token_data = tokens.get(token)
            .ok_or_else(|| RefreshTokenError::TokenNotFound {
                token: "hidden".to_string()
            })?;

        // Check if revoked
        if token_data.revoked {
            return Err(RefreshTokenError::TokenRevoked {
                token_id: token_data.token_id.clone()
            });
        }

        // Check if expired
        if Utc::now() > token_data.expires_at {
            return Err(RefreshTokenError::TokenExpired {
                token_id: token_data.token_id.clone(),
                expired_at: token_data.expires_at,
            });
        }

        Ok(token_data.clone())
    }

    /// Revoke a refresh token and its entire rotation chain
    pub async fn revoke_refresh_token(
        &self,
        token: &str,
        revoked_by: &str,
        reason: &str,
    ) -> Result<Vec<String>, RefreshTokenError> {
        let mut tokens = self.tokens.write().await;
        let mut revoked_token_ids = Vec::new();

        // Find the token to revoke
        if let Some(token_data) = tokens.get_mut(token) {
            if !token_data.revoked {
                token_data.revoked = true;
                revoked_token_ids.push(token_data.token_id.clone());
                
                tracing::info!(
                    token_id = %token_data.token_id,
                    user_id = %token_data.user_id,
                    revoked_by = %revoked_by,
                    reason = %reason,
                    "Refresh token revoked"
                );
            }

            // Also revoke all tokens in the same rotation chain
            let user_id = token_data.user_id.clone();
            let client_id = token_data.client_id.clone();
            
            // Revoke all tokens for this user/client combination to be safe
            for (_, other_token) in tokens.iter_mut() {
                if other_token.user_id == user_id 
                    && other_token.client_id == client_id 
                    && !other_token.revoked 
                {
                    other_token.revoked = true;
                    revoked_token_ids.push(other_token.token_id.clone());
                }
            }
        }

        Ok(revoked_token_ids)
    }

    /// Revoke all refresh tokens for a user
    pub async fn revoke_all_user_tokens(&self, user_id: &str) -> Result<Vec<String>, RefreshTokenError> {
        let mut tokens = self.tokens.write().await;
        let mut revoked_token_ids = Vec::new();

        for (_, token_data) in tokens.iter_mut() {
            if token_data.user_id == user_id && !token_data.revoked {
                token_data.revoked = true;
                revoked_token_ids.push(token_data.token_id.clone());
            }
        }

        tracing::info!(
            user_id = %user_id,
            count = %revoked_token_ids.len(),
            "All user refresh tokens revoked"
        );

        Ok(revoked_token_ids)
    }

    /// Clean up expired tokens (maintenance task)
    pub async fn cleanup_expired_tokens(&self) -> u32 {
        let mut tokens = self.tokens.write().await;
        let now = Utc::now();
        let initial_count = tokens.len();

        tokens.retain(|_, token_data| {
            token_data.expires_at > now
        });

        let cleaned_count = initial_count - tokens.len();
        
        if cleaned_count > 0 {
            tracing::info!(
                cleaned = %cleaned_count,
                remaining = %tokens.len(),
                "Cleaned up expired refresh tokens"
            );
        }

        cleaned_count as u32
    }

    /// Get refresh token statistics
    pub async fn get_stats(&self) -> RefreshTokenStats {
        let tokens = self.tokens.read().await;
        let now = Utc::now();
        
        let mut total_tokens = 0;
        let mut active_tokens = 0;
        let mut expired_tokens = 0;
        let mut revoked_tokens = 0;
        let mut users: std::collections::HashSet<String> = std::collections::HashSet::new();

        for token_data in tokens.values() {
            total_tokens += 1;
            users.insert(token_data.user_id.clone());

            if token_data.revoked {
                revoked_tokens += 1;
            } else if token_data.expires_at <= now {
                expired_tokens += 1;
            } else {
                active_tokens += 1;
            }
        }

        RefreshTokenStats {
            total_tokens,
            active_tokens,
            expired_tokens,
            revoked_tokens,
            unique_users: users.len() as u32,
        }
    }

    /// Generate a cryptographically secure refresh token
    fn generate_secure_token(&self, token_id: &str, user_id: &str) -> String {
        // Create a secure random component
        let random_bytes = (0..32)
            .map(|_| rand::random::<u8>())
            .collect::<Vec<u8>>();
        
        // Create deterministic component based on token_id and user_id
        let mut hasher = Sha256::new();
        hasher.update(token_id.as_bytes());
        hasher.update(user_id.as_bytes());
        hasher.update(&random_bytes);
        let hash = hasher.finalize();
        
        // Combine random and deterministic parts
        let mut token_bytes = Vec::new();
        token_bytes.extend_from_slice(&random_bytes);
        token_bytes.extend_from_slice(&hash[..16]); // Use first 16 bytes of hash
        
        // Encode as base64url (URL-safe base64)
        general_purpose::URL_SAFE_NO_PAD.encode(&token_bytes)
    }
}

impl Default for RefreshTokenService {
    fn default() -> Self {
        Self::new()
    }
}

// Global service instance
lazy_static::lazy_static! {
    pub static ref REFRESH_TOKEN_SERVICE: RefreshTokenService = RefreshTokenService::new();
}

/// Result of a token rotation operation
#[derive(Debug, Clone)]
pub struct RefreshTokenRotation {
    pub new_token: String,
    pub new_token_data: RefreshTokenData,
    pub old_token_id: String,
}

/// Refresh token error types
#[derive(Debug, thiserror::Error)]
pub enum RefreshTokenError {
    #[error("Refresh token not found")]
    TokenNotFound { token: String },
    
    #[error("Refresh token has been revoked: {token_id}")]
    TokenRevoked { token_id: String },
    
    #[error("Refresh token expired at {expired_at}: {token_id}")]
    TokenExpired { 
        token_id: String,
        expired_at: DateTime<Utc>,
    },
    
    #[error("Token rotation limit exceeded for {token_id}: {count} rotations")]
    RotationLimitExceeded { 
        token_id: String,
        count: u32,
    },
    
    #[error("Storage error: {message}")]
    StorageError { message: String },
}

/// Statistics about refresh tokens
#[derive(Debug, Serialize, Deserialize)]
pub struct RefreshTokenStats {
    pub total_tokens: u32,
    pub active_tokens: u32,
    pub expired_tokens: u32,
    pub revoked_tokens: u32,
    pub unique_users: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_refresh_token_creation() {
        let service = RefreshTokenService::new();
        
        let token = service.create_refresh_token(
            "user123", 
            "client456", 
            "openid profile", 
            Some("iPhone 14".to_string())
        ).await.unwrap();
        
        assert!(!token.is_empty());
        
        // Validate the token
        let token_data = service.validate_refresh_token(&token).await.unwrap();
        assert_eq!(token_data.user_id, "user123");
        assert_eq!(token_data.client_id, "client456");
        assert!(!token_data.revoked);
    }

    #[tokio::test]
    async fn test_refresh_token_rotation() {
        let service = RefreshTokenService::new();
        
        let original_token = service.create_refresh_token(
            "user123", 
            "client456", 
            "openid profile", 
            None
        ).await.unwrap();
        
        // Rotate the token
        let rotation = service.rotate_refresh_token(&original_token, None).await.unwrap();
        
        assert_ne!(rotation.new_token, original_token);
        assert_eq!(rotation.new_token_data.rotation_count, 1);
        
        // Original token should be revoked
        assert!(service.validate_refresh_token(&original_token).await.is_err());
        
        // New token should be valid
        assert!(service.validate_refresh_token(&rotation.new_token).await.is_ok());
    }

    #[tokio::test]
    async fn test_token_revocation() {
        let service = RefreshTokenService::new();
        
        let token = service.create_refresh_token(
            "user123", 
            "client456", 
            "openid profile", 
            None
        ).await.unwrap();
        
        // Token should be valid initially
        assert!(service.validate_refresh_token(&token).await.is_ok());
        
        // Revoke the token
        let revoked_ids = service.revoke_refresh_token(&token, "admin", "security").await.unwrap();
        assert!(!revoked_ids.is_empty());
        
        // Token should now be invalid
        assert!(service.validate_refresh_token(&token).await.is_err());
    }
}