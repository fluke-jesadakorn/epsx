// Unified permission constants and definitions
// Standard format: <domain>:<action>:<scope>

/// Core permission constants used throughout the application
pub mod permissions {
    // User management permissions
    pub const USERS_READ_ALL: &str = "users:read:all";
    pub const USERS_READ_OWN: &str = "users:read:own";
    pub const USERS_WRITE_ALL: &str = "users:write:all";
    pub const USERS_WRITE_OWN: &str = "users:write:own";
    pub const USERS_DELETE_ALL: &str = "users:delete:all";
    pub const USERS_MANAGE: &str = "users:manage:all";

    // Dashboard permissions
    pub const DASHBOARD_VIEW_BASIC: &str = "dashboard:view:basic";
    pub const DASHBOARD_VIEW_PREMIUM: &str = "dashboard:view:premium";
    pub const DASHBOARD_VIEW_ADMIN: &str = "dashboard:view:admin";

    // Analytics permissions
    pub const ANALYTICS_VIEW_BASIC: &str = "analytics:view:basic";
    pub const ANALYTICS_VIEW_PREMIUM: &str = "analytics:view:premium";
    pub const ANALYTICS_EXPORT: &str = "analytics:export:data";

    // Admin permissions
    pub const ADMIN_ACCESS: &str = "admin:access:all";
    pub const ADMIN_USERS_MANAGE: &str = "admin:users:manage";
    pub const ADMIN_SYSTEM_CONFIG: &str = "admin:system:configure";

    // Market data permissions
    pub const MARKET_DATA_BASIC: &str = "market:data:basic";
    pub const MARKET_DATA_REALTIME: &str = "market:data:realtime";
    pub const MARKET_DATA_HISTORICAL: &str = "market:data:historical";

    // Rankings permissions (tier-based)
    pub const RANKINGS_BASIC: &str = "rankings:view:basic";
    pub const RANKINGS_TECHNICAL: &str = "rankings:view:technical";
    pub const RANKINGS_AI_INSIGHTS: &str = "rankings:view:ai";
    pub const RANKINGS_CUSTOM: &str = "rankings:create:custom";

    // Wildcard permissions for admins
    pub const ALL_PERMISSIONS: &str = "*:*:*";
    pub const ADMIN_WILDCARD: &str = "admin:*:*";
    pub const USER_WILDCARD: &str = "users:*:own";
}

/// Access tier for permission requirements
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum AccessTier {
    Free,
    Bronze,
    Silver,  
    Gold,
    Platinum,
    Admin,
    SuperAdmin,
}

impl AccessTier {
    pub fn hierarchy_level(&self) -> u8 {
        match self {
            AccessTier::Free => 0,
            AccessTier::Bronze => 1,
            AccessTier::Silver => 2,
            AccessTier::Gold => 3,
            AccessTier::Platinum => 4,
            AccessTier::Admin => 5,
            AccessTier::SuperAdmin => 6,
        }
    }
    
    pub fn can_access_tier(&self, required_tier: &AccessTier) -> bool {
        self.hierarchy_level() >= required_tier.hierarchy_level()
    }
    
    pub fn from_package_tier(package_tier: &crate::dom::entities::iam::PackageTier) -> Self {
        match package_tier {
            crate::dom::entities::iam::PackageTier::Free => AccessTier::Free,
            crate::dom::entities::iam::PackageTier::Bronze => AccessTier::Bronze,
            crate::dom::entities::iam::PackageTier::Silver => AccessTier::Silver,
            crate::dom::entities::iam::PackageTier::Gold => AccessTier::Gold,
            crate::dom::entities::iam::PackageTier::Platinum => AccessTier::Platinum,
            crate::dom::entities::iam::PackageTier::Admin => AccessTier::Admin,
            crate::dom::entities::iam::PackageTier::SuperAdmin => AccessTier::SuperAdmin,
        }
    }
}

impl std::fmt::Display for AccessTier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AccessTier::Free => write!(f, "free"),
            AccessTier::Bronze => write!(f, "bronze"),
            AccessTier::Silver => write!(f, "silver"),
            AccessTier::Gold => write!(f, "gold"),
            AccessTier::Platinum => write!(f, "platinum"),
            AccessTier::Admin => write!(f, "admin"),
            AccessTier::SuperAdmin => write!(f, "super_admin"),
        }
    }
}

/// Package tier to permission mapping for easy permission derivation
pub fn get_package_tier_permissions(package_tier: &crate::dom::entities::iam::PackageTier) -> Vec<&'static str> {
    match package_tier {
        crate::dom::entities::iam::PackageTier::Free => vec![
            permissions::USERS_READ_OWN,
            permissions::USERS_WRITE_OWN,
            permissions::DASHBOARD_VIEW_BASIC,
            permissions::MARKET_DATA_BASIC,
        ],
        crate::dom::entities::iam::PackageTier::Bronze => vec![
            permissions::USERS_READ_OWN,
            permissions::USERS_WRITE_OWN,
            permissions::DASHBOARD_VIEW_BASIC,
            permissions::MARKET_DATA_BASIC,
            permissions::ANALYTICS_VIEW_BASIC,
        ],
        crate::dom::entities::iam::PackageTier::Silver => vec![
            permissions::USERS_READ_OWN,
            permissions::USERS_WRITE_OWN,
            permissions::DASHBOARD_VIEW_BASIC,
            permissions::MARKET_DATA_BASIC,
            permissions::MARKET_DATA_REALTIME,
            permissions::ANALYTICS_VIEW_BASIC,
            permissions::RANKINGS_BASIC,
        ],
        crate::dom::entities::iam::PackageTier::Gold => vec![
            permissions::USERS_READ_OWN,
            permissions::USERS_WRITE_OWN,
            permissions::DASHBOARD_VIEW_PREMIUM,
            permissions::MARKET_DATA_BASIC,
            permissions::MARKET_DATA_REALTIME,
            permissions::ANALYTICS_VIEW_PREMIUM,
            permissions::RANKINGS_BASIC,
            permissions::RANKINGS_TECHNICAL,
        ],
        crate::dom::entities::iam::PackageTier::Platinum => vec![
            permissions::USERS_READ_OWN,
            permissions::USERS_WRITE_OWN,
            permissions::DASHBOARD_VIEW_PREMIUM,
            permissions::MARKET_DATA_BASIC,
            permissions::MARKET_DATA_REALTIME,
            permissions::ANALYTICS_VIEW_PREMIUM,
            permissions::RANKINGS_BASIC,
            permissions::RANKINGS_TECHNICAL,
            permissions::RANKINGS_AI_INSIGHTS,
        ],
        crate::dom::entities::iam::PackageTier::Admin => vec![
            permissions::ADMIN_ACCESS,
            permissions::ADMIN_USERS_MANAGE,
            permissions::USERS_MANAGE,
            permissions::DASHBOARD_VIEW_ADMIN,
            permissions::MARKET_DATA_HISTORICAL,
            permissions::ANALYTICS_EXPORT,
            permissions::RANKINGS_CUSTOM,
            permissions::ALL_PERMISSIONS,
        ],
        crate::dom::entities::iam::PackageTier::SuperAdmin => vec![
            permissions::ADMIN_ACCESS,
            permissions::ADMIN_USERS_MANAGE,
            permissions::USERS_MANAGE,
            permissions::DASHBOARD_VIEW_ADMIN,
            permissions::MARKET_DATA_HISTORICAL,
            permissions::ANALYTICS_EXPORT,
            permissions::RANKINGS_CUSTOM,
            permissions::ALL_PERMISSIONS,
        ],
    }
}

/// Check if permission matches a pattern (supports wildcards)
pub fn permission_matches(permission: &str, pattern: &str) -> bool {
    if pattern == permissions::ALL_PERMISSIONS {
        return true;
    }
    
    if permission == pattern {
        return true;
    }
    
    // Handle wildcard patterns
    if pattern.ends_with(":*:*") {
        let domain = &pattern[..pattern.len() - 4];
        return permission.starts_with(&format!("{}:", domain));
    }
    
    if pattern.ends_with(":*") {
        let prefix = &pattern[..pattern.len() - 1];
        return permission.starts_with(prefix);
    }
    
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_access_tier_hierarchy() {
        assert!(AccessTier::Gold.can_access_tier(&AccessTier::Bronze));
        assert!(AccessTier::Admin.can_access_tier(&AccessTier::Platinum));
        assert!(!AccessTier::Bronze.can_access_tier(&AccessTier::Gold));
    }
    
    #[test]
    fn test_permission_matching() {
        assert!(permission_matches("users:read:all", "users:read:all"));
        assert!(permission_matches("users:read:all", "users:*"));
        assert!(permission_matches("users:read:all", "*:*:*"));
        assert!(!permission_matches("users:read:all", "admin:*"));
    }
    
    #[test]
    fn test_package_tier_permissions() {
        let admin_perms = get_package_tier_permissions(&crate::dom::entities::iam::PackageTier::Admin);
        assert!(admin_perms.contains(&permissions::ADMIN_ACCESS));
        assert!(admin_perms.contains(&permissions::USERS_MANAGE));
        
        let user_perms = get_package_tier_permissions(&crate::dom::entities::iam::PackageTier::Free);
        assert!(user_perms.contains(&permissions::USERS_READ_OWN));
        assert!(!user_perms.contains(&permissions::ADMIN_ACCESS));
    }
}