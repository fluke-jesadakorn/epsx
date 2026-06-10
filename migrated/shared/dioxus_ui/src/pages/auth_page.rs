use crate::primitives::*;

use dioxus::prelude::*;
use super::PageContext;
use super::PageMeta;

const AUTH_HYDRATION_SCRIPT: &str = "(function(){var t=localStorage.getItem('epsx_token');if(t){var d=new URLSearchParams(location.search).get('return_url')||'/';location.replace(d);}window.addEventListener('storage',function(){var t=localStorage.getItem('epsx_token');if(t)location.replace('/');});})();";

#[component]
pub fn AuthPage() -> Element {
    rsx! { RenderAuth {} }
}

#[component]
pub fn RenderAuth() -> Element {
    let mut status = use_signal(|| "idle".to_string()); // idle | connecting | signing | error
    let mut error_msg = use_signal(String::new);
    let mut demo_enabled = use_signal(|| true);

    // On mount, check if a wallet is already connected via window.epsxWallet
    // and pre-fill the address. Client-side hydration handles this.
    rsx! {
        div { class: "auth-page",
            div { class: "auth-page-grid",
                div { class: "auth-page-aside",
                    div { class: "auth-page-brand",
                        a { href: "/", span { dangerous_inner_html: "{epsx_templates::epsx_icon_svg()}" } }
                        a { href: "/", span { class: "gradient-text text-xl font-bold", "EPSX" } }
                    }
                    h1 { class: "auth-page-headline", "Sign in to EPSX" }
                    p { class: "auth-page-sub", "Connect a wallet to access dashboards, analytics, payments, and developer tools." }
                    ul { class: "auth-page-features",
                        li { span { Icon { name: "check".to_string(), size: Some(16) } } "SIWE — Sign-In With Ethereum" }
                        li { span { Icon { name: "check".to_string(), size: Some(16) } } "BSC mainnet (chain id 56)" }
                        li { span { Icon { name: "check".to_string(), size: Some(16) } } "No email, no password" }
                        li { span { Icon { name: "check".to_string(), size: Some(16) } } "Paymaster-sponsored gas for premium users" }
                    }
                }
                div { class: "auth-page-content",
                    div { class: "card card-glass auth-card",
                        h2 { class: "auth-card-title", "Choose a wallet" }
                        div { class: "wallet-list",
                            WalletOption { name: "MetaMask", icon: "wallet".to_string(), disabled: false,
                                onclick: move |_| {
                                    status.set("connecting".to_string());
                                    error_msg.set(String::new());
                                } }
                            WalletOption { name: "WalletConnect", icon: "plug".to_string(), disabled: false,
                                onclick: move |_| {
                                    status.set("connecting".to_string());
                                    error_msg.set(String::new());
                                } }
                            WalletOption { name: "Coinbase Wallet", icon: "wallet".to_string(), disabled: false,
                                onclick: move |_| {
                                    status.set("connecting".to_string());
                                    error_msg.set(String::new());
                                } }
                            WalletOption { name: "Trust Wallet", icon: "shield".to_string(), disabled: true,
                                onclick: move |_| {} }
                            WalletOption { name: "Binance Wallet", icon: "hexagon".to_string(), disabled: true,
                                onclick: move |_| {} }
                        }
                        if *demo_enabled.read() {
                            div { class: "auth-card-divider", "OR" }
                            button { class: "btn btn-outline btn-block", r#type: "button",
                                onclick: move |_| {
                                    status.set("signing".to_string());
                                    error_msg.set(String::new());
                                },
                                "Try the demo account"
                            }
                        }
                        if *status.read() == "connecting" || *status.read() == "signing" {
                            div { class: "auth-card-status flex items-center gap-2 mt-4",
                                div { class: "spinner spinner-sm" }
                                span { class: "text-sm text-muted-foreground",
                                    if *status.read() == "connecting" { "Waiting for wallet..." } else { "Signing message..." }
                                }
                            }
                        }
                        if !error_msg.read().is_empty() {
                            div { class: "auth-card-error text-sm text-danger mt-2", "{error_msg.read()}" }
                        }
                        p { class: "auth-card-foot text-xs text-muted-foreground",
                            "By connecting a wallet you agree to our "
                            a { href: "/terms", "Terms" }
                            " and "
                            a { href: "/privacy", "Privacy Policy" }
                            "."
                        }
                    }
                    script { dangerous_inner_html: AUTH_HYDRATION_SCRIPT }
                }
            }
        }
    }
}

#[component]
fn WalletOption(name: String, icon: String, disabled: bool, onclick: EventHandler<MouseEvent>) -> Element {
    rsx! {
        button {
            class: "wallet-option",
            r#type: "button",
            disabled: disabled,
            onclick: move |e| onclick.call(e),
            span { class: "wallet-option-icon", Icon { name: icon.clone(), size: Some(22) } }
            span { class: "wallet-option-name", "{name}" }
            span { class: "wallet-option-chev", Icon { name: "chevron-right".to_string(), size: Some(16) } }
        }
    }
}

pub fn render(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::marketing("Sign in");
    (meta, rsx! { RenderAuth {} })
}
