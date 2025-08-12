use std::collections::HashMap;
use serde::Serialize;

/// Granular Admin Module Definitions
#[derive(Debug, Clone, Serialize)]
pub struct AdminModule {
    pub code: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub category: &'static str,
    pub permissions: &'static [&'static str],
    pub api_endpoints: &'static [&'static str],
    pub frontend_routes: &'static [&'static str],
    pub access_level: &'static str,
}

/// All 10 granular admin modules
pub const ADMIN_MODULES: &[AdminModule] = &[
    AdminModule {
        code: "user_operations",
        name: "User Operations Manager",
        description: "User CRUD operations, status management, and basic profile editing",
        category: "management",
        permissions: &["user:read", "user:write", "user:status", "profile:edit"],
        api_endpoints: &["/api/v1/admin/users/*", "/api/v1/admin/firebase/users/*", "/api/v1/admin/users/*/unified"],
        frontend_routes: &["/users", "/users/*"],
        access_level: "write",
    },
    AdminModule {
        code: "permission_admin",
        name: "Permission Administrator",
        description: "Permission profiles, assignments, and temporary permission management",
        category: "management",
        permissions: &["permission:read", "permission:write", "profile:assign", "temp_permission:manage"],
        api_endpoints: &["/api/v1/admin/permission-profiles/*", "/api/v1/admin/temporary-permissions/*", "/api/v1/admin/permissions/*"],
        frontend_routes: &["/permission-profiles", "/permission-profiles/*"],
        access_level: "write",
    },
    AdminModule {
        code: "role_policy_manager",
        name: "Role & Policy Manager", 
        description: "Casbin roles, policies, and access control management",
        category: "management",
        permissions: &["role:read", "role:write", "policy:manage", "casbin:admin"],
        api_endpoints: &["/api/v1/admin/casbin/*", "/api/v1/admin/roles/*"],
        frontend_routes: &["/iam", "/iam/*"],
        access_level: "write",
    },
    AdminModule {
        code: "analytics_specialist",
        name: "Analytics Specialist",
        description: "Reporting, dashboards, and data analysis (read-only access)",
        category: "analytics", 
        permissions: &["analytics:read", "reports:generate", "metrics:view"],
        api_endpoints: &["/api/v1/admin/analytics/*"],
        frontend_routes: &["/analytics", "/analytics/*"],
        access_level: "read",
    },
    AdminModule {
        code: "billing_admin",
        name: "Billing Administrator",
        description: "Payment management, subscriptions, and package assignments",
        category: "commerce",
        permissions: &["billing:read", "billing:write", "subscription:manage", "package:assign"],
        api_endpoints: &["/api/v1/admin/users/*/billing", "/api/v1/admin/stock-ranking-packages/*"],
        frontend_routes: &["/billing", "/billing/*"],
        access_level: "write",
    },
    AdminModule {
        code: "system_admin",
        name: "System Administrator",
        description: "Database management, health monitoring, and system settings",
        category: "system",
        permissions: &["database:admin", "system:settings", "cache:manage", "health:monitor"],
        api_endpoints: &["/api/v1/admin/database/*", "/api/v1/admin/settings/*", "/api/v1/admin/casbin/cache/*"],
        frontend_routes: &["/database", "/settings", "/settings/*"],
        access_level: "admin",
    },
    AdminModule {
        code: "developer_relations",
        name: "Developer Relations",
        description: "API keys, developer portal, and integration management",
        category: "technical",
        permissions: &["api_key:manage", "developer:tools", "documentation:manage"],
        api_endpoints: &["/api/v1/admin/api-keys/*", "/api/v1/admin/developer-portal/*"],
        frontend_routes: &["/developer-portal", "/developer-portal/*"],
        access_level: "write",
    },
    AdminModule {
        code: "module_coordinator",
        name: "Module Coordinator",
        description: "Feature module assignments and access control",
        category: "management",
        permissions: &["module:read", "module:write", "module:assign", "feature:manage"],
        api_endpoints: &["/api/v1/admin/modules/*", "/api/v1/admin/users/*/modules"],
        frontend_routes: &["/modules", "/modules/*"],
        access_level: "write",
    },
    AdminModule {
        code: "compliance_audit",
        name: "Compliance & Audit Officer",
        description: "Security, audit reports, backups, and compliance management",
        category: "security",
        permissions: &["audit:read", "compliance:manage", "backup:create", "security:analyze"],
        api_endpoints: &["/api/v1/admin/permissions/audit-report", "/api/v1/admin/permissions/system-backup/*", "/api/v1/admin/analytics/security-risks"],
        frontend_routes: &["/compliance", "/audit/*"],
        access_level: "read",
    },
    AdminModule {
        code: "support_specialist",
        name: "Support Specialist",
        description: "User support and troubleshooting (read-only access)",
        category: "support",
        permissions: &["user:read", "support:tickets", "activity:view"],
        api_endpoints: &["/api/v1/admin/users/*/activity", "/api/v1/admin/support/*"],
        frontend_routes: &["/support", "/users/*/support"],
        access_level: "read",
    },
];

/// Get admin module by code
pub fn get_admin_module(code: &str) -> Option<&'static AdminModule> {
    ADMIN_MODULES.iter().find(|module| module.code == code)
}

/// Get all permissions for a list of module codes
pub fn get_permissions_for_modules(module_codes: &[String]) -> Vec<String> {
    let mut permissions = Vec::new();
    for code in module_codes {
        if let Some(module) = get_admin_module(code) {
            permissions.extend(module.permissions.iter().map(|&p| p.to_string()));
        }
    }
    permissions.sort();
    permissions.dedup();
    permissions
}

/// Check if user has access to specific API endpoint based on their modules
pub fn user_can_access_endpoint(user_modules: &[String], endpoint: &str) -> bool {
    for module_code in user_modules {
        if let Some(module) = get_admin_module(module_code) {
            for &pattern in module.api_endpoints {
                if endpoint_matches_pattern(endpoint, pattern) {
                    return true;
                }
            }
        }
    }
    false
}

/// Check if user has access to frontend route based on their modules
pub fn user_can_access_route(user_modules: &[String], route: &str) -> bool {
    for module_code in user_modules {
        if let Some(module) = get_admin_module(module_code) {
            for &pattern in module.frontend_routes {
                if route_matches_pattern(route, pattern) {
                    return true;
                }
            }
        }
    }
    false
}

/// Helper function to match API endpoint patterns
fn endpoint_matches_pattern(endpoint: &str, pattern: &str) -> bool {
    if pattern.ends_with("/*") {
        let prefix = &pattern[..pattern.len() - 2];
        endpoint.starts_with(prefix)
    } else {
        endpoint == pattern
    }
}

/// Helper function to match frontend route patterns  
fn route_matches_pattern(route: &str, pattern: &str) -> bool {
    if pattern.ends_with("/*") {
        let prefix = &pattern[..pattern.len() - 2];
        route.starts_with(prefix)
    } else {
        route == pattern
    }
}

/// Modern IAM system uses only granular admin modules
/// Legacy roles have been completely removed

/// Module-based permission validation
pub struct AdminModuleValidator;

impl AdminModuleValidator {
    /// Check if user has specific admin module
    pub fn has_admin_module(user_modules: &[String], required_module: &str) -> bool {
        user_modules.iter().any(|m| m == required_module)
    }
    
    /// Check if user has any admin modules (is an admin)
    pub fn has_any_admin_module(user_modules: &[String]) -> bool {
        !user_modules.is_empty()
    }
    
    /// Check if user has specific permission across all their modules
    pub fn has_permission(user_modules: &[String], required_permission: &str) -> bool {
        let user_permissions = get_permissions_for_modules(user_modules);
        user_permissions.iter().any(|p| p == required_permission)
    }
    
    /// Check if user can access specific endpoint
    pub fn can_access_endpoint(user_modules: &[String], endpoint: &str) -> bool {
        user_can_access_endpoint(user_modules, endpoint)
    }
    
    /// Check if user can access frontend route
    pub fn can_access_route(user_modules: &[String], route: &str) -> bool {
        user_can_access_route(user_modules, route)
    }
    
    /// Get user's effective access level (highest among their modules)
    pub fn get_effective_access_level(user_modules: &[String]) -> String {
        let mut highest_level = "none";
        
        for module_code in user_modules {
            if let Some(module) = get_admin_module(module_code) {
                match module.access_level {
                    "admin" => return "admin".to_string(),
                    "write" if highest_level != "admin" => highest_level = "write",
                    "read" if highest_level == "none" => highest_level = "read",
                    _ => {}
                }
            }
        }
        
        highest_level.to_string()
    }
}

/// Helper function to assign all admin modules to a user (for jesadakorn.kirtnu@gmail.com)
pub fn get_all_admin_module_codes() -> Vec<String> {
    ADMIN_MODULES.iter().map(|m| m.code.to_string()).collect()
}

/// Create modern JWT claims using admin module system
pub fn create_jwt_claims(user_modules: &[String]) -> HashMap<String, serde_json::Value> {
    let permissions = get_permissions_for_modules(user_modules);
    let access_level = AdminModuleValidator::get_effective_access_level(user_modules);
    let is_admin = AdminModuleValidator::has_any_admin_module(user_modules);
    
    let mut claims = HashMap::new();
    claims.insert("admin".to_string(), serde_json::Value::Bool(is_admin));
    claims.insert("access_level".to_string(), serde_json::Value::String(access_level));
    claims.insert("admin_modules".to_string(), serde_json::Value::Array(
        user_modules.iter().map(|m| serde_json::Value::String(m.clone())).collect()
    ));
    claims.insert("permissions".to_string(), serde_json::Value::Array(
        permissions.iter().map(|p| serde_json::Value::String(p.clone())).collect()
    ));
    claims
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_admin_module_lookup() {
        let module = get_admin_module("user_operations").unwrap();
        assert_eq!(module.name, "User Operations Manager");
        assert!(module.permissions.contains(&"user:read"));
    }
    
    #[test]
    fn test_endpoint_access() {
        let modules = vec!["user_operations".to_string()];
        assert!(user_can_access_endpoint(&modules, "/api/v1/admin/users/123"));
        assert!(!user_can_access_endpoint(&modules, "/api/v1/admin/billing/payments"));
    }
    
    #[test]
    fn test_permission_aggregation() {
        let modules = vec!["user_operations".to_string(), "analytics_specialist".to_string()];
        let permissions = get_permissions_for_modules(&modules);
        assert!(permissions.contains(&"user:read".to_string()));
        assert!(permissions.contains(&"analytics:read".to_string()));
    }
    
    #[test]
    fn test_access_level_resolution() {
        let modules = vec!["analytics_specialist".to_string(), "system_admin".to_string()];
        assert_eq!(AdminModuleValidator::get_effective_access_level(&modules), "admin");
    }
    
    #[test]
    fn test_all_modules_assignment() {
        let all_modules = get_all_admin_module_codes();
        assert_eq!(all_modules.len(), 10);
        assert!(all_modules.contains(&"user_operations".to_string()));
        assert!(all_modules.contains(&"system_admin".to_string()));
    }
}