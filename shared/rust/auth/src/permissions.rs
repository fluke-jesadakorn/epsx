//! Role-derived permission helpers — Wave 6C Track A.
//!
//! The admin / frontend BFFs (`apps/admin/src/ssr.rs`,
//! `apps/frontend/src/ssr.rs`) used to plumb `permissions: vec![]`
//! into the rendered `User`, which made `AdminAuthGate` intercept
//! page bodies even when the user was an admin (because the
//! gate's `required_permissions` check ran against an empty
//! permission set and produced a `missing` list).
//!
//! The fix is role-derived: every role on the JWT's `roles` claim
//! gets a sensible default permission set, and the BFF unions
//! those sets into the `User` it hands to the Dioxus SSR layer.
//! The wildcard syntax (`*:*:*`) is the same one the backend's
//! `apps/backend/src/core/permissions.rs::has_permission` already
//! understands, so admins get the implicit "I can do everything"
//! grant without an extra DB roundtrip.
//!
//! Defense in depth: even if the BFF forgets to populate
//! permissions, `AdminAuthGate` short-circuits the
//! missing-permission check when `user.is_admin()` is true. See
//! `shared/rust/dioxus_ui/src/auth/auth_gate.rs` Layer 3.

/// Returns the default permission set for a single role. The
/// returned strings use the `domain:action:scope` schema the
/// backend's `has_permission` understands (wildcards allowed).
///
/// - `admin` / `super_admin` → `["*:*:*"]` (full wildcard)
/// - `editor` / `content_manager` → read on every resource plus
///   write on `admin:content`
/// - `merchant` → full access to payments + wallet resources
/// - `user` → read-only on the wallet resource
/// - anything else → empty (the BFF will not auto-grant perms
///   for unknown role tags; service-derived perms are a future
///   wave)
pub fn default_permissions_for_role(role: &str) -> Vec<String> {
    match role {
        "admin" | "super_admin" => vec!["*:*:*".into()],
        "editor" | "content_manager" => vec![
            "*:*:*:read".into(),
            "admin:content:write".into(),
        ],
        "merchant" => vec![
            "epsx:payments:*".into(),
            "epsx:wallet:*".into(),
        ],
        "user" => vec!["epsx:wallet:read".into()],
        _ => vec![],
    }
}

/// Flattens a list of roles into a single sorted, deduped
/// permission set. This is the function the BFFs call when they
/// build the `User` they hand to Dioxus SSR.
///
/// The sort + dedup means the same `(user, role, perm)` tuple
/// always serializes identically, which keeps the SSR HTML
/// cache key stable across requests.
pub fn permissions_for_roles(roles: &[String]) -> Vec<String> {
    let mut perms: Vec<String> = roles
        .iter()
        .flat_map(|r| default_permissions_for_role(r))
        .collect();
    perms.sort();
    perms.dedup();
    perms
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_permissions_for_role_admin_returns_wildcard() {
        let perms = default_permissions_for_role("admin");
        assert_eq!(perms, vec!["*:*:*".to_string()]);
    }

    #[test]
    fn default_permissions_for_role_super_admin_returns_wildcard() {
        // super_admin is a separate role tag from admin but
        // gets the same wildcard grant. Catches regressions
        // where a future change accidentally demotes super_admin.
        let perms = default_permissions_for_role("super_admin");
        assert_eq!(perms, vec!["*:*:*".to_string()]);
    }

    #[test]
    fn default_permissions_for_role_editor_returns_read_plus_content_write() {
        let perms = default_permissions_for_role("editor");
        assert!(perms.contains(&"*:*:*:read".to_string()));
        assert!(perms.contains(&"admin:content:write".to_string()));
        assert_eq!(perms.len(), 2);
    }

    #[test]
    fn default_permissions_for_role_content_manager_returns_read_plus_content_write() {
        // content_manager is the legacy alias for editor.
        let perms = default_permissions_for_role("content_manager");
        assert!(perms.contains(&"*:*:*:read".to_string()));
        assert!(perms.contains(&"admin:content:write".to_string()));
    }

    #[test]
    fn default_permissions_for_role_merchant_returns_payment_and_wallet() {
        let perms = default_permissions_for_role("merchant");
        assert!(perms.contains(&"epsx:payments:*".to_string()));
        assert!(perms.contains(&"epsx:wallet:*".to_string()));
        assert_eq!(perms.len(), 2);
    }

    #[test]
    fn default_permissions_for_role_user_returns_wallet_read() {
        let perms = default_permissions_for_role("user");
        assert_eq!(perms, vec!["epsx:wallet:read".to_string()]);
    }

    #[test]
    fn default_permissions_for_role_unknown_returns_empty() {
        // Unknown role tags must not auto-grant anything — this
        // is the "default deny" rule from the design doc.
        assert!(default_permissions_for_role("").is_empty());
        assert!(default_permissions_for_role("guest").is_empty());
        assert!(default_permissions_for_role("ADMIN").is_empty()); // case-sensitive
    }

    #[test]
    fn permissions_for_roles_unions_and_dedups() {
        // admin contributes one perm; user contributes one perm;
        // editor contributes two. Total: 4 distinct perms.
        let roles = vec![
            "admin".to_string(),
            "user".to_string(),
            "editor".to_string(),
        ];
        let perms = permissions_for_roles(&roles);
        assert!(perms.contains(&"*:*:*".to_string()));
        assert!(perms.contains(&"epsx:wallet:read".to_string()));
        assert!(perms.contains(&"*:*:*:read".to_string()));
        assert!(perms.contains(&"admin:content:write".to_string()));
        // Sorted ascending — verifies the sort() in the helper.
        let mut expected = perms.clone();
        expected.sort();
        assert_eq!(perms, expected);
    }

    #[test]
    fn permissions_for_roles_dedupes_overlapping_perms() {
        // Two admin roles is a no-op; dedup should collapse to one entry.
        let roles = vec!["admin".to_string(), "admin".to_string()];
        let perms = permissions_for_roles(&roles);
        assert_eq!(perms, vec!["*:*:*".to_string()]);
    }

    #[test]
    fn permissions_for_roles_empty_input_returns_empty() {
        let perms = permissions_for_roles(&[]);
        assert!(perms.is_empty());
    }
}
