//! `AuthGate` — the "you must sign in" page wrapper.
//!
//! Wave 2 Track C additions:
//! - `required_permissions: Vec<String>` — render the list of missing
//!   permissions so the user knows why the gate fired.
//! - `return_url: Option<String>` — appended to the "Connect Wallet"
//!   link as a `?next=<url>` query string so the SIWE flow can bounce
//!   the user back to the original destination post-login.
//! - `AdminAuthGate` — admin-only variant. Wraps `AuthGate` and adds
//!   the admin-specific copy (shield icon, "Admin" badge, role check
//!   via `is_admin`).
//! - `is_gated: Option<bool>` — when `false`, the gate is a no-op and
//!   children are rendered unconditionally. Defaults to `true`. Useful
//!   for preview/dev environments where auth is bypassed.
//! - The user-facing reason text is now derived from a configurable
//!   `reason` prop (defaulting to "Sign in required").
//!
//! All existing call sites (e.g. `pages/account.rs`,
//! `pages/admin_pages/dashboard.rs`) keep compiling because every
//! new parameter has a `Default` via `#[props(default = ...)]`.

use crate::primitives::icon::Icon;

use dioxus::prelude::*;

/// Gate that renders `children` only when `user.is_some()`. When the
/// user is signed out, a "Sign in required" panel is shown with a
/// "Connect Wallet" link and a "Back to home" fallback.
///
/// New in Wave 2 Track C:
/// - `required_permissions` — when the user is signed in but missing
///   one of the listed permissions, the gate also fires. Renders a
///   bulleted list of the missing permissions in the panel.
/// - `return_url` — appended to the connect link as `?next=<url>`.
/// - `is_gated: bool` — when `false`, the gate is a no-op.
#[component]
pub fn AuthGate(
    user: Option<super::user::User>,
    /// Optional feature name (e.g. "your dashboard") shown in the
    /// gate copy. Kept from Wave 1.
    feature: Option<String>,
    /// When set, the gate fires if the user is missing any of these
    /// permissions. Ignored when the user is signed out (the sign-in
    /// gate takes priority).
    #[props(default = None)] required_permissions: Option<Vec<String>>,
    /// URL the user should be sent to after a successful sign-in.
    /// Forwarded as a `?next=...` query string on the connect link.
    #[props(default = None)] return_url: Option<String>,
    /// Optional override for the gate headline. Defaults to
    /// "Sign in required".
    #[props(default = None)] reason: Option<String>,
    /// When `false`, the gate is bypassed and `children` are rendered
    /// unconditionally. Defaults to `true`.
    #[props(default = true)] is_gated: bool,
    /// Optional click handler for the "Connect Wallet" button. When
    /// set, the link is replaced with a button (useful when the
    /// caller wants to open an in-page modal rather than navigate).
    #[props(default = None)] on_connect: Option<EventHandler<MouseEvent>>,
    /// Extra class names for the gate panel.
    #[props(default = None)] class_name: Option<String>,
    children: Element,
) -> Element {
    if !is_gated {
        return rsx! { Fragment { {children} } };
    }

    // Case 1 — user is signed in and has all required permissions.
    if let Some(u) = &user {
        let missing = match &required_permissions {
            None => vec![],
            Some(perms) => perms.iter().filter(|p| !u.has_permission(p)).cloned().collect(),
        };
        if missing.is_empty() {
            return rsx! { Fragment { {children} } };
        }
        let extra_cls2 = class_name.unwrap_or_default();
        // Case 2 — signed in but missing required permissions.
        return rsx! {
            div { class: "auth-gate auth-gate-missing {extra_cls2}", role: "alert",
                div { class: "auth-gate-icon", Icon { name: "shield".to_string(), size: Some(48) } }
                h2 { class: "auth-gate-title", "Permission required" }
                p { class: "auth-gate-description",
                    "Your account is signed in but does not have access to this page."
                }
                if !missing.is_empty() {
                    div { class: "auth-gate-perms",
                        p { "Missing permissions:" }
                        ul { for p in missing.iter() { li { "{p}" } } }
                    }
                }
                div { class: "auth-gate-actions",
                    a { class: "btn btn-primary", href: "/", "Back to home" }
                }
            }
        };
    }

    // Case 3 — signed out. The original Wave 1 behavior.
    let connect_href = match &return_url {
        Some(u) if !u.is_empty() => format!("/auth?next={}", u),
        _ => "/auth".to_string(),
    };
    let headline = reason.unwrap_or_else(|| "Sign in required".to_string());
    let extra_cls = class_name.unwrap_or_default();
    rsx! {
        div { class: "auth-gate {extra_cls}", role: "alert",
            div { class: "auth-gate-icon", Icon { name: "wallet".to_string(), size: Some(48) } }
            h2 { class: "auth-gate-title", "{headline}" }
            p { class: "auth-gate-description",
                if let Some(f) = feature {
                    "Sign in to access "
                    strong { "{f}" }
                    "."
                } else {
                    "Please connect your wallet to continue."
                }
            }
            if let Some(perms) = &required_permissions {
                if !perms.is_empty() {
                    div { class: "auth-gate-perms",
                        p { "Required permissions:" }
                        ul { for p in perms.iter() { li { "{p}" } } }
                    }
                }
            }
            div { class: "auth-gate-actions",
                if let Some(h) = on_connect.clone() {
                    button {
                        class: "btn btn-primary",
                        r#type: "button",
                        onclick: move |e| h.call(e),
                        "Connect Wallet"
                    }
                } else {
                    a { class: "btn btn-primary", href: "{connect_href}", "Connect Wallet" }
                }
                a { class: "btn btn-outline", href: "/", "Back to home" }
            }
        }
    }
}

/// Admin-only auth gate. Identical to `AuthGate` but:
/// 1. Fires if the user is signed out **or** signed in but
///    `!user.is_admin()`.
/// 2. Renders an "Admin" pill in the gate header.
/// 3. Default `reason` is "Admin access required".
#[component]
pub fn AdminAuthGate(
    user: Option<super::user::User>,
    #[props(default = None)] feature: Option<String>,
    #[props(default = None)] required_permissions: Option<Vec<String>>,
    #[props(default = None)] return_url: Option<String>,
    #[props(default = None)] reason: Option<String>,
    #[props(default = true)] is_gated: bool,
    #[props(default = None)] on_connect: Option<EventHandler<MouseEvent>>,
    #[props(default = None)] class_name: Option<String>,
    children: Element,
) -> Element {
    if !is_gated {
        return rsx! { Fragment { {children} } };
    }
    let has_admin = user.as_ref().map(|u| u.is_admin()).unwrap_or(false);
    if has_admin {
        // Wave 6C Track A — defense in depth. An admin user with
        // `required_permissions: Some(vec!["anything".into()])`
        // must always pass the gate, even if the BFF plumbed
        // `permissions: vec![]` (the pre-Wave-6C default). The
        // previous code entered this block but then ran the
        // `missing` check, which produced a non-empty list when
        // the user had no permissions and the gate fired.
        return rsx! { Fragment { {children} } };
    }
    let connect_href = match &return_url {
        Some(u) if !u.is_empty() => format!("/admin?next={}", u),
        _ => "/admin".to_string(),
    };
    let headline = reason.unwrap_or_else(|| "Admin access required".to_string());
    let extra_cls = class_name.unwrap_or_default();
    rsx! {
        div { class: "auth-gate auth-gate-admin {extra_cls}", role: "alert",
            div { class: "auth-gate-icon", Icon { name: "shield".to_string(), size: Some(48) } }
            span { class: "auth-gate-badge", "Admin" }
            h2 { class: "auth-gate-title", "{headline}" }
            p { class: "auth-gate-description",
                if let Some(f) = feature {
                    "Admin sign-in required to access "
                    strong { "{f}" }
                    "."
                } else {
                    "This area is restricted to platform administrators."
                }
            }
            if let Some(perms) = &required_permissions {
                if !perms.is_empty() {
                    div { class: "auth-gate-perms",
                        p { "Required permissions:" }
                        ul { for p in perms.iter() { li { "{p}" } } }
                    }
                }
            }
            div { class: "auth-gate-actions",
                if let Some(h) = on_connect.clone() {
                    button {
                        class: "btn btn-primary",
                        r#type: "button",
                        onclick: move |e| h.call(e),
                        "Connect Admin Wallet"
                    }
                } else {
                    a { class: "btn btn-primary", href: "{connect_href}", "Connect Admin Wallet" }
                }
                a { class: "btn btn-outline", href: "/", "Back to home" }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    //! Wave 6C Track A — auth-gate short-circuit tests.
    //!
    //! The previous `AdminAuthGate` entered the `has_admin` block
    //! but then ran the `required_permissions` missing check,
    //! which produced a non-empty list whenever the BFF plumbed
    //! `permissions: vec![]` (the pre-Wave-6C default). The fix
    //! is an unconditional `return` from the `has_admin` block.
    //!
    //! These two tests pin both halves of the contract:
    //! 1. Admin user + non-empty `required_permissions` → children
    //!    pass through (defense in depth).
    //! 2. Non-admin user + non-empty `required_permissions` →
    //!    the gate panel renders, children do not.
    use super::*;
    use crate::auth::user::{AuthMethod, User};

    fn render_to_string(el: dioxus::core::Element) -> String {
        dioxus_ssr::render_element(el)
    }

    fn admin_user() -> User {
        User {
            id: "u-admin".to_string(),
            address: "0xabcd…1234".to_string(),
            chain_id: "56".to_string(),
            roles: vec!["admin".to_string()],
            email: Some("admin@epsx.io".to_string()),
            tier: Some("Admin".to_string()),
            // Intentionally empty — the BFF would have populated
            // this in the post-Wave-6C code path, but the
            // short-circuit must not depend on that plumbing.
            permissions: vec![],
            last_login_at: None,
            auth_method: AuthMethod::Wallet,
            display_name: Some("Admin".to_string()),
        }
    }

    fn non_admin_user() -> User {
        User {
            id: "u-user".to_string(),
            address: "0xbeef…5678".to_string(),
            chain_id: "56".to_string(),
            roles: vec!["user".to_string()],
            email: None,
            tier: Some("Pro".to_string()),
            permissions: vec![],
            last_login_at: None,
            auth_method: AuthMethod::Wallet,
            display_name: None,
        }
    }

    #[test]
    fn admin_auth_gate_is_admin_short_circuits_required_permissions() {
        // Admin user + a required_permissions list that the
        // user does NOT actually hold. The short-circuit must
        // let the children through anyway.
        let user = admin_user();
        let el = rsx! {
            AdminAuthGate {
                user: Some(user),
                required_permissions: Some(vec!["anything:read".into()]),
                div { class: "gate-child", "child-marker-admin-pass" }
            }
        };
        let html = render_to_string(el);
        assert!(
            html.contains("child-marker-admin-pass"),
            "AdminAuthGate must pass children through for an admin user even when required_permissions are missing; got: {html}"
        );
    }

    #[test]
    fn admin_auth_gate_non_admin_still_enforces_required_permissions() {
        // Non-admin user with no permissions and a non-empty
        // required_permissions list. The gate must fire and
        // render the gate panel — children must NOT leak.
        let user = non_admin_user();
        let el = rsx! {
            AdminAuthGate {
                user: Some(user),
                required_permissions: Some(vec!["admin:content:write".into()]),
                div { class: "gate-child", "child-marker-non-admin-block" }
            }
        };
        let html = render_to_string(el);
        assert!(
            !html.contains("child-marker-non-admin-block"),
            "AdminAuthGate must NOT pass children through for a non-admin user missing required_permissions; got: {html}"
        );
        // Sanity: the gate panel itself renders. The
        // `auth-gate-admin` class is the stable marker.
        assert!(
            html.contains("auth-gate-admin"),
            "Expected the admin gate panel to render for a non-admin user; got: {html}"
        );
    }
}
