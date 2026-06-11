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
        // Run the same permission check the regular `AuthGate` does.
        if let Some(u) = &user {
            let missing: Vec<String> = match &required_permissions {
                None => vec![],
                Some(perms) => perms.iter().filter(|p| !u.has_permission(p)).cloned().collect(),
            };
            if missing.is_empty() {
                return rsx! { Fragment { {children} } };
            }
        }
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
