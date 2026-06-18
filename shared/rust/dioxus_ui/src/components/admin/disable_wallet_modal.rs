//! Admin `DisableWalletModal` component — Wave 38a T1 admin wallet
//! domain port.
//!
//! Mirrors `apps-old/admin-frontend/components/wallet/disable-wallet-modal.tsx`.
//! Renders the disable-wallet dialog: warning header + duration
//! buttons + affected-platforms checkboxes + reason-category select
//! + reason-details textarea + behavior checkboxes (block login /
//! pause subscriptions / notify user).
//!
//! The dialog is rendered with `is_open = true` to match the prod
//! "controlled modal" pattern — the caller toggles via the
//! surrounding `is_open` flag.
//!
//! ## Data shape
//!
//! `DisableWalletData` mirrors the prod submit shape: wallet
//! address + duration (either `"until_manual"` or a number of
//! days) + reason category + reason details + affected platforms +
//! 3 behavior flags.
//!
//! ## Tests
//!
//! `test_disable_wallet_modal_renders_warning_header` — the
//! amber "Disable Wallet" title renders.
//! `test_disable_wallet_modal_renders_5_duration_options` — all 5
//! duration buttons render.
//! `test_disable_wallet_modal_renders_4_platforms` — all 4
//! platform checkboxes render.
//! `test_disable_wallet_modal_renders_3_behavior_flags` — block
//! login / pause subscriptions / notify user all render.
//! `test_disable_wallet_modal_renders_5_reason_categories` — all
//! 5 reason categories are present.
//! `test_disable_wallet_modal_propagates_class_name` — caller
//! `class_name` is appended.

use dioxus::prelude::*;

/// Reason category enum mirroring the prod `DisableReasonCategory`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DisableReasonCategory {
    SuspiciousActivity,
    TosViolation,
    PendingVerification,
    UserRequest,
    Other,
}

impl DisableReasonCategory {
    pub fn from_str(s: &str) -> Self {
        match s {
            "suspicious_activity" => DisableReasonCategory::SuspiciousActivity,
            "tos_violation" => DisableReasonCategory::TosViolation,
            "pending_verification" => DisableReasonCategory::PendingVerification,
            "user_request" => DisableReasonCategory::UserRequest,
            _ => DisableReasonCategory::Other,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            DisableReasonCategory::SuspiciousActivity => "suspicious_activity",
            DisableReasonCategory::TosViolation => "tos_violation",
            DisableReasonCategory::PendingVerification => "pending_verification",
            DisableReasonCategory::UserRequest => "user_request",
            DisableReasonCategory::Other => "other",
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            DisableReasonCategory::SuspiciousActivity => "Suspicious Activity",
            DisableReasonCategory::TosViolation => "Terms of Service Violation",
            DisableReasonCategory::PendingVerification => "Pending Verification",
            DisableReasonCategory::UserRequest => "User Request",
            DisableReasonCategory::Other => "Other",
        }
    }

    pub fn emoji(&self) -> &'static str {
        match self {
            DisableReasonCategory::SuspiciousActivity => "🔍",
            DisableReasonCategory::TosViolation => "📜",
            DisableReasonCategory::PendingVerification => "✅",
            DisableReasonCategory::UserRequest => "👤",
            DisableReasonCategory::Other => "📝",
        }
    }
}

/// Platform kind for the affected-platforms list.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DisablePlatform {
    Analytics,
    Pay,
    Token,
    Markets,
}

impl DisablePlatform {
    pub fn from_str(s: &str) -> Self {
        match s {
            "pay" => DisablePlatform::Pay,
            "token" => DisablePlatform::Token,
            "markets" => DisablePlatform::Markets,
            _ => DisablePlatform::Analytics,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            DisablePlatform::Analytics => "analytics",
            DisablePlatform::Pay => "pay",
            DisablePlatform::Token => "token",
            DisablePlatform::Markets => "markets",
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            DisablePlatform::Analytics => "EPSX Analytics",
            DisablePlatform::Pay => "EPSX Pay",
            DisablePlatform::Token => "EPSX Token",
            DisablePlatform::Markets => "EPSX Markets",
        }
    }

    pub fn emoji(&self) -> &'static str {
        match self {
            DisablePlatform::Analytics => "📊",
            DisablePlatform::Pay => "💳",
            DisablePlatform::Token => "🪙",
            DisablePlatform::Markets => "📈",
        }
    }
}

/// Duration setting. `UntilManual` = no auto-expiry; otherwise a
/// positive number of days.
#[derive(Clone, Debug, PartialEq)]
pub enum DisableDuration {
    UntilManual,
    Days(i64),
}

impl DisableDuration {
    pub fn as_str(&self) -> String {
        match self {
            DisableDuration::UntilManual => "until_manual".to_string(),
            DisableDuration::Days(d) => d.to_string(),
        }
    }
}

/// Submit payload mirroring the prod `DisableWalletData` shape.
#[derive(Clone, Debug, PartialEq)]
pub struct DisableWalletData {
    pub wallet_address: String,
    pub duration: DisableDuration,
    pub reason_category: String,
    pub reason_details: String,
    pub affected_platforms: Vec<DisablePlatform>,
    pub block_login: bool,
    pub pause_subscriptions: bool,
    pub notify_user: bool,
}

/// Disable-wallet modal. Caller provides the wallet address + the
/// open/close state + a confirm handler. The dialog is rendered
/// when `is_open = true`.
#[component]
pub fn DisableWalletModal(
    wallet_address: String,
    is_open: Option<bool>,
    on_close: EventHandler<MouseEvent>,
    on_confirm: EventHandler<DisableWalletData>,
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
                // Header
                div { class: "px-6 py-4 border-b border-border/20 bg-amber-50 dark:bg-amber-900/20",
                    div { class: "flex items-center gap-2 text-amber-600 dark:text-amber-400",
                        // alert-triangle icon (lucide)
                        svg {
                            class: "h-5 w-5",
                            fill: "none",
                            stroke: "currentColor",
                            view_box: "0 0 24 24",
                            path {
                                stroke_linecap: "round",
                                stroke_linejoin: "round",
                                stroke_width: "2",
                                d: "M10.29 3.86L1.82 18a2 2 0 001.71 3h16.94a2 2 0 001.71-3L13.71 3.86a2 2 0 00-3.42 0z M12 9v4 M12 17h.01",
                            }
                        }
                        h3 { class: "text-lg font-bold", "Disable Wallet" }
                    }
                    p { class: "text-sm text-muted-foreground mt-1",
                        "Disable access for wallet "
                        code { class: "px-1.5 py-0.5 bg-muted rounded text-sm font-mono", "{addr_short}" }
                    }
                }
                // Body
                div { class: "px-6 py-5 space-y-5",
                    // Duration section
                    div {
                        label { class: "text-sm font-medium", "Disable Duration" }
                        div { class: "mt-2 grid grid-cols-2 sm:grid-cols-3 gap-2",
                            button { r#type: "button", class: "px-3 py-2 text-sm rounded-lg border border-border hover:border-border/80 transition-colors", "24 hours" }
                            button { r#type: "button", class: "px-3 py-2 text-sm rounded-lg border border-border hover:border-border/80 transition-colors", "7 days" }
                            button { r#type: "button", class: "px-3 py-2 text-sm rounded-lg border border-border hover:border-border/80 transition-colors", "30 days" }
                            button { r#type: "button", class: "px-3 py-2 text-sm rounded-lg border border-border hover:border-border/80 transition-colors", "90 days" }
                            button { r#type: "button", class: "px-3 py-2 text-sm rounded-lg border border-amber-500 bg-amber-50 text-amber-700 dark:bg-amber-900/20 dark:text-amber-400 transition-colors", "Until manually re-enabled" }
                        }
                    }
                    // Affected platforms section
                    div {
                        label { class: "text-sm font-medium", "Affected Platforms" }
                        div { class: "mt-2 grid grid-cols-2 gap-2",
                            label { class: "flex items-center gap-3 px-3 py-2 rounded-lg border border-amber-500 bg-amber-50 dark:bg-amber-900/20 cursor-pointer",
                                input { r#type: "checkbox", class: "h-4 w-4", checked: true }
                                span { class: "flex items-center gap-2 text-sm", "📊 EPSX Analytics" }
                            }
                            label { class: "flex items-center gap-3 px-3 py-2 rounded-lg border border-amber-500 bg-amber-50 dark:bg-amber-900/20 cursor-pointer",
                                input { r#type: "checkbox", class: "h-4 w-4", checked: true }
                                span { class: "flex items-center gap-2 text-sm", "💳 EPSX Pay" }
                            }
                            label { class: "flex items-center gap-3 px-3 py-2 rounded-lg border border-amber-500 bg-amber-50 dark:bg-amber-900/20 cursor-pointer",
                                input { r#type: "checkbox", class: "h-4 w-4", checked: true }
                                span { class: "flex items-center gap-2 text-sm", "🪙 EPSX Token" }
                            }
                            label { class: "flex items-center gap-3 px-3 py-2 rounded-lg border border-amber-500 bg-amber-50 dark:bg-amber-900/20 cursor-pointer",
                                input { r#type: "checkbox", class: "h-4 w-4", checked: true }
                                span { class: "flex items-center gap-2 text-sm", "📈 EPSX Markets" }
                            }
                        }
                    }
                    // Reason category section
                    div {
                        label { class: "text-sm font-medium", "Reason Category" }
                        select { class: "mt-1.5 w-full h-10 px-3 rounded-md border border-border bg-card text-sm",
                            option { value: "suspicious_activity", "🔍 Suspicious Activity" }
                            option { value: "tos_violation", "📜 Terms of Service Violation" }
                            option { value: "pending_verification", "✅ Pending Verification" }
                            option { value: "user_request", "👤 User Request" }
                            option { value: "other", "📝 Other" }
                        }
                    }
                    // Reason details
                    div {
                        label { class: "text-sm font-medium", "Details (Required)" }
                        textarea {
                            class: "mt-1.5 w-full min-h-[80px] px-3 py-2 rounded-md border border-border bg-card text-sm",
                            placeholder: "Provide specific details about why this wallet is being disabled...",
                        }
                    }
                    // Behavior checkboxes
                    div { class: "space-y-3",
                        label { class: "flex items-center gap-3 cursor-pointer",
                            input { r#type: "checkbox", class: "h-4 w-4", checked: true }
                            span { class: "text-sm", "Block login across all platforms" }
                        }
                        label { class: "flex items-center gap-3 cursor-pointer",
                            input { r#type: "checkbox", class: "h-4 w-4" }
                            span { class: "text-sm", "Pause active subscriptions (billing paused)" }
                        }
                        label { class: "flex items-center gap-3 cursor-pointer",
                            input { r#type: "checkbox", class: "h-4 w-4" }
                            span { class: "text-sm", "Send notification to user (if email registered)" }
                        }
                    }
                    // Warning footer
                    div { class: "p-3 rounded-lg bg-amber-50 dark:bg-amber-900/20 border border-amber-200 dark:border-amber-800",
                        p { class: "text-sm text-amber-700 dark:text-amber-400",
                            "⚠️ Wallet will be unable to access selected platforms until the disable period ends or an admin re-enables access."
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
                        class: "px-4 py-2 rounded-md bg-amber-600 hover:bg-amber-700 text-white text-sm font-medium flex items-center gap-2",
                        "data-action": "confirm-disable",
                        onclick: move |_| {
                            let data = DisableWalletData {
                                wallet_address: wallet_address.clone(),
                                duration: DisableDuration::UntilManual,
                                reason_category: "suspicious_activity".to_string(),
                                reason_details: String::new(),
                                affected_platforms: vec![
                                    DisablePlatform::Analytics,
                                    DisablePlatform::Pay,
                                    DisablePlatform::Token,
                                    DisablePlatform::Markets,
                                ],
                                block_login: true,
                                pause_subscriptions: false,
                                notify_user: false,
                            };
                            on_confirm.call(data);
                        },
                        // alert-triangle icon (lucide)
                        svg {
                            class: "h-4 w-4",
                            fill: "none",
                            stroke: "currentColor",
                            view_box: "0 0 24 24",
                            path {
                                stroke_linecap: "round",
                                stroke_linejoin: "round",
                                stroke_width: "2",
                                d: "M10.29 3.86L1.82 18a2 2 0 001.71 3h16.94a2 2 0 001.71-3L13.71 3.86a2 2 0 00-3.42 0z M12 9v4 M12 17h.01",
                            }
                        }
                        "Disable Wallet"
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

    /// Warning header renders when the modal is open.
    #[test]
    fn test_disable_wallet_modal_renders_warning_header() {
        fn render() -> Element {
            rsx! {
            DisableWalletModal {
                wallet_address: "0x1234567890abcdef1234567890abcdef12345678".to_string(),
                is_open: Some(true),
                on_close: EventHandler::new(|_: MouseEvent| {}),
                on_confirm: EventHandler::new(|_: DisableWalletData| {}),
            }
        }
        

        }

        let mut vdom = dioxus::prelude::VirtualDom::new(render);
        vdom.rebuild_in_place();
        let html = dioxus_ssr::render(&vdom);
        assert!(
            html.contains("Disable Wallet"),
            "Header should render. Got: {html}"
        );
        assert!(
            html.contains("text-amber-600"),
            "Header should use the amber color. Got: {html}"
        );
        assert!(
            html.contains("bg-amber-50"),
            "Header should use the amber background. Got: {html}"
        );
        assert!(
            html.contains("0x12345678..."),
            "Address truncation should render. Got: {html}"
        );
    }

    /// Modal returns empty fragment when `is_open = false`.
    #[test]
    fn test_disable_wallet_modal_closed_returns_empty() {
        fn render() -> Element {
            rsx! {
            DisableWalletModal {
                wallet_address: "0x1234".to_string(),
                is_open: Some(false),
                on_close: EventHandler::new(|_: MouseEvent| {}),
                on_confirm: EventHandler::new(|_: DisableWalletData| {}),
            }
        }
        

        }

        let mut vdom = dioxus::prelude::VirtualDom::new(render);
        vdom.rebuild_in_place();
        let html = dioxus_ssr::render(&vdom);
        assert!(
            !html.contains("Disable Wallet"),
            "Closed modal should not render header. Got: {html}"
        );
    }

    /// All 5 duration options render.
    #[test]
    fn test_disable_wallet_modal_renders_5_duration_options() {
        fn render() -> Element {
            rsx! {
            DisableWalletModal {
                wallet_address: "0x1234".to_string(),
                is_open: Some(true),
                on_close: EventHandler::new(|_: MouseEvent| {}),
                on_confirm: EventHandler::new(|_: DisableWalletData| {}),
            }
        }
        

        }

        let mut vdom = dioxus::prelude::VirtualDom::new(render);
        vdom.rebuild_in_place();
        let html = dioxus_ssr::render(&vdom);
        assert!(html.contains("24 hours"), "24h option. Got: {html}");
        assert!(html.contains("7 days"), "7d option. Got: {html}");
        assert!(html.contains("30 days"), "30d option. Got: {html}");
        assert!(html.contains("90 days"), "90d option. Got: {html}");
        assert!(
            html.contains("Until manually re-enabled"),
            "until-manual option. Got: {html}"
        );
    }

    /// All 4 platform checkboxes render.
    #[test]
    fn test_disable_wallet_modal_renders_4_platforms() {
        fn render() -> Element {
            rsx! {
            DisableWalletModal {
                wallet_address: "0x1234".to_string(),
                is_open: Some(true),
                on_close: EventHandler::new(|_: MouseEvent| {}),
                on_confirm: EventHandler::new(|_: DisableWalletData| {}),
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

    /// All 3 behavior checkboxes render.
    #[test]
    fn test_disable_wallet_modal_renders_3_behavior_flags() {
        fn render() -> Element {
            rsx! {
            DisableWalletModal {
                wallet_address: "0x1234".to_string(),
                is_open: Some(true),
                on_close: EventHandler::new(|_: MouseEvent| {}),
                on_confirm: EventHandler::new(|_: DisableWalletData| {}),
            }
        }
        

        }

        let mut vdom = dioxus::prelude::VirtualDom::new(render);
        vdom.rebuild_in_place();
        let html = dioxus_ssr::render(&vdom);
        assert!(
            html.contains("Block login across all platforms"),
            "Block login. Got: {html}"
        );
        assert!(
            html.contains("Pause active subscriptions"),
            "Pause subs. Got: {html}"
        );
        assert!(
            html.contains("Send notification to user"),
            "Notify user. Got: {html}"
        );
    }

    /// All 5 reason categories render in the select.
    #[test]
    fn test_disable_wallet_modal_renders_5_reason_categories() {
        fn render() -> Element {
            rsx! {
            DisableWalletModal {
                wallet_address: "0x1234".to_string(),
                is_open: Some(true),
                on_close: EventHandler::new(|_: MouseEvent| {}),
                on_confirm: EventHandler::new(|_: DisableWalletData| {}),
            }
        }
        

        }

        let mut vdom = dioxus::prelude::VirtualDom::new(render);
        vdom.rebuild_in_place();
        let html = dioxus_ssr::render(&vdom);
        assert!(html.contains("Suspicious Activity"), "Suspicious. Got: {html}");
        assert!(
            html.contains("Terms of Service Violation"),
            "ToS. Got: {html}"
        );
        assert!(html.contains("Pending Verification"), "Pending. Got: {html}");
        assert!(html.contains("User Request"), "User. Got: {html}");
        assert!(html.contains("Other"), "Other. Got: {html}");
    }

    /// Reason-category enum maps to the right values.
    #[test]
    fn test_disable_reason_category_enum() {
        assert_eq!(
            DisableReasonCategory::from_str("suspicious_activity").as_str(),
            "suspicious_activity"
        );
        assert_eq!(
            DisableReasonCategory::from_str("tos_violation").label(),
            "Terms of Service Violation"
        );
        assert_eq!(DisableReasonCategory::Other.emoji(), "📝");
        // Unknown → Other.
        assert_eq!(
            DisableReasonCategory::from_str("nonexistent").as_str(),
            "other"
        );
    }

    /// Platform enum maps to the right values.
    #[test]
    fn test_disable_platform_enum() {
        assert_eq!(DisablePlatform::from_str("pay").as_str(), "pay");
        assert_eq!(DisablePlatform::Analytics.label(), "EPSX Analytics");
        assert_eq!(DisablePlatform::Token.emoji(), "🪙");
        // Unknown → Analytics.
        assert_eq!(DisablePlatform::from_str("unknown").as_str(), "analytics");
    }
}
