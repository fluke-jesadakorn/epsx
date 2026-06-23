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
//! Click behavior:
//! 1. POST `/api/v1/pay/intents` with the typed props (via the
//!    BFF proxy at `pay.epsx.io/api/v1/pay/intents`).
//! 2. On success, redirect to `pay.epsx.io/checkout?intent={id}`.
//! 3. On failure, show an inline error toast (slice-5 will
//!    hydrate to use `epsx.toast()`).
//!
//! For slice-4 the click handler is implemented as a Dioxus
//! `onclick` closure. Slice-5 can swap in a `use_navigator` +
//! `Resource` if hydration feels janky.

use dioxus::prelude::*;

use crate::primitives::icon::Icon;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CheckoutVariant {
    /// Primary CTA — `bg-gradient-to-r from-cyan-500 to-blue-600 text-white`
    Gradient,
    /// Outline — `border-2 border-orange-500 text-orange-500`
    Outline,
    /// Solid — `bg-orange-500 text-white`
    Solid,
}

impl Default for CheckoutVariant {
    fn default() -> Self { Self::Gradient }
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
    let token = props.token.clone().unwrap_or_else(|| props.currency.clone());
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
    let final_class = props.class.clone().unwrap_or_else(|| base_class.to_string());

    // Build the POST body.
    let body_json = serde_json::json!({
        "amount": props.amount,
        "currency": props.currency,
        "chain_id": props.chain_id,
        "token": token,
        "description": props.description.clone().unwrap_or_default(),
    });
    let body_str = body_json.to_string();

    // wave49(slice-5): real onclick handler. After slice-4
    // shipped a no-op spawn, this now does the actual work:
    //
    // 1. POST to `/api/v1/pay/intent` (the BFF proxy at
    //    `props.api_base` forwards to `pay-svc`).
    // 2. Parse the response — the `intent.id` field is the
    //    redirect target.
    // 3. `window.location.href = /pay?intent={id}` to navigate.
    //
    // Implementation uses the Dioxus `document::eval` JS
    // bridge (same pattern Wave 1 used in `auth_modal`,
    // `modal`, and `rich_text`) because the SSR mode doesn't
    // expose `window` to Rust closures. For SSR-only render
    // (no hydration), the eval never runs and the button is
    // a visual no-op (the page-shell `disabled` state handles
    // the visible-but-inert case).
    let api_base = props.api_base.clone();
    let pay_url_fallback = props.pay_url.clone();
    let onclick = move |_evt| {
        let url = format!("{}/v1/pay/intent", api_base);
        let pay_url_fallback = pay_url_fallback.clone();
        let body_str = body_str.clone();
        spawn(async move {
            // Inline JS payload — runs in the browser context.
            // The fetch is intentionally NOT pre-encoded; the
            // browser-side JSON.stringify matches our Rust
            // serde_json output for the simple shapes we use.
            //
            // Follows the auth_modal.rs / modal.rs / rich_text.rs
            // pattern: `let _ = document::eval(script.as_str()).await;`
            // The `_` discard is intentional — the script is
            // fire-and-forget (we redirect on completion, so
            // there's nothing to await in the Rust closure).
            let script = format!(
                r#"
                (async () => {{
                    try {{
                        const resp = await fetch("{url}", {{
                            method: "POST",
                            headers: {{ "Content-Type": "application/json" }},
                            body: JSON.stringify({body}),
                        }});
                        if (!resp.ok) {{
                            console.error("CheckoutButton: pay-svc error", resp.status);
                            return;
                        }}
                        const data = await resp.json();
                        const intentId = data?.intent?.id;
                        if (intentId) {{
                            window.location.href = "/pay?intent=" + encodeURIComponent(intentId);
                        }} else {{
                            window.location.href = "{fallback}";
                        }}
                    }} catch (e) {{
                        console.error("CheckoutButton: fetch failed", e);
                        window.location.href = "{fallback}";
                    }}
                }})();
                "#,
                url = url,
                body = body_str,
                fallback = pay_url_fallback,
            );
            // Dispatch to the browser via the Dioxus eval bridge.
            // SSR is a no-op (the `RuntimeScope` doesn't have a
            // `document`); the button visual is correct without
            // hydration. On the client, this runs the IIFE
            // immediately and the user is redirected.
            let _ = document::eval(script.as_str()).await;
        });
    };

    rsx! {
        div { class: "checkout-button-wrap",
            button {
                class: "{final_class}",
                r#type: "button",
                onclick: onclick,
                "data-checkout-button": "true",
                "data-amount": "{props.amount}",
                "data-currency": "{props.currency}",
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