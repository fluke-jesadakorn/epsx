//! `<MarketingBackground>` — the PancakeSwap-style gradient background
//! shared across the marketing pages.
//!
//! This is the Wave 5 Track A extraction. The pattern is seen in:
//!   - `apps-old/frontend/app/page.tsx` (the home page wrapper div:
//!     `bg-gradient-to-br from-blue-50 via-orange-50 to-yellow-50` +
//!     inline floating orbs)
//!   - `apps-old/frontend/app/about/page.tsx` (the explicit
//!     "PancakeSwap-style vibrant background" block — 4 floating orbs,
//!     3 radial mesh overlays, 2 geometric shapes)
//!   - `apps-old/frontend/app/contact/page.tsx` (same gradient + orbs)
//!   - `apps-old/frontend/app/plans/page.tsx` (same gradient)
//!   - `apps-old/frontend/app/auth/page.tsx` (animated 3-orb pulse
//!     background, which is a related but distinct variant — this
//!     component intentionally does NOT cover the auth page; the
//!     auth page keeps its own inline background because it has
//!     additional `animate-pulse` rules)
//!
//! Track B's pages (contact, plans) should `use
//! crate::layout::marketing_bg::MarketingBackground;` to consume this
//! rather than duplicating the markup.
//!
//! The component is a presentational shell: it renders the fixed
//! background and a relative-positioned `children` slot on top. No
//! state, no callbacks, no auth gating. Pages decide their own
//! `MainLayout` / `AuthLayout` wrapping — this component only paints
//! the background and a content layer.
//!
//! # Usage
//!
//! ```ignore
//! use crate::layout::marketing_bg::MarketingBackground;
//!
//! rsx! {
//!     MarketingBackground {
//!         section { class: "hero", ... }
//!         section { class: "trust-bar", ... }
//!     }
//! }
//! ```
//!
//! # Markers
//!
//! The component renders a root `div.marketing-bg` so a test or visual
//! scraper can confirm the background is in place. CSS for the orbs
//! lives in `shared/rust/templates/src/lib.rs` under the
//! `// === wave5-page-depth-track-a ===` region.

use dioxus::prelude::*;

/// Marketing-page background. Renders the PancakeSwap-style gradient
/// + 4 floating orbs + 3 radial mesh overlays + 2 geometric
/// decorations, with a `children` slot for the page content on top.
///
/// The outer wrapper is `position: relative` and `z-index: 1` so the
/// fixed background underneath (`z-index: 0`) does not bleed over
/// scrollable content. The background is rendered as `position:
/// fixed` so it stays put as the user scrolls the marketing content.
#[allow(non_snake_case)] // PascalCase is intentional — Dioxus component convention.
#[component]
pub fn MarketingBackground(children: Element) -> Element {
    rsx! {
        // Outer wrapper. The class `marketing-bg` is a section-marker
        // that the test and the CSS both reference. The page-content
        // class lets the marketing page styles target the children
        // slot directly (e.g. for max-width container sizing).
        div { class: "marketing-bg",
            // The fixed background — gradient + orbs + meshes + shapes.
            div { class: "marketing-bg-fixed", "aria-hidden": "true",
                // Layer 1: base gradient
                div { class: "marketing-bg-gradient" }
                // Layer 2: four floating orbs (orange, blue, purple, green)
                div { class: "marketing-orb marketing-orb-orange" }
                div { class: "marketing-orb marketing-orb-blue" }
                div { class: "marketing-orb marketing-orb-purple" }
                div { class: "marketing-orb marketing-orb-green" }
                // Layer 3: three radial mesh overlays
                div { class: "marketing-mesh marketing-mesh-orange" }
                div { class: "marketing-mesh marketing-mesh-blue" }
                div { class: "marketing-mesh marketing-mesh-purple" }
                // Layer 4: two decorative geometric shapes
                div { class: "marketing-shape marketing-shape-square" }
                div { class: "marketing-shape marketing-shape-circle" }
            }
            // Content layer — relative + z-index 1 so it sits above
            // the fixed background. Children paint into this slot.
            div { class: "marketing-bg-content",
                { children }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Smoke test — `MarketingBackground` accepts an `Element` child
    /// and renders it inside a wrapper that contains the
    /// `marketing-bg` section marker. This is the
    /// "test_section_markers" check the Wave 5 design doc requires
    /// for the new component, and a regression guard for the API
    /// signature `pub fn MarketingBackground(children: Element)`.
    #[test]
    fn marketing_background_renders_children() {
        let html: String = dioxus_ssr::render_element(rsx! {
            MarketingBackground {
                div { class: "test-child-marker", "test-child-string-12345" }
            }
        });
        // Section marker for the component itself.
        assert!(
            html.contains("marketing-bg"),
            "MarketingBackground must render a root element with class 'marketing-bg'. Got: {}",
            html
        );
        // The child string must appear verbatim — proves the children
        // are forwarded into the content slot, not dropped or
        // escaped incorrectly.
        assert!(
            html.contains("test-child-string-12345"),
            "MarketingBackground must forward its children into the content slot. Got: {}",
            html
        );
        // All four orb classes are present — guards the full visual
        // structure (orange/blue/purple/green) from being silently
        // stripped by a refactor.
        for orb in &["marketing-orb-orange", "marketing-orb-blue", "marketing-orb-purple", "marketing-orb-green"] {
            assert!(
                html.contains(orb),
                "MarketingBackground must render orb class '{}'. Got: {}",
                orb, html
            );
        }
    }
}
