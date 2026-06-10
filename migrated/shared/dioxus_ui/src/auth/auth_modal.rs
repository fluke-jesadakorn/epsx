use crate::primitives::icon::Icon;

use dioxus::prelude::*;

#[component]
pub fn AuthModal(open: bool, on_close: EventHandler<MouseEvent>, demo_enabled: Option<bool>) -> Element {
    if !open { return rsx! { Fragment {} }; }
    let demo = demo_enabled.unwrap_or(false);
    rsx! {
        div { class: "modal-overlay", onclick: move |e| on_close.call(e),
            div { class: "auth-modal", role: "dialog", "aria-modal": "true",
                onclick: |e| e.stop_propagation(),
                div { class: "auth-modal-grid",
                    div { class: "auth-modal-aside",
                        div { class: "auth-modal-brand",
                            span { dangerous_inner_html: "{epsx_templates::epsx_icon_svg()}" }
                            span { class: "gradient-text text-2xl font-bold", "EPSX" }
                        }
                        h2 { class: "auth-modal-headline", "Sign in to your account" }
                        p { class: "auth-modal-sub", "Connect your Web3 wallet to access dashboards, analytics, and on-chain payments." }
                        ul { class: "auth-modal-features",
                            li { span { Icon { name: "check".to_string(), size: Some(16) } } span { "SIWE-based authentication" } }
                            li { span { Icon { name: "check".to_string(), size: Some(16) } } span { "BSC mainnet + testnet support" } }
                            li { span { Icon { name: "check".to_string(), size: Some(16) } } span { "Paymaster-sponsored gas (Premium)" } }
                        }
                    }
                    div { class: "auth-modal-content",
                        button { class: "modal-close absolute top-4 right-4", onclick: move |e| on_close.call(e), "✕" }
                        h3 { class: "auth-modal-title", "Choose your wallet" }
                        div { class: "wallet-list",
                            WalletOption { name: "MetaMask", icon: "wallet" }
                            WalletOption { name: "WalletConnect", icon: "wallet" }
                            WalletOption { name: "Coinbase Wallet", icon: "wallet" }
                            WalletOption { name: "Trust Wallet", icon: "wallet" }
                            WalletOption { name: "Binance Wallet", icon: "wallet" }
                        }
                        if demo {
                            div { class: "auth-modal-divider", "OR" }
                            button { class: "btn btn-outline btn-block", "Try demo account" }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn WalletOption(name: String, icon: String) -> Element {
    rsx! {
        button { class: "wallet-option",
            span { class: "wallet-icon", Icon { name: icon.clone(), size: Some(22) } }
            span { class: "wallet-name", "{name}" }
            span { class: "wallet-chev", Icon { name: "chevron-right".to_string(), size: Some(16) } }
        }
    }
}
