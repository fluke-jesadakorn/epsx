use std::collections::HashSet;
use chrono::{Utc, Duration};

use crate::domain::shared_kernel::{DomainError, DomainResult, Specification};
use crate::domain::user_management::{
    User, 
    Permission, 
};

/// Domain service for managing user permissions
/// This service contains business logic that spans multiple aggregates
/// or requires complex permission calculation logic
pub struct UserPermissionService;

impl UserPermissionService {
    /// Calculate effective permissions for a user
    /// This includes inherited permissions, role-based permissions, and temporary permissions
    pub fn calculate_effective_permissions(
        &self,
        user: &User,
        context: &PermissionContext
    ) -> HashSet<Permission> {
        let mut effective_permissions = HashSet::new();
        
        // Start with direct permissions
        for permission in user.permissions() {
            if permission.is_active() {
                effective_permissions.insert(permission.clone());
            }
        }
        
        // Add role-based permissions (if user has admin role)
        if self.user_has_admin_role(user) {
            effective_permissions.extend(self.get_admin_permissions());
        }
        
        // Add context-specific permissions
        effective_permissions.extend(self.get_context_permissions(user, context));
        
        effective_permissions
    }
    
    /// Check if a user can perform an action on a resource in a given context
    pub fn can_user_access(
        &self,
        user: &User,
        platform: &str,
        resource: &str,
        action: &str,
        context: &PermissionContext
    ) -> bool {
        // User must be active
        if !user.is_active() {
            return false;
        }
        
        // Calculate effective permissions
        let effective_permissions = self.calculate_effective_permissions(user, context);
        
        // Check if any permission grants access
        effective_permissions
            .iter()
            .any(|p| p.grants_access(platform, resource, action))
    }
    
    /// Generate default permissions for a new user based on their characteristics
    pub fn generate_default_permissions(
        &self,
        user: &User
    ) -> DomainResult<HashSet<Permission>> {
        let mut permissions = HashSet::new();
        
        // All users get basic read access
        permissions.insert(Permission::new("epsx:analytics:view")?);
        permissions.insert(Permission::new("epsx:user:read")?);
        
        // Email-based special permissions
        if let Some(domain) = user.email().domain() {
            match domain {
                "epsx.io" | "company.com" => {
                    // Internal users get additional permissions
                    permissions.insert(Permission::new("epsx:analytics:*")?);
                    permissions.insert(Permission::new("epsx:user:manage")?);
                }
                _ => {
                    // External users get standard permissions only
                }
            }
        }
        
        // Verified users get enhanced permissions
        if user.is_email_verified() {
            permissions.insert(Permission::new("epsx:notifications:manage")?);
        }
        
        Ok(permissions)
    }
    
    /// Check if two permission sets conflict with each other
    pub fn permissions_conflict(
        &self,
        permissions1: &HashSet<Permission>,
        permissions2: &HashSet<Permission>
    ) -> Vec<PermissionConflict> {
        let mut conflicts = Vec::new();
        
        for perm1 in permissions1 {
            for perm2 in permissions2 {
                if let Some(conflict) = self.check_permission_conflict(perm1, perm2) {
                    conflicts.push(conflict);
                }
            }
        }
        
        conflicts
    }
    
    /// Validate that a set of permissions is coherent and doesn't contain contradictions
    pub fn validate_permission_set(
        &self,
        permissions: &HashSet<Permission>
    ) -> DomainResult<()> {
        // Check for conflicting permissions
        let conflicts = self.permissions_conflict(permissions, permissions);
        if !conflicts.is_empty() {
            return Err(DomainError::business_rule_violation(
                format!("Permission set contains conflicts: {:?}", conflicts)
            ));
        }
        
        // Check for impossible combinations
        let has_admin = permissions.iter().any(|p| p.platform() == "admin");
        let has_readonly = permissions.iter().any(|p| p.action() == "read");
        
        if has_admin && permissions.len() == 1 && has_readonly {
            return Err(DomainError::business_rule_violation(
                "Admin users should have more than just read permissions"
            ));
        }
        
        Ok(())
    }
    
    /// Create time-limited permissions for temporary access
    pub fn create_temporary_permissions(
        &self,
        base_permissions: &HashSet<Permission>,
        duration: Duration
    ) -> DomainResult<HashSet<Permission>> {
        let expires_at = Utc::now() + duration;
        let mut temporary_permissions = HashSet::new();
        
        for permission in base_permissions {
            let temp_permission = Permission::new_with_expiration(
                permission.as_str(),
                expires_at
            )?;
            temporary_permissions.insert(temp_permission);
        }
        
        Ok(temporary_permissions)
    }
    
    /// Get permissions that are about to expire
    pub fn get_expiring_permissions(
        &self,
        user: &User,
        within: Duration
    ) -> Vec<Permission> {
        let cutoff = Utc::now() + within;
        
        user.permissions()
            .iter()
            .filter(|p| {
                if let Some(expires_at) = p.expires_at() {
                    expires_at <= cutoff
                } else {
                    false
                }
            })
            .cloned()
            .collect()
    }
    
    // Private helper methods
    
    fn user_has_admin_role(&self, user: &User) -> bool {
        user.permissions()
            .iter()
            .any(|p| p.platform() == "admin" && p.resource() == "*" && p.action() == "*")
    }
    
    fn get_admin_permissions(&self) -> HashSet<Permission> {
        let mut permissions = HashSet::new();
        
        // Admin users get all permissions
        if let Ok(perm) = Permission::new("admin:*:*") {
            permissions.insert(perm);
        }
        if let Ok(perm) = Permission::new("epsx:*:*") {
            permissions.insert(perm);
        }
        
        permissions
    }
    
    fn get_context_permissions(
        &self,
        _user: &User,
        context: &PermissionContext
    ) -> HashSet<Permission> {
        let mut permissions = HashSet::new();
        
        // Add permissions based on context (e.g., IP-based, time-based)
        if context.is_internal_network {
            if let Ok(perm) = Permission::new("epsx:internal:access") {
                permissions.insert(perm);
            }
        }
        
        if context.is_business_hours {
            if let Ok(perm) = Permission::new("epsx:business:access") {
                permissions.insert(perm);
            }
        }
        
        permissions
    }
    
    fn check_permission_conflict(
        &self,
        perm1: &Permission,
        perm2: &Permission
    ) -> Option<PermissionConflict> {
        // Example: Can't have both read-only and write permissions on the same resource
        if perm1.platform() == perm2.platform() 
            && perm1.resource() == perm2.resource()
            && perm1.action() == "read" 
            && perm2.action() == "*" {
            
            return Some(PermissionConflict {
                permission1: perm1.clone(),
                permission2: perm2.clone(),
                conflict_type: ConflictType::RedundantPermission,
                description: "Wildcard permission makes specific read permission redundant".to_string(),
            });
        }
        
        None
    }
}

/// Context information for permission calculations
#[derive(Debug, Clone)]
pub struct PermissionContext {
    pub is_internal_network: bool,
    pub is_business_hours: bool,
    pub client_ip: Option<String>,
    pub user_agent: Option<String>,
    pub session_age: Option<Duration>,
}

/// Represents a conflict between two permissions
#[derive(Debug, Clone)]
pub struct PermissionConflict {
    pub permission1: Permission,
    pub permission2: Permission,
    pub conflict_type: ConflictType,
    pub description: String,
}

#[derive(Debug, Clone)]
pub enum ConflictType {
    RedundantPermission,
    ContradictoryPermission,
    SecurityViolation,
}

/// Specification for checking if a user has admin privileges
pub struct IsAdminSpecification;

impl Specification<User> for IsAdminSpecification {
    fn is_satisfied_by(&self, user: &User) -> bool {
        user.is_active() && user.permissions()
            .iter()
            .any(|p| p.platform() == "admin" && p.action() == "*")
    }
}

/// Specification for checking if a user has specific platform access
pub struct HasPlatformAccessSpecification {
    platform: String,
}

impl HasPlatformAccessSpecification {
    pub fn new(platform: String) -> Self {
        Self { platform }
    }
}

impl Specification<User> for HasPlatformAccessSpecification {
    fn is_satisfied_by(&self, user: &User) -> bool {
        user.is_active() && user.permissions()
            .iter()
            .any(|p| p.platform() == self.platform || p.platform() == "*")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::user_management::value_objects::{Email, FirebaseUid};
    
    fn create_test_user() -> User {
        User::create(
            UserId::new(),
            FirebaseUid::new("test_uid").unwrap(),
            Email::new("test@example.com").unwrap(),
        ).unwrap()
    }
    
    #[test]
    fn generate_default_permissions_for_regular_user() {
        let service = UserPermissionService;
        let user = create_test_user();
        
        let permissions = service.generate_default_permissions(&user).unwrap();
        
        assert!(!permissions.is_empty());
        assert!(permissions.iter().any(|p| p.as_str() == "epsx:analytics:view"));
    }
    
    #[test]
    fn admin_specification_works() {
        let mut user = create_test_user();
        let admin_spec = IsAdminSpecification;
        
        // Initially not admin
        assert!(!admin_spec.is_satisfied_by(&user));
        
        // Grant admin permission
        let admin_perm = Permission::new("admin:*:*").unwrap();
        user.grant_permission(admin_perm, None).unwrap();
        
        // Now should be admin
        assert!(admin_spec.is_satisfied_by(&user));
    }
    
    #[test]
    fn platform_access_specification_works() {
        let mut user = create_test_user();
        let epsx_spec = HasPlatformAccessSpecification::new("epsx".to_string());
        
        // Initially no access
        assert!(!epsx_spec.is_satisfied_by(&user));
        
        // Grant epsx permission
        let epsx_perm = Permission::new("epsx:analytics:view").unwrap();
        user.grant_permission(epsx_perm, None).unwrap();
        
        // Now should have access
        assert!(epsx_spec.is_satisfied_by(&user));
    }
    
    #[test]
    fn temporary_permissions_creation() {
        let service = UserPermissionService;
        let mut base_permissions = HashSet::new();
        base_permissions.insert(Permission::new("epsx:temp:access").unwrap());
        
        let temp_permissions = service.create_temporary_permissions(
            &base_permissions,
            Duration::hours(1)
        ).unwrap();
        
        assert_eq!(temp_permissions.len(), 1);
        let temp_perm = temp_permissions.iter().next().unwrap();
        assert!(temp_perm.expires_at().is_some());
    }
}