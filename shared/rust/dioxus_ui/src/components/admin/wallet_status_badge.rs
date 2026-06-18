//! Admin `WalletStatusBadge` component — Wave 38a T1 admin wallet
//! domain port.
//!
//! Mirrors `apps-old/admin-frontend/components/wallet/wallet-status-badge.tsx`.
//! Renders the status pill for a wallet (Active / Disabled) using
//! the design-system badge classes (`bg-green-100 text-green-800
//! dark:bg-green-900/30 ...`).
//!
//! ## Tests
//!
//! `test_wallet_status_badge_active` — renders "Active" + the
//! green gradient classes.
//! `test_wallet_status_badge_disabled` — renders "Disabled" + the
//! amber gradient classes.
//! `test_wallet_status_badge_unknown_status_falls_back_to_active` —
//! an unknown status string falls back to the active config so the
//! UI never silently breaks.
//! `test_wallet_status_badge_propagates_class_name` — caller
//! `class_name` is appended to the root pill.

use dioxus::prelude::*;

/// Status enum mirroring the prod `WalletStatus` type. Maps to
/// the same gradient pill classes used by the prod badge.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum WalletStatusKind {
    Active,
    Disabled,
}

impl WalletStatusKind {
    /// Resolve a status string. Unknown values fall back to
    /// `Active` (matches prod: `?? STATUS_CONFIG.active`).
    pub fn from_str(s: &str) -> Self {
        match s {
            "active" => WalletStatusKind::Active,
            "disabled" => WalletStatusKind::Disabled,
            _ => WalletStatusKind::Active,
        }
    }

    /// Tailwind classes for the pill background + text.
    pub fn classes(&self) -> &'static str {
        match self {
            WalletStatusKind::Active => {
                "bg-green-100 text-green-800 dark:bg-green-900/30 dark:text-green-400 border-green-200 dark:border-green-800"
            }
            WalletStatusKind::Disabled => {
                "bg-amber-100 text-amber-800 dark:bg-amber-900/30 dark:text-amber-400 border-amber-200 dark:border-amber-800"
            }
        }
    }

    /// Display label.
    pub fn label(&self) -> &'static str {
        match self {
            WalletStatusKind::Active => "Active",
            WalletStatusKind::Disabled => "Disabled",
        }
    }

    /// Emoji glyph rendered before the label (matches prod).
    pub fn emoji(&self) -> &'static str {
        match self {
            WalletStatusKind::Active => "🟢",
            WalletStatusKind::Disabled => "⚠️",
        }
    }
}

/// Wallet status pill — Active / Disabled.
///
/// Mirrors the prod `WalletStatusBadge` from
/// `apps-old/admin-frontend/components/wallet/wallet-status-badge.tsx`
/// lines 38–46. Caller passes a status string ("active" /
/// "disabled") or a `WalletStatusKind` directly.
#[component]
pub fn WalletStatusBadge(
    /// Status string — `"active"` or `"disabled"`. Unknown values
    /// fall back to the active config (matches prod).
    status: String,
    /// Optional extra classes appended to the root pill.
    class_name: Option<String>,
) -> Element {
    let kind = WalletStatusKind::from_str(&status);
    let base = "px-3 py-1 border font-medium";
    let kind_cls = kind.classes();
    let mut cls = format!("{base} {kind_cls}");
    if let Some(extra) = class_name {
        if !extra.is_empty() {
            cls.push(' ');
            cls.push_str(&extra);
        }
    }
    let emoji = kind.emoji();
    let label = kind.label();
    rsx! {
        span { class: "inline-flex items-center rounded-full border {cls}",
            span { class: "mr-1.5", "{emoji}" }
            "{label}"
        }
    }
}

// =====================================================================
// Tests
// =====================================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// Active status renders "Active" + the green gradient classes.
    #[test]
    fn test_wallet_status_badge_active() {
        let el = rsx! {
            WalletStatusBadge {
                status: "active".to_string(),
            }
        };
        let html = dioxus_ssr::render_element(el);
        assert!(html.contains("Active"), "should render the Active label. Got: {html}");
        assert!(
            html.contains("bg-green-100"),
            "Active should use the green pill background. Got: {html}"
        );
        assert!(
            html.contains("🟢"),
            "Active should render the green dot emoji. Got: {html}"
        );
    }

    /// Disabled status renders "Disabled" + the amber gradient classes.
    #[test]
    fn test_wallet_status_badge_disabled() {
        let el = rsx! {
            WalletStatusBadge {
                status: "disabled".to_string(),
            }
        };
        let html = dioxus_ssr::render_element(el);
        assert!(html.contains("Disabled"), "should render the Disabled label. Got: {html}");
        assert!(
            html.contains("bg-amber-100"),
            "Disabled should use the amber pill background. Got: {html}"
        );
        assert!(
            html.contains("⚠️"),
            "Disabled should render the warning emoji. Got: {html}"
        );
    }

    /// Unknown status strings fall back to the active config
    /// (matches prod's `?? STATUS_CONFIG.active` fallback).
    #[test]
    fn test_wallet_status_badge_unknown_status_falls_back_to_active() {
        let el = rsx! {
            WalletStatusBadge {
                status: "this-does-not-exist".to_string(),
            }
        };
        let html = dioxus_ssr::render_element(el);
        assert!(
            html.contains("Active"),
            "Unknown status should fall back to Active. Got: {html}"
        );
    }

    /// Caller-supplied `class_name` is appended to the root pill.
    #[test]
    fn test_wallet_status_badge_propagates_class_name() {
        let el = rsx! {
            WalletStatusBadge {
                status: "active".to_string(),
                class_name: Some("ml-2 uppercase".to_string()),
            }
        };
        let html = dioxus_ssr::render_element(el);
        assert!(
            html.contains("ml-2"),
            "class_name should propagate to the root pill. Got: {html}"
        );
        assert!(
            html.contains("uppercase"),
            "class_name should propagate to the root pill. Got: {html}"
        );
    }

    /// The status enum maps to the right Tailwind gradient classes.
    #[test]
    fn test_status_kind_classes() {
        assert!(WalletStatusKind::Active.classes().contains("green-100"));
        assert!(WalletStatusKind::Disabled.classes().contains("amber-100"));
        assert_eq!(WalletStatusKind::Active.label(), "Active");
        assert_eq!(WalletStatusKind::Disabled.label(), "Disabled");
        assert_eq!(WalletStatusKind::Active.emoji(), "🟢");
        assert_eq!(WalletStatusKind::Disabled.emoji(), "⚠️");
    }
}
