use std::sync::Arc;
use std::net::IpAddr;
use chrono::{DateTime, Utc, Duration};
use uuid::Uuid;

use tracing::{debug, error, info, warn};

use sha2::{Sha256, Digest};

use base64::{engine::general_purpose, Engine};

use rand::{distributions::Alphanumeric, Rng};

use serde::{Serialize, Deserialize};


use crate::core::errors::{AppResult, AppError};

use crate::infra::db::diesel::{

    models::{RefreshToken, NewRefreshToken, UpdateRefreshToken, NewRevokedToken},
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
        let active_count = self.token_repo.count_active_tokens(&request.user_id).await?;
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
        let device_info_json = request.device_info.map(|info| {
            serde_json::to_value(info).unwrap_or_default()
        });

        let new_token = NewRefreshToken::new(
            request.user_id.clone(),
            token_hash,
            family_id,
            expires_at,
            device_info_json,
            request.ip_address,
            request.user_agent,
        );

        let token_record = self.token_repo.create(new_token).await?;

        info!("Created refresh token {} for user {} in family {}", 
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
        current_token: &str, 
        device_info: Option<DeviceInfo>,
        ip_address: Option<IpAddr>,
        user_agent: Option<String>,
    ) -> AppResult<RefreshTokenResponse> {
        if !self.config.enable_rotation {
            return Err(AppError::token_rotation_disabled());
        }

        let token_hash = self.hash_token(current_token);
        
        // Find the current token
        let current_token_record = self.token_repo.find_by_token_hash(&token_hash).await?
            .ok_or_else(|| AppError::invalid_token("Token not found or expired".to_string()))?;

        // Check for token reuse attack
        if current_token_record.is_revoked {
            warn!("Attempt to reuse revoked token {} from family {}", 
                  current_token_record.id, 
                  current_token_record.family_id);
            
            if self.config.revoke_family_on_reuse {
                // Revoke entire token family as security measure
                self.token_repo.revoke_family(
                    &current_token_record.family_id, 
                    "Token reuse detected - security measure"
                ).await?;
                
                // Also add to JTI blacklist
                let revoked = NewRevokedToken::revoke_refresh_token(
                    current_token_record.id.to_string(),
                    current_token_record.user_id.clone(),
                    current_token_record.expires_at,
                    Some("system".to_string()),
                    "Token family revoked due to reuse attempt".to_string(),
                );
                
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
        
        let token_record = self.token_repo.find_by_token_hash(&token_hash).await?
            .ok_or_else(|| AppError::invalid_token("Token not found or expired".to_string()))?;

        if token_record.is_revoked {
            return Err(AppError::invalid_token("Token has been revoked".to_string()));
        }

        if token_record.expires_at < Utc::now() {
            return Err(AppError::invalid_token("Token has expired".to_string()));
        }

        debug!("Validated refresh token {} for user {}", token_record.id, token_record.user_id);
        Ok(token_record)
    }

    /// Revoke a specific token
    pub async fn revoke_token(&self, token: &str, reason: &str) -> AppResult<()> {
        let token_hash = self.hash_token(token);
        
        if let Some(token_record) = self.token_repo.find_by_token_hash(&token_hash).await? {
            let update = UpdateRefreshToken::mark_revoked(reason.to_string());
            self.token_repo.update(&token_record.id, update).await?;

            // Add to JTI blacklist
            let revoked = NewRevokedToken::revoke_refresh_token(
                token_record.id.to_string(),
                token_record.user_id.clone(),
                token_record.expires_at,
                Some("manual".to_string()),
                reason.to_string(),
            );
            
            self.revoked_repo.revoke_token(revoked).await?;
            info!("Revoked token {} for user {} - reason: {}", token_record.id, token_record.user_id, reason);
        }
        
        Ok(())
    }

    /// Revoke all tokens for a user
    pub async fn revoke_user_tokens(&self, user_id: &str, reason: &str) -> AppResult<usize> {
        let count = self.token_repo.revoke_user_tokens(user_id, reason).await?;
        info!("Revoked {} tokens for user {} - reason: {}", count, user_id, reason);
        Ok(count)
    }

    /// Revoke entire token family
    pub async fn revoke_token_family(&self, family_id: &Uuid, reason: &str) -> AppResult<usize> {
        let count = self.token_repo.revoke_family(family_id, reason).await?;
        info!("Revoked {} tokens from family {} - reason: {}", count, family_id, reason);
        Ok(count)
    }

    /// Clean up expired tokens
    pub async fn cleanup_expired_tokens(&self) -> AppResult<usize> {
        let count = self.token_repo.cleanup_expired().await?;
        info!("Cleaned up {} expired refresh tokens", count);
        Ok(count)
    }

    /// Get active tokens for a user
    pub async fn get_user_tokens(&self, user_id: &str) -> AppResult<Vec<RefreshToken>> {
        self.token_repo.find_by_user_id(user_id).await
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