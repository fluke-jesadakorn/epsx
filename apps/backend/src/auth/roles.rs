// ============================================================================
use chrono::{DateTime, Utc};
use uuid::Uuid;
// PERMISSION-BASED ACCESS CONTROL HELPERS
// ============================================================================
// This file provides permission-based access control helpers and database utilities
// Uses structured permissions: "platform:resource:action" format (e.g., "epsx:analytics:view")
// No role enum - purely permission-based system leveraging auth/permissions.rs

use serde::{Deserialize, Serialize};


use std::collections::HashMap;


// Import the comprehensive permission system
use crate::auth::permissions::{

    check_permission_access, 
    has_admin_access,
    can_view_analytics,
    can_export_data, 
    can_access_realtime,
    can_manage_profile,
    can_receive_notifications,
    can_manage_billing,
    can_use_advanced_filters,
    PermissionSets,
};

// Import clean architecture services
use crate::app::services::{PermissionApplicationService, ApplicationPermissionError};


// ============================================================================
// PERMISSION-BASED USER CLAIMS (REPLACES ROLE-BASED CLAIMS)
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimpleUserClaims {
    pub firebase_uid: String,
    pub email: String,
    pub permissions: Vec<String>,    // Structured permissions instead of role
    pub display_name: Option<String>,
    pub name: Option<String>,
    pub avatar_url: Option<String>,
    pub is_active: bool,
    pub last_login_at: Option<chrono::DateTime<chrono::Utc>>,
}

// ============================================================================
// PERMISSION-BASED FEATURE ACCESS LOGIC
// ============================================================================

pub fn check_feature_access(user_permissions: &[String], feature: &str) -> bool {
    match feature {
        "view_eps" => can_view_analytics(user_permissions),
        "export_data" => can_export_data(user_permissions),
        "realtime" => can_access_realtime(user_permissions),
        "profile" => can_manage_profile(user_permissions),
        "notifications" => can_receive_notifications(user_permissions),
        "billing" => can_manage_billing(user_permissions),
        "advanced_filters" => can_use_advanced_filters(user_permissions),
        _ => has_admin_access(user_permissions), // Unknown features require admin
    }
}

pub fn check_permission_level_access(user_permissions: &[String], required_permission: &str) -> bool {
    check_permission_access(user_permissions, required_permission)
}

pub fn get_user_features(permissions: &[String]) -> Vec<String> {
    let all_features = [
        "view_eps", 
        "export_data", 
        "realtime", 
        "profile", 
        "notifications", 
        "billing", 
        "advanced_filters"
    ];
    
    all_features
        .iter()
        .filter(|feature| check_feature_access(permissions, feature))
        .map(|s| s.to_string())
        .collect()
}

// ============================================================================
// PERMISSION LEVEL HELPERS 
// ============================================================================

/// Get permission level based on user permissions
pub fn get_permission_level(permissions: &[String]) -> String {
    if has_admin_access(permissions) {
        "admin".to_string()
    } else if permissions.iter().any(|p| p.starts_with("epsx:")) {
        "user".to_string()  
    } else {
        "basic".to_string()
    }
}

/// Get default permissions for a given permission level
pub fn get_default_permissions_for_level(level: &str) -> Vec<String> {
    match level.to_lowercase().as_str() {
        "admin" => PermissionSets::admin(),
        "user" => PermissionSets::premium_user(),
        "basic" => PermissionSets::basic_user(),
        _ => PermissionSets::basic_user(), // Default fallback
    }
}

// ============================================================================
// CLEAN ARCHITECTURE INTEGRATION - REPLACES DIRECT DATABASE ACCESS
// ============================================================================

/// Get user claims using clean architecture services (async)
pub async fn get_user_claims_from_service(
    permission_service: &PermissionApplicationService,
    firebase_uid: &str
) -> Result<Option<SimpleUserClaims>, ApplicationPermissionError> {
    let user_claims = permission_service.get_user_claims(firebase_uid).await?;
    
    if let Some(claims) = user_claims {
        Ok(Some(SimpleUserClaims {
            firebase_uid: claims.firebase_uid,
            email: claims.email,
            permissions: claims.permissions,
            display_name: claims.display_name,
            name: claims.name,
            avatar_url: claims.avatar_url,
            is_active: claims.is_active,
            last_login_at: claims.last_login_at,
        }))
    } else {
        Ok(None)
    }
}

/// Check user feature access using clean architecture services (async)
pub async fn check_user_feature_access_service(
    permission_service: &PermissionApplicationService,
    firebase_uid: &str,
    feature: &str
) -> Result<bool, ApplicationPermissionError> {
    let user_permissions = permission_service.get_user_permissions(firebase_uid).await?;
    Ok(check_feature_access(&user_permissions, feature))
}

/// Check user permission access using clean architecture services (async)
pub async fn check_user_permission_access_service(
    permission_service: &PermissionApplicationService,
    firebase_uid: &str,
    required_permission: &str
) -> Result<bool, ApplicationPermissionError> {
    permission_service.check_user_permission(firebase_uid, required_permission).await
}

/// Require permission using clean architecture services (async)
pub async fn require_permission_service(
    permission_service: &PermissionApplicationService,
    firebase_uid: &str,
    required_permission: &str
) -> Result<SimpleUserClaims, ApplicationPermissionError> {
    let claims = permission_service.require_permission(firebase_uid, required_permission).await?;
    
    Ok(SimpleUserClaims {
        firebase_uid: claims.firebase_uid,
        email: claims.email,
        permissions: claims.permissions,
        display_name: claims.display_name,
        name: claims.name,
        avatar_url: claims.avatar_url,
        is_active: claims.is_active,
        last_login_at: claims.last_login_at,
    })
}

/// Require feature access using clean architecture services (async)
pub async fn require_feature_service(
    permission_service: &PermissionApplicationService,
    firebase_uid: &str,
    feature: &str
) -> Result<SimpleUserClaims, ApplicationPermissionError> {
    let user_claims = permission_service.get_user_claims(firebase_uid).await?
        .ok_or(ApplicationPermissionError::UserNotFound)?;
    
    // Check feature access
    if check_feature_access(&user_claims.permissions, feature) {
        Ok(SimpleUserClaims {
            firebase_uid: user_claims.firebase_uid,
            email: user_claims.email,
            permissions: user_claims.permissions,
            display_name: user_claims.display_name,
            name: user_claims.name,
            avatar_url: user_claims.avatar_url,
            is_active: user_claims.is_active,
            last_login_at: user_claims.last_login_at,
        })
    } else {
        Err(ApplicationPermissionError::InsufficientPermissions)
    }
}

/// Require admin access using clean architecture services (async)
pub async fn require_admin_service(
    permission_service: &PermissionApplicationService,
    firebase_uid: &str,
) -> Result<SimpleUserClaims, ApplicationPermissionError> {
    require_permission_service(permission_service, firebase_uid, "admin:*:*").await
}

// ============================================================================
// PERMISSION VALIDATION HELPERS
// ============================================================================

pub fn is_admin(permissions: &[String]) -> bool {
    has_admin_access(permissions)
}

pub fn has_any_access(permissions: &[String]) -> bool {
    !permissions.is_empty()
}

// Re-export permission helper functions for convenience
pub use crate::auth::permissions::{
    can_view_analytics as can_view_eps,
};

// ============================================================================
// ERROR TYPES
// ============================================================================

#[derive(Debug, thiserror::Error)]
pub enum PermissionError {
    #[error("Access denied: insufficient permissions")]
    InsufficientPermissions,
    
    #[error("Access denied: feature not available")]
    FeatureNotAvailable,
    
    #[error("User not found")]
    UserNotFound,
    
    #[error("Database error: {0}")]
    Database(#[from] diesel::result::Error),
    
    #[error("Invalid permission format")]
    InvalidPermission,
}

// ============================================================================
// MIDDLEWARE HELPERS - CLEAN ARCHITECTURE (ASYNC)
// ============================================================================
// New async helpers using clean architecture services

/// Async middleware helper for requiring permission using clean architecture
pub async fn require_permission_async(
    permission_service: &PermissionApplicationService,
    firebase_uid: &str,
    required_permission: &str
) -> Result<SimpleUserClaims, PermissionError> {
    match require_permission_service(permission_service, firebase_uid, required_permission).await {
        Ok(claims) => Ok(claims),
        Err(ApplicationPermissionError::UserNotFound) => Err(PermissionError::UserNotFound),
        Err(ApplicationPermissionError::InsufficientPermissions) => Err(PermissionError::InsufficientPermissions),
        Err(ApplicationPermissionError::InvalidPermissionFormat(_)) => Err(PermissionError::InvalidPermission),
        Err(_) => Err(PermissionError::Database(diesel::result::Error::DatabaseError(
            diesel::result::DatabaseErrorKind::Unknown,
            Box::new("Service error".to_string())
        ))),
    }
}

/// Async middleware helper for requiring feature access using clean architecture
pub async fn require_feature_async(
    permission_service: &PermissionApplicationService,
    firebase_uid: &str,
    feature: &str
) -> Result<SimpleUserClaims, PermissionError> {
    match require_feature_service(permission_service, firebase_uid, feature).await {
        Ok(claims) => Ok(claims),
        Err(ApplicationPermissionError::UserNotFound) => Err(PermissionError::UserNotFound),
        Err(ApplicationPermissionError::InsufficientPermissions) => Err(PermissionError::FeatureNotAvailable),
        Err(ApplicationPermissionError::InvalidPermissionFormat(_)) => Err(PermissionError::InvalidPermission),
        Err(_) => Err(PermissionError::Database(diesel::result::Error::DatabaseError(
            diesel::result::DatabaseErrorKind::Unknown,
            Box::new("Service error".to_string())
        ))),
    }
}

/// Async middleware helper for requiring admin access using clean architecture
pub async fn require_admin_async(
    permission_service: &PermissionApplicationService,
    firebase_uid: &str,
) -> Result<SimpleUserClaims, PermissionError> {
    require_permission_async(permission_service, firebase_uid, "admin:*:*").await
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permission_hierarchy() {
        let admin_permissions = vec!["admin:*:*".to_string()];
        let user_permissions = vec!["epsx:analytics:view".to_string(), "epsx:analytics:export".to_string()];
        
        // Admin can access everything
        assert!(check_permission_level_access(&admin_permissions, "admin:users:manage"));
        assert!(check_permission_level_access(&admin_permissions, "epsx:analytics:view"));
        
        // User can access specific permissions
        assert!(check_permission_level_access(&user_permissions, "epsx:analytics:view"));
        assert!(!check_permission_level_access(&user_permissions, "admin:users:manage"));
    }

    #[test]
    fn test_feature_access() {
        let admin_permissions = vec!["admin:*:*".to_string()];
        let user_permissions = vec![
            "epsx:analytics:view".to_string(),
            "epsx:analytics:export".to_string(),
            "epsx:realtime:access".to_string(),
            "epsx:profile:manage".to_string(),
            "epsx:notifications:receive".to_string(),
            "epsx:billing:manage".to_string(),
            "epsx:filters:advanced".to_string(),
        ];
        
        // Admin can access everything
        assert!(check_feature_access(&admin_permissions, "view_eps"));
        assert!(check_feature_access(&admin_permissions, "export_data"));
        assert!(check_feature_access(&admin_permissions, "realtime"));
        assert!(check_feature_access(&admin_permissions, "profile"));
        assert!(check_feature_access(&admin_permissions, "notifications"));
        assert!(check_feature_access(&admin_permissions, "billing"));
        assert!(check_feature_access(&admin_permissions, "advanced_filters"));
        
        // User can access granted features
        assert!(check_feature_access(&user_permissions, "view_eps"));
        assert!(check_feature_access(&user_permissions, "export_data"));
        assert!(check_feature_access(&user_permissions, "realtime"));
        assert!(check_feature_access(&user_permissions, "profile"));
        assert!(check_feature_access(&user_permissions, "notifications"));
        assert!(check_feature_access(&user_permissions, "billing"));
        assert!(check_feature_access(&user_permissions, "advanced_filters"));
    }

    #[test]
    fn test_permission_levels() {
        // Test permission level determination
        let admin_permissions = vec!["admin:*:*".to_string()];
        let user_permissions = vec!["epsx:analytics:view".to_string()];
        let basic_permissions = vec![];
        
        assert_eq!(get_permission_level(&admin_permissions), "admin");
        assert_eq!(get_permission_level(&user_permissions), "user");
        assert_eq!(get_permission_level(&basic_permissions), "basic");
        
        // Test default permissions for levels
        let admin_perms = get_default_permissions_for_level("admin");
        assert!(admin_perms.contains(&"admin:*:*".to_string()));
        
        let user_perms = get_default_permissions_for_level("user");
        assert!(user_perms.contains(&"epsx:analytics:view".to_string()));
    }

    #[test]
    fn test_admin_validation() {
        let admin_permissions = vec!["admin:*:*".to_string()];
        let user_permissions = vec!["epsx:analytics:view".to_string()];
        
        assert!(is_admin(&admin_permissions));
        assert!(!is_admin(&user_permissions));
        
        assert!(has_any_access(&admin_permissions));
        assert!(has_any_access(&user_permissions));
        assert!(!has_any_access(&[]));
    }
}

// ============================================================================
// SECURITY EVENT STUB (FOR REPLACED SECURITY SYSTEM)
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityEvent {
    pub id: Option<Uuid>,
    pub event_type: String,
    pub severity: String, 
    pub client_ip: Option<String>,
    pub user_id: Option<Uuid>,
    pub timestamp: DateTime<Utc>,
    pub details: HashMap<String, serde_json::Value>,
}

impl Default for SecurityEvent {
    fn default() -> Self {
        Self {
            id: Some(Uuid::new_v4()),
            event_type: "UNKNOWN".to_string(),
            severity: "LOW".to_string(),
            client_ip: None,
            user_id: None,
            timestamp: Utc::now(),
            details: HashMap::new(),
        }
    }
}