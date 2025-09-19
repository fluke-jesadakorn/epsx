/// Secure JWT Service - RS256 Only Implementation
/// Replaces all HS256 JWT implementations with secure RS256
/// 
/// This service provides a comprehensive, secure JWT implementation that:
/// - Uses only RS256 algorithm (prevents algorithm confusion attacks)
/// - Implements proper key rotation with key IDs
/// - Provides validation against common JWT vulnerabilities
/// - Enforces strict token lifetime limits
/// - Includes comprehensive security logging

use std::sync::Arc;
use jsonwebtoken::{encode, decode, Header, Validation, Algorithm, errors::ErrorKind};
use serde::{Deserialize, Serialize};
use chrono::{Utc, Duration};
use uuid::Uuid;
use tracing::{info, warn, error};

use super::key_manager::KeyManager;
use crate::infrastructure::cache::Cache;

/// Maximum token lifetime (24 hours for security)
const MAX_TOKEN_LIFETIME_HOURS: i64 = 24;

/// Standard JWT claims with comprehensive security fields
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SecureJWTClaims {
    // Standard JWT fields
    pub sub: String,    // Subject (User ID)
    pub iss: String,    // Issuer
    pub aud: String,    // Audience
    pub exp: i64,       // Expiration time
    pub iat: i64,       // Issued at
    pub nbf: i64,       // Not before
    pub jti: String,    // JWT ID (for revocation)
    
    // Custom claims
    pub email: Option<String>,
    pub name: Option<String>,
    pub role: Option<String>,
    pub permissions: Vec<String>,
    pub platform_context: Option<String>,
    pub tier: Option<String>,
    
    // Security fields
    pub token_type: String,     // "access", "refresh", "id"
    pub client_id: String,      // OAuth client ID
    pub scope: Option<String>,  // OAuth scopes
    pub device_fingerprint: Option<String>, // Device binding
}

/// Token creation request
#[derive(Debug, Clone)]
pub struct TokenRequest {
    pub user_id: String,
    pub email: Option<String>,
    pub name: Option<String>,
    pub role: Option<String>,
    pub permissions: Vec<String>,
    pub platform_context: Option<String>,
    pub tier: Option<String>,
    pub token_type: String,
    pub client_id: String,
    pub scope: Option<String>,
    pub device_fingerprint: Option<String>,
    pub lifetime_hours: Option<i64>,
}

/// Token validation result
#[derive(Debug, Clone)]
pub struct TokenValidationResult {
    pub claims: SecureJWTClaims,
    pub is_valid: bool,
    pub validation_time_ms: u64,
    pub key_id: String,
}

/// JWT Security Error types
#[derive(Debug, thiserror::Error)]
pub enum JWTSecurityError {
    #[error("Invalid algorithm: only RS256 is supported")]
    UnsupportedAlgorithm,
    #[error("Missing or invalid key ID")]
    InvalidKeyId,
    #[error("Token expired or invalid lifetime")]
    TokenExpired,
    #[error("Token not yet valid")]
    TokenNotYetValid,
    #[error("Unknown or expired signing key")]
    UnknownKey,
    #[error("Token format invalid")]
    InvalidFormat,
    #[error("Token signature verification failed")]
    SignatureVerificationFailed,
    #[error("Token lifetime exceeds maximum allowed")]
    ExcessiveLifetime,
    #[error("Token revoked or blacklisted")]
    TokenRevoked,
    #[error("Validation error: {0}")]
    ValidationFailed(String),
}

/// Secure JWT Service with RS256-only implementation
pub struct SecureJWTService {
    key_manager: Arc<KeyManager>,
    cache: Arc<dyn Cache>,
    issuer: String,
    default_audience: String,
}

impl SecureJWTService {
    /// Create new secure JWT service
    pub fn new(
        key_manager: Arc<KeyManager>,
        cache: Arc<dyn Cache>,
        issuer: String,
        default_audience: String,
    ) -> Self {
        info!("Initializing Secure JWT Service with RS256-only support");
        Self {
            key_manager,
            cache,
            issuer,
            default_audience,
        }
    }
    
    /// Create a secure JWT token with RS256 algorithm
    pub async fn create_token(&self, request: TokenRequest) -> Result<String, JWTSecurityError> {
        let start_time = std::time::Instant::now();
        
        // Validate token lifetime
        let lifetime_hours = request.lifetime_hours.unwrap_or(1); // Default 1 hour
        if lifetime_hours > MAX_TOKEN_LIFETIME_HOURS {
            warn!("Token lifetime {} hours exceeds maximum {}", lifetime_hours, MAX_TOKEN_LIFETIME_HOURS);
            return Err(JWTSecurityError::ExcessiveLifetime);
        }
        
        let now = Utc::now();
        let expires_at = now + Duration::hours(lifetime_hours);
        let jti = Uuid::new_v4().to_string();
        
        // Build claims with comprehensive security fields
        let claims = SecureJWTClaims {
            sub: request.user_id.clone(),
            iss: self.issuer.clone(),
            aud: self.default_audience.clone(),
            exp: expires_at.timestamp(),
            iat: now.timestamp(),
            nbf: now.timestamp(),
            jti: jti.clone(),
            email: request.email,
            name: request.name,
            role: request.role,
            permissions: request.permissions,
            platform_context: request.platform_context,
            tier: request.tier,
            token_type: request.token_type.clone(),
            client_id: request.client_id,
            scope: request.scope,
            device_fingerprint: request.device_fingerprint,
        };
        
        // Create RS256 header with key ID
        let mut header = Header::new(Algorithm::RS256);
        header.kid = Some(self.key_manager.current_key().kid.clone());
        
        // Sign token with current RSA key
        let token = encode(&header, &claims, &self.key_manager.current_key().encoding_key)
            .map_err(|e| {
                error!("Token encoding failed: {}", e);
                JWTSecurityError::ValidationFailed(format!("Encoding failed: {}", e))
            })?;
        
        // Cache token for revocation checking
        let cache_key = format!("jwt:{}:{}", request.token_type, jti);
        let cache_value = format!("{}:{}", request.user_id, expires_at.timestamp());
        self.cache.set(&cache_key, cache_value, Some((lifetime_hours * 3600) as u64));
        
        let elapsed = start_time.elapsed();
        info!(
            "Created {} token for user {} (lifetime: {}h) in {}ms",
            request.token_type,
            request.user_id,
            lifetime_hours,
            elapsed.as_millis()
        );
        
        Ok(token)
    }
    
    /// Validate JWT token with comprehensive security checks
    pub async fn validate_token(&self, token: &str) -> Result<TokenValidationResult, JWTSecurityError> {
        let start_time = std::time::Instant::now();
        
        // Basic format validation
        if token.is_empty() || token.len() > 8192 {
            return Err(JWTSecurityError::InvalidFormat);
        }
        
        let parts: Vec<&str> = token.split('.').collect();
        if parts.len() != 3 {
            return Err(JWTSecurityError::InvalidFormat);
        }
        
        // Decode header first to get algorithm and key ID
        let header = jsonwebtoken::decode_header(token)
            .map_err(|_| JWTSecurityError::InvalidFormat)?;
        
        // Enforce RS256 algorithm only
        if header.alg != Algorithm::RS256 {
            warn!("Rejected token with unsupported algorithm: {:?}", header.alg);
            return Err(JWTSecurityError::UnsupportedAlgorithm);
        }
        
        // Extract and validate key ID
        let key_id = header.kid.ok_or(JWTSecurityError::InvalidKeyId)?;
        if key_id.is_empty() || key_id.len() > 64 {
            return Err(JWTSecurityError::InvalidKeyId);
        }
        
        // Validate key ID format (alphanumeric, hyphens, underscores only)
        if !key_id.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
            warn!("Invalid key ID format: {}", key_id);
            return Err(JWTSecurityError::InvalidKeyId);
        }
        
        // Get the appropriate RSA key for verification
        let key_pair = self.key_manager.get_key(&key_id)
            .ok_or(JWTSecurityError::UnknownKey)?;
        
        // Key validation is now handled by KeyManager
        
        // Set up strict validation
        let mut validation = Validation::new(Algorithm::RS256);
        validation.set_issuer(&[&self.issuer]);
        validation.set_audience(&[&self.default_audience]);
        validation.validate_exp = true;
        validation.validate_nbf = true;
        validation.leeway = 30; // 30 seconds clock skew tolerance
        
        // Decode and validate token
        let token_data = decode::<SecureJWTClaims>(token, &key_pair.decoding_key, &validation)
            .map_err(|e| {
                match e.kind() {
                    ErrorKind::ExpiredSignature => {
                        warn!("Token expired: {}", token.chars().take(20).collect::<String>());
                        JWTSecurityError::TokenExpired
                    },
                    ErrorKind::ImmatureSignature => {
                        warn!("Token not yet valid: {}", token.chars().take(20).collect::<String>());
                        JWTSecurityError::TokenNotYetValid
                    },
                    ErrorKind::InvalidSignature => {
                        warn!("Token signature verification failed");
                        JWTSecurityError::SignatureVerificationFailed
                    },
                    _ => {
                        warn!("Token validation failed: {}", e);
                        JWTSecurityError::ValidationFailed(e.to_string())
                    }
                }
            })?;
        
        let claims = token_data.claims;
        
        // Check if token is revoked
        let cache_key = format!("jwt:{}:{}", claims.token_type, claims.jti);
        if self.cache.get(&cache_key).is_none() {
            warn!("Token not found in cache (revoked): {}", claims.jti);
            return Err(JWTSecurityError::TokenRevoked);
        }
        
        // Validate token lifetime against maximum
        let token_lifetime = claims.exp - claims.iat;
        if token_lifetime > MAX_TOKEN_LIFETIME_HOURS * 3600 {
            warn!("Token lifetime {} exceeds maximum", token_lifetime);
            return Err(JWTSecurityError::ExcessiveLifetime);
        }
        
        // Validate JTI format (UUID)
        if Uuid::parse_str(&claims.jti).is_err() {
            warn!("Invalid JTI format: {}", claims.jti);
            return Err(JWTSecurityError::InvalidFormat);
        }
        
        let elapsed = start_time.elapsed();
        let validation_time_ms = elapsed.as_millis() as u64;
        
        info!(
            "Validated {} token for user {} in {}ms",
            claims.token_type,
            claims.sub,
            validation_time_ms
        );
        
        Ok(TokenValidationResult {
            claims,
            is_valid: true,
            validation_time_ms,
            key_id,
        })
    }
    
    /// Revoke a token by removing it from cache
    pub async fn revoke_token(&self, jti: &str, token_type: &str) -> Result<(), JWTSecurityError> {
        // Validate JTI format
        if Uuid::parse_str(jti).is_err() {
            return Err(JWTSecurityError::InvalidFormat);
        }
        
        let cache_key = format!("jwt:{}:{}", token_type, jti);
        self.cache.delete(&cache_key);
        
        info!("Revoked {} token: {}", token_type, jti);
        Ok(())
    }
    
    /// Extract claims without full validation (for debugging/logging)
    pub fn decode_token_unsafe(&self, token: &str) -> Result<SecureJWTClaims, JWTSecurityError> {
        // Basic format check
        if token.split('.').count() != 3 {
            return Err(JWTSecurityError::InvalidFormat);
        }
        
        // Decode header to check algorithm
        let header = jsonwebtoken::decode_header(token)
            .map_err(|_| JWTSecurityError::InvalidFormat)?;
        
        if header.alg != Algorithm::RS256 {
            return Err(JWTSecurityError::UnsupportedAlgorithm);
        }
        
        let key_id = header.kid.ok_or(JWTSecurityError::InvalidKeyId)?;
        let key_pair = self.key_manager.get_key(&key_id)
            .ok_or(JWTSecurityError::UnknownKey)?;
        
        let mut validation = Validation::new(Algorithm::RS256);
        validation.validate_exp = false; // Skip expiration for unsafe decode
        validation.validate_nbf = false;
        validation.validate_aud = false;
        
        let token_data = decode::<SecureJWTClaims>(token, &key_pair.decoding_key, &validation)
            .map_err(|e| JWTSecurityError::ValidationFailed(e.to_string()))?;
        
        Ok(token_data.claims)
    }
    
    /// Get service health information
    pub fn health_check(&self) -> serde_json::Value {
        serde_json::json!({
            "service": "SecureJWTService",
            "algorithm": "RS256",
            "max_token_lifetime_hours": MAX_TOKEN_LIFETIME_HOURS,
            "current_key_id": self.key_manager.current_key().kid,
            "available_keys": self.key_manager.list_key_ids(),
            "security_features": [
                "RS256_only",
                "key_rotation",
                "token_revocation",
                "strict_validation",
                "lifetime_limits",
                "device_binding"
            ]
        })
    }
}