use jsonwebtoken::{decode, encode, Header, DecodingKey, Validation, Algorithm, errors::ErrorKind};
use serde::{Deserialize, Serialize};
use super::key_manager::KeyManager;
use std::sync::Arc;
use uuid::Uuid;

/// Complete JWT claims following RFC 7519 standard
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
    
    // Authorization
    pub role: String,
    pub permissions: Vec<String>,
    pub admin_modules: Vec<String>,
    pub package_tier: String,
    
    // Firebase integration
    pub firebase_uid: Option<String>,
}

#[derive(Debug, Clone)]
pub struct User {
    pub id: String,
    pub email: String,
    pub name: Option<String>,
    pub role: String,
    pub permissions: Vec<String>,
    pub admin_modules: Vec<String>,
    pub package_tier: String,
    pub firebase_uid: Option<String>,
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
    legacy_secret: Option<String>,
    issuer: String,
}

impl Service {
    pub fn new() -> Result<Self, Error> {
        let key_manager = Arc::new(KeyManager::from_env_or_generate()
            .map_err(|e| Error::Invalid(format!("Failed to initialize KeyManager: {}", e)))?);
            
        let legacy_secret = std::env::var("JWT_SECRET")
            .or_else(|_| std::env::var("NEXTAUTH_SECRET"))
            .ok();
            
        let issuer = std::env::var("OIDC_ISSUER")
            .unwrap_or_else(|_| "http://localhost:8080".to_string());
            
        if legacy_secret.is_some() {
            tracing::warn!("Using legacy HMAC secret for JWT backwards compatibility");
        }
        
        Ok(Self {
            key_manager,
            legacy_secret,
            issuer,
        })
    }

    /// Create JWT token with complete standard claims
    pub fn create(&self, user_data: UserData) -> Result<String, Error> {
        let now = chrono::Utc::now().timestamp() as usize;
        
        let claims = Claims {
            // Standard JWT claims
            sub: user_data.id.clone(),
            iss: self.issuer.clone(),
            aud: user_data.audience.unwrap_or_else(|| "epsx-api".to_string()),
            exp: now + user_data.ttl_seconds.unwrap_or(7200), // 2 hours default
            iat: now,
            nbf: now, // Valid immediately
            jti: Uuid::new_v4().to_string(), // Unique ID for revocation
            
            // User information
            email: user_data.email,
            name: user_data.name,
            
            // Authorization
            role: user_data.role.unwrap_or_else(|| "user".to_string()),
            permissions: user_data.permissions.unwrap_or_default(),
            admin_modules: user_data.admin_modules.unwrap_or_default(),
            package_tier: user_data.package_tier.unwrap_or_else(|| "FREE".to_string()),
            
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
        
        // Fallback to legacy HMAC
        if let Some(secret) = &self.legacy_secret {
            let key = DecodingKey::from_secret(secret.as_ref());
            let validation = Validation::new(Algorithm::HS256);
            
            match decode::<Claims>(token, &key, &validation) {
                Ok(token_data) => {
                    tracing::debug!("Successfully validated legacy HMAC token");
                    return Ok(token_data.claims);
                }
                Err(err) => match err.kind() {
                    ErrorKind::ExpiredSignature => return Err(Error::Expired),
                    ErrorKind::InvalidSignature => return Err(Error::InvalidSignature),
                    _ => tracing::debug!("Legacy HMAC validation failed: {}", err),
                }
            }
        }
        
        Err(Error::Invalid("Token validation failed".to_string()))
    }

    /// Decode token to user
    pub fn decode(&self, token: &str) -> Result<User, Error> {
        let claims = self.verify(token)?;
        
        Ok(User {
            id: claims.sub,
            email: claims.email,
            name: claims.name,
            role: claims.role,
            permissions: claims.permissions,
            admin_modules: claims.admin_modules,
            package_tier: claims.package_tier,
            firebase_uid: claims.firebase_uid,
        })
    }

    /// Check if user can access resource
    pub fn can(&self, user: &User, permission: &str) -> bool {
        // Check exact match
        if user.permissions.contains(&permission.to_string()) {
            return true;
        }

        // Check wildcard permissions
        for user_permission in &user.permissions {
            if user_permission.ends_with("*") {
                let prefix = &user_permission[..user_permission.len() - 1];
                if permission.starts_with(prefix) {
                    return true;
                }
            }
        }

        false
    }

    /// Check if user has role
    pub fn has_role(&self, user: &User, required_role: &str) -> bool {
        let role_hierarchy = [
            ("user", 1),
            ("premium", 2),
            ("moderator", 3),
            ("admin", 4),
            ("super_admin", 5),
        ].iter().cloned().collect::<std::collections::HashMap<_, _>>();

        let user_level = role_hierarchy.get(user.role.to_lowercase().as_str()).unwrap_or(&0);
        let required_level = role_hierarchy.get(required_role.to_lowercase().as_str()).unwrap_or(&1);

        user_level >= required_level
    }

    /// Check if user has module access
    pub fn has_module(&self, user: &User, module: &str) -> bool {
        user.admin_modules.contains(&module.to_string())
    }

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

    /// Get key manager for JWKS endpoint
    pub fn keys(&self) -> &KeyManager {
        &self.key_manager
    }
    
    // Middleware compatibility methods
    
    /// Extract user from token (for middleware)
    pub fn extract_user(&self, token: &str) -> Result<User, Error> {
        self.decode(token)
    }
    
    /// Validate admin endpoint access
    pub fn validate_admin_endpoint(&self, user: &User, path: &str) -> bool {
        // Check if user has admin role or full admin modules
        if self.has_role(user, "admin") {
            return true;
        }
        
        // Check if user has admin-full-004 profile
        if user.admin_modules.contains(&"admin-full-004".to_string()) {
            return true;
        }
        
        // Check specific admin modules for path-based access
        if path.contains("/admin/users") && user.admin_modules.contains(&"user-management".to_string()) {
            return true;
        }
        
        if path.contains("/admin/analytics") && user.admin_modules.contains(&"analytics-access".to_string()) {
            return true;
        }
        
        if path.contains("/admin/reports") && user.admin_modules.contains(&"reporting-access".to_string()) {
            return true;
        }
        
        if path.contains("/admin/audit") && user.admin_modules.contains(&"audit-logs".to_string()) {
            return true;
        }
        
        false
    }
    
    /// Check if user has permission (alias for can())
    pub fn has_permission(&self, user: &User, permission: &str) -> bool {
        self.can(user, permission)
    }
    
    /// Check if user has package tier (alias for has_tier())
    pub fn has_package_tier(&self, user: &User, required_tier: &str) -> bool {
        self.has_tier(user, required_tier)
    }
}

#[derive(Debug)]
pub struct UserData {
    pub id: String,
    pub email: String,
    pub name: Option<String>,
    pub role: Option<String>,
    pub permissions: Option<Vec<String>>,
    pub admin_modules: Option<Vec<String>>,
    pub package_tier: Option<String>,
    pub firebase_uid: Option<String>,
    pub audience: Option<String>,
    pub ttl_seconds: Option<usize>,
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
            role: Some("user".to_string()),
            permissions: Some(vec!["read".to_string()]),
            admin_modules: Some(vec![]),
            package_tier: Some("FREE".to_string()),
            firebase_uid: None,
            audience: None,
            ttl_seconds: Some(3600),
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
        let user = User {
            id: "user123".to_string(),
            email: "test@example.com".to_string(),
            name: None,
            role: "admin".to_string(),
            permissions: vec!["admin:*".to_string()],
            admin_modules: vec!["user_management".to_string()],
            package_tier: "GOLD".to_string(),
            firebase_uid: None,
        };
        
        assert!(service.can(&user, "admin:read"));
        assert!(service.has_role(&user, "user"));
        assert!(service.has_module(&user, "user_management"));
        assert!(service.has_tier(&user, "SILVER"));
    }
}