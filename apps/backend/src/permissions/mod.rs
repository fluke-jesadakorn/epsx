// Unified Permission Validation System for EPSX Trading Platform
//
// This module provides a comprehensive, enterprise-grade permission validation system
// that consolidates all permission logic across admin and user applications into a
// centralized, secure, and efficient validation framework.

pub mod core;
pub mod admin_modules;
pub mod package_tiers;
pub mod cache;
pub mod audit;
pub mod errors;

// Re-export core types for easier access
pub use core::{
    PermissionValidator, PermissionContext, PermissionResult, PermissionDecision,
    Permission, PermissionRequest, PermissionGrant, PermissionDenial,
    PermissionEngine, UnifiedPermissionSystem,
};

pub use admin_modules::{
    AdminModule, AdminModulePermission, AdminModuleValidator,
    AdminModuleAccess, AdminModuleContext,
};

pub use package_tiers::{
    PackageTierValidator, TierFeature, TierLimit, TierAccess,
};

// Validators removed - not implemented yet

pub use cache::{
    PermissionCache, CachedPermission, PermissionCacheConfig,
    PermissionCacheStats, CacheInvalidation,
};

pub use audit::{
    DatabasePermissionAudit, InMemoryPermissionAudit, PermissionAuditTrait,
    PermissionAuditEntry, AuditConfig, SecurityEvent, AuditQuery, AuditStatistics,
};

// Templates removed - not implemented yet

pub use errors::{
    PermissionError, ValidationError, PolicyError,
    CacheError, AuditError,
};

// Common traits and constants
pub mod traits {
    use async_trait::async_trait;
    use crate::dom::values::UserId;
    use super::*;
    
    /// Core trait for permission validation
    #[async_trait]
    pub trait PermissionValidator: Send + Sync {
        async fn validate(&self, context: &PermissionContext) -> PermissionDecision;
        async fn has_permission(&self, _user_id: &UserId, permission: &str, resource: &str) -> bool;
        async fn get_permissions(&self, _user_id: &UserId) -> Vec<Permission>;
    }
    
    /// Trait for cacheable permissions
    #[async_trait]
    pub trait Cacheable: Send + Sync {
        type Key;
        type Value;
        
        async fn cache_key(&self) -> Self::Key;
        async fn cache_value(&self) -> Self::Value;
        async fn invalidate_cache(&self) -> Result<(), CacheError>;
    }
    
    /// Trait for auditable permission actions
    #[async_trait]
    pub trait Auditable: Send + Sync {
        async fn audit(&self, entry: PermissionAuditEntry) -> Result<(), AuditError>;
        async fn security_event(&self, event: SecurityEvent) -> Result<(), AuditError>;
    }
}

// Common constants
pub mod constants {
    // Admin module permissions
    pub const USER_MANAGEMENT: &str = "user-management";
    pub const ANALYTICS_ACCESS: &str = "analytics-access";
    pub const SYSTEM_CONFIGURATION: &str = "system-configuration";
    pub const AUDIT_LOGS: &str = "audit-logs";
    pub const FINANCIAL_OVERSIGHT: &str = "financial-oversight";
    pub const CONTENT_MANAGEMENT: &str = "content-management";
    pub const SUPPORT_ACCESS: &str = "support-access";
    pub const SECURITY_MANAGEMENT: &str = "security-management";
    
    // Package tier features
    pub const BASIC_TRADING: &str = "basic-trading";
    pub const ADVANCED_ANALYTICS: &str = "advanced-analytics";
    pub const API_ACCESS: &str = "api-access";
    pub const PRIORITY_SUPPORT: &str = "priority-support";
    pub const ADVANCED_ORDERS: &str = "advanced-orders";
    pub const PORTFOLIO_TOOLS: &str = "portfolio-tools";
    pub const RESEARCH_REPORTS: &str = "research-reports";
    pub const INSTITUTIONAL_FEATURES: &str = "institutional-features";
    
    // Cache settings
    pub const PERMISSION_CACHE_TTL_SECONDS: u64 = 300; // 5 minutes
    pub const PERMISSION_CACHE_MAX_SIZE: u64 = 10000;
    
    // Performance thresholds
    pub const MAX_PERMISSION_CHECK_TIME_MS: u64 = 10;
    pub const MAX_CACHE_LOOKUP_TIME_MS: u64 = 1;
    
    // Security settings
    pub const MAX_PERMISSION_ELEVATION_TIME_MINUTES: i64 = 30;
    pub const PERMISSION_AUDIT_RETENTION_DAYS: i64 = 365;
}

// Utility functions
pub mod utils {
    use super::*;
    
    /// Check if permission matches pattern with wildcard support
    pub fn matches_permission_pattern(permission: &str, pattern: &str) -> bool {
        if pattern == "*" {
            return true;
        }
        
        if pattern.contains('*') {
            let regex_pattern = pattern.replace('*', ".*");
            regex::Regex::new(&regex_pattern)
                .map(|re| re.is_match(permission))
                .unwrap_or(false)
        } else {
            permission == pattern
        }
    }
    
    /// Generate unique permission key for caching
    pub fn generate_permission_key(user_id: &str, permission: &str, resource: &str) -> String {
        format!("perm:{}:{}:{}", user_id, permission, resource)
    }
    
    /// Check if permission is administrative
    pub fn is_admin_permission(permission: &str) -> bool {
        const ADMIN_PATTERNS: &[&str] = &[
            "admin:*",
            "system:*",
            "security:*",
            "audit:*",
            "user-management:*",
            "financial-oversight:*",
        ];
        
        ADMIN_PATTERNS.iter().any(|pattern| matches_permission_pattern(permission, pattern))
    }
    
    /// Extract module name from permission
    pub fn extract_module_name(permission: &str) -> Option<&str> {
        permission.split(':').next()
    }
    
    /// Validate permission format
    pub fn validate_permission_format(permission: &str) -> Result<(), ValidationError> {
        if permission.is_empty() {
            return Err(ValidationError::EmptyPermission);
        }
        
        if !permission.contains(':') {
            return Err(ValidationError::InvalidFormat("Permission must contain ':' separator".to_string()));
        }
        
        let parts: Vec<&str> = permission.split(':').collect();
        if parts.len() != 2 {
            return Err(ValidationError::InvalidFormat("Permission must have format 'module:action'".to_string()));
        }
        
        if parts[0].is_empty() || parts[1].is_empty() {
            return Err(ValidationError::InvalidFormat("Module and action cannot be empty".to_string()));
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dom::values::UserId;
    
    #[test]
    fn test_permission_pattern_matching() {
        assert!(utils::matches_permission_pattern("user:read", "user:*"));
        assert!(utils::matches_permission_pattern("user:read", "*"));
        assert!(!utils::matches_permission_pattern("user:read", "admin:*"));
        assert!(utils::matches_permission_pattern("user:read", "user:read"));
    }
    
    #[test]
    fn test_admin_permission_detection() {
        assert!(utils::is_admin_permission("admin:create"));
        assert!(utils::is_admin_permission("system:configure"));
        assert!(utils::is_admin_permission("user-management:delete"));
        assert!(!utils::is_admin_permission("user:read"));
    }
    
    #[test]
    fn test_permission_validation() {
        assert!(utils::validate_permission_format("user:read").is_ok());
        assert!(utils::validate_permission_format("").is_err());
        assert!(utils::validate_permission_format("invalid").is_err());
        assert!(utils::validate_permission_format("too:many:parts").is_err());
    }
    
    #[test]
    fn test_module_extraction() {
        assert_eq!(utils::extract_module_name("user:read"), Some("user"));
        assert_eq!(utils::extract_module_name("admin:delete"), Some("admin"));
        assert_eq!(utils::extract_module_name("invalid"), Some("invalid"));
    }
    
    #[test]
    fn test_permission_key_generation() {
        let key = utils::generate_permission_key("user123", "user:read", "profile");
        assert_eq!(key, "perm:user123:user:read:profile");
    }
}