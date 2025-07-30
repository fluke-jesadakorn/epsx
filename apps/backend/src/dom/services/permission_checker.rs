// Permission checking domain service

use crate::dom::values::{Role, PermSet};
use crate::dom::entities::User;

pub struct PermissionChecker;

impl PermissionChecker {
    pub fn can_access_feature(user: &User, feature: &str) -> bool {
        match feature {
            "basic_features" => true, // All users
            "premium_analytics" => user.has_perm("access:premium_features"),
            "advanced_rankings" => user.has_perm("read:advanced_analytics"),
            "user_management" => user.has_perm("manage:users"),
            "system_admin" => user.has_perm("manage:system"),
            "moderation" => user.has_perm("moderate:content"),
            _ => false,
        }
    }
    
    pub fn can_manage_user(manager: &User, target: &User) -> bool {
        // Can't manage yourself for role changes
        if manager.id() == target.id() {
            return false;
        }
        
        // Must have user management permission
        if !manager.has_perm("manage:users") {
            return false;
        }
        
        // Can only manage users with lower or equal hierarchy
        manager.role().hierarchy_level() >= target.role().hierarchy_level()
    }
    
    pub fn can_upgrade_user_to_role(manager: &User, target: &User, new_role: &Role) -> bool {
        // Must be able to manage the user
        if !Self::can_manage_user(manager, target) {
            return false;
        }
        
        // Can't upgrade beyond your own level
        if new_role.hierarchy_level() >= manager.role().hierarchy_level() {
            return false;
        }
        
        // Must be a valid upgrade path
        target.role().can_upgrade_to(new_role)
    }
    
    pub fn can_admin_modify_user(admin: &User, target: &User) -> bool {
        // Admin/SuperAdmin can modify lower hierarchy users
        // SuperAdmin can modify Admin, Admin cannot modify SuperAdmin
        
        // Can't modify yourself for safety
        if admin.id() == target.id() {
            return false;
        }
        
        match (admin.role(), target.role()) {
            (Role::SuperAdmin, _) => true, // SuperAdmin can modify anyone except themselves
            (Role::Admin, Role::SuperAdmin) => false, // Admin cannot modify SuperAdmin
            (Role::Admin, _) => true, // Admin can modify lower roles
            _ => false, // Only Admin and SuperAdmin can modify users
        }
    }
    
    pub fn required_permissions_for_role(role: &Role) -> Vec<&'static str> {
        match role {
            Role::Free => vec!["read:basic"],
            Role::User => vec!["read:own_data"],
            Role::Premium => vec![
                "read:own_data",
                "access:premium_features",
                "read:advanced_analytics",
            ],
            Role::Moderator => vec![
                "read:own_data",
                "access:premium_features",
                "read:advanced_analytics",
                "moderate:content",
                "read:user_reports",
            ],
            Role::Admin => vec![
                "read:own_data",
                "access:premium_features", 
                "read:advanced_analytics",
                "moderate:content",
                "read:user_reports",
                "manage:users",
                "read:all_data",
                "write:user_data",
            ],
            Role::SuperAdmin => vec![
                "read:own_data",
                "access:premium_features",
                "read:advanced_analytics", 
                "moderate:content",
                "read:user_reports",
                "manage:users",
                "read:all_data",
                "write:user_data",
                "manage:system",
                "write:all",
            ],
            Role::ApiClient => vec![
                "read:basic",
                "api:access",
            ],
        }
    }
    
    pub fn validate_permissions(perms: &PermSet, role: &Role) -> Vec<String> {
        let required = Self::required_permissions_for_role(role);
        let mut missing = Vec::new();
        
        for perm in required {
            if !perms.contains(perm) {
                missing.push(perm.to_string());
            }
        }
        
        missing
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dom::values::{Email, Subscription};
    
    fn create_test_user(role: Role) -> User {
        User::new(
            Email::new("test@example.com").unwrap(),
            role
        )
    }
    
    #[test]
    fn should_check_feature_access() {
        let user = create_test_user(Role::User);
        let premium_user = create_test_user(Role::Premium);
        let admin = create_test_user(Role::Admin);
        
        assert!(PermissionChecker::can_access_feature(&user, "basic_features"));
        assert!(!PermissionChecker::can_access_feature(&user, "premium_analytics"));
        
        assert!(PermissionChecker::can_access_feature(&premium_user, "premium_analytics"));
        assert!(!PermissionChecker::can_access_feature(&premium_user, "user_management"));
        
        assert!(PermissionChecker::can_access_feature(&admin, "user_management"));
        assert!(!PermissionChecker::can_access_feature(&admin, "system_admin"));
    }
    
    #[test]
    fn should_check_user_management_capability() {
        let admin = create_test_user(Role::Admin);
        let user = create_test_user(Role::User);
        let premium = create_test_user(Role::Premium);
        let super_admin = create_test_user(Role::SuperAdmin);
        
        // Admin can manage lower roles
        assert!(PermissionChecker::can_manage_user(&admin, &user));
        assert!(PermissionChecker::can_manage_user(&admin, &premium));
        
        // Admin cannot manage higher roles
        assert!(!PermissionChecker::can_manage_user(&admin, &super_admin));
        
        // Users cannot manage anyone
        assert!(!PermissionChecker::can_manage_user(&user, &premium));
        
        // Cannot manage yourself
        assert!(!PermissionChecker::can_manage_user(&admin, &admin));
    }
    
    #[test]
    fn should_check_role_upgrade_capability() {
        let admin = create_test_user(Role::Admin);
        let user = create_test_user(Role::User);
        
        // Admin can upgrade user to premium
        assert!(PermissionChecker::can_upgrade_user_to_role(&admin, &user, &Role::Premium));
        
        // Admin can upgrade user to moderator  
        assert!(PermissionChecker::can_upgrade_user_to_role(&admin, &user, &Role::Moderator));
        
        // Admin cannot upgrade user to admin or super admin (same or higher level)
        assert!(!PermissionChecker::can_upgrade_user_to_role(&admin, &user, &Role::Admin));
        assert!(!PermissionChecker::can_upgrade_user_to_role(&admin, &user, &Role::SuperAdmin));
    }
    
    #[test]
    fn should_get_required_permissions() {
        let user_perms = PermissionChecker::required_permissions_for_role(&Role::User);
        let admin_perms = PermissionChecker::required_permissions_for_role(&Role::Admin);
        
        assert!(user_perms.contains(&"read:own_data"));
        assert!(!user_perms.contains(&"manage:users"));
        
        assert!(admin_perms.contains(&"read:own_data"));
        assert!(admin_perms.contains(&"manage:users"));
        assert!(!admin_perms.contains(&"manage:system"));
    }
    
    #[test]
    fn should_validate_permissions() {
        let user_perms = PermSet::for_role(&Role::User);
        let missing = PermissionChecker::validate_permissions(&user_perms, &Role::Admin);
        
        assert!(!missing.is_empty());
        assert!(missing.contains(&"manage:users".to_string()));
    }
}