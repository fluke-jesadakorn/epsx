//! Admin `ReenableWalletModal` component — Wave 38a T1 admin wallet
//! domain port.
//!
//! Mirrors `apps-old/admin-frontend/components/wallet/reenable-wallet-modal.tsx`.
//! Renders the re-enable-wallet dialog: green header + disable-info
//! callout (optional) + platforms-to-re-enable checkboxes + 2
//! behavior checkboxes (restore permissions / resume subscriptions)
//! + resolution-note textarea.
//!
//! The dialog is rendered with `is_open = true` to match the prod
//! "controlled modal" pattern — the caller toggles via the
//! surrounding `is_open` flag.
//!
//! ## Tests
//!
//! `test_reenable_wallet_modal_renders_green_header` — the green
//! "Re-enable Wallet Access" title renders.
//! `test_reenable_wallet_modal_renders_disable_info_callout` —
//! when `disable_info` is supplied, the amber callout with
//! reason / by / at renders.
//! `test_reenable_wallet_modal_renders_4_platforms` — all 4
//! platform checkboxes render.
//! `test_reenable_wallet_modal_renders_2_behavior_flags` — restore
//! permissions + resume subscriptions both render.
//! `test_reenable_wallet_modal_closed_returns_empty` — when
//! `is_open = false`, the modal renders nothing.

use dioxus::prelude::*;

use super::disable_wallet_modal::DisablePlatform;

/// Disable-info payload surfaced in the re-enable callout.
#[derive(Clone, Debug, PartialEq)]
pub struct ReenableDisableInfo {
    pub disabled_by: String,
    pub disabled_at: String,
    pub reason_category: String,
    pub reason_details: String,
    pub expires_at: Option<String>,
    pub affected_platforms: Vec<DisablePlatform>,
}

/// Submit payload mirroring the prod `ReenableWalletData` shape.
#[derive(Clone, Debug, PartialEq)]
pub struct ReenableWalletData {
    pub wallet_address: String,
    pub platforms_to_enable: Vec<DisablePlatform>,
    pub restore_permissions: bool,
    pub resume_subscriptions: bool,
    pub resolution_note: String,
}

/// Re-enable wallet modal. Caller provides the wallet address +
/// the open/close state + optional disable info + a confirm
/// handler.
#[component]
pub fn ReenableWalletModal(
    wallet_address: String,
    /// Optional disable-info, rendered as an amber callout at
    /// the top of the modal.
    disable_info: Option<ReenableDisableInfo>,
    is_open: Option<bool>,
    on_close: EventHandler<MouseEvent>,
    on_confirm: EventHandler<ReenableWalletData>,
    /// Optional extra classes appended to the dialog wrapper.
    class_name: Option<String>,
) -> Element {
    let is_open = is_open.unwrap_or(false);
    if !is_open {
        return rsx! { Fragment {} };
    }
    let mut cls = "fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/50 backdrop-blur-sm".to_string();
    if let Some(extra) = class_name {
        if !extra.is_empty() {
            cls.push(' ');
            cls.push_str(&extra);
        }
    }
    let addr_short = if wallet_address.len() > 16 {
        format!("{}...{}", &wallet_address[..10], &wallet_address[wallet_address.len() - 6..])
    } else {
        wallet_address.clone()
    };
    rsx! {
        div { class: "{cls}",
            div { class: "bg-card rounded-2xl border border-border/20 shadow-2xl w-full max-w-lg overflow-hidden",
                // Header (green / emerald)
                div { class: "px-6 py-4 border-b border-border/20 bg-emerald-50 dark:bg-emerald-900/20",
                    div { class: "flex items-center gap-2 text-emerald-600 dark:text-emerald-400",
                        // unlock icon (lucide)
                        svg {
                            class: "h-5 w-5",
                            fill: "none",
                            stroke: "currentColor",
                            view_box: "0 0 24 24",
                            path {
                                stroke_linecap: "round",
                                stroke_linejoin: "round",
                                stroke_width: "2",
                                d: "M5 11h14a2 2 0 012 2v8a2 2 0 01-2 2H5a2 2 0 01-2-2v-8a2 2 0 012-2z M7 11V7a5 5 0 019.9-1",
                            }
                        }
                        h3 { class: "text-lg font-bold", "Re-enable Wallet Access" }
                    }
                    p { class: "text-sm text-muted-foreground mt-1",
                        "Restore access for wallet "
                        code { class: "px-1.5 py-0.5 bg-muted rounded text-sm font-mono", "{addr_short}" }
                    }
                }
                // Body
                div { class: "px-6 py-5 space-y-5",
                    // Disable-info callout (when supplied)
                    if let Some(info) = disable_info.clone() {
                        div { class: "p-4 rounded-xl bg-amber-50 dark:bg-amber-900/20 border border-amber-200 dark:border-amber-800",
                            h4 { class: "text-sm font-medium text-amber-800 dark:text-amber-300 mb-2", "Current Disable Information" }
                            div { class: "space-y-1.5 text-sm text-amber-700 dark:text-amber-400",
                                p { strong { "Disabled by: " } "{info.disabled_by}" }
                                p { strong { "Disabled on: " } "{info.disabled_at}" }
                                p { strong { "Reason: " } "{info.reason_category}" }
                                p { strong { "Details: " } "{info.reason_details}" }
                                if let Some(exp) = info.expires_at.clone() {
                                    p { strong { "Scheduled expiry: " } "{exp}" }
                                }
                            }
                        }
                    } else {
                        div { class: "p-4 rounded-xl bg-amber-50 dark:bg-amber-900/20 border border-amber-200 dark:border-amber-800",
                            p { class: "text-sm text-amber-700 dark:text-amber-400",
                                "This wallet is currently disabled. Select platforms and provide a resolution note to re-enable access."
                            }
                        }
                    }
                    // Platforms to re-enable
                    div {
                        label { class: "text-sm font-medium", "Platforms to Re-enable" }
                        div { class: "mt-2 grid grid-cols-2 gap-2",
                            label { class: "flex items-center gap-3 px-3 py-2 rounded-lg border border-green-500 bg-green-50 dark:bg-green-900/20 cursor-pointer",
                                input { r#type: "checkbox", class: "h-4 w-4", checked: true }
                                span { class: "flex items-center gap-2 text-sm", "📊 EPSX Analytics" }
                            }
                            label { class: "flex items-center gap-3 px-3 py-2 rounded-lg border border-green-500 bg-green-50 dark:bg-green-900/20 cursor-pointer",
                                input { r#type: "checkbox", class: "h-4 w-4", checked: true }
                                span { class: "flex items-center gap-2 text-sm", "💳 EPSX Pay" }
                            }
                            label { class: "flex items-center gap-3 px-3 py-2 rounded-lg border border-green-500 bg-green-50 dark:bg-green-900/20 cursor-pointer",
                                input { r#type: "checkbox", class: "h-4 w-4", checked: true }
                                span { class: "flex items-center gap-2 text-sm", "🪙 EPSX Token" }
                            }
                            label { class: "flex items-center gap-3 px-3 py-2 rounded-lg border border-green-500 bg-green-50 dark:bg-green-900/20 cursor-pointer",
                                input { r#type: "checkbox", class: "h-4 w-4", checked: true }
                                span { class: "flex items-center gap-2 text-sm", "📈 EPSX Markets" }
                            }
                        }
                    }
                    // Behavior checkboxes
                    div { class: "space-y-3",
                        label { class: "flex items-center gap-3 cursor-pointer",
                            input { r#type: "checkbox", class: "h-4 w-4", checked: true }
                            span { class: "text-sm", "Restore all previous permissions" }
                        }
                        label { class: "flex items-center gap-3 cursor-pointer",
                            input { r#type: "checkbox", class: "h-4 w-4", checked: true }
                            span { class: "text-sm", "Resume paused subscriptions" }
                        }
                    }
                    // Resolution note
                    div {
                        label { class: "text-sm font-medium", "Resolution Note (Required)" }
                        textarea {
                            class: "mt-1.5 w-full min-h-[80px] px-3 py-2 rounded-md border border-border bg-card text-sm",
                            placeholder: "e.g., Investigation complete - false positive, user verified identity...",
                        }
                    }
                }
                // Footer
                div { class: "px-6 py-4 border-t border-border/20 flex justify-end gap-2 bg-muted/10",
                    button {
                        r#type: "button",
                        class: "px-4 py-2 rounded-md border border-border bg-card text-sm font-medium hover:bg-muted",
                        onclick: move |e| on_close.call(e),
                        "Cancel"
                    }
                    button {
                        r#type: "button",
                        class: "px-4 py-2 rounded-md bg-emerald-600 hover:bg-emerald-700 text-white text-sm font-medium flex items-center gap-2",
                        "data-action": "confirm-reenable",
                        onclick: move |_| {
                            let data = ReenableWalletData {
                                wallet_address: wallet_address.clone(),
                                platforms_to_enable: vec![
                                    DisablePlatform::Analytics,
                                    DisablePlatform::Pay,
                                    DisablePlatform::Token,
                                    DisablePlatform::Markets,
                                ],
                                restore_permissions: true,
                                resume_subscriptions: true,
                                resolution_note: String::new(),
                            };
                            on_confirm.call(data);
                        },
                        // unlock icon (lucide)
                        svg {
                            class: "h-4 w-4",
                            fill: "none",
                            stroke: "currentColor",
                            view_box: "0 0 24 24",
                            path {
                                stroke_linecap: "round",
                                stroke_linejoin: "round",
                                stroke_width: "2",
                                d: "M5 11h14a2 2 0 012 2v8a2 2 0 01-2 2H5a2 2 0 01-2-2v-8a2 2 0 012-2z M7 11V7a5 5 0 019.9-1",
                            }
                        }
                        "Re-enable Access"
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

    /// Green header renders when modal is open.
    #[test]
    fn test_reenable_wallet_modal_renders_green_header() {
        fn render() -> Element {
            rsx! {
            ReenableWalletModal {
                wallet_address: "0x1234567890abcdef1234567890abcdef12345678".to_string(),
                is_open: Some(true),
                on_close: EventHandler::new(|_: MouseEvent| {}),
                on_confirm: EventHandler::new(|_: ReenableWalletData| {}),
            }
        }
        

        }

        let mut vdom = dioxus::prelude::VirtualDom::new(render);
        vdom.rebuild_in_place();
        let html = dioxus_ssr::render(&vdom);
        assert!(
            html.contains("Re-enable Wallet Access"),
            "Header should render. Got: {html}"
        );
        assert!(
            html.contains("text-emerald-600"),
            "Header should use the emerald color. Got: {html}"
        );
        assert!(
            html.contains("0x12345678..."),
            "Address truncation should render. Got: {html}"
        );
    }

    /// Modal returns empty fragment when `is_open = false`.
    #[test]
    fn test_reenable_wallet_modal_closed_returns_empty() {
        fn render() -> Element {
            rsx! {
            ReenableWalletModal {
                wallet_address: "0x1234".to_string(),
                is_open: Some(false),
                on_close: EventHandler::new(|_: MouseEvent| {}),
                on_confirm: EventHandler::new(|_: ReenableWalletData| {}),
            }
        }
        

        }

        let mut vdom = dioxus::prelude::VirtualDom::new(render);
        vdom.rebuild_in_place();
        let html = dioxus_ssr::render(&vdom);
        assert!(
            !html.contains("Re-enable Wallet Access"),
            "Closed modal should not render. Got: {html}"
        );
    }

    /// Disable-info callout renders when `disable_info` is supplied.
    #[test]
    fn test_reenable_wallet_modal_renders_disable_info_callout() {
        fn render() -> Element {
            let info = ReenableDisableInfo {
                disabled_by: "admin@example.com".to_string(),
                disabled_at: "2026-06-15".to_string(),
                reason_category: "suspicious_activity".to_string(),
                reason_details: "Suspicious API call pattern".to_string(),
                expires_at: Some("2026-06-22".to_string()),
                affected_platforms: vec![DisablePlatform::Analytics, DisablePlatform::Pay],
            };
            rsx! {
            ReenableWalletModal {
                wallet_address: "0x1234".to_string(),
                disable_info: Some(info),
                is_open: Some(true),
                on_close: EventHandler::new(|_: MouseEvent| {}),
                on_confirm: EventHandler::new(|_: ReenableWalletData| {}),
            }
        }
        }

        let mut vdom = dioxus::prelude::VirtualDom::new(render);
        vdom.rebuild_in_place();
        let html = dioxus_ssr::render(&vdom);
        assert!(
            html.contains("Current Disable Information"),
            "Callout heading should render. Got: {html}"
        );
        assert!(
            html.contains("admin@example.com"),
            "Disabled-by should render. Got: {html}"
        );
        assert!(
            html.contains("2026-06-15"),
            "Disabled-at should render. Got: {html}"
        );
        assert!(
            html.contains("suspicious_activity"),
            "Reason category should render. Got: {html}"
        );
        assert!(
            html.contains("Scheduled expiry"),
            "Expires-at row should render. Got: {html}"
        );
    }

    /// All 4 platform checkboxes render.
    #[test]
    fn test_reenable_wallet_modal_renders_4_platforms() {
        fn render() -> Element {
            rsx! {
            ReenableWalletModal {
                wallet_address: "0x1234".to_string(),
                is_open: Some(true),
                on_close: EventHandler::new(|_: MouseEvent| {}),
                on_confirm: EventHandler::new(|_: ReenableWalletData| {}),
            }
        }
        

        }

        let mut vdom = dioxus::prelude::VirtualDom::new(render);
        vdom.rebuild_in_place();
        let html = dioxus_ssr::render(&vdom);
        assert!(html.contains("EPSX Analytics"), "Analytics. Got: {html}");
        assert!(html.contains("EPSX Pay"), "Pay. Got: {html}");
        assert!(html.contains("EPSX Token"), "Token. Got: {html}");
        assert!(html.contains("EPSX Markets"), "Markets. Got: {html}");
    }

    /// All 2 behavior checkboxes render.
    #[test]
    fn test_reenable_wallet_modal_renders_2_behavior_flags() {
        fn render() -> Element {
            rsx! {
            ReenableWalletModal {
                wallet_address: "0x1234".to_string(),
                is_open: Some(true),
                on_close: EventHandler::new(|_: MouseEvent| {}),
                on_confirm: EventHandler::new(|_: ReenableWalletData| {}),
            }
        }
        

        }

        let mut vdom = dioxus::prelude::VirtualDom::new(render);
        vdom.rebuild_in_place();
        let html = dioxus_ssr::render(&vdom);
        assert!(
            html.contains("Restore all previous permissions"),
            "Restore permissions. Got: {html}"
        );
        assert!(
            html.contains("Resume paused subscriptions"),
            "Resume subs. Got: {html}"
        );
    }
}
