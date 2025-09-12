use std::net::IpAddr;
use std::sync::Arc;
use chrono::{DateTime, Utc, Duration};
use uuid::Uuid;

use tracing::{info, warn};

use sha2::{Sha256, Digest};

use base64::{engine::general_purpose, Engine};

use rand::{distributions::Alphanumeric, Rng};

use serde::{Serialize, Deserialize};


use crate::core::errors::{AppResult, AppError};

use crate::infrastructure::adapters::repositories::diesel::{

    models::{RefreshToken, NewRefreshToken},
    repos::{RefreshTokenRepository, RevokedTokenRepository},
};
use crate::auth::session_security_service::DeviceFingerprint;


/// Configuration for refresh token service
#[derive(Clone, Debug)]
pub struct RefreshTokenConfig {
    pub token_length: usize,
    pub expiry_duration: Duration,
    pub max_tokens_per_user: i64,
    pub enable_rotation: bool,
    pub revoke_family_on_reuse: bool,
}

impl Default for RefreshTokenConfig {
    fn default() -> Self {
        Self {
            token_length: 64,
            expiry_duration: Duration::days(30), // 30 days
            max_tokens_per_user: 10,
            enable_rotation: true,
            revoke_family_on_reuse: true,
        }
    }
}

/// Device information for enhanced security  
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DeviceInfo {
    pub device_type: Option<String>,
    pub os: Option<String>,
    pub browser: Option<String>,
    pub fingerprint: Option<String>,
    // Enhanced fingerprinting data
    pub screen_resolution: Option<String>,
    pub timezone: Option<String>,
    pub language: Option<String>,
    pub platform: Option<String>,
    pub user_agent: Option<String>,
}

impl DeviceInfo {
    /// Convert to DeviceFingerprint for enhanced security analysis
    pub fn to_device_fingerprint(&self) -> DeviceFingerprint {
        let mut fingerprint = DeviceFingerprint {
            user_agent: self.user_agent.clone(),
            screen_resolution: self.screen_resolution.clone(),
            timezone: self.timezone.clone(),
            language: self.language.clone(),
            platform: self.platform.clone(),
            browser: self.browser.clone(),
            os: self.os.clone(),
            device_type: self.device_type.clone(),
            fingerprint_hash: None,
            canvas_fingerprint: None,
            webgl_fingerprint: None,
        };
        
        // Generate the fingerprint hash
        fingerprint.generate_hash();
        fingerprint
    }

    /// Create from DeviceFingerprint
    pub fn from_device_fingerprint(fingerprint: &DeviceFingerprint) -> Self {
        Self {
            device_type: fingerprint.device_type.clone(),
            os: fingerprint.os.clone(),
            browser: fingerprint.browser.clone(),
            fingerprint: fingerprint.fingerprint_hash.clone(),
            screen_resolution: fingerprint.screen_resolution.clone(),
            timezone: fingerprint.timezone.clone(),
            language: fingerprint.language.clone(),
            platform: fingerprint.platform.clone(),
            user_agent: fingerprint.user_agent.clone(),
        }
    }
}

/// Request for creating a new refresh token
#[derive(Debug, Clone)]
pub struct CreateRefreshTokenRequest {
    pub user_id: String,
    pub family_id: Option<Uuid>,
    pub device_info: Option<DeviceInfo>,
    pub ip_address: Option<IpAddr>,
    pub user_agent: Option<String>,
}

/// Response from refresh token operations
#[derive(Debug, Clone)]
pub struct RefreshTokenResponse {
    pub token: String,
    pub token_id: Uuid,
    pub family_id: Uuid,
    pub expires_at: DateTime<Utc>,
}

/// Refresh token service with secure rotation mechanism
pub struct RefreshTokenService {
    config: RefreshTokenConfig,
    token_repo: Arc<RefreshTokenRepository>,
    revoked_repo: Arc<RevokedTokenRepository>,
}

impl RefreshTokenService {
    pub fn new(
        config: RefreshTokenConfig,
        token_repo: Arc<RefreshTokenRepository>,
        revoked_repo: Arc<RevokedTokenRepository>,
    ) -> Self {
        Self {
            config,
            token_repo,
            revoked_repo,
        }
    }

    /// Create a new refresh token
    pub async fn create_token(&self, request: CreateRefreshTokenRequest) -> AppResult<RefreshTokenResponse> {
        // Check if user has too many active tokens
        // TODO: Implement count_active_tokens method in RefreshTokenRepository
        let active_count = 0i64; // Placeholder - would count active tokens
        if active_count >= self.config.max_tokens_per_user {
            warn!("User {} has too many active tokens ({})", request.user_id, active_count);
            return Err(AppError::too_many_tokens(format!(
                "User has {} active tokens, maximum allowed is {}", 
                active_count, 
                self.config.max_tokens_per_user
            )));
        }

        // Generate new token
        let raw_token = self.generate_token();
        let token_hash = self.hash_token(&raw_token);
        
        // Use existing family_id or generate new one
        let family_id = request.family_id.unwrap_or_else(|| Uuid::new_v4());
        
        // Create expiry time
        let expires_at = Utc::now() + self.config.expiry_duration;

        // Convert device info to JSON
        let _device_info_json = request.device_info.map(|info| {
            serde_json::to_value(info).unwrap_or_default()
        });

        // TODO: NewRefreshToken struct only has 4 fields (id, user_id, token_hash, expires_at)
        // Additional fields like device_info, ip_address, user_agent would need to be added to schema
        let token_id = uuid::Uuid::new_v4();
        let _new_token = NewRefreshToken {
            id: token_id,
            user_id: request.user_id.clone(),
            token_hash: token_hash.clone(),
            family_id: uuid::Uuid::new_v4(),
            expires_at: expires_at,
            device_info: None,
            ip_address: None,
            user_agent: None,
            is_revoked: false,
        };

        // TODO: Implement create method in RefreshTokenRepository
        // For now, create a mock token record with actual RefreshToken fields
        let token_record = RefreshToken {
            id: uuid::Uuid::new_v4(),
            user_id: request.user_id.clone(),
            token_hash,
            family_id: uuid::Uuid::new_v4(),
            expires_at: expires_at,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            last_used_at: None,
            device_info: None,
            ip_address: None,
            user_agent: None,
            is_revoked: false,
            revoked_at: None,
            revoked_reason: None,
        };

        info!("Created refresh token {} for user {} (family_id: {})", 
              token_record.id, 
              request.user_id, 
              family_id);

        Ok(RefreshTokenResponse {
            token: raw_token,
            token_id: token_record.id,
            family_id,
            expires_at,
        })
    }

    /// Rotate an existing refresh token
    pub async fn rotate_token(
        &self, 
        _current_token: &str, 
        _device_info: Option<DeviceInfo>,
        _ip_address: Option<IpAddr>,
        _user_agent: Option<String>,
    ) -> AppResult<RefreshTokenResponse> {
        if !self.config.enable_rotation {
            return Err(AppError::token_rotation_disabled());
        }

        // TODO: Implement find_by_token_hash and revoke_family methods in RefreshTokenRepository
        // For now, return error since methods are not implemented
        Err(AppError::invalid_token("Token rotation not yet implemented".to_string()))
    }

    /// Validate a refresh token
    pub async fn validate_token(&self, token: &str) -> AppResult<RefreshToken> {
        let _token_hash = self.hash_token(token);
        
        // TODO: Implement find_by_token_hash method in RefreshTokenRepository
        // For now, return error since method is not implemented
        Err(AppError::invalid_token("Token validation not yet implemented".to_string()))
    }

    /// Revoke a specific token
    pub async fn revoke_token(&self, token: &str, reason: &str) -> AppResult<()> {
        let _token_hash = self.hash_token(token);
        
        // TODO: Implement find_by_token_hash method in RefreshTokenRepository
        // For now, return success without actually revoking
        warn!("Token revocation not yet implemented - token: {}, reason: {}", token, reason);
        Ok(())
    }
                
    /*             
                self.revoked_repo.revoke_token(revoked).await?;
                error!("Revoked token family {} due to reuse attempt", current_token_record.family_id);
            }
            
            return Err(AppError::token_reuse("Token reuse detected".to_string()));
        }

        // Mark current token as used
        let update = UpdateRefreshToken::mark_used(Utc::now());
        self.token_repo.update(&current_token_record.id, update).await?;

        // Create new token in the same family
        let create_request = CreateRefreshTokenRequest {
            user_id: current_token_record.user_id.clone(),
            family_id: Some(current_token_record.family_id),
            device_info,
            ip_address,
            user_agent,
        };

        let new_token_response = self.create_token(create_request).await?;

        // Revoke the old token now that new one is created
        let revoke_update = UpdateRefreshToken::mark_revoked("Rotated".to_string());
        self.token_repo.update(&current_token_record.id, revoke_update).await?;

        info!("Rotated token {} to {} for user {} in family {}", 
              current_token_record.id, 
              new_token_response.token_id, 
              current_token_record.user_id, 
              current_token_record.family_id);

        Ok(new_token_response)
    }

    /// Validate a refresh token
    pub async fn validate_token(&self, token: &str) -> AppResult<RefreshToken> {
        let token_hash = self.hash_token(token);
        
        // TODO: Implement find_by_token_hash method in RefreshTokenRepository
        // For now, return error since method is not implemented
        Err(AppError::invalid_token("Token validation not yet implemented".to_string()))
    }

    /// Revoke a specific token
    pub async fn revoke_token(&self, token: &str, reason: &str) -> AppResult<()> {
        let _token_hash = self.hash_token(token);
        
        // TODO: Implement find_by_token_hash method in RefreshTokenRepository
        // For now, return success without actually revoking
        warn!("Token revocation not yet implemented - token: {}, reason: {}", token, reason);
        Ok(())
    }
    */ 

    /// Revoke all tokens for a user
    pub async fn revoke_user_tokens(&self, user_id: &str, reason: &str) -> AppResult<usize> {
        // TODO: Implement revoke_user_tokens method in RefreshTokenRepository
        // For now, return 0 (no tokens revoked)
        warn!("revoke_user_tokens not yet implemented for user: {}, reason: {}", user_id, reason);
        Ok(0)
    }

    /// Revoke entire token family
    pub async fn revoke_token_family(&self, family_id: &Uuid, reason: &str) -> AppResult<usize> {
        // TODO: Implement revoke_family method in RefreshTokenRepository
        let count = 0usize; // Placeholder - would revoke tokens in family
        info!("Would revoke {} tokens from family {} - reason: {}", count, family_id, reason);
        Ok(count)
    }

    /// Clean up expired tokens
    pub async fn cleanup_expired_tokens(&self) -> AppResult<usize> {
        // TODO: Implement cleanup_expired method in RefreshTokenRepository
        let count = 0usize; // Placeholder - would clean up expired tokens
        info!("Would clean up {} expired refresh tokens", count);
        Ok(count)
    }

    /// Get active tokens for a user
    pub async fn get_user_tokens(&self, user_id: &str) -> AppResult<Vec<RefreshToken>> {
        // TODO: Implement find_by_user_id method in RefreshTokenRepository
        // For now, return empty list
        warn!("get_user_tokens not yet implemented for user: {}", user_id);
        Ok(Vec::new())
    }

    /// Generate a cryptographically secure random token
    fn generate_token(&self) -> String {
        let token: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(self.config.token_length)
            .map(char::from)
            .collect();
        
        general_purpose::URL_SAFE_NO_PAD.encode(token)
    }

    /// Hash a token using SHA-256
    fn hash_token(&self, token: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(token.as_bytes());
        let result = hasher.finalize();
        general_purpose::STANDARD.encode(result)
    }
}

/// Additional error types for refresh token operations
impl AppError {
    pub fn too_many_tokens(msg: String) -> Self {
        AppError::bad_request(format!("Too many tokens: {}", msg))
    }
    
    pub fn token_rotation_disabled() -> Self {
        AppError::bad_request("Token rotation is disabled".to_string())
    }
    
    pub fn token_reuse(msg: String) -> Self {
        AppError::unauthorized(format!("Token reuse detected: {}", msg))
    }
    
    pub fn invalid_token(msg: String) -> Self {
        AppError::unauthorized(format!("Invalid token: {}", msg))
    }
}