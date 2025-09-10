// Stateless Permission Validation Service
// Zero-database permission checking using JWT claims with temporal controls

use crate::domain::authentication::value_objects::SecureAccessTokenClaims;
use serde::Serialize;
use chrono::{DateTime, Utc};
use tracing::{debug, warn};

#[derive(Debug)]
pub enum PermissionError {
    InvalidPermissionFormat(String),
    PermissionDenied(String),
    ExpiredPermission(String),
    InsufficientPrivileges(String),
    InvalidRole(String),
}

impl std::fmt::Display for PermissionError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            PermissionError::InvalidPermissionFormat(msg) => write!(f, "Invalid permission format: {}", msg),
            PermissionError::PermissionDenied(msg) => write!(f, "Permission denied: {}", msg),
            PermissionError::ExpiredPermission(msg) => write!(f, "Permission expired: {}", msg),
            PermissionError::InsufficientPrivileges(msg) => write!(f, "Insufficient privileges: {}", msg),
            PermissionError::InvalidRole(msg) => write!(f, "Invalid role: {}", msg),
        }
    }
}

impl std::error::Error for PermissionError {}

/// Permission matching strategies
#[derive(Debug, Clone, PartialEq)]
pub enum PermissionMatch {
    Exact,          // Exact permission match
    Wildcard,       // Wildcard pattern match (admin:*:*)
    RoleBased,      // Role-based inheritance
    Temporal,       // Time-scoped permission
}

/// Permission validation result with detailed context
#[derive(Debug, Clone, Serialize)]
pub struct PermissionValidationResult {
    pub granted: bool,
    pub match_type: String,
    pub matched_permission: Option<String>,
    pub required_permission: String,
    pub user_id: String,
    pub expires_at: Option<DateTime<Utc>>,
    pub granted_by: Option<String>,
}

/// Permission summary for user
#[derive(Debug, Clone, Serialize)]
pub struct PermissionSummary {
    pub user_id: String,
    pub total_permissions: usize,
    pub active_permissions: usize,
    pub expired_permissions: usize,
    pub roles: Vec<String>,
    pub security_level: u8,
    pub permissions_expire_at: Option<DateTime<Utc>>,
    pub platform_contexts: Vec<String>,
}

/// Stateless permission validation service
/// 
/// This service validates permissions entirely from JWT claims without database queries.
/// Features:
/// - Exact permission matching
/// - Wildcard permission patterns (*:*:*)
/// - Role-based permission inheritance
/// - Temporal permissions with auto-expiry
/// - Platform-scoped permissions
/// - Performance: ~1-5ms validation vs 50-100ms database queries
pub struct StatelessPermissionService;

impl StatelessPermissionService {
    /// Check if user has required permission
    /// 
    /// # Examples
    /// ```
    /// use crate::domain::authorization::services::StatelessPermissionService;
    /// 
    /// let has_permission = StatelessPermissionService::check_permission(
    ///     &claims,
    ///     "admin:users:read"
    /// )?;
    /// ```
    pub fn check_permission(
        claims: &SecureAccessTokenClaims,
        required_permission: &str,
    ) -> Result<bool, PermissionError> {
        let result = Self::validate_permission_detailed(claims, required_permission)?;
        Ok(result.granted)
    }
    
    /// Validate permission with detailed result information
    pub fn validate_permission_detailed(
        claims: &SecureAccessTokenClaims,
        required_permission: &str,
    ) -> Result<PermissionValidationResult, PermissionError> {
        // Validate permission format
        Self::validate_permission_format(required_permission)?;
        
        // Get active (non-expired) permissions from JWT claims
        let active_permissions = claims.get_active_permissions();
        
        debug!(
            user_id = %claims.sub,
            required_permission = required_permission,
            active_permissions_count = active_permissions.len(),
            "Validating permission"
        );
        
        // Check exact permission match
        if let Some(matched_perm) = Self::find_exact_match(&active_permissions, required_permission) {
            let expires_at = Self::extract_permission_expiry(&matched_perm);
            
            return Ok(PermissionValidationResult {
                granted: true,
                match_type: "exact".to_string(),
                matched_permission: Some(matched_perm),
                required_permission: required_permission.to_string(),
                user_id: claims.sub.clone(),
                expires_at,
                granted_by: Some(claims.granted_by.clone()),
            });
        }
        
        // Check wildcard permissions
        if let Some(matched_perm) = Self::find_wildcard_match(&active_permissions, required_permission) {
            let expires_at = Self::extract_permission_expiry(&matched_perm);
            
            return Ok(PermissionValidationResult {
                granted: true,
                match_type: "wildcard".to_string(),
                matched_permission: Some(matched_perm),
                required_permission: required_permission.to_string(),
                user_id: claims.sub.clone(),
                expires_at,
                granted_by: Some(claims.granted_by.clone()),
            });
        }
        
        // Check role-based permissions
        if let Some(role) = Self::find_role_based_match(&claims.roles, required_permission) {
            return Ok(PermissionValidationResult {
                granted: true,
                match_type: "role_based".to_string(),
                matched_permission: Some(format!("role:{}", role)),
                required_permission: required_permission.to_string(),
                user_id: claims.sub.clone(),
                expires_at: None, // Role-based permissions don't expire
                granted_by: Some(claims.granted_by.clone()),
            });
        }
        
        // Permission denied
        warn!(
            user_id = %claims.sub,
            required_permission = required_permission,
            active_permissions = ?active_permissions,
            roles = ?claims.roles,
            "Permission denied - no matching permission or role"
        );
        
        Ok(PermissionValidationResult {
            granted: false,
            match_type: "denied".to_string(),
            matched_permission: None,
            required_permission: required_permission.to_string(),
            user_id: claims.sub.clone(),
            expires_at: None,
            granted_by: None,
        })
    }
    
    /// Check multiple permissions (all must be granted)
    pub fn check_all_permissions(
        claims: &SecureAccessTokenClaims,
        required_permissions: &[&str],
    ) -> Result<bool, PermissionError> {
        for permission in required_permissions {
            if !Self::check_permission(claims, permission)? {
                return Ok(false);
            }
        }
        Ok(true)
    }
    
    /// Check multiple permissions (any one must be granted)
    pub fn check_any_permission(
        claims: &SecureAccessTokenClaims,
        required_permissions: &[&str],
    ) -> Result<bool, PermissionError> {
        for permission in required_permissions {
            if Self::check_permission(claims, permission)? {
                return Ok(true);
            }
        }
        Ok(false)
    }
    
    /// Get comprehensive permission summary for user
    pub fn get_permission_summary(claims: &SecureAccessTokenClaims) -> PermissionSummary {
        let active_permissions = claims.get_active_permissions();
        let expired_count = claims.permissions.len() - active_permissions.len();
        
        // Find earliest permission expiry
        let permissions_expire_at = claims.permissions.iter()
            .filter_map(|p| Self::extract_permission_expiry(p))
            .min();
        
        // Extract platform contexts from permissions
        let platform_contexts: Vec<String> = active_permissions.iter()
            .filter_map(|p| p.split(':').next().map(|s| s.to_string()))
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();
        
        PermissionSummary {
            user_id: claims.sub.clone(),
            total_permissions: claims.permissions.len(),
            active_permissions: active_permissions.len(),
            expired_permissions: expired_count,
            roles: claims.roles.clone(),
            security_level: claims.security_level,
            permissions_expire_at,
            platform_contexts,
        }
    }
    
    /// Filter permissions by platform context
    pub fn get_platform_permissions(
        claims: &SecureAccessTokenClaims,
        platform: &str,
    ) -> Vec<String> {
        claims.get_active_permissions().into_iter()
            .filter(|p| p.starts_with(&format!("{}:", platform)))
            .collect()
    }
    
    /// Check if user has admin privileges
    pub fn is_admin(claims: &SecureAccessTokenClaims) -> bool {
        // Check for admin role
        if claims.roles.contains(&"admin".to_string()) {
            return true;
        }
        
        // Check for admin wildcard permissions
        let active_permissions = claims.get_active_permissions();
        active_permissions.iter().any(|p| {
            p == "admin:*:*" || p.starts_with("admin:*:*:")
        })
    }
    
    /// Check if user has elevated privileges on any platform
    pub fn has_elevated_privileges(claims: &SecureAccessTokenClaims) -> bool {
        let active_permissions = claims.get_active_permissions();
        
        // Check for wildcard permissions on any platform
        active_permissions.iter().any(|p| {
            let parts: Vec<&str> = p.split(':').collect();
            if parts.len() >= 3 {
                parts[1] == "*" || parts[2] == "*"
            } else {
                false
            }
        }) || claims.roles.contains(&"admin".to_string())
    }
    
    // Private helper methods
    
    fn validate_permission_format(permission: &str) -> Result<(), PermissionError> {
        let parts: Vec<&str> = permission.split(':').collect();
        
        if parts.len() < 3 {
            return Err(PermissionError::InvalidPermissionFormat(
                format!("Permission must have format 'platform:resource:action', got: {}", permission)
            ));
        }
        
        // Validate platform name (no empty parts)
        if parts[0].is_empty() || parts[1].is_empty() || parts[2].is_empty() {
            return Err(PermissionError::InvalidPermissionFormat(
                format!("Permission parts cannot be empty: {}", permission)
            ));
        }
        
        Ok(())
    }
    
    fn find_exact_match(permissions: &[String], required: &str) -> Option<String> {
        permissions.iter()
            .find(|&p| p == required)
            .cloned()
    }
    
    fn find_wildcard_match(permissions: &[String], required: &str) -> Option<String> {
        let required_parts: Vec<&str> = required.split(':').collect();
        
        for permission in permissions {
            let perm_parts: Vec<&str> = permission.split(':').collect();
            
            // Skip temporal permissions for wildcard matching (use first 3 parts)
            let perm_parts = if perm_parts.len() > 3 && perm_parts[3].parse::<i64>().is_ok() {
                &perm_parts[..3]
            } else {
                &perm_parts
            };
            
            if perm_parts.len() != required_parts.len() {
                continue;
            }
            
            let matches = perm_parts.iter().zip(required_parts.iter())
                .all(|(perm_part, req_part)| {
                    *perm_part == "*" || *perm_part == *req_part
                });
            
            if matches {
                return Some(permission.clone());
            }
        }
        
        None
    }
    
    fn find_role_based_match(roles: &[String], required: &str) -> Option<String> {
        let required_parts: Vec<&str> = required.split(':').collect();
        if required_parts.len() < 1 {
            return None;
        }
        
        let platform = required_parts[0];
        
        // Role hierarchy and platform mapping
        for role in roles {
            let role_grants_permission = match role.as_str() {
                "admin" => {
                    // Admin role grants all admin:* permissions
                    platform == "admin"
                },
                "manager" => {
                    // Manager role grants epsx:* permissions
                    platform == "epsx" && !required.contains("delete") && !required.contains("admin")
                },
                "analyst" => {
                    // Analyst role grants read permissions on epsx platform
                    platform == "epsx" && required.contains("read")
                },
                "support" => {
                    // Support role grants limited permissions
                    platform == "epsx" && (required.contains("read") || required.contains("list"))
                },
                _ => false,
            };
            
            if role_grants_permission {
                return Some(role.clone());
            }
        }
        
        None
    }
    
    fn extract_permission_expiry(permission: &str) -> Option<DateTime<Utc>> {
        let parts: Vec<&str> = permission.split(':').collect();
        
        // Check if last part is a timestamp
        if let Some(last_part) = parts.last() {
            if let Ok(timestamp) = last_part.parse::<i64>() {
                return DateTime::from_timestamp(timestamp, 0);
            }
        }
        
        None
    }
}

/// Macro for easy permission checking in handlers
/// Usage: require_permission!(claims, "admin:users:read");
#[macro_export]
macro_rules! require_stateless_permission {
    ($claims:expr, $permission:expr) => {
        if !crate::domain::authorization::services::StatelessPermissionService::check_permission($claims, $permission)? {
            tracing::warn!(
                user_id = %$claims.sub,
                required_permission = $permission,
                "Stateless permission check failed"
            );
            return Err(axum::http::StatusCode::FORBIDDEN);
        }
    };
}

/// Macro for checking any of multiple permissions
/// Usage: require_any_permission!(claims, &["admin:users:read", "admin:users:write"]);
#[macro_export]
macro_rules! require_any_stateless_permission {
    ($claims:expr, $permissions:expr) => {
        if !crate::domain::authorization::services::StatelessPermissionService::check_any_permission($claims, $permissions)? {
            tracing::warn!(
                user_id = %$claims.sub,
                required_permissions = ?$permissions,
                "Stateless permission check failed - none granted"
            );
            return Err(axum::http::StatusCode::FORBIDDEN);
        }
    };
}

/// Macro for checking all of multiple permissions
/// Usage: require_all_permissions!(claims, &["admin:users:read", "admin:users:write"]);
#[macro_export]
macro_rules! require_all_stateless_permissions {
    ($claims:expr, $permissions:expr) => {
        if !crate::domain::authorization::services::StatelessPermissionService::check_all_permissions($claims, $permissions)? {
            tracing::warn!(
                user_id = %$claims.sub,
                required_permissions = ?$permissions,
                "Stateless permission check failed - not all granted"
            );
            return Err(axum::http::StatusCode::FORBIDDEN);
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::shared_kernel::value_objects::UserId;
    
    fn create_test_claims(permissions: Vec<String>, roles: Vec<String>) -> SecureAccessTokenClaims {
        let user_id = UserId::from_string("test_user".to_string()).unwrap();
        SecureAccessTokenClaims::new(
            &user_id,
            permissions,
            roles,
            "test_admin",
            "test_device_fingerprint",
            Utc::now() + chrono::Duration::hours(1),
        ).unwrap()
    }
    
    #[test]
    fn test_exact_permission_match() {
        let claims = create_test_claims(
            vec!["epsx:users:read".to_string(), "admin:analytics:write".to_string()],
            vec![]
        );
        
        assert!(StatelessPermissionService::check_permission(&claims, "epsx:users:read").unwrap());
        assert!(StatelessPermissionService::check_permission(&claims, "admin:analytics:write").unwrap());
        assert!(!StatelessPermissionService::check_permission(&claims, "epsx:users:write").unwrap());
    }
    
    #[test]
    fn test_wildcard_permission_match() {
        let claims = create_test_claims(
            vec!["admin:*:*".to_string(), "epsx:users:*".to_string()],
            vec![]
        );
        
        // Wildcard should match
        assert!(StatelessPermissionService::check_permission(&claims, "admin:users:read").unwrap());
        assert!(StatelessPermissionService::check_permission(&claims, "admin:analytics:write").unwrap());
        assert!(StatelessPermissionService::check_permission(&claims, "epsx:users:read").unwrap());
        assert!(StatelessPermissionService::check_permission(&claims, "epsx:users:delete").unwrap());
        
        // Should not match different platforms
        assert!(!StatelessPermissionService::check_permission(&claims, "other:users:read").unwrap());
        assert!(!StatelessPermissionService::check_permission(&claims, "epsx:analytics:read").unwrap());
    }
    
    #[test]
    fn test_temporal_permissions() {
        let future_timestamp = (Utc::now() + chrono::Duration::hours(1)).timestamp();
        let past_timestamp = (Utc::now() - chrono::Duration::hours(1)).timestamp();
        
        let claims = create_test_claims(
            vec![
                "epsx:read:permanent".to_string(),
                format!("admin:temp:write:{}", future_timestamp),
                format!("admin:expired:delete:{}", past_timestamp),
            ],
            vec![]
        );
        
        // Permanent permission should work
        assert!(StatelessPermissionService::check_permission(&claims, "epsx:read:permanent").unwrap());
        
        // Valid temporal permission should work
        assert!(StatelessPermissionService::check_permission(&claims, "admin:temp:write").unwrap());
        
        // Expired temporal permission should not work
        assert!(!StatelessPermissionService::check_permission(&claims, "admin:expired:delete").unwrap());
    }
    
    #[test]
    fn test_role_based_permissions() {
        let claims = create_test_claims(
            vec![],
            vec!["admin".to_string(), "analyst".to_string()]
        );
        
        // Admin role should grant admin permissions
        assert!(StatelessPermissionService::check_permission(&claims, "admin:users:read").unwrap());
        assert!(StatelessPermissionService::check_permission(&claims, "admin:analytics:write").unwrap());
        
        // Analyst role should grant read permissions on epsx
        assert!(StatelessPermissionService::check_permission(&claims, "epsx:analytics:read").unwrap());
        
        // Analyst should not get write permissions
        assert!(!StatelessPermissionService::check_permission(&claims, "epsx:analytics:write").unwrap());
    }
    
    #[test]
    fn test_permission_validation_result() {
        let claims = create_test_claims(
            vec!["admin:users:read".to_string()],
            vec![]
        );
        
        let result = StatelessPermissionService::validate_permission_detailed(
            &claims, 
            "admin:users:read"
        ).unwrap();
        
        assert!(result.granted);
        assert_eq!(result.match_type, "exact");
        assert_eq!(result.matched_permission, Some("admin:users:read".to_string()));
        assert_eq!(result.required_permission, "admin:users:read");
        assert!(result.granted_by.is_some());
    }
    
    #[test]
    fn test_permission_summary() {
        let future_timestamp = (Utc::now() + chrono::Duration::hours(1)).timestamp();
        let claims = create_test_claims(
            vec![
                "epsx:users:read".to_string(),
                format!("admin:temp:write:{}", future_timestamp),
                "admin:analytics:*".to_string(),
            ],
            vec!["analyst".to_string()]
        );
        
        let summary = StatelessPermissionService::get_permission_summary(&claims);
        
        assert_eq!(summary.total_permissions, 3);
        assert_eq!(summary.active_permissions, 3); // All should be active
        assert_eq!(summary.expired_permissions, 0);
        assert_eq!(summary.roles, vec!["analyst"]);
        assert!(summary.permissions_expire_at.is_some());
        assert!(summary.platform_contexts.contains(&"epsx".to_string()));
        assert!(summary.platform_contexts.contains(&"admin".to_string()));
    }
    
    #[test]
    fn test_admin_privileges_check() {
        // Test with admin role
        let admin_claims = create_test_claims(
            vec![],
            vec!["admin".to_string()]
        );
        assert!(StatelessPermissionService::is_admin(&admin_claims));
        assert!(StatelessPermissionService::has_elevated_privileges(&admin_claims));
        
        // Test with admin wildcard permission
        let admin_perm_claims = create_test_claims(
            vec!["admin:*:*".to_string()],
            vec![]
        );
        assert!(StatelessPermissionService::is_admin(&admin_perm_claims));
        assert!(StatelessPermissionService::has_elevated_privileges(&admin_perm_claims));
        
        // Test regular user
        let user_claims = create_test_claims(
            vec!["epsx:users:read".to_string()],
            vec!["user".to_string()]
        );
        assert!(!StatelessPermissionService::is_admin(&user_claims));
        assert!(!StatelessPermissionService::has_elevated_privileges(&user_claims));
    }
    
    #[test]
    fn test_multiple_permission_checks() {
        let claims = create_test_claims(
            vec!["epsx:users:read".to_string(), "epsx:analytics:read".to_string()],
            vec![]
        );
        
        // All permissions should pass
        assert!(StatelessPermissionService::check_all_permissions(
            &claims, 
            &["epsx:users:read", "epsx:analytics:read"]
        ).unwrap());
        
        // Should fail if any permission missing
        assert!(!StatelessPermissionService::check_all_permissions(
            &claims, 
            &["epsx:users:read", "epsx:users:write"]
        ).unwrap());
        
        // Any permission should pass if one is granted
        assert!(StatelessPermissionService::check_any_permission(
            &claims, 
            &["epsx:users:write", "epsx:users:read"]
        ).unwrap());
        
        // Should fail if no permissions granted
        assert!(!StatelessPermissionService::check_any_permission(
            &claims, 
            &["epsx:users:write", "epsx:users:delete"]
        ).unwrap());
    }
}