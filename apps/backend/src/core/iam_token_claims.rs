use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use crate::core::permission_constants::{get_permissions_for_modules, AdminModuleValidator};
use crate::config::env::get_env_var;

/// IAM-enhanced access token claims for OpenID Connect tokens
/// Includes granular admin modules, permissions, and subscription information
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AccessTokenClaims {
    // Standard OpenID Connect claims
    pub sub: String,                    // Subject (Firebase UID)
    pub iss: String,                    // Issuer (our backend URL)
    pub aud: Vec<String>,              // Audience (client IDs)
    pub iat: i64,                      // Issued at (Unix timestamp)
    pub exp: i64,                      // Expires at (Unix timestamp)
    pub nbf: i64,                      // Not before (Unix timestamp)
    pub jti: String,                   // JWT ID (for revocation)
    
    // User identity claims
    pub email: String,                 // User's email address
    pub email_verified: bool,          // Email verification status
    pub name: Option<String>,          // User's display name
    pub picture: Option<String>,       // User's profile picture URL
    
    // Legacy role system (for backward compatibility)
    pub role: String,                  // Legacy role (admin, user, moderator, etc.)
    
    // Enhanced IAM claims
    pub admin_modules: Vec<String>,    // Granular admin module codes
    pub permissions: Vec<String>,      // Computed permissions from modules
    pub access_level: String,          // Effective access level (read, write, admin)
    
    // Subscription and billing
    pub subscription_tier: String,     // Subscription tier (basic, premium, enterprise)
    pub subscription_status: String,   // Active, suspended, expired, etc.
    pub subscription_expires_at: Option<DateTime<Utc>>, // Subscription expiry
    
    // Session information
    pub session_id: String,            // Unique session identifier
    pub session_type: String,          // Session type (admin, user, api)
    pub client_id: String,            // OAuth client ID
    pub scope: String,                // Granted scopes
    
    // Security and audit
    pub auth_time: i64,               // Authentication timestamp
    pub amr: Vec<String>,             // Authentication Method Reference
    pub acr: String,                  // Authentication Context Class Reference
    pub nonce: Option<String>,        // Nonce for replay protection
    
    // Metadata
    pub ip_address: Option<String>,   // Client IP address
    pub user_agent: Option<String>,   // Client user agent
    pub risk_score: Option<f64>,      // Authentication risk score (0.0-1.0)
}

/// ID token claims for user identity (separate from access token)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IdTokenClaims {
    // Standard OpenID Connect ID token claims
    pub sub: String,                  // Subject (Firebase UID)
    pub iss: String,                  // Issuer (our backend URL)  
    pub aud: String,                  // Audience (client ID)
    pub iat: i64,                     // Issued at
    pub exp: i64,                     // Expires at
    pub nbf: i64,                     // Not before
    
    // User identity
    pub email: String,                // User's email
    pub email_verified: bool,         // Email verification status
    pub name: Option<String>,         // Display name
    pub given_name: Option<String>,   // First name
    pub family_name: Option<String>,  // Last name
    pub picture: Option<String>,      // Profile picture URL
    pub locale: Option<String>,       // User's locale
    
    // Authentication context
    pub auth_time: i64,              // When authentication occurred
    pub nonce: Option<String>,       // Nonce for client validation
    pub amr: Vec<String>,            // Authentication methods used
    pub acr: String,                 // Authentication context class
}

/// Claims for refresh tokens
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RefreshTokenClaims {
    pub sub: String,                 // Subject (Firebase UID)
    pub iss: String,                 // Issuer
    pub aud: Vec<String>,           // Audiences
    pub iat: i64,                    // Issued at
    pub exp: i64,                    // Expires at (longer TTL)
    pub jti: String,                 // JWT ID
    pub session_id: String,          // Session ID
    pub client_id: String,           // Client ID
    pub scope: String,               // Original scopes
    pub token_family: String,        // Token family for rotation
}

impl AccessTokenClaims {
    /// Create new access token claims from user data and IAM information
    pub fn new(
        user_id: String,
        email: String,
        name: Option<String>,
        admin_modules: Vec<String>,
        subscription_tier: String,
        client_id: String,
        scope: String,
        expires_in_seconds: i64,
    ) -> Self {
        let now = Utc::now();
        let iat = now.timestamp();
        let exp = iat + expires_in_seconds;
        
        // Compute permissions from admin modules
        let permissions = get_permissions_for_modules(&admin_modules);
        
        // Determine effective access level
        let access_level = AdminModuleValidator::get_effective_access_level(&admin_modules);
        
        // Determine legacy role for backward compatibility
        let role = if admin_modules.is_empty() {
            "user".to_string()
        } else if access_level == "admin" {
            "admin".to_string()  
        } else {
            "moderator".to_string()
        };

        Self {
            // Standard claims
            sub: user_id.clone(),
            iss: get_env_var("OIDC_ISSUER").unwrap_or_else(|_| "http://localhost:8080".to_string()),
            aud: vec![client_id.clone()],
            iat,
            exp,
            nbf: iat,
            jti: uuid::Uuid::new_v4().to_string(),
            
            // User identity
            email: email.clone(),
            email_verified: true, // Assume verified since coming from Firebase
            name,
            picture: None,
            
            // Legacy role
            role,
            
            // Enhanced IAM
            admin_modules: admin_modules.clone(),
            permissions,
            access_level,
            
            // Subscription
            subscription_tier,
            subscription_status: "active".to_string(),
            subscription_expires_at: None,
            
            // Session
            session_id: uuid::Uuid::new_v4().to_string(),
            session_type: if admin_modules.is_empty() { "user".to_string() } else { "admin".to_string() },
            client_id,
            scope,
            
            // Security
            auth_time: iat,
            amr: vec!["firebase".to_string()],
            acr: "1".to_string(), // Single-factor authentication
            nonce: None,
            
            // Metadata
            ip_address: None,
            user_agent: None,
            risk_score: Some(0.1), // Low risk by default
        }
    }
    
    /// Check if the token has a specific permission
    pub fn has_permission(&self, permission: &str) -> bool {
        self.permissions.iter().any(|p| p == permission)
    }
    
    /// Check if the token has a specific admin module
    pub fn has_admin_module(&self, module: &str) -> bool {
        self.admin_modules.iter().any(|m| m == module)
    }
    
    /// Check if the token has admin access
    pub fn is_admin(&self) -> bool {
        !self.admin_modules.is_empty()
    }
    
    /// Check if the token has premium subscription
    pub fn has_premium_access(&self) -> bool {
        matches!(
            self.subscription_tier.as_str(),
            "premium" | "enterprise" | "platinum" | "gold"
        )
    }
    
    /// Get remaining token lifetime in seconds
    pub fn remaining_lifetime(&self) -> i64 {
        self.exp - Utc::now().timestamp()
    }
    
    /// Check if token is expired
    pub fn is_expired(&self) -> bool {
        Utc::now().timestamp() > self.exp
    }
    
    /// Check if token is valid (not expired and not before nbf)
    pub fn is_valid(&self) -> bool {
        let now = Utc::now().timestamp();
        now >= self.nbf && now < self.exp
    }
}

impl IdTokenClaims {
    /// Create new ID token claims from user data
    pub fn new(
        user_id: String,
        email: String,
        name: Option<String>,
        client_id: String,
        expires_in_seconds: i64,
        nonce: Option<String>,
    ) -> Self {
        let now = Utc::now();
        let iat = now.timestamp();
        let exp = iat + expires_in_seconds;
        
        Self {
            sub: user_id,
            iss: get_env_var("OIDC_ISSUER").unwrap_or_else(|_| "http://localhost:8080".to_string()),
            aud: client_id,
            iat,
            exp,
            nbf: iat,
            
            email: email.clone(),
            email_verified: true,
            name,
            given_name: None,
            family_name: None,
            picture: None,
            locale: Some("en-US".to_string()),
            
            auth_time: iat,
            nonce,
            amr: vec!["firebase".to_string()],
            acr: "1".to_string(),
        }
    }
}

impl RefreshTokenClaims {
    /// Create new refresh token claims
    pub fn new(
        user_id: String,
        client_id: String,
        scope: String,
        session_id: String,
        expires_in_seconds: i64,
    ) -> Self {
        let now = Utc::now();
        let iat = now.timestamp();
        let exp = iat + expires_in_seconds;
        
        Self {
            sub: user_id,
            iss: get_env_var("OIDC_ISSUER").unwrap_or_else(|_| "http://localhost:8080".to_string()),
            aud: vec![client_id.clone()],
            iat,
            exp,
            jti: uuid::Uuid::new_v4().to_string(),
            session_id,
            client_id,
            scope,
            token_family: uuid::Uuid::new_v4().to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_access_token_claims_creation() {
        let claims = AccessTokenClaims::new(
            "user123".to_string(),
            "user@example.com".to_string(),
            Some("Test User".to_string()),
            vec!["user_operations".to_string(), "analytics_specialist".to_string()],
            "premium".to_string(),
            "epsx-frontend".to_string(),
            "openid profile email".to_string(),
            3600,
        );
        
        assert_eq!(claims.sub, "user123");
        assert_eq!(claims.email, "user@example.com");
        assert_eq!(claims.subscription_tier, "premium");
        assert!(claims.has_admin_module("user_operations"));
        assert!(claims.has_premium_access());
        assert!(claims.is_admin());
        assert!(!claims.is_expired());
    }
    
    #[test]
    fn test_permission_checking() {
        let claims = AccessTokenClaims::new(
            "admin123".to_string(),
            "admin@example.com".to_string(),
            None,
            vec!["system_admin".to_string()],
            "enterprise".to_string(),
            "epsx-admin".to_string(),
            "openid profile email admin".to_string(),
            3600,
        );
        
        assert!(claims.has_permission("database:admin"));
        assert!(claims.has_permission("system:settings"));
        assert_eq!(claims.access_level, "admin");
        assert_eq!(claims.role, "admin");
    }
    
    #[test]
    fn test_regular_user_claims() {
        let claims = AccessTokenClaims::new(
            "user456".to_string(),
            "user@example.com".to_string(),
            Some("Regular User".to_string()),
            vec![], // No admin modules
            "basic".to_string(),
            "epsx-frontend".to_string(),
            "openid profile email".to_string(),
            3600,
        );
        
        assert!(!claims.is_admin());
        assert!(!claims.has_premium_access());
        assert_eq!(claims.role, "user");
        assert_eq!(claims.access_level, "none");
        assert_eq!(claims.session_type, "user");
    }
}