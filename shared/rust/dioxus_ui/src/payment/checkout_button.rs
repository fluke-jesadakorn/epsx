//! `CheckoutButton` — reusable CTA that creates a pay intent
//! and redirects to `pay.epsx.io`.
//!
//! wave49(slice-4): Shared component for the pricing pages
//! (`/plans`) + anywhere else a "Get Started" / "Subscribe" /
//! "Pay Now" button needs to deep-link into the pay flow.
//!
//! Usage:
//! ```ignore
//! CheckoutButton {
//!     amount: "100".to_string(),
//!     currency: "USDT".to_string(),
//!     chain_id: "56".to_string(),
//!     token: "USDT".to_string(),
//!     description: Some("Pro plan subscription".to_string()),
//!     label: Some("Get Started".to_string()),
//!     variant: Some(CheckoutVariant::Gradient),
//! }
//! ```
//!
//! Click behavior is delegated through typed data attributes to the generated
//! Rust/WASM browser runtime.

use dioxus::prelude::*;

use crate::primitives::icon::Icon;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum CheckoutVariant {
    /// Primary CTA — `bg-gradient-to-r from-cyan-500 to-blue-600 text-white`
    #[default]
    Gradient,
    /// Outline — `border-2 border-orange-500 text-orange-500`
    Outline,
    /// Solid — `bg-orange-500 text-white`
    Solid,
}

#[derive(Props, Clone, PartialEq)]
pub struct CheckoutButtonProps {
    /// Amount in the smallest token unit (or human-readable —
    /// the pay-svc accepts both). For USDT: pass `100` for
    /// 100 USDT (6-decimal scale handled server-side).
    pub amount: String,
    /// Display currency code (e.g. `USDT`). Also used as the
    /// default token symbol if `token` is not provided.
    pub currency: String,
    /// Chain id — `56` for BSC mainnet, `97` for BSC testnet.
    #[props(default = "56".to_string())]
    pub chain_id: String,
    /// Token symbol. Defaults to `currency`.
    #[props(default)]
    pub token: Option<String>,
    /// Optional human-readable description (shown on the
    /// checkout screen).
    #[props(default)]
    pub description: Option<String>,
    /// Button label. Defaults to "Get Started".
    #[props(default = "Get Started".to_string())]
    pub label: String,
    /// Visual variant.
    #[props(default)]
    pub variant: CheckoutVariant,
    /// Optional override for the pay subdomain. Defaults to
    /// `pay.epsx.io` (set via `EPSX_PAY_URL` env var on the
    /// BFF, baked in at compile time via the `pay_url` prop).
    #[props(default = "https://pay.epsx.io".to_string())]
    pub pay_url: String,
    /// Optional override for the BFF base URL. Defaults to
    /// `/api` (relative — works when the button is rendered
    /// by an app that already proxies through the pay BFF).
    #[props(default = "/api".to_string())]
    pub api_base: String,
    /// Full class override (skips variant + default class).
    #[props(default)]
    pub class: Option<String>,
}

#[component]
pub fn CheckoutButton(props: CheckoutButtonProps) -> Element {
    // Derive the token symbol (fall back to currency).
    let token = props
        .token
        .clone()
        .unwrap_or_else(|| props.currency.clone());
    let chain_label = match props.chain_id.as_str() {
        "56" => "BSC (BEP-20)",
        "97" => "BSC Testnet",
        other => other,
    };

    // Class selection by variant.
    let base_class = match props.variant {
        CheckoutVariant::Gradient =>
            "checkout-button checkout-button-gradient w-full py-4 rounded-xl font-bold text-base transition-all duration-300 relative overflow-hidden bg-gradient-to-r from-cyan-500 to-blue-600 text-white hover:shadow-lg hover:shadow-cyan-500/25",
        CheckoutVariant::Outline =>
            "checkout-button checkout-button-outline w-full py-4 rounded-xl font-bold text-base transition-all duration-300 border-2 border-orange-500 text-orange-500 hover:bg-orange-500/10",
        CheckoutVariant::Solid =>
            "checkout-button checkout-button-solid w-full py-4 rounded-xl font-bold text-base transition-all duration-300 bg-orange-500 text-white hover:bg-orange-600",
    };
    let final_class = props
        .class
        .clone()
        .unwrap_or_else(|| base_class.to_string());

    rsx! {
        div { class: "checkout-button-wrap",
            button {
                class: "{final_class}",
                r#type: "button",
                "data-checkout-button": "true",
                "data-epsx-action": "create-checkout",
                "data-amount": "{props.amount}",
                "data-currency": "{props.currency}",
                "data-token": "{token}",
                "data-chain-id": "{props.chain_id}",
                "data-description": "{props.description.clone().unwrap_or_default()}",
                "data-api-base": "{props.api_base}",
                "data-chain": "{chain_label}",
                "data-pay-url": "{props.pay_url}",
                span { class: "checkout-button-label relative flex items-center justify-center gap-2",
                    Icon { name: "trending-up".to_string(), size: Some(16), class_name: Some("w-4 h-4".to_string()) }
                    "{props.label}"
                }
            }
        }
    }
}
