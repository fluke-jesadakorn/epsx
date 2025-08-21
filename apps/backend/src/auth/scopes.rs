/*!
 * OAuth 2.0 Scope Management
 * 
 * Implements comprehensive scope validation and control for OAuth 2.0 and OpenID Connect.
 * Supports hierarchical scopes, dynamic permission mapping, and fine-grained access control.
 */

use std::collections::{HashMap, HashSet};
use serde::{Serialize, Deserialize};
use thiserror::Error;

/// OAuth 2.0 scope definition
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Scope {
    pub name: String,
    pub description: String,
    pub category: ScopeCategory,
    pub required_role: Option<String>,
    pub permissions: Vec<String>,
    pub sensitive: bool, // Requires additional consent
}

/// Scope categories for organization
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ScopeCategory {
    OpenID,     // OIDC scopes
    Profile,    // User profile scopes
    Trading,    // Trading platform scopes
    Admin,      // Administrative scopes
    Analytics,  // Analytics scopes
    Billing,    // Billing scopes
}

/// Scope validation service
pub struct ScopeService {
    scopes: HashMap<String, Scope>,
    role_hierarchy: HashMap<String, u32>,
}

impl ScopeService {
    /// Create a new scope service with predefined scopes
    pub fn new() -> Self {
        let mut service = Self {
            scopes: HashMap::new(),
            role_hierarchy: Self::create_role_hierarchy(),
        };
        
        service.register_builtin_scopes();
        service
    }

    /// Register all built-in OAuth/OIDC scopes
    fn register_builtin_scopes(&mut self) {
        // OpenID Connect Core 1.0 scopes
        self.register_scope(Scope {
            name: "openid".to_string(),
            description: "OpenID Connect authentication".to_string(),
            category: ScopeCategory::OpenID,
            required_role: None,
            permissions: vec!["auth:authenticate".to_string()],
            sensitive: false,
        });

        self.register_scope(Scope {
            name: "profile".to_string(),
            description: "Basic profile information".to_string(),
            category: ScopeCategory::Profile,
            required_role: None,
            permissions: vec!["profile:read".to_string()],
            sensitive: false,
        });

        self.register_scope(Scope {
            name: "email".to_string(),
            description: "Email address access".to_string(),
            category: ScopeCategory::Profile,
            required_role: None,
            permissions: vec!["email:read".to_string()],
            sensitive: false,
        });

        // Trading platform scopes
        self.register_scope(Scope {
            name: "trading:read".to_string(),
            description: "Read trading data and positions".to_string(),
            category: ScopeCategory::Trading,
            required_role: None,
            permissions: vec!["trading:read", "market:read"].iter().map(|s| s.to_string()).collect(),
            sensitive: false,
        });

        self.register_scope(Scope {
            name: "trading:write".to_string(),
            description: "Execute trades and manage positions".to_string(),
            category: ScopeCategory::Trading,
            required_role: Some("premium".to_string()),
            permissions: vec!["trading:read", "trading:write", "market:read", "orders:create"].iter().map(|s| s.to_string()).collect(),
            sensitive: true,
        });

        self.register_scope(Scope {
            name: "portfolio:read".to_string(),
            description: "View portfolio information".to_string(),
            category: ScopeCategory::Trading,
            required_role: None,
            permissions: vec!["portfolio:read", "positions:read"].iter().map(|s| s.to_string()).collect(),
            sensitive: false,
        });

        self.register_scope(Scope {
            name: "portfolio:write".to_string(),
            description: "Modify portfolio settings".to_string(),
            category: ScopeCategory::Trading,
            required_role: Some("premium".to_string()),
            permissions: vec!["portfolio:read", "portfolio:write", "positions:read"].iter().map(|s| s.to_string()).collect(),
            sensitive: true,
        });

        // Premium analytics scopes
        self.register_scope(Scope {
            name: "analytics:basic".to_string(),
            description: "Basic market analytics".to_string(),
            category: ScopeCategory::Analytics,
            required_role: None,
            permissions: vec!["analytics:basic", "market:read"].iter().map(|s| s.to_string()).collect(),
            sensitive: false,
        });

        self.register_scope(Scope {
            name: "analytics:advanced".to_string(),
            description: "Advanced analytics and insights".to_string(),
            category: ScopeCategory::Analytics,
            required_role: Some("premium".to_string()),
            permissions: vec!["analytics:basic", "analytics:advanced", "market:read", "insights:read"].iter().map(|s| s.to_string()).collect(),
            sensitive: false,
        });

        // Administrative scopes
        self.register_scope(Scope {
            name: "admin:users".to_string(),
            description: "User management access".to_string(),
            category: ScopeCategory::Admin,
            required_role: Some("moderator".to_string()),
            permissions: vec!["users:read", "users:write", "profiles:read", "profiles:write"].iter().map(|s| s.to_string()).collect(),
            sensitive: true,
        });

        self.register_scope(Scope {
            name: "admin:system".to_string(),
            description: "System administration access".to_string(),
            category: ScopeCategory::Admin,
            required_role: Some("admin".to_string()),
            permissions: vec!["system:read", "system:write", "config:read", "config:write"].iter().map(|s| s.to_string()).collect(),
            sensitive: true,
        });

        self.register_scope(Scope {
            name: "billing:read".to_string(),
            description: "View billing information".to_string(),
            category: ScopeCategory::Billing,
            required_role: None,
            permissions: vec!["billing:read", "subscription:read"].iter().map(|s| s.to_string()).collect(),
            sensitive: false,
        });

        self.register_scope(Scope {
            name: "billing:write".to_string(),
            description: "Manage billing and subscriptions".to_string(),
            category: ScopeCategory::Billing,
            required_role: Some("moderator".to_string()),
            permissions: vec!["billing:read", "billing:write", "subscription:read", "subscription:write"].iter().map(|s| s.to_string()).collect(),
            sensitive: true,
        });

        self.register_scope(Scope {
            name: "admin_modules".to_string(),
            description: "Access to granular admin modules".to_string(),
            category: ScopeCategory::Admin,
            required_role: None,
            permissions: vec!["admin_modules:read", "admin_modules:validate"].iter().map(|s| s.to_string()).collect(),
            sensitive: true,
        });
    }

    /// Register a new scope
    pub fn register_scope(&mut self, scope: Scope) {
        tracing::debug!(
            scope_name = %scope.name,
            category = ?scope.category,
            required_role = ?scope.required_role,
            permission_count = %scope.permissions.len(),
            "Registering OAuth scope"
        );
        
        self.scopes.insert(scope.name.clone(), scope);
    }

    /// Validate that requested scopes are valid and user is authorized
    pub fn validate_scopes(
        &self,
        requested_scopes: &str,
        user_role: &str,
        client_id: &str,
    ) -> Result<ValidatedScopes, ScopeError> {
        let scope_names = self.parse_scope_string(requested_scopes);
        
        if scope_names.is_empty() {
            return Err(ScopeError::EmptyScope);
        }

        let mut validated_scopes = Vec::new();
        let mut all_permissions = HashSet::new();
        let mut sensitive_scopes = Vec::new();

        // Validate each requested scope
        for scope_name in &scope_names {
            let scope = self.scopes.get(scope_name)
                .ok_or_else(|| ScopeError::InvalidScope {
                    scope: scope_name.clone()
                })?;

            // Check role requirement
            if let Some(required_role) = &scope.required_role {
                if !self.has_required_role(user_role, required_role) {
                    return Err(ScopeError::InsufficientRole {
                        scope: scope_name.clone(),
                        required_role: required_role.clone(),
                        user_role: user_role.to_string(),
                    });
                }
            }

            // Check client authorization for sensitive scopes
            if scope.sensitive && !self.is_client_authorized_for_sensitive_scopes(client_id) {
                return Err(ScopeError::ClientNotAuthorized {
                    scope: scope_name.clone(),
                    client_id: client_id.to_string(),
                });
            }

            validated_scopes.push(scope.clone());
            all_permissions.extend(scope.permissions.iter().cloned());
            
            if scope.sensitive {
                sensitive_scopes.push(scope.name.clone());
            }
        }

        // Ensure openid scope is included for OIDC
        if !scope_names.contains(&"openid".to_string()) && self.requires_openid_scope(&scope_names) {
            return Err(ScopeError::MissingOpenIDScope);
        }

        Ok(ValidatedScopes {
            granted_scopes: scope_names.join(" "),
            scope_objects: validated_scopes,
            permissions: all_permissions.into_iter().collect(),
            sensitive_scopes,
        })
    }

    /// Parse space-separated scope string into individual scope names
    pub fn parse_scope_string(&self, scope_string: &str) -> Vec<String> {
        scope_string
            .split_whitespace()
            .map(|s| s.to_string())
            .collect()
    }

    /// Check if user has required role
    fn has_required_role(&self, user_role: &str, required_role: &str) -> bool {
        let user_level = self.role_hierarchy.get(user_role).copied().unwrap_or(0);
        let required_level = self.role_hierarchy.get(required_role).copied().unwrap_or(1);
        
        user_level >= required_level
    }

    /// Check if client is authorized for sensitive scopes
    fn is_client_authorized_for_sensitive_scopes(&self, client_id: &str) -> bool {
        // In a production system, this would check against a client registry
        // For now, we'll allow trusted first-party clients
        match client_id {
            "epsx-frontend" | "epsx-admin" => true,
            _ => false, // Third-party clients would need explicit approval
        }
    }

    /// Check if scope combination requires openid scope
    fn requires_openid_scope(&self, scopes: &[String]) -> bool {
        // If any OIDC profile scopes are requested, openid is required
        scopes.iter().any(|scope| matches!(scope.as_str(), "profile" | "email"))
    }

    /// Create role hierarchy mapping
    fn create_role_hierarchy() -> HashMap<String, u32> {
        let mut hierarchy = HashMap::new();
        
        hierarchy.insert("user".to_string(), 1);
        hierarchy.insert("premium".to_string(), 2);
        hierarchy.insert("moderator".to_string(), 3);
        hierarchy.insert("admin".to_string(), 4);
        hierarchy.insert("super_admin".to_string(), 5);
        
        hierarchy
    }

    /// Get scope information
    pub fn get_scope(&self, scope_name: &str) -> Option<&Scope> {
        self.scopes.get(scope_name)
    }

    /// Get all available scopes for a user role
    pub fn get_available_scopes(&self, user_role: &str) -> Vec<&Scope> {
        self.scopes
            .values()
            .filter(|scope| {
                scope.required_role
                    .as_ref()
                    .map_or(true, |required| self.has_required_role(user_role, required))
            })
            .collect()
    }

    /// Get scope statistics
    pub fn get_scope_stats(&self) -> ScopeStats {
        let mut stats = ScopeStats {
            total_scopes: self.scopes.len() as u32,
            categories: HashMap::new(),
            sensitive_count: 0,
        };

        for scope in self.scopes.values() {
            *stats.categories.entry(format!("{:?}", scope.category)).or_insert(0) += 1;
            
            if scope.sensitive {
                stats.sensitive_count += 1;
            }
        }

        stats
    }

    /// Reduce scopes based on user permissions (for token refresh)
    pub fn reduce_scopes_for_user(
        &self,
        requested_scopes: &str,
        user_role: &str,
        user_permissions: &[String],
    ) -> Result<String, ScopeError> {
        let validated = self.validate_scopes(requested_scopes, user_role, "system")?;
        
        // Filter scopes based on actual user permissions
        let mut granted_scopes = Vec::new();
        
        for scope_obj in &validated.scope_objects {
            // Check if user has all required permissions for this scope
            let has_all_permissions = scope_obj.permissions.iter()
                .all(|perm| user_permissions.contains(perm));
            
            if has_all_permissions {
                granted_scopes.push(scope_obj.name.clone());
            } else {
                tracing::debug!(
                    scope = %scope_obj.name,
                    required_permissions = ?scope_obj.permissions,
                    user_permissions = ?user_permissions,
                    "Scope filtered due to insufficient permissions"
                );
            }
        }
        
        Ok(granted_scopes.join(" "))
    }
}

impl Default for ScopeService {
    fn default() -> Self {
        Self::new()
    }
}

// Global scope service instance
lazy_static::lazy_static! {
    pub static ref SCOPE_SERVICE: ScopeService = ScopeService::new();
}

/// Result of scope validation
#[derive(Debug, Clone)]
pub struct ValidatedScopes {
    pub granted_scopes: String,
    pub scope_objects: Vec<Scope>,
    pub permissions: Vec<String>,
    pub sensitive_scopes: Vec<String>,
}

/// Scope validation errors
#[derive(Debug, Error)]
pub enum ScopeError {
    #[error("Empty scope string")]
    EmptyScope,
    
    #[error("Invalid scope: {scope}")]
    InvalidScope { scope: String },
    
    #[error("Insufficient role for scope '{scope}': requires '{required_role}', user has '{user_role}'")]
    InsufficientRole {
        scope: String,
        required_role: String,
        user_role: String,
    },
    
    #[error("Client '{client_id}' not authorized for sensitive scope '{scope}'")]
    ClientNotAuthorized {
        scope: String,
        client_id: String,
    },
    
    #[error("OpenID scope is required when requesting profile scopes")]
    MissingOpenIDScope,
}

/// Scope statistics
#[derive(Debug, Serialize)]
pub struct ScopeStats {
    pub total_scopes: u32,
    pub categories: HashMap<String, u32>,
    pub sensitive_count: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scope_validation() {
        let service = ScopeService::new();
        
        // Test valid scopes
        let result = service.validate_scopes("openid profile email", "user", "epsx-frontend");
        assert!(result.is_ok());
        
        let validated = result.unwrap();
        assert_eq!(validated.granted_scopes, "openid profile email");
        assert_eq!(validated.scope_objects.len(), 3);
        assert!(validated.permissions.contains(&"auth:authenticate".to_string()));
        assert!(validated.permissions.contains(&"profile:read".to_string()));
        assert!(validated.permissions.contains(&"email:read".to_string()));
    }

    #[test]
    fn test_insufficient_role() {
        let service = ScopeService::new();
        
        // User role trying to access premium scope
        let result = service.validate_scopes("trading:write", "user", "epsx-frontend");
        assert!(result.is_err());
        
        if let Err(ScopeError::InsufficientRole { scope, required_role, user_role }) = result {
            assert_eq!(scope, "trading:write");
            assert_eq!(required_role, "premium");
            assert_eq!(user_role, "user");
        } else {
            panic!("Expected InsufficientRole error");
        }
    }

    #[test]
    fn test_scope_reduction() {
        let service = ScopeService::new();
        
        let user_permissions = vec![
            "auth:authenticate".to_string(),
            "profile:read".to_string(),
            "trading:read".to_string(),
        ];
        
        let reduced = service.reduce_scopes_for_user(
            "openid profile trading:read trading:write",
            "premium",
            &user_permissions,
        );
        
        assert!(reduced.is_ok());
        let granted = reduced.unwrap();
        assert!(granted.contains("openid"));
        assert!(granted.contains("profile"));
        assert!(granted.contains("trading:read"));
        assert!(!granted.contains("trading:write")); // Should be filtered out
    }
}