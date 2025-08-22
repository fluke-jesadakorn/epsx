use jsonwebtoken::{decode, encode, Header, DecodingKey, Validation, Algorithm, errors::ErrorKind};
use serde::{Deserialize, Serialize};
use super::key_manager::KeyManager;
use std::sync::Arc;
use crate::config::env::get_env_var;

/**
 * Modern JWT handling for Auth.js v5 integration
 * Replaces complex Casbin policy system with simple JWT-based permissions
 */

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EPSXClaims {
    pub sub: String,  // User ID
    pub email: String,
    pub name: Option<String>,
    pub admin_modules: Vec<String>,
    pub permissions: Vec<String>,
    pub package_tier: String,
    pub role: String,
    pub firebase_uid: Option<String>,
    pub exp: usize,
    pub iat: usize,
}

#[derive(Debug, Clone)]
pub struct AuthenticatedUser {
    pub user_id: String,
    pub email: String,
    pub name: Option<String>,
    pub admin_modules: Vec<String>,
    pub permissions: Vec<String>,
    pub package_tier: String,
    pub role: String,
    pub firebase_uid: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum JWTError {
    #[error("Invalid token: {0}")]
    InvalidToken(String),
    #[error("Token expired")]
    Expired,
    #[error("Missing claims: {0}")]
    MissingClaims(String),
    #[error("Invalid signature")]
    InvalidSignature,
    #[error("Permission denied")]
    PermissionDenied,
}

pub struct JWTService {
    key_manager: Arc<KeyManager>,
    // Legacy support for HMAC-based tokens during transition
    legacy_secret: Option<String>,
}

impl JWTService {
    pub fn new() -> Result<Self, JWTError> {
        let key_manager = Arc::new(KeyManager::from_env_or_generate()
            .map_err(|e| JWTError::InvalidToken(format!("Failed to initialize KeyManager: {}", e)))?); 
            
        // Support legacy HMAC tokens during transition
        let legacy_secret = get_env_var("JWT_SECRET")
            .or_else(|_| get_env_var("JWT_SECRET"))
            .ok();
            
        if legacy_secret.is_some() {
            tracing::warn!("Using legacy HMAC secret for JWT backwards compatibility. Consider migrating to RSA keys.");
        }
        
        Ok(Self {
            key_manager,
            legacy_secret,
        })
    }

    /**
     * Validate and decode JWT token (supports both RSA and legacy HMAC)
     */
    pub fn validate_token(&self, token: &str) -> Result<EPSXClaims, JWTError> {
        // First try to decode header to get kid (Key ID)
        let header = jsonwebtoken::decode_header(token)
            .map_err(|e| JWTError::InvalidToken(format!("Invalid token header: {}", e)))?;
            
        // Try RSA validation first
        if let Some(kid) = &header.kid {
            if let Some(key_pair) = self.key_manager.get_key(kid) {
                let validation = Validation::new(Algorithm::RS256);
                match decode::<EPSXClaims>(token, &key_pair.decoding_key, &validation) {
                    Ok(token_data) => return Ok(token_data.claims),
                    Err(err) => {
                        tracing::debug!("RSA token validation failed for kid {}: {}", kid, err);
                    }
                }
            }
        } else {
            // Try with current key if no kid specified
            let current_key = self.key_manager.current_key();
            let validation = Validation::new(Algorithm::RS256);
            match decode::<EPSXClaims>(token, &current_key.decoding_key, &validation) {
                Ok(token_data) => return Ok(token_data.claims),
                Err(err) => {
                    tracing::debug!("RSA token validation failed with current key: {}", err);
                }
            }
        }
        
        // Fallback to legacy HMAC validation
        if let Some(secret) = &self.legacy_secret {
            let key = DecodingKey::from_secret(secret.as_ref());
            let validation = Validation::new(Algorithm::HS256);
            
            match decode::<EPSXClaims>(token, &key, &validation) {
                Ok(token_data) => {
                    tracing::debug!("Successfully validated legacy HMAC token");
                    return Ok(token_data.claims);
                }
                Err(err) => match err.kind() {
                    ErrorKind::ExpiredSignature => return Err(JWTError::Expired),
                    ErrorKind::InvalidSignature => return Err(JWTError::InvalidSignature),
                    _ => {
                        tracing::debug!("Legacy HMAC token validation failed: {}", err);
                    }
                }
            }
        }
        
        Err(JWTError::InvalidToken("Token validation failed with all available methods".to_string()))
    }

    /**
     * Extract authenticated user from JWT claims
     */
    pub fn extract_user(&self, token: &str) -> Result<AuthenticatedUser, JWTError> {
        let claims = self.validate_token(token)?;

        Ok(AuthenticatedUser {
            user_id: claims.sub.clone(),
            email: claims.email.clone(),
            name: claims.name.clone(),
            admin_modules: claims.admin_modules.clone(),
            permissions: claims.permissions.clone(),
            package_tier: claims.package_tier.clone(),
            role: claims.role.clone(),
            firebase_uid: claims.firebase_uid.clone(),
        })
    }

    /**
     * Check if user has specific permission
     * Replaces complex Casbin policy evaluation
     */
    pub fn has_permission(&self, user: &AuthenticatedUser, permission: &str) -> bool {
        // Check exact match
        if user.permissions.contains(&permission.to_string()) {
            return true;
        }

        // Check wildcard permissions
        for user_permission in &user.permissions {
            if user_permission.ends_with(".*") || user_permission.ends_with(":*") {
                let prefix = &user_permission[..user_permission.len() - 2];
                if permission.starts_with(&format!("{}.", prefix)) || 
                   permission.starts_with(&format!("{}:", prefix)) {
                    return true;
                }
            }
            if user_permission == "*" {
                return true;
            }
        }

        false
    }

    /**
     * Check if user has specific admin module
     */
    pub fn has_admin_module(&self, user: &AuthenticatedUser, module: &str) -> bool {
        user.admin_modules.contains(&module.to_string())
    }

    /**
     * Check if user has package tier or higher
     */
    pub fn has_package_tier(&self, user: &AuthenticatedUser, required_tier: &str) -> bool {
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

    /**
     * Check role hierarchy
     */
    pub fn has_role(&self, user: &AuthenticatedUser, required_role: &str) -> bool {
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

    /**
     * Check if user is admin (has any admin modules)
     */
    pub fn is_admin(&self, user: &AuthenticatedUser) -> bool {
        !user.admin_modules.is_empty()
    }

    /**
     * Check if user is system admin
     */
    pub fn is_system_admin(&self, user: &AuthenticatedUser) -> bool {
        self.has_admin_module(user, "system_admin") || user.role == "super_admin"
    }

    /**
     * Validate admin access for specific endpoint
     * Replaces complex admin module service logic
     */
    pub fn validate_admin_endpoint(&self, user: &AuthenticatedUser, path: &str) -> bool {
        // System admin has access to everything
        if self.is_system_admin(user) {
            return true;
        }

        // Map endpoints to required admin modules
        let endpoint_modules = [
            ("/api/admin/users", "user_operations"),
            ("/api/admin/permissions", "permission_admin"),
            ("/api/admin/billing", "billing_admin"),
            ("/api/admin/analytics", "analytics_specialist"),
            ("/api/admin/modules", "module_coordinator"),
            ("/api/admin/developer", "developer_relations"),
            ("/api/admin/audit", "compliance_audit"),
            ("/api/admin/support", "support_specialist"),
        ];

        for (endpoint_prefix, required_module) in &endpoint_modules {
            if path.starts_with(endpoint_prefix) {
                return self.has_admin_module(user, required_module);
            }
        }

        // Default: require any admin module for admin endpoints
        path.starts_with("/api/admin") && self.is_admin(user)
    }

    /**
     * Create and sign JWT token with RSA key
     */
    pub fn create_token(&self, user_data: UserClaimsInput) -> Result<String, JWTError> {
        let claims = self.create_user_claims(user_data);
        let current_key = self.key_manager.current_key();
        
        // Create header with key ID
        let mut header = Header::new(Algorithm::RS256);
        header.kid = Some(current_key.kid.clone());
        
        encode(&header, &claims, &current_key.encoding_key)
            .map_err(|e| JWTError::InvalidToken(format!("Failed to encode JWT: {}", e)))
    }
    
    /**
     * Create user claims for JWT generation (used during login)
     */
    pub fn create_user_claims(&self, user_data: UserClaimsInput) -> EPSXClaims {
        let now = chrono::Utc::now().timestamp() as usize;
        
        EPSXClaims {
            sub: user_data.user_id,
            email: user_data.email,
            name: user_data.name,
            admin_modules: user_data.admin_modules.unwrap_or_default(),
            permissions: user_data.permissions.unwrap_or_else(|| vec!["user:read".to_string()]),
            package_tier: user_data.package_tier.unwrap_or_else(|| "FREE".to_string()),
            role: user_data.role.unwrap_or_else(|| "user".to_string()),
            firebase_uid: user_data.firebase_uid,
            exp: now + 7200, // 2 hours
            iat: now,
        }
    }
    
    /**
     * Get the current key manager (for JWKS endpoint)
     */
    pub fn key_manager(&self) -> &KeyManager {
        &self.key_manager
    }
    
    /**
     * Rotate JWT signing keys
     */
    pub fn rotate_keys(&mut self) -> Result<String, JWTError> {
        Arc::get_mut(&mut self.key_manager)
            .ok_or_else(|| JWTError::InvalidToken("Cannot rotate keys: KeyManager is shared".to_string()))?
            .rotate_keys()
            .map_err(|e| JWTError::InvalidToken(format!("Key rotation failed: {}", e)))
    }
}

#[derive(Debug)]
pub struct UserClaimsInput {
    pub user_id: String,
    pub email: String,
    pub name: Option<String>,
    pub admin_modules: Option<Vec<String>>,
    pub permissions: Option<Vec<String>>,
    pub package_tier: Option<String>,
    pub role: Option<String>,
    pub firebase_uid: Option<String>,
}

impl Default for JWTService {
    fn default() -> Self {
        Self::new().expect("Failed to create JWT service")
    }
}

lazy_static::lazy_static! {
    pub static ref JWT_SERVICE: JWTService = JWTService::new()
        .expect("Failed to initialize JWT service");
}