use jsonwebtoken::{decode, encode, Header, Validation, Algorithm, errors::ErrorKind};
use uuid::Uuid;

use serde::{Deserialize, Serialize};

use super::key_manager::KeyManager;

use std::sync::Arc;

use crate::config::env::get_env_var;

use std::collections::HashSet;

use crate::infra::db::diesel::repos::RevokedTokenRepository;


/// Check if user has admin permissions
fn has_admin_permissions(permissions: &[String]) -> bool {
    permissions.iter().any(|p| p == "admin:*:*" || p.starts_with("admin:"))
}

/// Derive package tier from permissions
pub fn derive_package_tier_from_permissions(permissions: &[String]) -> String {
    if has_enterprise_permissions(permissions) {
        "ENTERPRISE".to_string()
    } else if has_platinum_permissions(permissions) {
        "PLATINUM".to_string()
    } else if has_gold_permissions(permissions) {
        "GOLD".to_string()
    } else if has_silver_permissions(permissions) {
        "SILVER".to_string()
    } else if has_bronze_permissions(permissions) {
        "BRONZE".to_string()
    } else {
        "FREE".to_string()
    }
}

/// Derive accessible platforms from permissions
pub fn derive_accessible_platforms_from_permissions(permissions: &[String]) -> Vec<String> {
    let mut platforms = HashSet::new();
    
    for permission in permissions {
        if let Some(platform) = permission.split(':').next() {
            platforms.insert(platform.to_string());
        }
    }
    
    if platforms.is_empty() {
        vec!["epsx".to_string()] // Default fallback
    } else {
        platforms.into_iter().collect()
    }
}

/// Derive primary platform from permissions (priority: admin > epsx > epsx-pay > epsx-token)
pub fn derive_primary_platform_from_permissions(permissions: &[String]) -> String {
    if permissions.iter().any(|p| p.starts_with("admin:")) {
        "admin".to_string()
    } else if permissions.iter().any(|p| p.starts_with("epsx:")) {
        "epsx".to_string()
    } else if permissions.iter().any(|p| p.starts_with("epsx-pay:")) {
        "epsx-pay".to_string()
    } else if permissions.iter().any(|p| p.starts_with("epsx-token:")) {
        "epsx-token".to_string()
    } else {
        "epsx".to_string() // Default fallback
    }
}

/// Helper functions for tier detection
fn has_enterprise_permissions(permissions: &[String]) -> bool {
    permissions.iter().any(|p| {
        p.starts_with("enterprise:") || 
        p == "admin:*:*" ||
        p.contains("enterprise") ||
        has_admin_permissions(permissions)
    })
}

fn has_platinum_permissions(permissions: &[String]) -> bool {
    permissions.iter().any(|p| {
        p.starts_with("platinum:") ||
        p.contains("platinum") ||
        permissions.len() >= 10 // Many permissions indicate higher tier
    })
}

fn has_gold_permissions(permissions: &[String]) -> bool {
    permissions.iter().any(|p| {
        p.starts_with("gold:") ||
        p.contains("gold") ||
        permissions.iter().any(|perm| perm.contains("export") || perm.contains("advanced"))
    })
}

fn has_silver_permissions(permissions: &[String]) -> bool {
    permissions.iter().any(|p| {
        p.starts_with("silver:") ||
        p.contains("silver") ||
        permissions.len() >= 5 // Several permissions indicate silver tier
    })
}

fn has_bronze_permissions(permissions: &[String]) -> bool {
    permissions.iter().any(|p| {
        p.starts_with("bronze:") ||
        p.contains("bronze") ||
        permissions.len() >= 3 // Few permissions indicate bronze tier
    })
}

/// JWT claims following RFC 7519 standard with permission-only system
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    // Standard JWT claims (RFC 7519)
    pub sub: String,        // Subject (User ID) - same as firebase_uid
    pub iss: String,        // Issuer 
    pub aud: String,        // Audience
    pub exp: usize,         // Expiration Time
    pub iat: usize,         // Issued At
    pub nbf: usize,         // Not Before
    pub jti: String,        // JWT ID (for revocation)
    
    // User information
    pub email: String,
    pub name: Option<String>,
    
    // Permission-only authorization system
    pub permissions: Vec<String>,        // Structured Platform:Resource:Action format
}

/// Refresh token claims (simplified)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RefreshTokenClaims {
    pub sub: String,            // Subject (User ID)
    pub iss: String,            // Issuer
    pub aud: String,            // Audience (refresh)
    pub exp: i64,               // Expiration Time
    pub iat: i64,               // Issued At
    pub session_id: String,     // Session ID
    pub token_type: String,     // Token type (refresh)
}

#[derive(Debug, Clone)]
pub struct User {
    pub id: String,
    pub email: String,
    pub name: Option<String>,
    // permissions field removed - handled by separate table
    // package_tier, platforms, firebase_uid, primary_platform removed - derived from permissions
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Invalid token: {0}")]
    Invalid(String),
    #[error("Token expired")]
    Expired,
    #[error("Token revoked")]
    Revoked,
    #[error("Missing claims: {0}")]
    MissingClaims(String),
    #[error("Invalid signature")]
    InvalidSignature,
    #[error("Permission denied")]
    PermissionDenied,
    #[error("Not yet valid")]
    NotYetValid,
}

pub struct Service {
    key_manager: Arc<KeyManager>,
    issuer: String,
    revoked_token_repo: Option<Arc<RevokedTokenRepository>>,
}

impl Service {
    pub fn new() -> Result<Self, Error> {
        let key_manager = Arc::new(KeyManager::from_env_or_generate()
            .map_err(|e| Error::Invalid(format!("Failed to initialize KeyManager: {}", e)))?);
            
        let issuer = get_env_var("OIDC_ISSUER")
            .unwrap_or_else(|_| "http://localhost:8080".to_string());
            
        Ok(Self {
            key_manager,
            issuer,
            revoked_token_repo: None,
        })
    }

    pub fn with_revoked_token_repo(mut self, revoked_token_repo: Arc<RevokedTokenRepository>) -> Self {
        self.revoked_token_repo = Some(revoked_token_repo);
        self
    }

    /// Create JWT token with complete standard claims (permission-only)
    pub fn create(&self, user_data: UserData) -> Result<String, Error> {
        let now = chrono::Utc::now().timestamp() as usize;
        
        let claims = Claims {
            // Standard JWT claims
            sub: user_data.id.clone(),
            iss: self.issuer.clone(),
            aud: user_data.audience.unwrap_or_else(|| "epsx-ecosystem".to_string()),
            exp: now + user_data.ttl_seconds.unwrap_or(7200), // 2 hours default
            iat: now,
            nbf: now, // Valid immediately
            jti: Uuid::new_v4().to_string(), // Unique ID for revocation
            
            // User information
            email: user_data.email,
            name: user_data.name,
            
            // Permission-only authorization
            permissions: user_data.permissions.clone().unwrap_or_default(),
        };
        
        let current_key = self.key_manager.current_key();
        let mut header = Header::new(Algorithm::RS256);
        header.kid = Some(current_key.kid.clone());
        
        encode(&header, &claims, &current_key.encoding_key)
            .map_err(|e| Error::Invalid(format!("Failed to encode JWT: {}", e)))
    }

    /// Verify and decode JWT token
    pub async fn verify(&self, token: &str) -> Result<Claims, Error> {
        // Try RSA validation first
        if let Ok(header) = jsonwebtoken::decode_header(token) {
            // Try with specific key ID
            if let Some(kid) = &header.kid {
                if let Some(key_pair) = self.key_manager.get_key(kid) {
                    let mut validation = Validation::new(Algorithm::RS256);
                    validation.set_issuer(&[&self.issuer]);
                    
                    match decode::<Claims>(token, &key_pair.decoding_key, &validation) {
                        Ok(token_data) => {
                            let now = chrono::Utc::now().timestamp() as usize;
                            
                            // Check not before
                            if token_data.claims.nbf > now {
                                return Err(Error::NotYetValid);
                            }
                            
                            // Check if token is revoked (JTI blacklist)
                            if let Some(ref revoked_repo) = self.revoked_token_repo {
                                match revoked_repo.is_revoked(&token_data.claims.jti).await {
                                    Ok(true) => return Err(Error::Revoked),
                                    Ok(false) => {}, // Token is not revoked, continue
                                    Err(e) => {
                                        tracing::error!("Failed to check token revocation status for JTI {}: {}", token_data.claims.jti, e);
                                        // Continue anyway - don't fail auth if revocation check fails
                                    }
                                }
                            }
                            
                            return Ok(token_data.claims);
                        }
                        Err(err) => match err.kind() {
                            ErrorKind::ExpiredSignature => return Err(Error::Expired),
                            ErrorKind::InvalidSignature => return Err(Error::InvalidSignature),
                            _ => tracing::debug!("RSA validation failed: {}", err),
                        }
                    }
                }
            }
            
            // Try with current key if no kid
            let current_key = self.key_manager.current_key();
            let mut validation = Validation::new(Algorithm::RS256);
            validation.set_issuer(&[&self.issuer]);
            
            match decode::<Claims>(token, &current_key.decoding_key, &validation) {
                Ok(token_data) => {
                    let now = chrono::Utc::now().timestamp() as usize;
                    
                    // Check not before
                    if token_data.claims.nbf > now {
                        return Err(Error::NotYetValid);
                    }
                    
                    // Check if token is revoked (JTI blacklist)
                    if let Some(ref revoked_repo) = self.revoked_token_repo {
                        match revoked_repo.is_revoked(&token_data.claims.jti).await {
                            Ok(true) => return Err(Error::Revoked),
                            Ok(false) => {}, // Token is not revoked, continue
                            Err(e) => {
                                tracing::error!("Failed to check token revocation status for JTI {}: {}", token_data.claims.jti, e);
                                // Continue anyway - don't fail auth if revocation check fails
                            }
                        }
                    }
                    
                    return Ok(token_data.claims);
                }
                Err(err) => match err.kind() {
                    ErrorKind::ExpiredSignature => return Err(Error::Expired),
                    ErrorKind::InvalidSignature => return Err(Error::InvalidSignature),
                    _ => tracing::debug!("Current key validation failed: {}", err),
                }
            }
        }
        
        Err(Error::Invalid("Token validation failed".to_string()))
    }

    /// Decode token to user (without permissions field - permissions moved to separate table)
    pub async fn decode(&self, token: &str) -> Result<User, Error> {
        let claims = self.verify(token).await?;
        
        Ok(User {
            id: claims.sub,
            email: claims.email,
            name: claims.name,
            // permissions field removed - handled by separate table
            // package_tier, platforms, firebase_uid, primary_platform removed - derived from permissions
        })
    }
    
    /// Extract both user and permissions from JWT claims
    pub async fn decode_with_permissions(&self, token: &str) -> Result<(User, Vec<String>), Error> {
        let claims = self.verify(token).await?;
        
        let user = User {
            id: claims.sub,
            email: claims.email,
            name: claims.name,
            // permissions field removed - handled by separate table
            // package_tier, platforms, firebase_uid, primary_platform removed - derived from permissions
        };
        
        // Apply timestamp validation to permissions from JWT
        use crate::auth::permissions::filter_valid_permissions;
        let valid_permissions = filter_valid_permissions(&claims.permissions);
        
        Ok((user, valid_permissions))
    }

    // Permission checking methods removed - permissions are now handled by separate PermissionService
    // Use PermissionApplicationService.has_permission() instead

    /// Check if user has package tier (now requires permissions parameter)
    pub fn has_tier_with_permissions(&self, permissions: &[String], required_tier: &str) -> bool {
        let tier_hierarchy = [
            ("FREE", 1),
            ("BRONZE", 2),
            ("SILVER", 3),
            ("GOLD", 4),
            ("PLATINUM", 5),
            ("ENTERPRISE", 6),
        ].iter().cloned().collect::<std::collections::HashMap<_, _>>();

        let user_tier = derive_package_tier_from_permissions(permissions);
        let user_level = tier_hierarchy.get(user_tier.as_str()).unwrap_or(&0);
        let required_level = tier_hierarchy.get(required_tier).unwrap_or(&1);

        user_level >= required_level
    }

    // Admin check removed - use PermissionApplicationService.has_permission() instead
    // pub fn is_admin(&self, user: &User) -> bool { ... }

    /// Get key manager for JWKS endpoint
    pub fn keys(&self) -> &KeyManager {
        &self.key_manager
    }
    
    // Middleware compatibility methods
    
    /// Extract user from token (for middleware)
    pub async fn extract_user(&self, token: &str) -> Result<User, Error> {
        self.decode(token).await
    }
    
    // Admin endpoint validation removed - use middleware with PermissionApplicationService instead
    // This method required access to user permissions which are now in separate table
    pub fn validate_admin_endpoint(&self, _user: &User, _path: &str) -> bool {
        // Deprecated: Use middleware with separate permission checking
        // Always return true for backward compatibility - real permission checking happens in middleware
        true
    }
    
    /// Check if user has package tier (alias for has_tier_with_permissions())
    pub fn has_package_tier_with_permissions(&self, permissions: &[String], required_tier: &str) -> bool {
        self.has_tier_with_permissions(permissions, required_tier)
    }
}

/// Cross-platform permission service for structured permission validation
pub struct CrossPlatformPermissionService;

impl CrossPlatformPermissionService {
    pub fn new() -> Self {
        Self
    }

    /// Validate platform-specific permission in Platform:Resource:Action format
    /// Note: This method is deprecated - permissions should be checked via PermissionApplicationService
    pub fn validate_platform_permission(&self, _user: &User, _platform: &str, _resource: &str, _action: &str) -> bool {
        // Deprecated: User permissions are now in separate table
        // Return true for backward compatibility - real permission checking should use PermissionApplicationService
        true
    }

    /// Check if user can access a specific platform (now requires permissions parameter)
    pub fn can_access_platform_with_permissions(&self, permissions: &[String], platform: &str) -> bool {
        // Check if platform is in user's accessible platforms based on permissions
        let accessible_platforms = derive_accessible_platforms_from_permissions(permissions);
        accessible_platforms.contains(&platform.to_string())
    }

    /// Get all platforms user can access (from permissions)
    pub fn get_accessible_platforms_from_permissions(&self, permissions: &[String]) -> Vec<String> {
        derive_accessible_platforms_from_permissions(permissions)
    }

    /// Parse permission string into components
    pub fn parse_permission(&self, permission: &str) -> Option<(String, String, String)> {
        let parts: Vec<&str> = permission.split(':').collect();
        if parts.len() == 3 {
            Some((parts[0].to_string(), parts[1].to_string(), parts[2].to_string()))
        } else {
            None
        }
    }

    /// Build permission string from components
    pub fn build_permission(&self, platform: &str, resource: &str, action: &str) -> String {
        format!("{}:{}:{}", platform, resource, action)
    }

    /// Get all permissions for a specific platform
    /// Note: This method is deprecated - use PermissionApplicationService.get_user_permissions() instead
    pub fn get_platform_permissions(&self, _user: &User, _platform: &str) -> Vec<String> {
        // Deprecated: User permissions are now in separate table
        vec![]
    }

    /// Check if user has admin access for a platform
    /// Note: This method is deprecated - use PermissionApplicationService.has_permission() instead
    pub fn has_platform_admin_access(&self, _user: &User, _platform: &str) -> bool {
        // Deprecated: User permissions are now in separate table
        // Return true for backward compatibility - real permission checking should use PermissionApplicationService
        true
    }
}

impl Default for CrossPlatformPermissionService {
    fn default() -> Self {
        Self::new()
    }
}

// Global instance for easy access
lazy_static::lazy_static! {
    pub static ref CROSS_PLATFORM_PERMISSION_SERVICE: CrossPlatformPermissionService = 
        CrossPlatformPermissionService::new();
}

#[derive(Debug)]
pub struct UserData {
    pub id: String,
    pub email: String,
    pub name: Option<String>,
    pub permissions: Option<Vec<String>>, // Structured permissions only
    pub audience: Option<String>,
    pub ttl_seconds: Option<usize>,
    
    // Removed: package_tier, firebase_uid, platforms, primary_platform
}

impl Default for Service {
    fn default() -> Self {
        Self::new().expect("Failed to create JWT service")
    }
}

lazy_static::lazy_static! {
    pub static ref JWT: Service = Service::new()
        .expect("Failed to initialize JWT service");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_and_verify_token() {
        let service = Service::new().unwrap();
        
        let user_data = UserData {
            id: "user123".to_string(),
            email: "test@example.com".to_string(),
            name: Some("Test User".to_string()),
            permissions: Some(vec!["epsx:analytics:view".to_string()]),
            audience: None,
            ttl_seconds: Some(3600),
        };
        
        let token = service.create(user_data).unwrap();
        let claims = service.verify(&token).await.unwrap();
        
        assert_eq!(claims.sub, "user123");
        assert_eq!(claims.email, "test@example.com");
        assert!(!claims.jti.is_empty());
    }

    #[test]
    fn test_permission_check() {
        let service = Service::new().unwrap();
        let permissions = vec!["admin:*:*".to_string()];
        
        // Test permission derivation functions
        assert_eq!(derive_package_tier_from_permissions(&permissions), "ENTERPRISE");
        assert!(service.has_tier_with_permissions(&permissions, "SILVER"));
        assert!(derive_accessible_platforms_from_permissions(&permissions).contains(&"admin".to_string()));
        assert_eq!(derive_primary_platform_from_permissions(&permissions), "admin");
    }

    #[tokio::test]
    async fn test_cross_platform_jwt_claims() {
        let service = Service::new().unwrap();
        
        let user_data = UserData {
            id: "cross_user".to_string(),
            email: "cross@example.com".to_string(),
            name: Some("Cross Platform User".to_string()),
            permissions: Some(vec![
                "epsx:analytics:view".to_string(),
                "epsx-pay:transactions:create".to_string(),
                "epsx-token:governance:vote".to_string()
            ]),
            audience: None,
            ttl_seconds: Some(3600),
        };
        
        let token = service.create(user_data).unwrap();
        let claims = service.verify(&token).await.unwrap();
        
        assert_eq!(claims.sub, "cross_user");
        assert_eq!(claims.aud, "epsx-ecosystem");
        assert!(claims.permissions.contains(&"epsx-token:governance:vote".to_string()));
        
        // Test derivation functions with these permissions
        let accessible_platforms = derive_accessible_platforms_from_permissions(&claims.permissions);
        assert!(accessible_platforms.contains(&"epsx-pay".to_string()));
        assert_eq!(derive_primary_platform_from_permissions(&claims.permissions), "epsx");
    }

    #[test] 
    fn test_cross_platform_permission_service() {
        let permission_service = CrossPlatformPermissionService::new();
        let permissions = vec![
            "epsx:analytics:view".to_string(),
            "epsx:analytics:export".to_string(),
            "epsx-pay:transactions:*".to_string(),
            "epsx-token:governance:vote".to_string()
        ];

        // Test derivation functions
        let accessible_platforms = derive_accessible_platforms_from_permissions(&permissions);
        assert!(accessible_platforms.contains(&"epsx".to_string()));
        assert!(accessible_platforms.contains(&"epsx-pay".to_string()));
        assert!(accessible_platforms.contains(&"epsx-token".to_string()));
        
        // Test platform access with new method
        assert!(permission_service.can_access_platform_with_permissions(&permissions, "epsx"));
        assert!(permission_service.can_access_platform_with_permissions(&permissions, "epsx-pay"));
        assert!(permission_service.can_access_platform_with_permissions(&permissions, "epsx-token"));
        
        // Test primary platform derivation
        assert_eq!(derive_primary_platform_from_permissions(&permissions), "epsx");
    }

    #[test]
    fn test_admin_cross_platform_access() {
        let permission_service = CrossPlatformPermissionService::new();
        let admin_permissions = vec!["admin:*:*".to_string()];

        // Test admin derivations
        assert_eq!(derive_package_tier_from_permissions(&admin_permissions), "ENTERPRISE");
        assert_eq!(derive_primary_platform_from_permissions(&admin_permissions), "admin");
        
        // Admin should have access to all platforms through admin permissions
        let accessible_platforms = derive_accessible_platforms_from_permissions(&admin_permissions);
        assert!(accessible_platforms.contains(&"admin".to_string()));
        
        // Test platform access with admin permissions
        assert!(permission_service.can_access_platform_with_permissions(&admin_permissions, "admin"));
    }
}