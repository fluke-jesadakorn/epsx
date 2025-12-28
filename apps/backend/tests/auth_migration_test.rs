use epsx::auth::UnifiedWeb3AuthService;

/// Integration tests for auth migration
#[cfg(test)]
mod tests {
    use super::*;

    // Verify UnifiedWeb3AuthService::has_permission logic (Unit Test logic)
    #[test]
    fn test_has_permission_logic() {
        let permissions = vec![
            "admin:users:read".to_string(),
            "epsx:analytics:*".to_string(),
        ];

        // Exact match
        assert!(UnifiedWeb3AuthService::has_permission(&permissions, "admin:users:read"));
        
        // Wildcard match
        assert!(UnifiedWeb3AuthService::has_permission(&permissions, "epsx:analytics:view"));

        // No match
        assert!(!UnifiedWeb3AuthService::has_permission(&permissions, "admin:users:write"));
        
        // Testing super admin separately
        let super_perms = vec!["admin:*:*".to_string()];
        assert!(UnifiedWeb3AuthService::has_permission(&super_perms, "any:permission:check"));
    }

    // Verify UnifiedWeb3AuthService::is_admin logic
    #[test]
    fn test_is_admin_logic() {
        let admin_perms = vec!["admin:users:read".to_string()];
        assert!(UnifiedWeb3AuthService::is_admin(&admin_perms));

        let super_admin = vec!["admin:*:*".to_string()];
        assert!(UnifiedWeb3AuthService::is_admin(&super_admin));

        let user_perms = vec!["epsx:view".to_string()];
        assert!(!UnifiedWeb3AuthService::is_admin(&user_perms));
    }
}
