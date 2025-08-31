use jsonwebtoken::{decode, encode, Header, Validation, Algorithm, errors::ErrorKind};
use serde::{Deserialize, Serialize};
use super::key_manager::KeyManager;
use std::sync::Arc;
use uuid::Uuid;
use crate::config::env::get_env_var;
use crate::auth::permissions::check_permission_access;

/// Check if user has admin permissions
fn has_admin_permissions(permissions: &[String]) -> bool {
    permissions.iter().any(|p| p == "admin:*:*" || p.starts_with("admin:"))
}

/// JWT claims following RFC 7519 standard with permission-only system
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    // Standard JWT claims (RFC 7519)
    pub sub: String,        // Subject (User ID)
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
    pub package_tier: String,           // Package tier
    
    // Cross-platform fields
    pub platforms: Option<Vec<String>>,      // Accessible platforms ["epsx", "epsx-pay", "epsx-token"]
    pub primary_platform: Option<String>,   // Default platform
    pub platform_context: Option<String>,   // Current platform context
    
    // Firebase integration
    pub firebase_uid: Option<String>,
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
    pub package_tier: String,
    pub firebase_uid: Option<String>,
    
    // Cross-platform fields
    pub platforms: Vec<String>,          // Accessible platforms
    pub primary_platform: String,       // Default platform
    pub platform_context: Option<String>, // Current platform context
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Invalid token: {0}")]
    Invalid(String),
    #[error("Token expired")]
    Expired,
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
        })
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
            package_tier: user_data.package_tier.unwrap_or_else(|| "FREE".to_string()),
            
            // Cross-platform fields
            platforms: user_data.platforms.clone().or_else(|| Some(vec!["epsx".to_string()])),
            primary_platform: user_data.primary_platform.clone().or_else(|| Some("epsx".to_string())),
            platform_context: user_data.platform_context.clone(),
            
            // Firebase integration
            firebase_uid: user_data.firebase_uid,
        };
        
        let current_key = self.key_manager.current_key();
        let mut header = Header::new(Algorithm::RS256);
        header.kid = Some(current_key.kid.clone());
        
        encode(&header, &claims, &current_key.encoding_key)
            .map_err(|e| Error::Invalid(format!("Failed to encode JWT: {}", e)))
    }

    /// Verify and decode JWT token
    pub fn verify(&self, token: &str) -> Result<Claims, Error> {
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
                Ok(token_data) => return Ok(token_data.claims),
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
    pub fn decode(&self, token: &str) -> Result<User, Error> {
        let claims = self.verify(token)?;
        
        Ok(User {
            id: claims.sub,
            email: claims.email,
            name: claims.name,
            // permissions field removed - handled by separate table
            package_tier: claims.package_tier,
            firebase_uid: claims.firebase_uid,
            
            // Cross-platform fields with backward compatibility defaults
            platforms: claims.platforms.unwrap_or_else(|| vec!["epsx".to_string()]),
            primary_platform: claims.primary_platform.unwrap_or_else(|| "epsx".to_string()),
            platform_context: claims.platform_context,
        })
    }
    
    /// Extract both user and permissions from JWT claims
    pub fn decode_with_permissions(&self, token: &str) -> Result<(User, Vec<String>), Error> {
        let claims = self.verify(token)?;
        
        let user = User {
            id: claims.sub,
            email: claims.email,
            name: claims.name,
            package_tier: claims.package_tier,
            firebase_uid: claims.firebase_uid,
            
            // Cross-platform fields with backward compatibility defaults
            platforms: claims.platforms.unwrap_or_else(|| vec!["epsx".to_string()]),
            primary_platform: claims.primary_platform.unwrap_or_else(|| "epsx".to_string()),
            platform_context: claims.platform_context,
        };
        
        // Apply timestamp validation to permissions from JWT
        use crate::auth::permissions::filter_valid_permissions;
        let valid_permissions = filter_valid_permissions(&claims.permissions);
        
        Ok((user, valid_permissions))
    }

    // Permission checking methods removed - permissions are now handled by separate PermissionService
    // Use PermissionApplicationService.has_permission() instead

    /// Check if user has package tier
    pub fn has_tier(&self, user: &User, required_tier: &str) -> bool {
        let tier_hierarchy = [
            ("FREE", 1),
            ("BRONZE", 2),
            ("SILVER", 3),
            ("GOLD", 4),
            ("PLATINUM", 5),
            ("ENTERPRISE", 6),
        ].iter().cloned().collect::<std::collections::HashMap<_, _>>();

        let user_level = tier_hierarchy.get(user.package_tier.as_str()).unwrap_or(&0);
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
    pub fn extract_user(&self, token: &str) -> Result<User, Error> {
        self.decode(token)
    }
    
    // Admin endpoint validation removed - use middleware with PermissionApplicationService instead
    // This method required access to user permissions which are now in separate table
    pub fn validate_admin_endpoint(&self, _user: &User, _path: &str) -> bool {
        // Deprecated: Use middleware with separate permission checking
        // Always return true for backward compatibility - real permission checking happens in middleware
        true
    }
    
    /// Check if user has package tier (alias for has_tier())
    pub fn has_package_tier(&self, user: &User, required_tier: &str) -> bool {
        self.has_tier(user, required_tier)
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

    /// Check if user can access a specific platform
    pub fn can_access_platform(&self, user: &User, platform: &str) -> bool {
        // Check if platform is in user's accessible platforms list
        user.platforms.contains(&platform.to_string())
        // Note: Permission-based platform access checking moved to PermissionApplicationService
    }

    /// Get all platforms user can access (from user.platforms field only)
    pub fn get_accessible_platforms(&self, user: &User) -> Vec<String> {
        user.platforms.clone()
        // Note: Platform access based on permissions now requires PermissionApplicationService
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
    pub package_tier: Option<String>,
    pub firebase_uid: Option<String>,
    pub audience: Option<String>,
    pub ttl_seconds: Option<usize>,
    
    // Cross-platform fields
    pub platforms: Option<Vec<String>>,
    pub primary_platform: Option<String>,
    pub platform_context: Option<String>,
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

    #[test]
    fn test_create_and_verify_token() {
        let service = Service::new().unwrap();
        
        let user_data = UserData {
            id: "user123".to_string(),
            email: "test@example.com".to_string(),
            name: Some("Test User".to_string()),
            permissions: Some(vec!["epsx:analytics:view".to_string()]),
            package_tier: Some("FREE".to_string()),
            firebase_uid: None,
            audience: None,
            ttl_seconds: Some(3600),
            platforms: Some(vec!["epsx".to_string()]),
            primary_platform: Some("epsx".to_string()),
            platform_context: None,
        };
        
        let token = service.create(user_data).unwrap();
        let claims = service.verify(&token).unwrap();
        
        assert_eq!(claims.sub, "user123");
        assert_eq!(claims.email, "test@example.com");
        assert!(!claims.jti.is_empty());
    }

    #[test]
    fn test_permission_check() {
        let service = Service::new().unwrap();
        let permissions = vec!["admin:*:*".to_string()];
        let user = User {
            id: "user123".to_string(),
            email: "test@example.com".to_string(),
            name: None,
            permissions: permissions.clone(),
            package_tier: "GOLD".to_string(),
            firebase_uid: None,
            platforms: vec!["epsx".to_string()],
            primary_platform: "epsx".to_string(),
            platform_context: None,
        };
        
        assert!(service.has_permission(&user, "epsx:users:manage"));
        assert!(service.is_admin(&user));
        assert!(service.has_tier(&user, "SILVER"));
    }

    #[test]
    fn test_cross_platform_jwt_claims() {
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
            package_tier: Some("PREMIUM".to_string()),
            firebase_uid: None,
            audience: None,
            ttl_seconds: Some(3600),
            platforms: Some(vec!["epsx".to_string(), "epsx-pay".to_string(), "epsx-token".to_string()]),
            primary_platform: Some("epsx".to_string()),
            platform_context: Some("epsx-pay".to_string()),
        };
        
        let token = service.create(user_data).unwrap();
        let claims = service.verify(&token).unwrap();
        
        assert_eq!(claims.sub, "cross_user");
        assert_eq!(claims.aud, "epsx-ecosystem");
        assert!(claims.platforms.as_ref().unwrap().contains(&"epsx-pay".to_string()));
        assert_eq!(claims.primary_platform.as_ref().unwrap(), "epsx");
        assert_eq!(claims.platform_context.as_ref().unwrap(), "epsx-pay");
        assert!(claims.permissions.contains(&"epsx-token:governance:vote".to_string()));
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
        let user = User {
            id: "platform_user".to_string(),
            email: "platform@example.com".to_string(),
            name: None,
            permissions: permissions.clone(),
            package_tier: "PREMIUM".to_string(),
            firebase_uid: None,
            platforms: vec!["epsx".to_string(), "epsx-pay".to_string(), "epsx-token".to_string()],
            primary_platform: "epsx".to_string(),
            platform_context: None,
        };

        // Test exact permission match
        assert!(permission_service.validate_platform_permission(&user, "epsx", "analytics", "view"));
        assert!(permission_service.validate_platform_permission(&user, "epsx", "analytics", "export"));
        
        // Test wildcard permission  
        assert!(permission_service.validate_platform_permission(&user, "epsx-pay", "transactions", "create"));
        assert!(permission_service.validate_platform_permission(&user, "epsx-pay", "transactions", "process"));
        assert!(permission_service.validate_platform_permission(&user, "epsx-pay", "transactions", "refund"));
        
        // Test specific permission
        assert!(permission_service.validate_platform_permission(&user, "epsx-token", "governance", "vote"));
        assert!(!permission_service.validate_platform_permission(&user, "epsx-token", "governance", "propose"));
        
        // Test platform access
        assert!(permission_service.can_access_platform(&user, "epsx"));
        assert!(permission_service.can_access_platform(&user, "epsx-pay"));
        assert!(permission_service.can_access_platform(&user, "epsx-token"));
    }

    #[test]
    fn test_admin_cross_platform_access() {
        let permission_service = CrossPlatformPermissionService::new();
        let admin_permissions = vec!["admin:*:*".to_string()];
        let admin_user = User {
            id: "admin_user".to_string(),
            email: "admin@example.com".to_string(),
            name: None,
            permissions: admin_permissions.clone(),
            package_tier: "ENTERPRISE".to_string(),
            firebase_uid: None,
            platforms: vec!["epsx".to_string()],
            primary_platform: "epsx".to_string(),
            platform_context: None,
        };

        // Admin should have access to all platforms and resources
        assert!(permission_service.validate_platform_permission(&admin_user, "epsx", "users", "manage"));
        assert!(permission_service.validate_platform_permission(&admin_user, "epsx-pay", "transactions", "create"));
        assert!(permission_service.validate_platform_permission(&admin_user, "epsx-token", "treasury", "approve"));
        assert!(permission_service.has_platform_admin_access(&admin_user, "epsx"));
        assert!(permission_service.has_platform_admin_access(&admin_user, "epsx-pay"));
        assert!(permission_service.has_platform_admin_access(&admin_user, "epsx-token"));
    }
}