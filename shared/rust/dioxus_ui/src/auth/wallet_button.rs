use crate::primitives::icon::Icon;

use dioxus::prelude::*;
use super::user::User;

#[component]
pub fn WalletConnectButton(user: Option<User>, on_connect: Option<EventHandler<MouseEvent>>) -> Element {
    if let Some(u) = user {
        rsx! {
            a { class: "btn btn-primary", href: "/profile",
                span { Icon { name: "wallet".to_string(), size: Some(16) } }
                span { "{u.short_address()}" }
            }
        }
    } else {
        rsx! {
            button { class: "btn btn-gradient", onclick: move |e| if let Some(h) = &on_connect { h.call(e); },
                span { "Connect Wallet" }
            }
        }
    }
}

#[component]
pub fn ConnectedWalletDropdown(user: User) -> Element {
    rsx! {
        div { class: "connected-wallet",
            div { class: "wallet-pill",
                span { class: "wallet-status-dot" }
                span { class: "wallet-address", "{user.short_address()}" }
            }
            div { class: "wallet-balance",
                span { "0.00" }
                span { class: "text-muted-foreground text-sm", "BNB" }
            }
        }
    }
}
