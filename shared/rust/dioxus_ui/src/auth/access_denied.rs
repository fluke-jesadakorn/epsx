//! `AccessDenied` — full-page "you can't see this" panel.
//!
//! Wave 2 Track C additions:
//! - `return_url: Option<String>` — overrides the "Back to home"
//!   link with a custom URL (e.g. the dashboard, or a deep link).
//! - `icon: Option<String>` — pick a different lucide icon (default
//!   "shield"). Common alternates: "lock", "alert-circle".
//! - `show_back: Option<bool>` — toggle the back-to-home button
//!   (kept as `Option<bool>` to mirror the TS `AccessDenied`).
//! - `contact_href: Option<String>` — replaces the "Request Access"
//!   link. Defaults to `/contact`.
//!
//! All existing call sites (`pages/access_denied.rs`,
//! `pages/admin_pages/access_denied.rs`,
//! `pages/admin_pages/unauthorized.rs`) keep compiling.

use crate::primitives::icon::Icon;

use dioxus::prelude::*;

#[component]
pub fn AccessDenied(
    /// Free-form reason text. Defaults to
    /// "You do not have permission to access this page".
    #[props(default = None)] reason: Option<String>,
    /// Optional list of required permissions. Rendered as a
    /// monospaced bullet list inside an "alert" block.
    #[props(default = None)] required_permissions: Option<Vec<String>>,
    /// Show the "Go Home" back-button. Defaults to `true`.
    #[props(default = None)] show_back: Option<bool>,
    /// Extra class names for the outer wrapper.
    #[props(default = None)] class_name: Option<String>,
    /// URL the "Go Home" / "Back" link points to. Defaults to `/`.
    #[props(default = None)] return_url: Option<String>,
    /// Lucide icon name. Defaults to "shield". Other reasonable
    /// choices: "lock", "alert-circle", "info".
    #[props(default = None)] icon: Option<String>,
    /// Href for the "Request Access" CTA. Defaults to `/contact`.
    #[props(default = None)] contact_href: Option<String>,
) -> Element {
    let reason_val = reason.unwrap_or_else(|| {
        "You do not have permission to access this page".to_string()
    });
    let show_back_val = show_back.unwrap_or(true);
    let back_href = return_url.unwrap_or_else(|| "/".to_string());
    let contact_href_val = contact_href.unwrap_or_else(|| "/contact".to_string());
    let icon_val = icon.unwrap_or_else(|| "shield".to_string());
    let extra_cls = class_name.unwrap_or_default();
    rsx! {
        div { class: "access-denied {extra_cls}", role: "alert",
            div { class: "access-denied-icon",
                Icon { name: icon_val.clone(), size: Some(64), class_name: Some("text-destructive".to_string()) }
            }
            h1 { class: "access-denied-title", "Access Denied" }
            p { class: "access-denied-reason", "{reason_val}" }
            if let Some(perms) = required_permissions {
                if !perms.is_empty() {
                    div { class: "access-denied-perms",
                        p { "Required permissions:" }
                        ul { for p in perms.iter() {
                            li { class: "access-denied-perm", "{p}" }
                        } }
                    }
                }
            }
            div { class: "access-denied-actions",
                if show_back_val {
                    a { class: "btn btn-outline", href: "{back_href}", "Back" }
                }
                a { class: "btn btn-primary", href: "{contact_href_val}", "Request Access" }
            }
        }
    }
}
