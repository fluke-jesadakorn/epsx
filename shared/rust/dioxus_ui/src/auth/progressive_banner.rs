use crate::primitives::icon::Icon;

use dioxus::prelude::*;

#[component]
pub fn ProgressiveAuthBanner(feature: Option<String>) -> Element {
    rsx! {
        div { class: "progressive-auth-banner", role: "status",
            div { class: "banner-icon", Icon { name: "info".to_string(), size: Some(20) } }
            div { class: "banner-content",
                p { class: "banner-title",
                    if let Some(f) = feature {
                        "Sign in to {f}"
                    } else {
                        "Connect your wallet for the full experience"
                    }
                }
                p { class: "banner-subtitle text-sm text-muted-foreground", "Browsing works without signing in, but you need a wallet to act." }
            }
            a { class: "btn btn-sm btn-primary", href: "/auth", "Sign in" }
        }
    }
}
