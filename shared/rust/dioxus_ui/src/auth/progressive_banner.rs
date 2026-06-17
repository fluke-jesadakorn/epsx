//! `ProgressiveAuthBanner` — inline "sign in to unlock" strip.
//!
//! Wave 2 Track C additions over the Wave 1 scaffold:
//! - `cta_label: Option<String>` — override the button label
//!   (default: "Sign in"). The TS source allows passing a custom
//!   message; the Rust port mirrors that on the CTA side.
//! - `on_click: Option<EventHandler<MouseEvent>>` — fires when the
//!   CTA is activated. Useful when the parent wants to open an
//!   in-page modal rather than navigate to `/auth`.
//! - `dismissible: Option<bool>` — when `true`, a small "✕" button
//!   appears on the right edge. Emits `on_dismiss` when clicked.
//! - `on_dismiss: Option<EventHandler<MouseEvent>>` — fired by the
//!   dismiss button. When unset, the dismiss button is hidden even
//!   if `dismissible` is `true` (matches the TS prop name).
//! - `href: Option<String>` — when `on_click` is unset, the CTA is
//!   rendered as a link pointing to this URL (default: `/auth`).
//! - `message` / `description` — separate the headline (bold) from
//!   the sub-line, matching the TS `AuthBanner` shape.
//! - `icon: Option<String>` — override the leading icon (default
//!   "info"). Common alternates: "wallet", "shield".

use crate::primitives::icon::Icon;

use dioxus::prelude::*;

#[component]
pub fn ProgressiveAuthBanner(
    /// Optional feature name (e.g. "your dashboard") shown in the
    /// banner copy. Kept from Wave 1.
    #[props(default = None)] feature: Option<String>,
    /// Override for the banner headline. When set, replaces the
    /// auto-generated "Sign in to <feature>" copy.
    #[props(default = None)] message: Option<String>,
    /// Sub-line below the headline. Default text matches Wave 1.
    #[props(default = None)] description: Option<String>,
    /// CTA label. Defaults to "Sign in".
    #[props(default = None)] cta_label: Option<String>,
    /// Click handler for the CTA. When set, the CTA becomes a
    /// button; when unset, the CTA is an anchor tag pointing to
    /// `href` (default `/auth`).
    #[props(default = None)] on_click: Option<EventHandler<MouseEvent>>,
    /// Href for the CTA when `on_click` is unset. Defaults to
    /// `/auth`.
    #[props(default = None)] href: Option<String>,
    /// Show the dismiss button. Defaults to `false`. The dismiss
    /// button is only rendered when `on_dismiss` is also set.
    #[props(default = false)] dismissible: bool,
    /// Fired when the user clicks the dismiss button. When unset,
    /// the dismiss button is hidden even if `dismissible` is true.
    #[props(default = None)] on_dismiss: Option<EventHandler<MouseEvent>>,
    /// Override for the leading icon. Defaults to "info".
    #[props(default = None)] icon: Option<String>,
    /// Extra class names for the banner wrapper.
    #[props(default = None)] class_name: Option<String>,
) -> Element {
    let icon_val = icon.unwrap_or_else(|| "info".to_string());
    let cta_label_val = cta_label.unwrap_or_else(|| "Sign in".to_string());
    let href_val = href.unwrap_or_else(|| "/auth".to_string());
    let description_val = description.unwrap_or_else(|| {
        "Browsing works without signing in, but you need a wallet to act.".to_string()
    });
    let extra_cls = class_name.unwrap_or_default();
    let has_dismiss = dismissible && on_dismiss.is_some();
    let message_val = message.clone().unwrap_or_else(|| {
        if let Some(f) = &feature {
            format!("Sign in to {f}")
        } else {
            "Connect your wallet for the full experience".to_string()
        }
    });
    rsx! {
        div { class: "progressive-auth-banner {extra_cls}", role: "status",
            div { class: "banner-icon", Icon { name: icon_val, size: Some(20) } }
            div { class: "banner-content",
                p { class: "banner-title", "{message_val}" }
                p { class: "banner-subtitle text-sm text-muted-foreground", "{description_val}" }
            }
            if let Some(h) = on_click.clone() {
                button {
                    class: "btn btn-sm btn-primary",
                    r#type: "button",
                    onclick: move |e| h.call(e),
                    "{cta_label_val}"
                }
            } else {
                a { class: "btn btn-sm btn-primary", href: "{href_val}", "{cta_label_val}" }
            }
            if has_dismiss {
                if let Some(h) = on_dismiss.clone() {
                    button {
                        class: "progressive-auth-banner-dismiss",
                        r#type: "button",
                        "aria-label": "Dismiss",
                        onclick: move |e| h.call(e),
                        Icon { name: "x".to_string(), size: Some(14) }
                    }
                }
            }
        }
    }
}
