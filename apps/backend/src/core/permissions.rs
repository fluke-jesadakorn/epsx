/// Unified permission checking — single source of truth.
/// All permission wildcard matching goes through these functions.
/// Format: "platform:resource:action" (e.g., "admin:users:manage")

/// Check if permission set satisfies a required permission.
/// Supports: exact match, `*:*` super-admin, `platform:*:*`, `platform:resource:*`
pub fn has_permission(user_permissions: &[String], required: &str) -> bool {
    user_permissions.iter().any(|p| {
        if p == required {
            return true;
        }

        // super-admin wildcard
        if p == "*:*" {
            return true;
        }

        let parts: Vec<&str> = required.splitn(3, ':').collect();
        if parts.len() >= 3 {
            // platform:*:*
            let platform_wc = format!("{}:*:*", parts[0]);
            if *p == platform_wc {
                return true;
            }

            // platform:resource:*
            let resource_wc = format!("{}:{}:*", parts[0], parts[1]);
            if *p == resource_wc {
                return true;
            }
        }

        false
    })
}

/// Check if permission set contains admin privileges.
/// True for: `admin:*:*`, `admin:dashboard:view`, `*:*`
pub fn is_admin(user_permissions: &[String]) -> bool {
    user_permissions.iter().any(|p| {
        p == "admin:*:*" || p == "admin:dashboard:view" || p == "*:*"
    })
}

/// Check if any of the required permissions are satisfied.
pub fn has_any_permission(user_permissions: &[String], required: &[&str]) -> bool {
    required.iter().any(|r| has_permission(user_permissions, r))
}

/// Extract platform prefix: `"admin:users:read"` → `"admin"`
pub fn permission_platform(permission: &str) -> &str {
    permission.split(':').next().unwrap_or(permission)
}

/// Check if any permission belongs to admin platform.
/// Replaces scattered `starts_with("admin:")` checks.
pub fn has_admin_platform_permission(perms: &[String]) -> bool {
    perms.iter().any(|p| permission_platform(p) == "admin")
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── has_permission ──────────────────────────────────────────

    #[test]
    fn exact_match() {
        let perms = vec!["epsx:analytics:read".into()];
        assert!(has_permission(&perms, "epsx:analytics:read"));
    }

    #[test]
    fn no_match() {
        let perms = vec!["epsx:analytics:read".into()];
        assert!(!has_permission(&perms, "admin:users:manage"));
    }

    #[test]
    fn platform_wildcard() {
        let perms = vec!["admin:*:*".into()];
        assert!(has_permission(&perms, "admin:users:manage"));
        assert!(has_permission(&perms, "admin:permissions:read"));
        assert!(!has_permission(&perms, "epsx:analytics:read"));
    }

    #[test]
    fn resource_wildcard() {
        let perms = vec!["epsx:analytics:*".into()];
        assert!(has_permission(&perms, "epsx:analytics:read"));
        assert!(has_permission(&perms, "epsx:analytics:write"));
        assert!(!has_permission(&perms, "epsx:export:csv"));
    }

    #[test]
    fn super_admin_wildcard() {
        let perms = vec!["*:*".into()];
        assert!(has_permission(&perms, "admin:users:manage"));
        assert!(has_permission(&perms, "epsx:analytics:read"));
        assert!(has_permission(&perms, "epsx-pay:payments:create"));
    }

    #[test]
    fn empty_permissions() {
        let perms: Vec<String> = vec![];
        assert!(!has_permission(&perms, "epsx:analytics:read"));
    }

    #[test]
    fn two_part_permission_no_wildcard() {
        let perms = vec!["analytics:basic".into()];
        assert!(has_permission(&perms, "analytics:basic"));
        assert!(!has_permission(&perms, "analytics:premium"));
    }

    #[test]
    fn multiple_permissions() {
        let perms = vec![
            "epsx:analytics:read".into(),
            "epsx:export:csv".into(),
            "admin:dashboard:view".into(),
        ];
        assert!(has_permission(&perms, "epsx:analytics:read"));
        assert!(has_permission(&perms, "epsx:export:csv"));
        assert!(has_permission(&perms, "admin:dashboard:view"));
        assert!(!has_permission(&perms, "admin:users:manage"));
    }

    // ── is_admin ────────────────────────────────────────────────

    #[test]
    fn is_admin_with_wildcard() {
        let perms = vec!["admin:*:*".into()];
        assert!(is_admin(&perms));
    }

    #[test]
    fn is_admin_with_dashboard_view() {
        let perms = vec!["admin:dashboard:view".into()];
        assert!(is_admin(&perms));
    }

    #[test]
    fn is_admin_with_super_admin() {
        let perms = vec!["*:*".into()];
        assert!(is_admin(&perms));
    }

    #[test]
    fn is_admin_false_for_regular_user() {
        let perms = vec!["epsx:analytics:read".into()];
        assert!(!is_admin(&perms));
    }

    #[test]
    fn is_admin_false_for_non_admin_prefix() {
        // Having some admin-scoped permission does NOT make you admin
        let perms = vec!["admin:users:read".into()];
        assert!(!is_admin(&perms));
    }

    #[test]
    fn is_admin_empty() {
        let perms: Vec<String> = vec![];
        assert!(!is_admin(&perms));
    }

    // ── has_any_permission ──────────────────────────────────────

    #[test]
    fn has_any_one_matches() {
        let perms = vec!["epsx:analytics:read".into()];
        assert!(has_any_permission(&perms, &["admin:users:manage", "epsx:analytics:read"]));
    }

    #[test]
    fn has_any_none_match() {
        let perms = vec!["epsx:analytics:read".into()];
        assert!(!has_any_permission(&perms, &["admin:users:manage", "epsx:export:csv"]));
    }

    // ── permission_platform ─────────────────────────────────────

    #[test]
    fn platform_extraction() {
        assert_eq!(permission_platform("admin:users:read"), "admin");
        assert_eq!(permission_platform("epsx:analytics:read"), "epsx");
        assert_eq!(permission_platform("epsx-pay:payments:create"), "epsx-pay");
        assert_eq!(permission_platform("standalone"), "standalone");
    }

    // ── has_admin_platform_permission ───────────────────────────

    #[test]
    fn has_admin_platform_true() {
        let perms = vec!["admin:users:manage".into(), "epsx:analytics:read".into()];
        assert!(has_admin_platform_permission(&perms));
    }

    #[test]
    fn has_admin_platform_false() {
        let perms = vec!["epsx:analytics:read".into()];
        assert!(!has_admin_platform_permission(&perms));
    }

    #[test]
    fn has_admin_platform_empty() {
        let perms: Vec<String> = vec![];
        assert!(!has_admin_platform_permission(&perms));
    }

    #[test]
    fn has_admin_platform_wildcard() {
        let perms = vec!["admin:*:*".into()];
        assert!(has_admin_platform_permission(&perms));
    }
}
