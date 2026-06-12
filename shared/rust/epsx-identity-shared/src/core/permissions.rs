//! Permission rule logic — verbatim copy of
//! `apps/backend/src/core/permissions.rs` (CLAUDE.md "Permissions & Plan
//! Logic — Backend Only"). The backend binary's middleware continues
//! to use the canonical implementation; this duplicate allows the
//! extracted `epsx-identity-shared` crate to compile standalone. In a
//! later wave, the backend can `pub use` from here and delete its
//! local copy.

pub fn has_permission(user_permissions: &[String], required: &str) -> bool {
    user_permissions.iter().any(|p| {
        if p == required {
            return true;
        }
        if p == "*:*" || p == "*:*:*" {
            return true;
        }
        let parts: Vec<&str> = required.splitn(3, ':').collect();
        if parts.len() >= 3 {
            let platform_wc = format!("{}:*:*", parts[0]);
            if *p == platform_wc {
                return true;
            }
            let resource_wc = format!("{}:{}:*", parts[0], parts[1]);
            if *p == resource_wc {
                return true;
            }
        }
        false
    })
}

pub fn is_admin(user_permissions: &[String]) -> bool {
    user_permissions.iter().any(|p| {
        p == "admin:*:*" || p == "admin:dashboard:view" || p == "*:*" || p == "*:*:*"
    })
}

pub fn has_any_permission(user_permissions: &[String], required: &[&str]) -> bool {
    required.iter().any(|r| has_permission(user_permissions, r))
}

pub fn permission_platform(permission: &str) -> &str {
    permission.split(':').next().unwrap_or(permission)
}

pub fn has_admin_platform_permission(perms: &[String]) -> bool {
    perms.iter().any(|p| permission_platform(p) == "admin")
}
