// ============================================================================
// PERMISSION RESOLVER STUB - REPLACING COMPLEX PERMISSION RESOLVER
// ============================================================================
// Simple stub for removed permission resolver system

use crate::dom::values::UserId;
use crate::auth::roles::{Role, check_feature_access};
use std::str::FromStr;

// Simple stub permission resolver
pub struct PermissionResolver;

impl PermissionResolver {
    pub fn new() -> Self {
        Self
    }

    pub async fn can_access_feature(&self, _user_id: &UserId, feature: &str) -> Result<bool, String> {
        // For now, always allow - use simple roles system instead
        match feature {
            "view_eps" => Ok(true), // All users can view EPS
            _ => Ok(false), // Other features require proper role checking
        }
    }

    pub async fn get_user_permissions(&self, _user_id: &UserId) -> Result<Vec<String>, String> {
        // Return basic permissions for compatibility
        Ok(vec![
            "view_eps".to_string(),
        ])
    }

    // Simple role-based checking
    pub fn check_simple_role_feature(&self, role_str: &str, feature: &str) -> bool {
        match Role::from_str(role_str) {
            Ok(role) => check_feature_access(&role, feature),
            Err(_) => false,
        }
    }
}

impl Default for PermissionResolver {
    fn default() -> Self {
        Self::new()
    }
}