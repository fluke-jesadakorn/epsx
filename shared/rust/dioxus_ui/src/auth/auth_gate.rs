use crate::primitives::icon::Icon;

use dioxus::prelude::*;

#[component]
pub fn AuthGate(user: Option<super::user::User>, feature: Option<String>, children: Element) -> Element {
    if user.is_some() { return rsx! { Fragment { {children} } }; }
    rsx! {
        div { class: "auth-gate", role: "alert",
            div { class: "auth-gate-icon", Icon { name: "wallet".to_string(), size: Some(48) } }
            h2 { class: "auth-gate-title", "Sign in required" }
            p { class: "auth-gate-description",
                if let Some(f) = feature {
                    "Sign in to access "
                    strong { "{f}" }
                    "."
                } else {
                    "Please connect your wallet to continue."
                }
            }
            div { class: "auth-gate-actions",
                a { class: "btn btn-primary", href: "/auth", "Connect Wallet" }
                a { class: "btn btn-outline", href: "/", "Back to home" }
            }
        }
    }
}
