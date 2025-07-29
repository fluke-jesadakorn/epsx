// Centralized permission constants to avoid hardcoding throughout the codebase

/// Resource-based permissions for the application
/// Format: "{action}:{resource}" or "{action}:{scope}"
pub struct Permissions;

impl Permissions {
    // Basic permissions
    pub const READ_OWN: &'static str = "read:own";
    pub const WRITE_OWN: &'static str = "write:own";
    pub const READ_ALL: &'static str = "read:all";
    pub const WRITE_ALL: &'static str = "write:all";
    pub const DELETE_ALL: &'static str = "delete:all";
    
    // User management permissions
    pub const MANAGE_USERS: &'static str = "manage:users";
    pub const DELETE_USERS: &'static str = "delete:users";
    pub const READ_USER_REPORTS: &'static str = "read:user_reports";
    pub const WRITE_USER_DATA: &'static str = "write:user_data";
    
    // System permissions
    pub const MANAGE_SYSTEM: &'static str = "manage:system";
    pub const MANAGE_ADMIN: &'static str = "manage:admin";
    
    // Content permissions
    pub const MODERATE_CONTENT: &'static str = "moderate:content";
    pub const MODERATE_USERS: &'static str = "moderate:users";
    pub const WRITE_CONTENT: &'static str = "write:content";
    
    // Premium features
    pub const ACCESS_PREMIUM: &'static str = "access:premium";
    pub const ACCESS_PREMIUM_FEATURES: &'static str = "access:premium_features";
    pub const READ_PREMIUM: &'static str = "read:premium";
    pub const READ_ADVANCED_ANALYTICS: &'static str = "read:advanced_analytics";
    
    // Data access permissions
    pub const READ_BASIC: &'static str = "read:basic";
    pub const READ_OWN_DATA: &'static str = "read:own_data";
    pub const READ_ALL_DATA: &'static str = "read:all_data";
    pub const WRITE_OWN_DATA: &'static str = "write:own_data";
}

/// Permission groups for role-based access control
pub struct PermissionGroups;

impl PermissionGroups {
    /// Free tier permissions
    pub fn free_tier() -> Vec<&'static str> {
        vec![
            Permissions::READ_BASIC,
            Permissions::READ_OWN,
        ]
    }
    
    /// User tier permissions
    pub fn user_tier() -> Vec<&'static str> {
        vec![
            Permissions::READ_OWN,
            Permissions::WRITE_OWN,
            Permissions::READ_OWN_DATA,
            Permissions::WRITE_OWN_DATA,
        ]
    }
    
    /// Premium tier permissions
    pub fn premium_tier() -> Vec<&'static str> {
        let mut perms = Self::user_tier();
        perms.extend_from_slice(&[
            Permissions::ACCESS_PREMIUM,
            Permissions::ACCESS_PREMIUM_FEATURES,
            Permissions::READ_PREMIUM,
            Permissions::READ_ADVANCED_ANALYTICS,
        ]);
        perms
    }
    
    /// Moderator permissions
    pub fn moderator() -> Vec<&'static str> {
        let mut perms = Self::premium_tier();
        perms.extend_from_slice(&[
            Permissions::READ_ALL,
            Permissions::MODERATE_CONTENT,
            Permissions::MODERATE_USERS,
            Permissions::WRITE_CONTENT,
            Permissions::READ_USER_REPORTS,
        ]);
        perms
    }
    
    /// Admin permissions
    pub fn admin() -> Vec<&'static str> {
        let mut perms = Self::moderator();
        perms.extend_from_slice(&[
            Permissions::WRITE_ALL,
            Permissions::MANAGE_USERS,
            Permissions::DELETE_USERS,
            Permissions::READ_ALL_DATA,
            Permissions::WRITE_USER_DATA,
        ]);
        perms
    }
    
    /// Super admin permissions
    pub fn super_admin() -> Vec<&'static str> {
        let mut perms = Self::admin();
        perms.extend_from_slice(&[
            Permissions::DELETE_ALL,
            Permissions::MANAGE_SYSTEM,
            Permissions::MANAGE_ADMIN,
        ]);
        perms
    }
}

/// Feature-based permission checks
pub struct FeaturePermissions;

impl FeaturePermissions {
    /// Check if user can access premium analytics
    pub fn can_access_premium_analytics(user_permissions: &[String]) -> bool {
        user_permissions.iter().any(|p| 
            p == Permissions::ACCESS_PREMIUM_FEATURES || 
            p == Permissions::READ_ADVANCED_ANALYTICS
        )
    }
    
    /// Check if user can access advanced rankings
    pub fn can_access_advanced_rankings(user_permissions: &[String]) -> bool {
        user_permissions.iter().any(|p| 
            p == Permissions::READ_ADVANCED_ANALYTICS ||
            p == Permissions::READ_ALL
        )
    }
    
    /// Check if user can manage other users
    pub fn can_manage_users(user_permissions: &[String]) -> bool {
        user_permissions.iter().any(|p| 
            p == Permissions::MANAGE_USERS ||
            p == Permissions::MANAGE_SYSTEM
        )
    }
    
    /// Check if user can access system admin features
    pub fn can_access_system_admin(user_permissions: &[String]) -> bool {
        user_permissions.iter().any(|p| 
            p == Permissions::MANAGE_SYSTEM ||
            p == Permissions::MANAGE_ADMIN
        )
    }
    
    /// Check if user can moderate content
    pub fn can_moderate(user_permissions: &[String]) -> bool {
        user_permissions.iter().any(|p| 
            p == Permissions::MODERATE_CONTENT ||
            p == Permissions::MODERATE_USERS
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permission_groups() {
        let free_perms = PermissionGroups::free_tier();
        assert!(free_perms.contains(&Permissions::READ_BASIC));
        assert!(!free_perms.contains(&Permissions::MANAGE_USERS));
        
        let admin_perms = PermissionGroups::admin();
        assert!(admin_perms.contains(&Permissions::READ_OWN));
        assert!(admin_perms.contains(&Permissions::MANAGE_USERS));
        assert!(!admin_perms.contains(&Permissions::MANAGE_SYSTEM));
        
        let super_admin_perms = PermissionGroups::super_admin();
        assert!(super_admin_perms.contains(&Permissions::MANAGE_SYSTEM));
    }
    
    #[test]
    fn test_feature_permissions() {
        let user_perms = vec![
            Permissions::READ_OWN.to_string(),
            Permissions::ACCESS_PREMIUM_FEATURES.to_string()
        ];
        
        assert!(FeaturePermissions::can_access_premium_analytics(&user_perms));
        assert!(!FeaturePermissions::can_manage_users(&user_perms));
        
        let admin_perms = vec![
            Permissions::MANAGE_USERS.to_string(),
            Permissions::READ_ALL.to_string()
        ];
        
        assert!(FeaturePermissions::can_manage_users(&admin_perms));
        assert!(FeaturePermissions::can_access_advanced_rankings(&admin_perms));
    }
}