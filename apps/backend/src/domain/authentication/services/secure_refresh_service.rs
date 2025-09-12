// Secure Refresh Token Service
// Implements refresh token rotation with family tracking and security monitoring

use crate::domain::authentication::value_objects::SecureAccessToken;
use crate::domain::shared_kernel::value_objects::UserId;
use chrono::{DateTime, Utc, Duration};
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use tracing::{info, warn};

#[derive(Debug)]
pub enum RefreshError {
    TokenNotFound,
    TokenRevoked(String),
    TokenExpired,
    FamilyCompromised(String),
    InvalidToken(String),
    DeviceMismatch(String),
    SecurityViolation(String),
    RateLimited(String),
    DatabaseError(String),
}

impl std::fmt::Display for RefreshError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            RefreshError::TokenNotFound => write!(f, "Refresh token not found"),
            RefreshError::TokenRevoked(msg) => write!(f, "Token revoked: {}", msg),
            RefreshError::TokenExpired => write!(f, "Refresh token expired"),
            RefreshError::FamilyCompromised(msg) => write!(f, "Token family compromised: {}", msg),
            RefreshError::InvalidToken(msg) => write!(f, "Invalid token: {}", msg),
            RefreshError::DeviceMismatch(msg) => write!(f, "Device mismatch: {}", msg),
            RefreshError::SecurityViolation(msg) => write!(f, "Security violation: {}", msg),
            RefreshError::RateLimited(msg) => write!(f, "Rate limited: {}", msg),
            RefreshError::DatabaseError(msg) => write!(f, "Database error: {}", msg),
        }
    }
}

impl std::error::Error for RefreshError {}

/// Refresh token family for rotation tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefreshTokenFamily {
    pub family_id: Uuid,
    pub user_id: UserId,
    pub created_at: DateTime<Utc>,
    pub last_used_at: DateTime<Utc>,
    pub device_fingerprint: String,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub is_active: bool,
    pub revoked_at: Option<DateTime<Utc>>,
    pub revoked_reason: Option<String>,
    pub usage_count: u32,
    pub max_lifetime_hours: u32,
}

/// Individual refresh token within a family
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefreshToken {
    pub token_id: Uuid,
    pub family_id: Uuid,
    pub user_id: UserId,
    pub token_hash: String,  // SHA256 hash of actual token
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub used_at: Option<DateTime<Utc>>,
    pub is_used: bool,
    pub device_fingerprint: String,
    pub successor_id: Option<Uuid>,  // Next token in rotation chain
}

/// Refresh request context
#[derive(Debug, Clone)]
pub struct RefreshContext {
    pub device_fingerprint: String,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub timestamp: DateTime<Utc>,
}

/// Refresh result with new tokens
#[derive(Debug, Clone)]
pub struct RefreshResult {
    pub new_access_token: SecureAccessToken,
    pub new_refresh_token: String,
    pub expires_in: i64,
    pub refresh_count: u32,
    pub security_warnings: Vec<String>,
}

/// Secure refresh token service with family rotation
pub struct SecureRefreshService {
    // In production, these would be proper repository interfaces
    refresh_token_store: std::sync::RwLock<std::collections::HashMap<String, RefreshToken>>,
    token_families: std::sync::RwLock<std::collections::HashMap<Uuid, RefreshTokenFamily>>,
    revoked_tokens: std::sync::RwLock<std::collections::HashSet<String>>,
}

impl SecureRefreshService {
    pub fn new() -> Self {
        Self {
            refresh_token_store: std::sync::RwLock::new(std::collections::HashMap::new()),
            token_families: std::sync::RwLock::new(std::collections::HashMap::new()),
            revoked_tokens: std::sync::RwLock::new(std::collections::HashSet::new()),
        }
    }
    
    /// Create new refresh token family for user
    pub fn create_refresh_family(
        &self,
        user_id: &UserId,
        device_fingerprint: &str,
        ip_address: Option<String>,
        user_agent: Option<String>,
        max_lifetime_hours: u32,
    ) -> Result<(RefreshTokenFamily, String), RefreshError> {
        let family_id = Uuid::new_v4();
        let now = Utc::now();
        
        // Create token family
        let family = RefreshTokenFamily {
            family_id,
            user_id: user_id.clone(),
            created_at: now,
            last_used_at: now,
            device_fingerprint: device_fingerprint.to_string(),
            ip_address,
            user_agent,
            is_active: true,
            revoked_at: None,
            revoked_reason: None,
            usage_count: 0,
            max_lifetime_hours,
        };
        
        // Generate first refresh token
        let refresh_token = self.generate_refresh_token(&family)?;
        let token_string = self.create_token_string(&refresh_token);
        
        // Store family and token
        {
            let mut families = self.token_families.write().unwrap();
            families.insert(family_id, family.clone());
        }
        
        {
            let mut tokens = self.refresh_token_store.write().unwrap();
            tokens.insert(token_string.clone(), refresh_token);
        }
        
        info!(
            user_id = %user_id.as_str(),
            family_id = %family_id,
            device_fingerprint = %device_fingerprint,
            "Created new refresh token family"
        );
        
        Ok((family, token_string))
    }
    
    /// Refresh access token using refresh token rotation
    pub fn refresh_access_token(
        &self,
        refresh_token: &str,
        context: RefreshContext,
        requested_permissions: Vec<String>,
        granted_by: &str,
    ) -> Result<RefreshResult, RefreshError> {
        // Validate refresh token
        let old_token = self.validate_refresh_token(refresh_token, &context)?;
        
        // Get token family
        let family = self.get_token_family(&old_token.family_id)?;
        
        // Security checks
        self.perform_security_checks(&old_token, &family, &context)?;
        
        // Check family lifetime
        if self.is_family_expired(&family) {
            let mut family_mut = family;
            self.revoke_family(&mut family_mut, "Family lifetime exceeded")?;
            return Err(RefreshError::TokenExpired);
        }
        
        // Mark old token as used
        self.mark_token_used(&old_token, context.timestamp)?;
        
        // Generate new refresh token
        let new_refresh_token = self.generate_refresh_token(&family)?;
        let new_refresh_token_string = self.create_token_string(&new_refresh_token);
        
        // Update family usage
        let mut family = family;
        family.usage_count += 1;
        family.last_used_at = context.timestamp;
        
        // Generate new access token
        let expires_in_minutes = 60; // 1 hour
        let new_access_token = SecureAccessToken::generate(
            &old_token.user_id,
            requested_permissions,
            vec![], // Roles would come from user lookup
            granted_by,
            &context.device_fingerprint,
            expires_in_minutes,
        ).map_err(|e| RefreshError::InvalidToken(e.to_string()))?;
        
        // Store new tokens
        {
            let mut tokens = self.refresh_token_store.write().unwrap();
            tokens.insert(new_refresh_token_string.clone(), new_refresh_token);
        }
        
        {
            let mut families = self.token_families.write().unwrap();
            families.insert(family.family_id, family.clone());
        }
        
        // Security monitoring
        let security_warnings = self.analyze_refresh_security(&old_token, &family, &context);
        
        info!(
            user_id = %old_token.user_id.as_str(),
            family_id = %family.family_id,
            usage_count = family.usage_count,
            security_warnings = security_warnings.len(),
            "Access token refreshed successfully"
        );
        
        Ok(RefreshResult {
            new_access_token,
            new_refresh_token: new_refresh_token_string,
            expires_in: expires_in_minutes as i64 * 60,
            refresh_count: family.usage_count,
            security_warnings,
        })
    }
    
    /// Revoke specific refresh token
    pub fn revoke_refresh_token(&self, refresh_token: &str, reason: &str) -> Result<(), RefreshError> {
        let token = {
            let tokens = self.refresh_token_store.read().unwrap();
            tokens.get(refresh_token).cloned()
                .ok_or(RefreshError::TokenNotFound)?
        };
        
        // Revoke entire family for security
        let mut family = self.get_token_family(&token.family_id)?;
        self.revoke_family(&mut family, reason)?;
        
        warn!(
            user_id = %token.user_id.as_str(),
            family_id = %token.family_id,
            reason = %reason,
            "Refresh token revoked"
        );
        
        Ok(())
    }
    
    /// Revoke all refresh tokens for user
    pub fn revoke_all_user_tokens(&self, user_id: &UserId, reason: &str) -> Result<u32, RefreshError> {
        let mut revoked_count = 0;
        
        // Find all families for user
        let family_ids: Vec<Uuid> = {
            let families = self.token_families.read().unwrap();
            families.values()
                .filter(|f| f.user_id == *user_id && f.is_active)
                .map(|f| f.family_id)
                .collect()
        };
        
        // Revoke each family
        for family_id in family_ids {
            if let Ok(mut family) = self.get_token_family(&family_id) {
                if self.revoke_family(&mut family, reason).is_ok() {
                    revoked_count += 1;
                }
            }
        }
        
        warn!(
            user_id = %user_id.as_str(),
            revoked_count = revoked_count,
            reason = %reason,
            "All user refresh tokens revoked"
        );
        
        Ok(revoked_count)
    }
    
    /// Get refresh token statistics for monitoring
    pub fn get_refresh_statistics(&self, user_id: &UserId) -> RefreshStatistics {
        let families = self.token_families.read().unwrap();
        let user_families: Vec<&RefreshTokenFamily> = families.values()
            .filter(|f| f.user_id == *user_id)
            .collect();
        
        let active_families = user_families.iter().filter(|f| f.is_active).count();
        let total_usage: u32 = user_families.iter().map(|f| f.usage_count).sum();
        let oldest_family = user_families.iter()
            .min_by_key(|f| f.created_at)
            .map(|f| f.created_at);
        let most_recent_use = user_families.iter()
            .max_by_key(|f| f.last_used_at)
            .map(|f| f.last_used_at);
        
        RefreshStatistics {
            user_id: user_id.clone(),
            active_families: active_families as u32,
            total_families: user_families.len() as u32,
            total_usage,
            oldest_family_created: oldest_family,
            most_recent_use,
        }
    }
    
    // Private helper methods
    
    fn validate_refresh_token(&self, token: &str, context: &RefreshContext) -> Result<RefreshToken, RefreshError> {
        // Check if token is revoked
        {
            let revoked = self.revoked_tokens.read().unwrap();
            if revoked.contains(token) {
                return Err(RefreshError::TokenRevoked("Token is in revocation list".to_string()));
            }
        }
        
        // Get token from store
        let stored_token = {
            let tokens = self.refresh_token_store.read().unwrap();
            tokens.get(token).cloned()
                .ok_or(RefreshError::TokenNotFound)?
        };
        
        // Check if token is already used
        if stored_token.is_used {
            // Token reuse detected - possible attack
            warn!(
                token_id = %stored_token.token_id,
                family_id = %stored_token.family_id,
                "Refresh token reuse detected - possible attack"
            );
            
            // Revoke entire family
            if let Ok(mut family) = self.get_token_family(&stored_token.family_id) {
                let _ = self.revoke_family(&mut family, "Token reuse detected");
            }
            
            return Err(RefreshError::FamilyCompromised("Token reuse detected".to_string()));
        }
        
        // Check expiry
        if context.timestamp >= stored_token.expires_at {
            return Err(RefreshError::TokenExpired);
        }
        
        Ok(stored_token)
    }
    
    fn perform_security_checks(
        &self,
        token: &RefreshToken,
        family: &RefreshTokenFamily,
        context: &RefreshContext,
    ) -> Result<(), RefreshError> {
        // Device fingerprint check
        if token.device_fingerprint != context.device_fingerprint {
            return Err(RefreshError::DeviceMismatch(
                "Device fingerprint changed".to_string()
            ));
        }
        
        // Rate limiting check
        let time_since_last_use = context.timestamp.signed_duration_since(family.last_used_at);
        if time_since_last_use < Duration::seconds(30) {
            return Err(RefreshError::RateLimited(
                "Refresh too frequent".to_string()
            ));
        }
        
        // Usage count check
        if family.usage_count >= 1000 {
            return Err(RefreshError::SecurityViolation(
                "Excessive refresh token usage".to_string()
            ));
        }
        
        Ok(())
    }
    
    fn is_family_expired(&self, family: &RefreshTokenFamily) -> bool {
        let family_age = Utc::now().signed_duration_since(family.created_at);
        family_age >= Duration::hours(family.max_lifetime_hours as i64)
    }
    
    fn revoke_family(&self, family: &mut RefreshTokenFamily, reason: &str) -> Result<(), RefreshError> {
        family.is_active = false;
        family.revoked_at = Some(Utc::now());
        family.revoked_reason = Some(reason.to_string());
        
        // Add all tokens from this family to revocation list
        let tokens_to_revoke: Vec<String> = {
            let tokens = self.refresh_token_store.read().unwrap();
            tokens.iter()
                .filter(|(_, token)| token.family_id == family.family_id)
                .map(|(token_str, _)| token_str.clone())
                .collect()
        };
        
        {
            let mut revoked = self.revoked_tokens.write().unwrap();
            for token in tokens_to_revoke {
                revoked.insert(token);
            }
        }
        
        // Update family in store
        {
            let mut families = self.token_families.write().unwrap();
            families.insert(family.family_id, family.clone());
        }
        
        Ok(())
    }
    
    fn mark_token_used(&self, token: &RefreshToken, used_at: DateTime<Utc>) -> Result<(), RefreshError> {
        let mut updated_token = token.clone();
        updated_token.is_used = true;
        updated_token.used_at = Some(used_at);
        
        // Find token string to update
        let token_string = {
            let tokens = self.refresh_token_store.read().unwrap();
            tokens.iter()
                .find(|(_, stored_token)| stored_token.token_id == token.token_id)
                .map(|(token_str, _)| token_str.clone())
                .ok_or(RefreshError::TokenNotFound)?
        };
        
        // Update token in store
        {
            let mut tokens = self.refresh_token_store.write().unwrap();
            tokens.insert(token_string, updated_token);
        }
        
        Ok(())
    }
    
    fn generate_refresh_token(&self, family: &RefreshTokenFamily) -> Result<RefreshToken, RefreshError> {
        let token_id = Uuid::new_v4();
        let now = Utc::now();
        let expires_at = now + Duration::days(30); // 30 days
        
        // Generate secure random token
        let token_data = format!("{}:{}:{}", family.family_id, token_id, now.timestamp());
        let token_hash = self.hash_token(&token_data);
        
        Ok(RefreshToken {
            token_id,
            family_id: family.family_id,
            user_id: family.user_id.clone(),
            token_hash,
            expires_at,
            created_at: now,
            used_at: None,
            is_used: false,
            device_fingerprint: family.device_fingerprint.clone(),
            successor_id: None,
        })
    }
    
    fn create_token_string(&self, token: &RefreshToken) -> String {
        // In production, this would be a cryptographically secure token
        format!("rt_{}_{}", token.family_id, token.token_id)
    }
    
    fn hash_token(&self, token_data: &str) -> String {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(token_data.as_bytes());
        format!("{:x}", hasher.finalize())
    }
    
    fn get_token_family(&self, family_id: &Uuid) -> Result<RefreshTokenFamily, RefreshError> {
        let families = self.token_families.read().unwrap();
        families.get(family_id)
            .cloned()
            .ok_or(RefreshError::TokenNotFound)
    }
    
    fn analyze_refresh_security(
        &self,
        _token: &RefreshToken,
        family: &RefreshTokenFamily,
        _context: &RefreshContext,
    ) -> Vec<String> {
        let mut warnings = Vec::new();
        
        // Check for high usage
        if family.usage_count > 100 {
            warnings.push("High refresh token usage".to_string());
        }
        
        // Check family age
        let family_age = Utc::now().signed_duration_since(family.created_at);
        if family_age > Duration::days(7) {
            warnings.push("Long-lived refresh token family".to_string());
        }
        
        warnings
    }
}

/// Refresh token statistics for monitoring
#[derive(Debug, Clone, Serialize)]
pub struct RefreshStatistics {
    pub user_id: UserId,
    pub active_families: u32,
    pub total_families: u32,
    pub total_usage: u32,
    pub oldest_family_created: Option<DateTime<Utc>>,
    pub most_recent_use: Option<DateTime<Utc>>,
}

// Singleton for global access
use std::sync::OnceLock;
static REFRESH_SERVICE: OnceLock<SecureRefreshService> = OnceLock::new();

/// Get global refresh service instance
pub fn get_refresh_service() -> &'static SecureRefreshService {
    REFRESH_SERVICE.get_or_init(|| SecureRefreshService::new())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_refresh_token_family_creation() {
        let service = SecureRefreshService::new();
        let user_id = UserId::from_string("user123".to_string()).unwrap();
        
        let result = service.create_refresh_family(
            &user_id,
            "device_fp_123",
            Some("192.168.1.1".to_string()),
            Some("Mozilla/5.0".to_string()),
            168, // 1 week
        );
        
        assert!(result.is_ok());
        let (family, token) = result.unwrap();
        assert_eq!(family.user_id, user_id);
        assert!(family.is_active);
        assert!(token.starts_with("rt_"));
    }
    
    #[test]
    fn test_refresh_token_rotation() {
        let service = SecureRefreshService::new();
        let user_id = UserId::from_string("user123".to_string()).unwrap();
        
        // Create initial family
        let (_family, refresh_token) = service.create_refresh_family(
            &user_id,
            "device_fp_123",
            Some("192.168.1.1".to_string()),
            Some("Mozilla/5.0".to_string()),
            168,
        ).unwrap();
        
        // Refresh the token
        let context = RefreshContext {
            device_fingerprint: "device_fp_123".to_string(),
            ip_address: Some("192.168.1.1".to_string()),
            user_agent: Some("Mozilla/5.0".to_string()),
            timestamp: Utc::now(),
        };
        
        let result = service.refresh_access_token(
            &refresh_token,
            context,
            vec!["epsx:read".to_string()],
            "test_admin"
        );
        
        assert!(result.is_ok());
        let refresh_result = result.unwrap();
        assert!(!refresh_result.new_refresh_token.is_empty());
        assert_eq!(refresh_result.refresh_count, 1);
    }
    
    #[test]
    fn test_token_reuse_detection() {
        let service = SecureRefreshService::new();
        let user_id = UserId::from_string("user123".to_string()).unwrap();
        
        // Create initial family
        let (_family, refresh_token) = service.create_refresh_family(
            &user_id,
            "device_fp_123",
            None,
            None,
            168,
        ).unwrap();
        
        let context = RefreshContext {
            device_fingerprint: "device_fp_123".to_string(),
            ip_address: None,
            user_agent: None,
            timestamp: Utc::now(),
        };
        
        // First refresh should work
        let result1 = service.refresh_access_token(
            &refresh_token,
            context.clone(),
            vec!["epsx:read".to_string()],
            "test_admin"
        );
        assert!(result1.is_ok());
        
        // Second refresh with same token should fail (reuse detection)
        let result2 = service.refresh_access_token(
            &refresh_token,
            context,
            vec!["epsx:read".to_string()],
            "test_admin"
        );
        assert!(result2.is_err());
        assert!(matches!(result2.unwrap_err(), RefreshError::FamilyCompromised(_)));
    }
    
    #[test]
    fn test_device_mismatch_detection() {
        let service = SecureRefreshService::new();
        let user_id = UserId::from_string("user123".to_string()).unwrap();
        
        // Create initial family
        let (_family, refresh_token) = service.create_refresh_family(
            &user_id,
            "device_fp_123",
            None,
            None,
            168,
        ).unwrap();
        
        // Try to refresh with different device fingerprint
        let context = RefreshContext {
            device_fingerprint: "different_device".to_string(),
            ip_address: None,
            user_agent: None,
            timestamp: Utc::now(),
        };
        
        let result = service.refresh_access_token(
            &refresh_token,
            context,
            vec!["epsx:read".to_string()],
            "test_admin"
        );
        
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), RefreshError::DeviceMismatch(_)));
    }
}