//! Admin `WalletCard` component — Wave 38a T1 admin wallet domain
//! port.
//!
//! Mirrors `apps-old/admin-frontend/components/wallet/wallet-card.tsx`.
//! Renders a single wallet card with the design-system glass
//! styling: rounded border, hover-lift, dual blur orbs.
//!
//! The card has 3 logical sections (Identity / Stats / Actions).
//! For SSR simplicity, each section is rendered as a slot (string)
//! rather than composing child components — the prod
//! `WalletCardSections` (sub-components) are not ported in this
//! wave. Migration note: callers should compose the 3 slot
//! strings as needed; a future wave can split this into child
//! components (`WalletCardIdentity`, `WalletCardStats`,
//! `WalletCardActions`).
//!
//! ## Visual layout
//!
//! - Outer wrapper: rounded-xl border + bg-card/60 + hover
//!   shadow + hover border (#7645d9/30).
//! - Two decorative blur orbs (cyan top-left, purple bottom-right).
//! - Inner content row: rounded-xl bg-white/[0.02] + flex-col
//!   with the 3 sections.
//!
//! ## Tests
//!
//! `test_wallet_card_renders_glass_wrapper` — the glass card
//! classes render.
//! `test_wallet_card_selected_ring` — `is_selected = true` adds
//! the cyan ring.
//! `test_wallet_card_disabled_opacity` — `is_disabled = true` adds
//! opacity-60 + grayscale.
//! `test_wallet_card_renders_all_sections` — identity / stats /
//! actions all render.

use dioxus::prelude::*;

/// Wallet card data payload. Mirrors a subset of the prod
/// `WalletData` shape used by the card.
#[derive(Clone, Debug, PartialEq)]
pub struct WalletCardData {
    pub wallet_address: String,
    pub status: String,
    pub label: Option<String>,
}

/// Single-wallet card with the design-system glass styling.
/// Caller passes the 3 slot strings (identity / stats / actions).
/// Action handlers are forwarded as `EventHandler`s.
#[component]
pub fn WalletCard(
    /// Card data payload (address + status + label).
    wallet: WalletCardData,
    /// Whether the card's checkbox / selection ring is active.
    is_selected: Option<bool>,
    /// Whether the card is disabled (visually muted).
    is_disabled: Option<bool>,
    /// Identity slot content (avatar + address + label).
    identity: String,
    /// Stats slot content (badges, plan name, etc.).
    stats: String,
    /// Actions slot content (buttons, dropdown).
    actions: String,
    /// Optional extra classes appended to the outer wrapper.
    class_name: Option<String>,
) -> Element {
    let is_selected = is_selected.unwrap_or(false);
    let is_disabled = is_disabled.unwrap_or(false);
    let mut cls = "group relative w-full overflow-hidden rounded-xl border border-border/40 bg-card/60 p-1 transition-all duration-300 hover:border-[#7645d9]/30 hover:shadow-2xl hover:shadow-[#7645d9]/10".to_string();
    if is_selected {
        cls.push_str(" ring-2 ring-[#1fc7d4] bg-[#1fc7d4]/5");
    }
    if is_disabled {
        cls.push_str(" opacity-60 grayscale-[0.5]");
    }
    if let Some(extra) = class_name {
        if !extra.is_empty() {
            cls.push(' ');
            cls.push_str(&extra);
        }
    }
    let data_addr = wallet.wallet_address.clone();
    rsx! {
        div {
            class: "{cls}",
            "data-wallet-card": "true",
            "data-wallet-address": "{data_addr}",
            // Decorative blur orbs
            div { class: "absolute -left-16 -top-16 h-32 w-32 rounded-full bg-[#1fc7d4]/5 blur-[50px] transition-all duration-500 group-hover:bg-[#1fc7d4]/10" }
            div { class: "absolute -right-16 -bottom-16 h-32 w-32 rounded-full bg-[#7645d9]/5 blur-[50px] transition-all duration-500 group-hover:bg-[#7645d9]/10" }
            // Inner content row
            div { class: "relative flex flex-col gap-6 rounded-xl bg-white/[0.02] p-4 sm:p-5",
                // Identity slot
                div { class: "wallet-card-identity",
                    // The identity string is rendered via
                    // `dangerous_inner_html` so callers can pass
                    // any HTML structure (matches the prod
                    // `WalletCardIdentity` sub-component).
                    div { dangerous_inner_html: "{identity}" }
                }
                // Stats slot
                div { class: "wallet-card-stats",
                    div { dangerous_inner_html: "{stats}" }
                }
                // Actions slot
                div { class: "wallet-card-actions",
                    div { dangerous_inner_html: "{actions}" }
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

    fn sample_card() -> WalletCardData {
        WalletCardData {
            wallet_address: "0x1234567890abcdef1234567890abcdef12345678".to_string(),
            status: "active".to_string(),
            label: Some("VIP".to_string()),
        }
    }

    /// The base card emits the glass wrapper classes.
    #[test]
    fn test_wallet_card_renders_glass_wrapper() {
        let el = rsx! {
            WalletCard {
                wallet: sample_card(),
                identity: "<div>Identity</div>".to_string(),
                stats: "<div>Stats</div>".to_string(),
                actions: "<div>Actions</div>".to_string(),
            }
        };
        let html = dioxus_ssr::render_element(el);
        assert!(
            html.contains("rounded-xl border border-border/40 bg-card/60"),
            "Card should use the glass wrapper classes. Got: {html}"
        );
        assert!(
            html.contains("hover:border-[#7645d9]/30"),
            "Card should have hover border. Got: {html}"
        );
        // Decorative blur orbs.
        assert!(
            html.contains("blur-[50px]"),
            "Card should render blur orbs. Got: {html}"
        );
    }

    /// `is_selected = true` adds the cyan ring.
    #[test]
    fn test_wallet_card_selected_ring() {
        let el = rsx! {
            WalletCard {
                wallet: sample_card(),
                is_selected: Some(true),
                identity: "<div>Identity</div>".to_string(),
                stats: "<div>Stats</div>".to_string(),
                actions: "<div>Actions</div>".to_string(),
            }
        };
        let html = dioxus_ssr::render_element(el);
        assert!(
            html.contains("ring-2 ring-[#1fc7d4]"),
            "Selected card should have cyan ring. Got: {html}"
        );
        assert!(
            html.contains("bg-[#1fc7d4]/5"),
            "Selected card should have cyan bg. Got: {html}"
        );
    }

    /// `is_disabled = true` adds opacity-60 + grayscale.
    #[test]
    fn test_wallet_card_disabled_opacity() {
        let el = rsx! {
            WalletCard {
                wallet: sample_card(),
                is_disabled: Some(true),
                identity: "<div>Identity</div>".to_string(),
                stats: "<div>Stats</div>".to_string(),
                actions: "<div>Actions</div>".to_string(),
            }
        };
        let html = dioxus_ssr::render_element(el);
        assert!(
            html.contains("opacity-60"),
            "Disabled card should have opacity-60. Got: {html}"
        );
        assert!(
            html.contains("grayscale-[0.5]"),
            "Disabled card should have grayscale. Got: {html}"
        );
    }

    /// All 3 sections (identity / stats / actions) render.
    #[test]
    fn test_wallet_card_renders_all_sections() {
        let el = rsx! {
            WalletCard {
                wallet: sample_card(),
                identity: "<span data-slot='identity'>IDENTITY</span>".to_string(),
                stats: "<span data-slot='stats'>STATS</span>".to_string(),
                actions: "<span data-slot='actions'>ACTIONS</span>".to_string(),
            }
        };
        let html = dioxus_ssr::render_element(el);
        assert!(html.contains("IDENTITY"), "Identity slot should render. Got: {html}");
        assert!(html.contains("STATS"), "Stats slot should render. Got: {html}");
        assert!(html.contains("ACTIONS"), "Actions slot should render. Got: {html}");
        // Slot containers.
        assert!(html.contains("wallet-card-identity"), "Should have identity container. Got: {html}");
        assert!(html.contains("wallet-card-stats"), "Should have stats container. Got: {html}");
        assert!(html.contains("wallet-card-actions"), "Should have actions container. Got: {html}");
    }

    /// The wallet address is exposed via `data-wallet-address`.
    #[test]
    fn test_wallet_card_data_address_attribute() {
        let el = rsx! {
            WalletCard {
                wallet: sample_card(),
                identity: "<div>Id</div>".to_string(),
                stats: "<div>St</div>".to_string(),
                actions: "<div>Ac</div>".to_string(),
            }
        };
        let html = dioxus_ssr::render_element(el);
        assert!(
            html.contains("data-wallet-address="),
            "Card should expose data-wallet-address. Got: {html}"
        );
        assert!(
            html.contains("data-wallet-card=\"true\""),
            "Card should expose data-wallet-card marker. Got: {html}"
        );
    }

    /// `class_name` is propagated.
    #[test]
    fn test_wallet_card_propagates_class_name() {
        let el = rsx! {
            WalletCard {
                wallet: sample_card(),
                identity: "<div>I</div>".to_string(),
                stats: "<div>S</div>".to_string(),
                actions: "<div>A</div>".to_string(),
                class_name: Some("mt-4 max-w-md".to_string()),
            }
        };
        let html = dioxus_ssr::render_element(el);
        assert!(
            html.contains("mt-4"),
            "class_name should propagate. Got: {html}"
        );
        assert!(
            html.contains("max-w-md"),
            "class_name should propagate. Got: {html}"
        );
    }
}
