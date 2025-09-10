// Secure Authentication Tokens with Enhanced Claims
// Implements stateless granular permissions with cryptographic validation

use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use sha2::{Sha256, Digest};
use uuid::Uuid;
use crate::infrastructure::security::{get_key_manager, KeyError};
use crate::domain::shared_kernel::value_objects::UserId;
use jsonwebtoken::{encode, decode, Header, Algorithm, Validation};

#[derive(Debug)]
pub enum TokenError {
    GenerationFailed(String),
    InvalidToken(String),
    SecurityViolation(String),
    KeyManagementError(String),
    PermissionIntegrityError(String),
    DeviceBindingError(String),
    ExpiredToken(String),
}

impl std::fmt::Display for TokenError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            TokenError::GenerationFailed(msg) => write!(f, "Token generation failed: {}", msg),
            TokenError::InvalidToken(msg) => write!(f, "Invalid token: {}", msg),
            TokenError::SecurityViolation(msg) => write!(f, "Security violation: {}", msg),
            TokenError::KeyManagementError(msg) => write!(f, "Key management error: {}", msg),
            TokenError::PermissionIntegrityError(msg) => write!(f, "Permission integrity error: {}", msg),
            TokenError::DeviceBindingError(msg) => write!(f, "Device binding error: {}", msg),
            TokenError::ExpiredToken(msg) => write!(f, "Token expired: {}", msg),
        }
    }
}

impl std::error::Error for TokenError {}

impl From<KeyError> for TokenError {
    fn from(err: KeyError) -> Self {
        TokenError::KeyManagementError(err.to_string())
    }
}

impl From<jsonwebtoken::errors::Error> for TokenError {
    fn from(err: jsonwebtoken::errors::Error) -> Self {
        TokenError::InvalidToken(err.to_string())
    }
}

/// Enhanced JWT claims with granular permissions and security features
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecureAccessTokenClaims {
    // Standard OIDC claims
    pub sub: String,                    // Subject (user ID)
    pub iss: String,                    // Issuer
    pub aud: String,                    // Audience
    pub iat: i64,                       // Issued at
    pub exp: i64,                       // Expires at
    pub jti: String,                    // JWT ID
    
    // Enhanced permission claims
    pub permissions: Vec<String>,        // ["admin:users:read", "epsx:analytics:write:1735689600"]
    pub permission_hash: String,         // SHA256 hash for integrity validation
    pub roles: Vec<String>,             // ["admin", "analyst"]
    pub granted_by: String,             // Admin who granted permissions
    pub platform_context: String,       // "epsx", "admin", "epsx-pay"
    
    // Security features
    pub device_fingerprint: String,     // Device binding for security
    pub ip_subnet: Option<String>,      // IP restriction (e.g., "192.168.1.0/24")
    pub security_level: u8,            // Risk-based access control (1-5)
    pub token_version: u32,            // For immediate token invalidation
    pub session_id: String,            // Session tracking
    
    // Temporal controls
    pub max_session_duration: Option<i64>, // Maximum session length in seconds
    pub requires_mfa: bool,            // MFA required for sensitive operations
    pub can_refresh: bool,             // Token can be refreshed
    pub refresh_count: u32,            // Number of times token has been refreshed
}

impl SecureAccessTokenClaims {
    /// Create new secure access token claims with validation
    pub fn new(
        user_id: &UserId,
        permissions: Vec<String>,
        roles: Vec<String>,
        granted_by: &str,
        device_fingerprint: &str,
        expires_at: DateTime<Utc>,
    ) -> Result<Self, TokenError> {
        let permission_hash = Self::compute_permission_hash(&permissions);
        let now = Utc::now();
        
        // Validate input parameters
        if permissions.is_empty() && roles.is_empty() {
            return Err(TokenError::GenerationFailed(
                "Either permissions or roles must be provided".to_string()
            ));
        }
        
        if device_fingerprint.len() < 16 {
            return Err(TokenError::DeviceBindingError(
                "Device fingerprint too short, must be at least 16 characters".to_string()
            ));
        }
        
        Ok(Self {
            sub: format!("auth:{}", user_id.as_str()),
            iss: "https://auth.epsx.io".to_string(),
            aud: "epsx-api".to_string(),
            iat: now.timestamp(),
            exp: expires_at.timestamp(),
            jti: Uuid::new_v4().to_string(),
            
            permissions,
            permission_hash,
            roles,
            granted_by: granted_by.to_string(),
            platform_context: "epsx".to_string(),
            
            device_fingerprint: device_fingerprint.to_string(),
            ip_subnet: None, // TODO: Extract from request IP
            security_level: 3, // Default medium security
            token_version: 1,
            session_id: Uuid::new_v4().to_string(),
            
            max_session_duration: Some(8 * 3600), // 8 hours default
            requires_mfa: false,
            can_refresh: true,
            refresh_count: 0,
        })
    }
    
    /// Validate permission integrity using cryptographic hash
    pub fn validate_integrity(&self) -> Result<(), TokenError> {
        let computed_hash = Self::compute_permission_hash(&self.permissions);
        if computed_hash != self.permission_hash {
            return Err(TokenError::PermissionIntegrityError(
                "Permission claims have been tampered with".to_string()
            ));
        }
        Ok(())
    }
    
    /// Validate device binding
    pub fn validate_device_binding(&self, request_fingerprint: &str) -> Result<(), TokenError> {
        if self.device_fingerprint != request_fingerprint {
            return Err(TokenError::DeviceBindingError(format!(
                "Token bound to different device. Expected: {}, Got: {}",
                self.device_fingerprint,
                request_fingerprint
            )));
        }
        Ok(())
    }
    
    /// Get active (non-expired) permissions
    pub fn get_active_permissions(&self) -> Vec<String> {
        let now = Utc::now().timestamp();
        
        self.permissions.iter()
            .filter(|perm| {
                // Filter out expired temporal permissions
                if let Some(timestamp) = Self::extract_timestamp(perm) {
                    timestamp > now
                } else {
                    true // Permanent permission
                }
            })
            .cloned()
            .collect()
    }
    
    /// Check if token is expired
    pub fn is_expired(&self) -> bool {
        Utc::now().timestamp() > self.exp
    }
    
    /// Check if session duration exceeded
    pub fn is_session_expired(&self) -> bool {
        if let Some(max_duration) = self.max_session_duration {
            let session_age = Utc::now().timestamp() - self.iat;
            session_age > max_duration
        } else {
            false
        }
    }
    
    /// Check if token has specific permission
    pub fn has_permission(&self, required_permission: &str) -> bool {
        let active_permissions = self.get_active_permissions();
        
        // Check exact match
        if active_permissions.contains(&required_permission.to_string()) {
            return true;
        }
        
        // Check wildcard permissions
        Self::check_wildcard_permissions(&active_permissions, required_permission)
    }
    
    /// Get user ID from subject claim
    pub fn get_user_id(&self) -> Result<UserId, TokenError> {
        let user_id_str = self.sub
            .strip_prefix("auth:")
            .ok_or_else(|| TokenError::InvalidToken(
                "Invalid subject format, expected 'auth:' prefix".to_string()
            ))?;
        
        UserId::from_string(user_id_str.to_string())
            .map_err(|e| TokenError::InvalidToken(format!("Invalid user ID format: {}", e)))
    }
    
    /// Create new claims for token refresh (increments refresh count)
    pub fn create_refresh_claims(&self, new_expires_at: DateTime<Utc>) -> Result<Self, TokenError> {
        if !self.can_refresh {
            return Err(TokenError::SecurityViolation(
                "Token refresh not allowed".to_string()
            ));
        }
        
        if self.refresh_count >= 10 {
            return Err(TokenError::SecurityViolation(
                "Maximum refresh count exceeded".to_string()
            ));
        }
        
        let mut new_claims = self.clone();
        new_claims.exp = new_expires_at.timestamp();
        new_claims.iat = Utc::now().timestamp();
        new_claims.jti = Uuid::new_v4().to_string();
        new_claims.refresh_count += 1;
        
        Ok(new_claims)
    }
    
    // Private helper methods
    fn compute_permission_hash(permissions: &[String]) -> String {
        let mut hasher = Sha256::new();
        
        // Sort permissions for consistent hashing
        let mut sorted_permissions = permissions.to_vec();
        sorted_permissions.sort();
        
        for perm in &sorted_permissions {
            hasher.update(perm.as_bytes());
        }
        
        format!("{:x}", hasher.finalize())
    }
    
    fn extract_timestamp(permission: &str) -> Option<i64> {
        permission.split(':').last()
            .and_then(|s| s.parse().ok())
    }
    
    fn check_wildcard_permissions(permissions: &[String], required: &str) -> bool {
        let required_parts: Vec<&str> = required.split(':').collect();
        
        for permission in permissions {
            let perm_parts: Vec<&str> = permission.split(':').collect();
            
            // Skip temporal permissions for wildcard matching
            let perm_parts = if perm_parts.len() > 3 && perm_parts[3].parse::<i64>().is_ok() {
                &perm_parts[..3]
            } else {
                &perm_parts
            };
            
            if perm_parts.len() != required_parts.len() {
                continue;
            }
            
            let matches = perm_parts.iter().zip(required_parts.iter())
                .all(|(p, r)| *p == "*" || *p == *r);
            
            if matches {
                return true;
            }
        }
        
        false
    }
}

/// Secure JWT token wrapper with cryptographic validation
#[derive(Debug, Clone)]
pub struct SecureAccessToken {
    token: String,
    claims: SecureAccessTokenClaims,
}

impl SecureAccessToken {
    /// Generate new secure JWT token with RS256 signature
    pub fn generate(
        user_id: &UserId,
        permissions: Vec<String>,
        roles: Vec<String>,
        granted_by: &str,
        device_fingerprint: &str,
        expires_in_minutes: u32,
    ) -> Result<Self, TokenError> {
        let expires_at = Utc::now() + chrono::Duration::minutes(expires_in_minutes as i64);
        
        let claims = SecureAccessTokenClaims::new(
            user_id,
            permissions,
            roles,
            granted_by,
            device_fingerprint,
            expires_at,
        )?;
        
        let key_manager = get_key_manager()?;
        let encoding_key = key_manager.get_encoding_key()?;
        let header = Header::new(Algorithm::RS256);
        
        let token = encode(&header, &claims, &encoding_key)?;
        
        tracing::info!(
            user_id = %user_id.as_str(),
            permissions_count = claims.permissions.len(),
            expires_at = %expires_at,
            jti = %claims.jti,
            "Secure access token generated with RS256 signature"
        );
        
        Ok(Self { token, claims })
    }
    
    /// Validate and parse secure JWT token with full security checks
    pub fn from_jwt(token: String, request_device_fingerprint: &str) -> Result<Self, TokenError> {
        let key_manager = get_key_manager()?;
        let decoding_key = key_manager.get_decoding_key()?;
        let validation = Validation::new(Algorithm::RS256);
        
        let token_data = decode::<SecureAccessTokenClaims>(&token, &decoding_key, &validation)?;
        let claims = token_data.claims;
        
        // Perform comprehensive security validation
        claims.validate_integrity()?;
        claims.validate_device_binding(request_device_fingerprint)?;
        
        if claims.is_expired() {
            return Err(TokenError::ExpiredToken(
                "Access token has expired".to_string()
            ));
        }
        
        if claims.is_session_expired() {
            return Err(TokenError::ExpiredToken(
                "Session duration exceeded".to_string()
            ));
        }
        
        tracing::debug!(
            user_id = %claims.sub,
            jti = %claims.jti,
            permissions_count = claims.permissions.len(),
            "Secure access token validated successfully"
        );
        
        Ok(Self { token, claims })
    }
    
    /// Create refreshed token
    pub fn refresh(&self, expires_in_minutes: u32) -> Result<Self, TokenError> {
        let expires_at = Utc::now() + chrono::Duration::minutes(expires_in_minutes as i64);
        let new_claims = self.claims.create_refresh_claims(expires_at)?;
        
        let key_manager = get_key_manager()?;
        let encoding_key = key_manager.get_encoding_key()?;
        let header = Header::new(Algorithm::RS256);
        
        let token = encode(&header, &new_claims, &encoding_key)?;
        
        tracing::info!(
            user_id = %new_claims.sub,
            old_jti = %self.claims.jti,
            new_jti = %new_claims.jti,
            refresh_count = new_claims.refresh_count,
            "Access token refreshed successfully"
        );
        
        Ok(Self { token, claims: new_claims })
    }
    
    // Getters
    pub fn token(&self) -> &str { &self.token }
    pub fn claims(&self) -> &SecureAccessTokenClaims { &self.claims }
    pub fn expires_at(&self) -> DateTime<Utc> {
        DateTime::from_timestamp(self.claims.exp, 0)
            .unwrap_or_else(|| Utc::now())
    }
    pub fn user_id(&self) -> Result<UserId, TokenError> { self.claims.get_user_id() }
    pub fn permissions(&self) -> Vec<String> { self.claims.get_active_permissions() }
    pub fn jti(&self) -> &str { &self.claims.jti }
    pub fn session_id(&self) -> &str { &self.claims.session_id }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_permission_integrity_validation() {
        let user_id = UserId::from_string("user123".to_string()).unwrap();
        let permissions = vec!["epsx:read".to_string(), "admin:users:write".to_string()];
        
        let claims = SecureAccessTokenClaims::new(
            &user_id,
            permissions,
            vec![],
            "admin",
            "device_fingerprint_123456789",
            Utc::now() + chrono::Duration::hours(1),
        ).unwrap();
        
        // Valid integrity check should pass
        assert!(claims.validate_integrity().is_ok());
        
        // Modify permissions without updating hash (simulating tampering)
        let mut tampered_claims = claims.clone();
        tampered_claims.permissions.push("admin:*:*".to_string());
        
        // Integrity check should fail
        assert!(tampered_claims.validate_integrity().is_err());
    }
    
    #[test]
    fn test_temporal_permissions() {
        let user_id = UserId::from_string("user123".to_string()).unwrap();
        let future_timestamp = (Utc::now() + chrono::Duration::hours(1)).timestamp();
        let past_timestamp = (Utc::now() - chrono::Duration::hours(1)).timestamp();
        
        let permissions = vec![
            "epsx:read".to_string(), // Permanent
            format!("admin:temp:write:{}", future_timestamp), // Valid temporal
            format!("admin:expired:delete:{}", past_timestamp), // Expired temporal
        ];
        
        let claims = SecureAccessTokenClaims::new(
            &user_id,
            permissions,
            vec![],
            "admin",
            "device_fingerprint_123456789",
            Utc::now() + chrono::Duration::hours(1),
        ).unwrap();
        
        let active_permissions = claims.get_active_permissions();
        
        // Should include permanent and valid temporal permissions
        assert!(active_permissions.contains(&"epsx:read".to_string()));
        assert!(active_permissions.iter().any(|p| p.starts_with("admin:temp:write")));
        
        // Should exclude expired temporal permissions
        assert!(!active_permissions.iter().any(|p| p.starts_with("admin:expired:delete")));
    }
    
    #[test]
    fn test_wildcard_permissions() {
        let user_id = UserId::from_string("user123".to_string()).unwrap();
        let permissions = vec!["admin:*:*".to_string(), "epsx:users:read".to_string()];
        
        let claims = SecureAccessTokenClaims::new(
            &user_id,
            permissions,
            vec![],
            "admin",
            "device_fingerprint_123456789",
            Utc::now() + chrono::Duration::hours(1),
        ).unwrap();
        
        // Wildcard should match
        assert!(claims.has_permission("admin:users:delete"));
        assert!(claims.has_permission("admin:analytics:read"));
        
        // Exact match should work
        assert!(claims.has_permission("epsx:users:read"));
        
        // Non-matching should fail
        assert!(!claims.has_permission("epsx:users:write"));
    }
    
    #[test]
    fn test_device_binding_validation() {
        let user_id = UserId::from_string("user123".to_string()).unwrap();
        let device_fingerprint = "device_fingerprint_123456789";
        
        let claims = SecureAccessTokenClaims::new(
            &user_id,
            vec!["epsx:read".to_string()],
            vec![],
            "admin",
            device_fingerprint,
            Utc::now() + chrono::Duration::hours(1),
        ).unwrap();
        
        // Same device should validate
        assert!(claims.validate_device_binding(device_fingerprint).is_ok());
        
        // Different device should fail
        assert!(claims.validate_device_binding("different_device").is_err());
    }
}