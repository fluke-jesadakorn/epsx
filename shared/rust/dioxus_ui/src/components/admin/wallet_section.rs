//! Admin `WalletSection` component — Wave 38a T1 admin wallet
//! domain port.
//!
//! Mirrors `apps-old/admin-frontend/components/wallet/wallet-section.tsx`
//! — a layout container used to wrap the wallet-management page
//! sections. Provides the design-system section card (rounded-2xl
//! bg-card border border-border/20 shadow-xl overflow-hidden) +
//! an optional top accent bar (cyan → purple gradient).
//!
//! ## Visual layout
//!
//! - Outer wrapper: `rounded-2xl bg-card border border-border/20
//!   shadow-xl overflow-hidden`.
//! - Optional 3px top accent bar (cyan → purple gradient) when
//!   `accent = true` (the default).
//! - Optional header row (title + action slot).
//! - Body slot for caller content.
//!
//! ## Tests
//!
//! `test_wallet_section_renders_card_wrapper` — the design-system
//! section card classes render.
//! `test_wallet_section_renders_accent_bar_when_enabled` /
//! `_hides_accent_bar_when_disabled` — the top accent bar is
//! conditional.
//! `test_wallet_section_renders_title` / `_hides_title_when_absent`
//! — the title slot is conditional.
//! `test_wallet_section_propagates_class_name` — caller
//! `class_name` is appended.

use dioxus::prelude::*;

/// Wallet section — design-system section card with optional
/// accent bar + optional title row + body slot.
#[component]
pub fn WalletSection(
    /// Optional section title (rendered as `<h2>`).
    title: Option<String>,
    /// Optional subtitle / description below the title.
    subtitle: Option<String>,
    /// Whether to render the cyan → purple top accent bar
    /// (default: true).
    accent: Option<bool>,
    /// Optional action slot content (rendered in the top-right).
    action: Option<String>,
    /// Optional extra classes appended to the outer wrapper.
    class_name: Option<String>,
    children: Element,
) -> Element {
    let accent = accent.unwrap_or(true);
    let mut cls =
        "rounded-2xl bg-card border border-border/20 shadow-xl overflow-hidden".to_string();
    if let Some(extra) = class_name {
        if !extra.is_empty() {
            cls.push(' ');
            cls.push_str(&extra);
        }
    }
    rsx! {
        section { class: "{cls}",
            if accent {
                div { class: "h-[3px] bg-gradient-to-r from-[#1fc7d4] to-[#7645d9]" }
            }
            if title.is_some() || action.is_some() {
                div { class: "px-5 py-4 flex items-center justify-between gap-3 border-b border-border/20",
                    div {
                        if let Some(t) = title.clone() {
                            if !t.is_empty() {
                                h2 { class: "text-lg font-bold text-foreground", "{t}" }
                            }
                        }
                        if let Some(s) = subtitle.clone() {
                            if !s.is_empty() {
                                p { class: "text-sm text-muted-foreground mt-0.5", "{s}" }
                            }
                        }
                    }
                    if let Some(a) = action.clone() {
                        if !a.is_empty() {
                            div { class: "wallet-section-action shrink-0",
                                div { dangerous_inner_html: "{a}" }
                            }
                        }
                    }
                }
            }
            div { class: "wallet-section-body",
                {children}
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

    /// The base section card wrapper classes render.
    #[test]
    fn test_wallet_section_renders_card_wrapper() {
        let el = rsx! {
            WalletSection {
                "Body content"
            }
        };
        let html = dioxus_ssr::render_element(el);
        assert!(
            html.contains("rounded-2xl bg-card border border-border/20 shadow-xl overflow-hidden"),
            "Section should use the design-system wrapper classes. Got: {html}"
        );
        assert!(
            html.contains("Body content"),
            "Section should render body children. Got: {html}"
        );
    }

    /// Accent bar renders when `accent = true` (default).
    #[test]
    fn test_wallet_section_renders_accent_bar_when_enabled() {
        let el = rsx! {
            WalletSection {
                "Body"
            }
        };
        let html = dioxus_ssr::render_element(el);
        assert!(
            html.contains("bg-gradient-to-r from-[#1fc7d4] to-[#7645d9]"),
            "Default accent should render. Got: {html}"
        );
    }

    /// Accent bar is hidden when `accent = false`.
    #[test]
    fn test_wallet_section_hides_accent_bar_when_disabled() {
        let el = rsx! {
            WalletSection {
                accent: Some(false),
                "Body"
            }
        };
        let html = dioxus_ssr::render_element(el);
        assert!(
            !html.contains("bg-gradient-to-r from-[#1fc7d4] to-[#7645d9]"),
            "Accent bar should be hidden when accent=false. Got: {html}"
        );
    }

    /// Title renders as h2.
    #[test]
    fn test_wallet_section_renders_title() {
        let el = rsx! {
            WalletSection {
                title: Some("Wallet Overview".to_string()),
                subtitle: Some("Manage all your wallets".to_string()),
                "Body"
            }
        };
        let html = dioxus_ssr::render_element(el);
        assert!(
            html.contains("Wallet Overview"),
            "Title should render. Got: {html}"
        );
        assert!(
            html.contains("Manage all your wallets"),
            "Subtitle should render. Got: {html}"
        );
        assert!(html.contains("<h2"), "Title should be an h2. Got: {html}");
    }

    /// Title slot absent → no header row.
    #[test]
    fn test_wallet_section_hides_title_when_absent() {
        let el = rsx! {
            WalletSection {
                "Body only"
            }
        };
        let html = dioxus_ssr::render_element(el);
        assert!(
            !html.contains("<h2"),
            "Header row should not render when title is absent. Got: {html}"
        );
    }

    /// `class_name` is propagated.
    #[test]
    fn test_wallet_section_propagates_class_name() {
        let el = rsx! {
            WalletSection {
                class_name: Some("mt-4 max-w-3xl".to_string()),
                "Body"
            }
        };
        let html = dioxus_ssr::render_element(el);
        assert!(
            html.contains("mt-4"),
            "class_name should propagate. Got: {html}"
        );
        assert!(
            html.contains("max-w-3xl"),
            "class_name should propagate. Got: {html}"
        );
    }
}
