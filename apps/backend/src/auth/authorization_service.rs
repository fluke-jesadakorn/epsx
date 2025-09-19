// Unified Authorization Service
// Consolidates: permissions.rs, granular_permissions.rs, policy_engine.rs, hierarchy_resolver.rs, scopes.rs

use std::collections::{HashMap, HashSet};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tracing::{debug, warn};
use anyhow::Result;

/// Permission structure
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Permission {
    pub platform: String,
    pub resource: String,
    pub action: String,
    pub expires_at: Option<DateTime<Utc>>,
}

impl Permission {
    pub fn new(platform: &str, resource: &str, action: &str) -> Self {
        Self {
            platform: platform.to_string(),
            resource: resource.to_string(),
            action: action.to_string(),
            expires_at: None,
        }
    }
    
    pub fn with_expiry(platform: &str, resource: &str, action: &str, expires_at: DateTime<Utc>) -> Self {
        Self {
            platform: platform.to_string(),
            resource: resource.to_string(),
            action: action.to_string(),
            expires_at: Some(expires_at),
        }
    }
    
    pub fn from_string(permission_str: &str) -> Result<Self> {
        let parts: Vec<&str> = permission_str.split(':').collect();
        
        if parts.len() < 3 {
            return Err(anyhow::anyhow!("Permission must be in format 'platform:resource:action', got: {}", permission_str));
        }
        
        // Check for embedded timestamp (format: platform:resource:action:timestamp)
        if parts.len() == 4 {
            if let Ok(timestamp) = parts[3].parse::<i64>() {
                let expires_at = DateTime::from_timestamp(timestamp, 0)
                    .ok_or_else(|| anyhow::anyhow!("Invalid timestamp in permission"))?;
                return Ok(Self::with_expiry(parts[0], parts[1], parts[2], expires_at));
            }
        }
        
        Ok(Self::new(parts[0], parts[1], parts[2]))
    }
    
    pub fn to_string(&self) -> String {
        match self.expires_at {
            Some(expires) => format!("{}:{}:{}:{}", self.platform, self.resource, self.action, expires.timestamp()),
            None => format!("{}:{}:{}", self.platform, self.resource, self.action),
        }
    }
    
    pub fn is_expired(&self) -> bool {
        match self.expires_at {
            Some(expires) => Utc::now() > expires,
            None => false,
        }
    }
    
    /// Check if this permission matches another (supports wildcards)
    pub fn matches(&self, required: &Permission) -> bool {
        // Exact match
        if self.platform == required.platform 
           && self.resource == required.resource 
           && self.action == required.action {
            return !self.is_expired();
        }
        
        // Wildcard matching
        if self.matches_with_wildcards(required) {
            return !self.is_expired();
        }
        
        false
    }
    
    fn matches_with_wildcards(&self, required: &Permission) -> bool {
        let platform_match = self.platform == "*" || self.platform == required.platform;
        let resource_match = self.resource == "*" || self.resource == required.resource;
        let action_match = self.action == "*" || self.action == required.action;
        
        platform_match && resource_match && action_match
    }
}

/// Authorization context
#[derive(Debug, Clone)]
pub struct AuthContext {
    pub user_id: String,
    pub permissions: Vec<Permission>,
    pub role: Option<String>,
    pub session_id: Option<String>,
}

impl AuthContext {
    pub fn new(user_id: String, permission_strings: Vec<String>, role: Option<String>) -> Self {
        let permissions = permission_strings
            .into_iter()
            .filter_map(|p| Permission::from_string(&p).ok())
            .collect();
            
        Self {
            user_id,
            permissions,
            role,
            session_id: None,
        }
    }
    
    pub fn with_session(mut self, session_id: String) -> Self {
        self.session_id = Some(session_id);
        self
    }
}

/// Authorization decision
#[derive(Debug, Clone)]
pub enum AuthDecision {
    Allow,
    Deny(String),
}

/// Unified Authorization Service
pub struct AuthorizationService {
    // Cache for permission hierarchies and policies
    permission_cache: HashMap<String, Vec<Permission>>,
}

impl AuthorizationService {
    pub fn new() -> Self {
        Self {
            permission_cache: HashMap::new(),
        }
    }
    
    /// Check if user has required permission
    pub fn check_permission(&self, context: &AuthContext, required_permission: &str) -> AuthDecision {
        let required = match Permission::from_string(required_permission) {
            Ok(perm) => perm,
            Err(e) => {
                warn!("Invalid permission format '{}': {}", required_permission, e);
                return AuthDecision::Deny(format!("Invalid permission format: {}", e));
            }
        };
        
        debug!("Checking permission '{}' for user: {}", required_permission, context.user_id);
        
        // Filter out expired permissions
        let active_permissions: Vec<&Permission> = context.permissions
            .iter()
            .filter(|p| !p.is_expired())
            .collect();
        
        // Check direct permission match
        for permission in &active_permissions {
            if permission.matches(&required) {
                debug!("Permission granted via direct match: {}", permission.to_string());
                return AuthDecision::Allow;
            }
        }
        
        // Check admin wildcard permissions
        for permission in &active_permissions {
            if permission.platform == "admin" && permission.resource == "*" && permission.action == "*" {
                debug!("Permission granted via admin wildcard");
                return AuthDecision::Allow;
            }
        }
        
        debug!("Permission denied for '{}': no matching permission found", required_permission);
        AuthDecision::Deny(format!("Insufficient permissions for: {}", required_permission))
    }
    
    /// Check multiple permissions (all must pass)
    pub fn check_all_permissions(&self, context: &AuthContext, required_permissions: &[String]) -> AuthDecision {
        for permission in required_permissions {
            match self.check_permission(context, permission) {
                AuthDecision::Allow => continue,
                AuthDecision::Deny(reason) => return AuthDecision::Deny(reason),
            }
        }
        AuthDecision::Allow
    }
    
    /// Check if any of the permissions pass
    pub fn check_any_permissions(&self, context: &AuthContext, required_permissions: &[String]) -> AuthDecision {
        for permission in required_permissions {
            if matches!(self.check_permission(context, permission), AuthDecision::Allow) {
                return AuthDecision::Allow;
            }
        }
        AuthDecision::Deny("None of the required permissions found".to_string())
    }
    
    /// Get all active (non-expired) permissions for user
    pub fn get_active_permissions(&self, context: &AuthContext) -> Vec<Permission> {
        context.permissions
            .iter()
            .filter(|p| !p.is_expired())
            .cloned()
            .collect()
    }
    
    /// Get permissions expiring soon (within specified hours)
    pub fn get_expiring_permissions(&self, context: &AuthContext, within_hours: i64) -> Vec<Permission> {
        let threshold = Utc::now() + chrono::Duration::hours(within_hours);
        
        context.permissions
            .iter()
            .filter(|p| {
                if let Some(expires) = p.expires_at {
                    expires <= threshold && expires > Utc::now()
                } else {
                    false
                }
            })
            .cloned()
            .collect()
    }
    
    /// Clean up expired permissions
    pub fn cleanup_expired_permissions(&mut self, context: &mut AuthContext) -> usize {
        let original_count = context.permissions.len();
        context.permissions.retain(|p| !p.is_expired());
        let removed_count = original_count - context.permissions.len();
        
        if removed_count > 0 {
            debug!("Cleaned up {} expired permissions for user: {}", removed_count, context.user_id);
        }
        
        removed_count
    }
    
    /// Get permission statistics
    pub fn get_permission_stats(&self, context: &AuthContext) -> PermissionStats {
        let total = context.permissions.len();
        let active = context.permissions.iter().filter(|p| !p.is_expired()).count();
        let expired = total - active;
        let expiring_soon = self.get_expiring_permissions(context, 24).len();
        
        let mut platforms = HashSet::new();
        let mut resources = HashSet::new();
        let mut actions = HashSet::new();
        
        for permission in &context.permissions {
            if !permission.is_expired() {
                platforms.insert(permission.platform.clone());
                resources.insert(permission.resource.clone());
                actions.insert(permission.action.clone());
            }
        }
        
        PermissionStats {
            total_permissions: total,
            active_permissions: active,
            expired_permissions: expired,
            expiring_in_24h: expiring_soon,
            unique_platforms: platforms.len(),
            unique_resources: resources.len(),
            unique_actions: actions.len(),
        }
    }
}

/// Permission statistics
#[derive(Debug, Serialize)]
pub struct PermissionStats {
    pub total_permissions: usize,
    pub active_permissions: usize,
    pub expired_permissions: usize,
    pub expiring_in_24h: usize,
    pub unique_platforms: usize,
    pub unique_resources: usize,
    pub unique_actions: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_permission_parsing() {
        let perm = Permission::from_string("epsx:analytics:view").unwrap();
        assert_eq!(perm.platform, "epsx");
        assert_eq!(perm.resource, "analytics");
        assert_eq!(perm.action, "view");
        assert!(perm.expires_at.is_none());
    }
    
    #[test]
    fn test_embedded_timestamp_permission() {
        let timestamp = Utc::now().timestamp() + 3600; // 1 hour from now
        let perm_str = format!("epsx:analytics:view:{}", timestamp);
        let perm = Permission::from_string(&perm_str).unwrap();
        
        assert_eq!(perm.platform, "epsx");
        assert_eq!(perm.resource, "analytics");
        assert_eq!(perm.action, "view");
        assert!(perm.expires_at.is_some());
        assert!(!perm.is_expired());
    }
    
    #[test]
    fn test_permission_matching() {
        let auth_service = AuthorizationService::new();
        
        let permissions = vec![
            "epsx:analytics:view".to_string(),
            "admin:*:*".to_string(),
        ];
        
        let context = AuthContext::new("user123".to_string(), permissions, Some("user".to_string()));
        
        // Direct match
        assert!(matches!(auth_service.check_permission(&context, "epsx:analytics:view"), AuthDecision::Allow));
        
        // Admin wildcard should allow anything
        assert!(matches!(auth_service.check_permission(&context, "some:random:permission"), AuthDecision::Allow));
        
        // Should deny unknown permission for non-admin
        let user_permissions = vec!["epsx:basic:read".to_string()];
        let user_context = AuthContext::new("user456".to_string(), user_permissions, Some("user".to_string()));
        
        assert!(matches!(auth_service.check_permission(&user_context, "epsx:admin:write"), AuthDecision::Deny(_)));
    }
    
    #[test]
    fn test_expired_permissions() {
        let auth_service = AuthorizationService::new();
        
        let past_timestamp = Utc::now().timestamp() - 3600; // 1 hour ago
        let expired_permission = format!("epsx:analytics:view:{}", past_timestamp);
        
        let context = AuthContext::new("user123".to_string(), vec![expired_permission], None);
        
        // Expired permission should be denied
        assert!(matches!(auth_service.check_permission(&context, "epsx:analytics:view"), AuthDecision::Deny(_)));
    }
}