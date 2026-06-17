use crate::primitives::icon::Icon;

use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

/// How the user authenticated. Mirrors the discriminator in the
/// `useSharedAuth()` TS provider — the Rust side just stores the tag
/// so the UI can render the right pill / icon.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthMethod {
    Wallet,
    Email,
    Demo,
    OAuth,
    Siwe,
    Unknown,
}

impl Default for AuthMethod {
    fn default() -> Self { Self::Unknown }
}

/// Existing user shape — extended with the optional auth metadata
/// (display name, last login timestamp, method tag). All new fields
/// are `Option`/`Default` so existing serialized users keep working.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub address: String,
    pub chain_id: String,
    pub roles: Vec<String>,
    pub email: Option<String>,
    pub tier: Option<String>,
    pub permissions: Vec<String>,
    /// ISO-8601 timestamp of the most recent successful sign-in.
    /// `None` when the user has never logged in (e.g. freshly
    /// provisioned by an admin).
    #[serde(default)]
    pub last_login_at: Option<String>,
    /// How the user most recently authenticated. Defaults to
    /// `AuthMethod::Unknown` for legacy records.
    #[serde(default)]
    pub auth_method: AuthMethod,
    /// Human-friendly display name (e.g. ENS, OAuth name). Used by
    /// the connected-wallet dropdown header and account overview.
    #[serde(default)]
    pub display_name: Option<String>,
}

impl User {
    pub fn is_authed(&self) -> bool { !self.id.is_empty() }

    pub fn short_address(&self) -> String {
        if self.address.len() < 10 { return self.address.clone(); }
        format!("{}…{}", &self.address[..6], &self.address[self.address.len()-4..])
    }

    pub fn is_admin(&self) -> bool {
        self.roles.iter().any(|r| r == "admin" || r == "super_admin" || r == "Admin")
    }

    pub fn has_permission(&self, p: &str) -> bool {
        // Wave 7 — wildcard-aware permission check.
        //
        // `User::permissions` is populated by the admin/frontend BFF
        // from the JWT role list (see `apps/{admin,frontend}/src/auth.rs`
        // → `permissions_for_roles`). The BFF may emit either
        // 2-segment perms (`<feature>:<action>`, e.g. `analytics:read`,
        // `admin:*`) or 3-segment perms (`<platform>:<resource>:<action>`,
        // e.g. `admin:users:manage`) — the same two grammars the page
        // gates use. `has_permission` accepts both.
        //
        // Wildcard rules (mirrors `apps/backend/src/core/permissions.rs`
        // for 3-segment; adds 2-segment `prefix:*` for the UI layer):
        //
        //   1. Exact match: `<p> == <r>` → ✓
        //   2. Super-admin: `p == "*:*"` or `p == "*:*:*"` → ✓
        //   3. 3-segment platform wildcard: `p == "<pl>:*:*"` matches
        //      any 3-segment required perm with the same platform.
        //   4. 3-segment resource wildcard: `p == "<pl>:<re>:*"` matches
        //      any 3-segment required perm with the same platform+resource.
        //   5. 2-segment prefix wildcard: `p` is 2-segment and ends in
        //      `:*`; matches any 2-segment required perm with the same
        //      prefix. (`admin:*` matches `admin:auth`, `admin:read`, ...)
        //
        // Cross-grammar wildcards (2-seg `admin:*` matching 3-seg
        // `admin:users:read`) are intentionally NOT supported — they
        // would silently grant access across grammars and make the
        // perms a BFF emits hard to reason about.
        //
        // Behavior change from pre-wave7: the old impl was exact-match
        // only, which is a strict subset of this impl. Every previous
        // call site that passed still passes; new wildcard-bearing
        // perms now resolve correctly.
        self.permissions.iter().any(|x| permission_matches(x, p))
    }

    /// Returns `true` when the user has the given role (case-insensitive,
    /// exact match). Empty / unknown role tags never match.
    pub fn has_role(&self, role: &str) -> bool {
        if role.is_empty() { return false; }
        self.roles.iter().any(|r| r.eq_ignore_ascii_case(role))
    }

    /// Returns `true` when the user has at least one of the given roles.
    /// Empty input list returns `false` (consistent with the
    /// "any-permission" pattern used elsewhere in the app).
    pub fn has_any_role(&self, roles: &[&str]) -> bool {
        if roles.is_empty() { return false; }
        roles.iter().any(|r| self.has_role(r))
    }

    /// Returns the display name when set, falling back to the short
    /// wallet address, then to the bare email local-part, then to
    /// a generic "Guest" placeholder. Never returns an empty string.
    pub fn display_name_or_fallback(&self) -> String {
        if let Some(n) = &self.display_name {
            if !n.is_empty() { return n.clone(); }
        }
        if !self.address.is_empty() { return self.short_address(); }
        if let Some(email) = &self.email {
            if let Some(at) = email.find('@') {
                return email[..at].to_string();
            }
            return email.clone();
        }
        "Guest".to_string()
    }

    /// Tiny helper to render a leading icon appropriate for the user's
    /// auth method. Useful for the connected-wallet dropdown header
    /// and the access-denied audit row. Returns a wallet icon for
    /// all on-chain methods, mail for email/OAuth, and the generic
    /// user icon for everything else.
    pub fn method_icon(&self) -> String {
        match self.auth_method {
            AuthMethod::Wallet | AuthMethod::Siwe => "wallet".to_string(),
            AuthMethod::Email => "mail".to_string(),
            AuthMethod::OAuth => "shield".to_string(),
            AuthMethod::Demo => "user".to_string(),
            AuthMethod::Unknown => {
                if self.is_authed() { "user".to_string() } else { "user".to_string() }
            }
        }
    }
}

impl Default for User {
    fn default() -> Self {
        Self {
            id: String::new(),
            address: String::new(),
            chain_id: "56".to_string(),
            roles: vec![],
            email: None,
            tier: None,
            permissions: vec![],
            last_login_at: None,
            auth_method: AuthMethod::Unknown,
            display_name: None,
        }
    }
}

/// Convenience render helper: a small "Logged in via <method>" pill
/// that the connected-wallet dropdown and account pages can drop in
/// without re-implementing the icon/label mapping.
#[component]
pub fn AuthMethodPill(user: User) -> Element {
    let label = match user.auth_method {
        AuthMethod::Wallet => "Wallet",
        AuthMethod::Siwe => "SIWE",
        AuthMethod::Email => "Email",
        AuthMethod::OAuth => "OAuth",
        AuthMethod::Demo => "Demo",
        AuthMethod::Unknown => "Signed in",
    };
    let icon = user.method_icon();
    rsx! {
        span { class: "auth-method-pill",
            span { class: "auth-method-pill-icon", Icon { name: icon, size: Some(12) } }
            span { class: "auth-method-pill-label", "{label}" }
        }
    }
}

/// Check whether a held permission `held` satisfies a required
/// permission `req`. Exposed at module scope so the BFF test
/// harnesses and the gate's permission filter can call it
/// independently of `User`.
///
/// See `User::has_permission` for the full rules. Summary:
/// - exact match
/// - `held` is a 2-seg or 3-seg super-admin wildcard
/// - both 3-seg: `held` is `pl:*:*` or `pl:re:*` for the same `pl`/`re`
/// - both 2-seg: `held` is `prefix:*` and `req` starts with `prefix:`
/// - cross-grammar wildcards (2-seg matching 3-seg or vice versa) → false
pub fn permission_matches(held: &str, req: &str) -> bool {
    // 1. Exact match.
    if held == req {
        return true;
    }

    // 2. Super-admin wildcards — match any required perm.
    if held == "*:*" || held == "*:*:*" {
        return true;
    }

    let held_parts: Vec<&str> = held.split(':').collect();
    let req_parts: Vec<&str> = req.split(':').collect();

    match (held_parts.len(), req_parts.len()) {
        // 3. Both 3-segment: platform / resource / action wildcards.
        (3, 3) => {
            // held is a 3-seg wildcard only if its last segment is `*`.
            if held_parts[2] != "*" {
                return false;
            }
            // platform wildcard: `<pl>:*:*`
            if held_parts[1] == "*" {
                return held_parts[0] == req_parts[0];
            }
            // resource wildcard: `<pl>:<re>:*`
            held_parts[0] == req_parts[0] && held_parts[1] == req_parts[1]
        }
        // 4. Both 2-segment: prefix wildcard.
        (2, 2) => {
            // held wildcard only if its last segment is `*`.
            if held_parts[1] != "*" {
                return false;
            }
            // `prefix:*` matches `prefix:anything`.
            held_parts[0] == req_parts[0]
        }
        // 5. Cross-grammar: no match (intentional).
        _ => false,
    }
}

#[cfg(test)]
mod perm_match_tests {
    //! Wildcard-aware permission matching — the heart of the
    //! wave7 `has_permission` upgrade. Mirrors the test layout in
    //! `apps/backend/src/core/permissions.rs` (3-seg cases) and
    //! adds the 2-seg prefix wildcard that the UI layer needs.
    use super::permission_matches;

    // ── 2-segment exact + prefix wildcard ──────────────────────

    #[test]
    fn two_seg_exact_match() {
        assert!(permission_matches("analytics:read", "analytics:read"));
    }

    #[test]
    fn two_seg_no_match() {
        assert!(!permission_matches("analytics:read", "policies:read"));
    }

    #[test]
    fn two_seg_prefix_wildcard() {
        // `admin:*` is the admin platform catch-all used by the
        // AdminAuthGate. It must match any 2-seg admin perm.
        assert!(permission_matches("admin:*", "admin:auth"));
        assert!(permission_matches("admin:*", "admin:read"));
        assert!(permission_matches("admin:*", "admin:manage"));
        // Not admin perms.
        assert!(!permission_matches("admin:*", "analytics:read"));
        assert!(!permission_matches("admin:*", "policies:manage"));
    }

    #[test]
    fn two_seg_prefix_wildcard_does_not_match_3seg() {
        // Cross-grammar: 2-seg `admin:*` must NOT silently grant
        // 3-seg `admin:users:read` access.
        assert!(!permission_matches("admin:*", "admin:users:read"));
    }

    // ── 3-segment exact + wildcards (mirrors backend) ──────────

    #[test]
    fn three_seg_exact_match() {
        assert!(permission_matches("admin:users:read", "admin:users:read"));
    }

    #[test]
    fn three_seg_no_match() {
        assert!(!permission_matches("admin:users:read", "admin:users:write"));
        assert!(!permission_matches("admin:users:read", "epsx:analytics:read"));
    }

    #[test]
    fn three_seg_platform_wildcard() {
        assert!(permission_matches("admin:*:*", "admin:users:read"));
        assert!(permission_matches("admin:*:*", "admin:permissions:read"));
        // Cross-platform: must not match.
        assert!(!permission_matches("admin:*:*", "epsx:analytics:read"));
    }

    #[test]
    fn three_seg_resource_wildcard() {
        assert!(permission_matches("admin:users:*", "admin:users:read"));
        assert!(permission_matches("admin:users:*", "admin:users:write"));
        // Cross-resource: must not match.
        assert!(!permission_matches("admin:users:*", "admin:groups:read"));
    }

    // ── Super-admin wildcards (any grammar) ────────────────────

    #[test]
    fn super_admin_two_seg_matches_anything() {
        assert!(permission_matches("*:*", "analytics:read"));
        assert!(permission_matches("*:*", "admin:auth"));
        assert!(permission_matches("*:*", "any:thing"));
    }

    #[test]
    fn super_admin_three_seg_matches_anything() {
        assert!(permission_matches("*:*:*", "admin:users:read"));
        assert!(permission_matches("*:*:*", "epsx:analytics:read"));
        assert!(permission_matches("*:*:*", "epsx-pay:payments:create"));
    }

    // ── Cross-grammar: must not match ──────────────────────────

    #[test]
    fn three_seg_held_does_not_match_two_seg_req() {
        assert!(!permission_matches("admin:users:*", "admin:read"));
        assert!(!permission_matches("admin:*:*", "analytics:read"));
    }

    #[test]
    fn two_seg_held_does_not_match_three_seg_req() {
        // 2-seg `analytics:read` is a literal UI-layer perm; it must
        // NOT match a 3-seg required perm like `epsx:analytics:read`.
        assert!(!permission_matches("analytics:read", "epsx:analytics:read"));
    }

    // ── Edge cases ─────────────────────────────────────────────

    #[test]
    fn empty_held_does_not_satisfy_a_required_perm() {
        // The held set has no useful perms; required perms should
        // not match anything (no wildcards present).
        assert!(!permission_matches("", "analytics:read"));
        assert!(!permission_matches("", "admin:read"));
    }

    #[test]
    fn empty_required_does_not_match_a_held_perm() {
        // A required perm of "" is a programming error (no gate
        // declares an empty required perm), but defensively it
        // should not match a non-empty held perm. (Exact match
        // between two empty strings would return true, but that's
        // a degenerate case and we don't test it.)
        assert!(!permission_matches("analytics:read", ""));
    }

    #[test]
    fn non_wildcard_does_not_act_as_wildcard() {
        // A non-wildcard held perm must not match a different req.
        assert!(!permission_matches("admin:auth", "admin:read"));
        assert!(!permission_matches("admin:users:read", "admin:users:write"));
    }

    #[test]
    fn single_segment_does_not_match() {
        // `<pl>:*` requires 2 segments; a single `*` is not a valid
        // wildcard and must not match anything.
        assert!(!permission_matches("*", "analytics:read"));
        assert!(!permission_matches("admin", "admin:read"));
    }

    #[test]
    fn exact_match_wins_even_for_unusual_grammars() {
        // The function's first rule is exact-match equality. Two
        // 4-segment strings that happen to be equal will match —
        // this is defensive against unknown grammars (no call
        // site emits ≥4-segment perms, but if two records both
        // carry the same 4-seg string, they're treated as
        // equivalent). Documented behavior, not a security gap:
        // no wildcard logic ever upgrades a non-match to a match.
        assert!(permission_matches("a:b:c:d", "a:b:c:d"));
    }
}
