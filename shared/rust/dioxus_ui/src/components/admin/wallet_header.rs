//! Admin `WalletHeader` component — Wave 38a T1 admin wallet domain
//! port.
//!
//! Mirrors `apps-old/admin-frontend/components/wallet/wallet-header.tsx`.
//! Renders the wallet detail header: address + copy button +
//! status badge + timing metadata + optional disable-info callout.
//!
//! ## Tests
//!
//! `test_wallet_header_renders_address` — the wallet address
//! renders in monospace.
//! `test_wallet_header_renders_status_badge` — the status badge
//! renders.
//! `test_wallet_header_renders_disable_info` — disabled wallets
//! get the amber callout with the disable reason.

use dioxus::prelude::*;

use super::wallet_status_badge::WalletStatusBadge;

/// Wallet header data payload. Mirrors a subset of the prod
/// `WalletData` shape used by the header.
#[derive(Clone, Debug, PartialEq)]
pub struct WalletHeaderData {
    pub wallet_address: String,
    pub status: String,
    pub created_at: String,
    pub last_auth_at: Option<String>,
    /// Optional disable-info, surfaced as a callout when
    /// `status = "disabled"`.
    pub disable_reason: Option<String>,
    pub disable_disabled_by: Option<String>,
    pub disable_disabled_at: Option<String>,
    pub disable_expires_at: Option<String>,
}

/// Wallet detail header — address + status + timing metadata.
/// Caller supplies the data + an optional `on_copy` click handler.
#[component]
pub fn WalletHeader(
    wallet: WalletHeaderData,
    /// Optional copy-address click handler.
    on_copy: Option<EventHandler<MouseEvent>>,
    /// Optional extra classes appended to the outer wrapper.
    class_name: Option<String>,
) -> Element {
    let mut cls =
        "rounded-xl bg-gradient-to-r from-blue-50 to-purple-50 dark:from-blue-900/10 dark:to-purple-900/10 p-5 border border-blue-100 dark:border-blue-900/30".to_string();
    if let Some(extra) = class_name {
        if !extra.is_empty() {
            cls.push(' ');
            cls.push_str(&extra);
        }
    }
    let status_str = wallet.status.clone();
    let disable_reason = wallet.disable_reason.clone();
    let disabled_by = wallet.disable_disabled_by.clone();
    let disabled_at = wallet.disable_disabled_at.clone();
    let expires_at = wallet.disable_expires_at.clone();
    let is_disabled = wallet.status == "disabled";
    let last_auth = wallet.last_auth_at.clone();
    let created_at = wallet.created_at.clone();
    let address_display = wallet.wallet_address.clone();
    rsx! {
        div { class: "{cls}",
            // Address row
            div { class: "flex items-center gap-3 mb-4",
                div { class: "flex-1 min-w-0",
                    code { class: "text-sm font-mono font-bold text-foreground break-all bg-white/50 dark:bg-black/20 px-2 py-1 rounded",
                        "{address_display}"
                    }
                }
                div { class: "relative flex-shrink-0",
                    button {
                        r#type: "button",
                        class: "p-2 rounded-lg hover:bg-white dark:hover:bg-gray-800 text-gray-500 hover:text-blue-600 dark:hover:text-blue-400 transition-colors shadow-sm",
                        title: "Copy address",
                        onclick: move |e| {
                            if let Some(cb) = on_copy {
                                cb.call(e);
                            }
                        },
                        // copy icon (lucide)
                        svg {
                            class: "h-4 w-4",
                            fill: "none",
                            stroke: "currentColor",
                            view_box: "0 0 24 24",
                            path {
                                stroke_linecap: "round",
                                stroke_linejoin: "round",
                                stroke_width: "2",
                                d: "M16 4h2a2 2 0 012 2v14a2 2 0 01-2 2H6a2 2 0 01-2-2V6a2 2 0 012-2h2 M9 2h6a1 1 0 011 1v2a1 1 0 01-1 1H9a1 1 0 01-1-1V3a1 1 0 011-1z",
                            }
                        }
                    }
                }
            }
            // Status + timing metadata
            div { class: "flex flex-wrap items-center gap-y-3 gap-x-6",
                WalletStatusBadge { status: status_str }
                div { class: "flex items-center text-xs text-muted-foreground",
                    // clock icon (lucide)
                    svg {
                        class: "h-3.5 w-3.5 mr-1.5 opacity-70",
                        fill: "none",
                        stroke: "currentColor",
                        view_box: "0 0 24 24",
                        path {
                            stroke_linecap: "round",
                            stroke_linejoin: "round",
                            stroke_width: "2",
                            d: "M12 8v4l3 3 M12 22a10 10 0 100-20 10 10 0 000 20z",
                        }
                    }
                    span { "Created {created_at}" }
                }
                if let Some(la) = last_auth {
                    div { class: "flex items-center text-xs text-muted-foreground",
                        span { class: "w-1 h-1 rounded-full bg-muted-foreground mx-2 hidden sm:block" }
                        span { "Last active {la}" }
                    }
                }
            }
            // Disable callout
            if is_disabled {
                if let Some(reason) = disable_reason {
                    div { class: "mt-5 p-3.5 rounded-lg bg-amber-50 dark:bg-amber-900/20 border border-amber-100 dark:border-amber-900/30",
                        div { class: "flex items-center gap-2 text-amber-800 dark:text-amber-400 font-semibold text-sm mb-1.5",
                            span { "⚠️ Access Restricted" }
                        }
                        p { class: "text-sm text-gray-700 dark:text-muted-foreground leading-relaxed",
                            strong { class: "text-amber-700 dark:text-amber-500", "Reason: " }
                            " {reason}"
                        }
                        div { class: "mt-2.5 pt-2.5 border-t border-amber-100 dark:border-amber-900/30 text-[11px] text-muted-foreground flex flex-wrap gap-x-3 gap-y-1",
                            if let Some(by) = disabled_by {
                                span { "Disabled by " strong { "{by}" } }
                                span { "•" }
                            }
                            if let Some(at) = disabled_at {
                                span { "{at}" }
                                span { "•" }
                            }
                            if let Some(exp) = expires_at {
                                span { class: "text-amber-600 dark:text-amber-500", "Expires {exp}" }
                            }
                        }
                    }
                }
            }
        }
    }
}

// =====================================================================
// Tests
// =====================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_header() -> WalletHeaderData {
        WalletHeaderData {
            wallet_address: "0x1234567890abcdef1234567890abcdef12345678".to_string(),
            status: "active".to_string(),
            created_at: "2 days ago".to_string(),
            last_auth_at: Some("1 hour ago".to_string()),
            disable_reason: None,
            disable_disabled_by: None,
            disable_disabled_at: None,
            disable_expires_at: None,
        }
    }

    /// The address renders in monospace.
    #[test]
    fn test_wallet_header_renders_address() {
        fn render() -> Element {
            rsx! {
            WalletHeader {
                wallet: sample_header(),
            }
        }
        

        }

        let mut vdom = dioxus::prelude::VirtualDom::new(render);
        vdom.rebuild_in_place();
        let html = dioxus_ssr::render(&vdom);
        assert!(
            html.contains("0x1234567890abcdef"),
            "Address should render in monospace. Got: {html}"
        );
        assert!(
            html.contains("font-mono"),
            "Address should use font-mono. Got: {html}"
        );
    }

    /// The status badge renders.
    #[test]
    fn test_wallet_header_renders_status_badge() {
        fn render() -> Element {
            rsx! {
            WalletHeader {
                wallet: sample_header(),
            }
        }
        

        }

        let mut vdom = dioxus::prelude::VirtualDom::new(render);
        vdom.rebuild_in_place();
        let html = dioxus_ssr::render(&vdom);
        assert!(
            html.contains("Active"),
            "Status badge should render. Got: {html}"
        );
        assert!(
            html.contains("bg-green-100"),
            "Active status should use the green gradient. Got: {html}"
        );
    }

    /// Disabled wallet renders the amber callout.
    #[test]
    fn test_wallet_header_renders_disable_info() {
        fn render() -> Element {
            let mut header = sample_header();
            header.status = "disabled".to_string();
            header.disable_reason = Some("Suspicious activity detected".to_string());
            header.disable_disabled_by = Some("admin@example.com".to_string());
            header.disable_disabled_at = Some("2026-06-15".to_string());
            header.disable_expires_at = Some("2026-06-22".to_string());
            rsx! {
            WalletHeader {
                wallet: header,
            }
        }
        }

        let mut vdom = dioxus::prelude::VirtualDom::new(render);
        vdom.rebuild_in_place();
        let html = dioxus_ssr::render(&vdom);
        assert!(
            html.contains("Access Restricted"),
            "Disabled header should render the Access Restricted callout. Got: {html}"
        );
        assert!(
            html.contains("Suspicious activity detected"),
            "Disable reason should render. Got: {html}"
        );
        assert!(
            html.contains("bg-amber-50"),
            "Disabled callout should use amber. Got: {html}"
        );
        assert!(
            html.contains("admin@example.com"),
            "Disabled-by should render. Got: {html}"
        );
        assert!(
            html.contains("Expires 2026-06-22"),
            "Expires-at should render. Got: {html}"
        );
    }

    /// Active wallet does NOT render the disable callout.
    #[test]
    fn test_wallet_header_no_disable_callout_for_active() {
        fn render() -> Element {
            rsx! {
            WalletHeader {
                wallet: sample_header(),
            }
        }
        

        }

        let mut vdom = dioxus::prelude::VirtualDom::new(render);
        vdom.rebuild_in_place();
        let html = dioxus_ssr::render(&vdom);
        assert!(
            !html.contains("Access Restricted"),
            "Active header should not render disable callout. Got: {html}"
        );
    }

    /// The last-auth-at row renders when present.
    #[test]
    fn test_wallet_header_renders_last_auth() {
        fn render() -> Element {
            rsx! {
            WalletHeader {
                wallet: sample_header(),
            }
        }
        

        }

        let mut vdom = dioxus::prelude::VirtualDom::new(render);
        vdom.rebuild_in_place();
        let html = dioxus_ssr::render(&vdom);
        assert!(
            html.contains("Last active 1 hour ago"),
            "Last-auth row should render. Got: {html}"
        );
    }

    /// `class_name` is propagated.
    #[test]
    fn test_wallet_header_propagates_class_name() {
        fn render() -> Element {
            rsx! {
            WalletHeader {
                wallet: sample_header(),
                class_name: Some("mt-6".to_string()),
            }
        }
        

        }

        let mut vdom = dioxus::prelude::VirtualDom::new(render);
        vdom.rebuild_in_place();
        let html = dioxus_ssr::render(&vdom);
        assert!(
            html.contains("mt-6"),
            "class_name should propagate. Got: {html}"
        );
    }
}
