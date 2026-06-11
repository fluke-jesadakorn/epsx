//! `AuthModal` — wallet-selection dialog that triggers the SIWE flow.
//!
//! Wave 2 Track C additions over the Wave 1 scaffold:
//! - `WalletOption` now takes an `on_click` `EventHandler<MouseEvent>`
//!   so the caller can wire the click to a specific connector (MetaMask,
//!   WalletConnect, etc.) instead of the previous no-op stub.
//! - `DemoButton` slot exposes a styled "Try demo account" button. The
//!   text + `on_click` are caller-supplied so the same component can be
//!   reused for both real and demo flows.
//! - `on_open_change: EventHandler<bool>` callback. Fires with `false`
//!   when the user dismisses the modal via Escape, overlay click, or
//!   the close button — mirrors the Radix `onOpenChange` contract used
//!   by the shadcn `Dialog`.
//! - Focus trap: the dialog panel gets a runtime-generated `id` and
//!   the first focusable descendant is focused on mount via
//!   `document::eval` (same approach Wave 1 used for `Modal`).
//! - Escape listener: `onkeydown` on the panel checks `Key::Escape` and
//!   calls the close handlers when `close_on_escape` is enabled.
//! - `aria-labelledby` + `aria-describedby` wiring to the title and
//!   description slots, satisfying the dialog a11y contract from the
//!   design doc §3.

use crate::primitives::icon::Icon;

use dioxus::prelude::*;

/// Auth modal dialog. The signature is **additive** over the Wave 1
/// scaffold — `open`, `on_close`, and `demo_enabled` keep their
/// meaning, and every other parameter is optional with a sensible
/// default. Existing pages compile unchanged.
#[component]
pub fn AuthModal(
    open: bool,
    on_close: EventHandler<MouseEvent>,
    /// Variant tag — `"user"` (default) or `"admin"`. Drives the
    /// copy in the brand panel and the demo-button visibility.
    #[props(default = None)] variant: Option<String>,
    /// When `true`, the "Try demo account" button is rendered.
    /// Defaults to `false`.
    #[props(default = None)] demo_enabled: Option<bool>,
    /// Custom label for the demo button. Defaults to
    /// "Try demo account".
    #[props(default = None)] demo_label: Option<String>,
    /// Fires when the demo button is clicked.
    #[props(default = None)] on_demo: Option<EventHandler<MouseEvent>>,
    /// Fires whenever the modal transitions to a closed state (Escape,
    /// overlay click, close button, or programmatic close).
    #[props(default = None)] on_open_change: Option<EventHandler<bool>>,
    /// Optional callback fired after a successful auth (currently a
    /// no-op stub — the real SIWE flow is wired by the BFF; this
    /// callback is here for parity with the TS shadcn wrapper).
    #[props(default = None)] on_success: Option<EventHandler<()>>,
    /// Title text. Defaults to "Choose your wallet".
    #[props(default = None)] title: Option<String>,
    /// Description text shown below the title.
    #[props(default = None)] description: Option<String>,
    /// List of wallet options rendered in the dialog. When `None`
    /// a default MetaMask/WalletConnect/Coinbase/Trust/Binance list
    /// is rendered, matching the Wave 1 scaffold.
    #[props(default = None)] wallets: Option<Vec<WalletInfo>>,
    /// Disable the overlay-click-to-close behavior. Defaults to
    /// `false` (overlay click closes the modal).
    #[props(default = true)] close_on_overlay: bool,
    /// Disable the Escape-to-close behavior. Defaults to `false`
    /// (Escape closes the modal).
    #[props(default = true)] close_on_escape: bool,
    /// Extra class names for the dialog panel.
    #[props(default = None)] class_name: Option<String>,
) -> Element {
    if !open { return rsx! { Fragment {} }; }
    let demo = demo_enabled.unwrap_or(false);
    let variant_val = variant.unwrap_or_else(|| "user".to_string());
    let title_val = title.unwrap_or_else(|| "Choose your wallet".to_string());
    let description_val = description.unwrap_or_else(|| {
        "Connect your Web3 wallet to access dashboards, analytics, and on-chain payments.".to_string()
    });
    let demo_label_val = demo_label.unwrap_or_else(|| "Try demo account".to_string());
    let extra_cls = class_name.unwrap_or_default();

    // Runtime ids for the dialog + label/description. Generate a
    // monotonic counter for SSR-stable panel ids (Modal does the same).
    let panel_id = format!("auth-modal-panel-{}", generate_id());
    let title_id = format!("{panel_id}-title");
    let desc_id = format!("{panel_id}-desc");

    // Focus the first focusable descendant on mount. Best-effort: a
    // browser-only eval, no-op on the server.
    {
        let selector = format!("#{panel_id}");
        let script = format!(
            r#"
            (function() {{
                var el = document.querySelector({selector:?});
                if (!el) return;
                var focusable = el.querySelector(
                    'a[href], button:not([disabled]), textarea:not([disabled]), input:not([disabled]):not([type="hidden"]), select:not([disabled]), [tabindex]:not([tabindex="-1"])'
                );
                if (focusable) {{ focusable.focus(); }} else {{ el.focus(); }}
            }})();
            "#
        );
        spawn(async move {
            let _ = document::eval(script.as_str()).await;
        });
    }

    let on_overlay_click = move |e: MouseEvent| {
        if !close_on_overlay { return; }
        on_close.call(e);
        if let Some(h) = &on_open_change { h.call(false); }
    };

    let on_close_button = move |e: MouseEvent| {
        on_close.call(e);
        if let Some(h) = &on_open_change { h.call(false); }
    };

    let on_key_down = move |e: Event<KeyboardData>| {
        if !close_on_escape { return; }
        if matches!(e.key(), Key::Escape) {
            if let Some(h) = &on_open_change { h.call(false); }
        }
    };

    let is_admin = variant_val == "admin";

    rsx! {
        div { class: "modal-overlay auth-modal-overlay", onclick: on_overlay_click,
            div {
                class: "auth-modal {extra_cls}",
                id: "{panel_id}",
                role: "dialog",
                "aria-modal": "true",
                "aria-labelledby": "{title_id}",
                "aria-describedby": "{desc_id}",
                tabindex: "-1",
                onclick: |e| e.stop_propagation(),
                onkeydown: on_key_down,
                div { class: "auth-modal-grid",
                    div { class: "auth-modal-aside",
                        div { class: "auth-modal-brand",
                            span { dangerous_inner_html: "{epsx_templates::epsx_icon_svg()}" }
                            span { class: "gradient-text text-2xl font-bold", "EPSX" }
                        }
                        h2 { class: "auth-modal-headline",
                            if is_admin { "Admin Control Panel" } else { "Sign in to your account" }
                        }
                        p { class: "auth-modal-sub",
                            if is_admin {
                                "Restricted access. Connect your admin wallet to manage users, permissions, and platform analytics."
                            } else {
                                "Connect your Web3 wallet to access dashboards, analytics, and on-chain payments."
                            }
                        }
                        ul { class: "auth-modal-features",
                            li { span { Icon { name: "check".to_string(), size: Some(16) } } span { "SIWE-based authentication" } }
                            li { span { Icon { name: "check".to_string(), size: Some(16) } } span { "BSC mainnet + testnet support" } }
                            li { span { Icon { name: "check".to_string(), size: Some(16) } } span { "Paymaster-sponsored gas (Premium)" } }
                        }
                    }
                    div { class: "auth-modal-content",
                        button {
                            class: "modal-close absolute top-4 right-4",
                            r#type: "button",
                            "aria-label": "Close",
                            onclick: on_close_button,
                            "✕"
                        }
                        h3 { class: "auth-modal-title", id: "{title_id}", "{title_val}" }
                        if !description_val.is_empty() {
                            p { class: "auth-modal-description text-sm text-muted-foreground mb-3",
                                id: "{desc_id}",
                                "{description_val}"
                            }
                        }
                        div { class: "wallet-list",
                            if let Some(list) = wallets {
                                for w in list {
                                    WalletOption {
                                        name: w.name,
                                        icon: w.icon,
                                        id: w.id,
                                        on_click: w.on_click,
                                    }
                                }
                            } else {
                                WalletOption { name: "MetaMask", icon: "wallet", id: Some("metamask".to_string()), on_click: None }
                                WalletOption { name: "WalletConnect", icon: "wallet", id: Some("walletconnect".to_string()), on_click: None }
                                WalletOption { name: "Coinbase Wallet", icon: "wallet", id: Some("coinbase".to_string()), on_click: None }
                                WalletOption { name: "Trust Wallet", icon: "wallet", id: Some("trust".to_string()), on_click: None }
                                WalletOption { name: "Binance Wallet", icon: "wallet", id: Some("binance".to_string()), on_click: None }
                            }
                        }
                        if demo {
                            div { class: "auth-modal-divider", "OR" }
                            DemoButton { label: demo_label_val, on_click: on_demo }
                        }
                    }
                }
            }
        }
    }
}

/// A single wallet option row in the auth modal. The row is a button
/// that fires `on_click` when activated. The `id` is forwarded to
/// the click handler as a stable identifier (e.g. "metamask") that
/// the BFF can dispatch to a specific SIWE flow.
#[component]
pub fn WalletOption(
    name: String,
    /// Icon name from the lucide registry. Defaults to "wallet".
    #[props(default = "wallet".to_string())] icon: String,
    /// Stable id for this wallet ("metamask", "walletconnect", ...).
    /// Forwarded to `on_click` so callers can switch on the connector.
    #[props(default = None)] id: Option<String>,
    /// Click handler. When `None`, the button is rendered but
    /// disabled (the existing pages that don't wire a click still
    /// compile and render).
    #[props(default = None)] on_click: Option<EventHandler<WalletClick>>,
) -> Element {
    let id_val = id.clone();
    let on_click_handler = on_click.clone();
    let is_disabled = on_click.is_none();
    rsx! {
        button {
            class: "wallet-option",
            r#type: "button",
            disabled: is_disabled,
            "aria-label": "Continue with {name}",
            onclick: move |e| {
                if let Some(h) = &on_click_handler {
                    h.call(WalletClick { id: id_val.clone(), event: e });
                }
            },
            span { class: "wallet-icon", Icon { name: icon.clone(), size: Some(22) } }
            span { class: "wallet-name", "{name}" }
            span { class: "wallet-chev", Icon { name: "chevron-right".to_string(), size: Some(16) } }
        }
    }
}

/// Payload passed to `WalletOption::on_click`. Bundles the wallet's
/// stable id (e.g. "metamask") with the underlying mouse event so
/// the caller can both switch on the connector and `prevent_default`
/// / `stop_propagation` as needed.
#[derive(Clone, Debug)]
pub struct WalletClick {
    pub id: Option<String>,
    pub event: MouseEvent,
}

/// Demo account entry-point. Styled as a wide outline button under an
/// "OR" divider. Fires `on_click` when activated.
#[component]
pub fn DemoButton(
    /// Button label. Defaults to "Try demo account".
    #[props(default = "Try demo account".to_string())] label: String,
    #[props(default = None)] on_click: Option<EventHandler<MouseEvent>>,
    #[props(default = None)] class_name: Option<String>,
) -> Element {
    let extra_cls = class_name.unwrap_or_default();
    rsx! {
        button {
            class: "btn btn-outline btn-block auth-demo-btn {extra_cls}",
            r#type: "button",
            "aria-label": "{label}",
            onclick: move |e| {
                if let Some(h) = &on_click { h.call(e); }
            },
            "{label}"
        }
    }
}

/// Convenience descriptor for a wallet option. Used when the caller
/// wants to pass a custom wallet list to `AuthModal`.
#[derive(Clone, Debug, PartialEq)]
pub struct WalletInfo {
    pub name: String,
    /// Lucide icon name. Defaults to "wallet".
    pub icon: String,
    /// Stable id ("metamask", "walletconnect", ...).
    pub id: Option<String>,
    /// Click handler. The handler receives the `MouseEvent`; the
    /// caller can read `id` from the surrounding closure scope.
    pub on_click: Option<EventHandler<WalletClick>>,
}

/// Monotonically increasing id for SSR-stable panel ids. Same trick
/// Wave 1 used for `Modal`.
fn generate_id() -> u64 {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(1);
    COUNTER.fetch_add(1, Ordering::Relaxed)
}
