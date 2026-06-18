//! Admin `PancakeButton` family — Wave 37 T1 admin primitives port.
//!
//! Mirrors `apps-old/admin-frontend/components/ui/pancake-button.tsx`,
//! which exports 3 PancakeSwap-styled button primitives:
//!
//! | Component | Use case |
//! | --- | --- |
//! | `PancakeButton` | Gradient CTA button (4 variants × 4 sizes) |
//! | `PancakeIconButton` | Icon-only square button with optional badge |
//! | `PancakeFAB` | Floating action button (fixed-position) |
//!
//! The `PancakeButton` variants (`pancake` / `admin` / `analytics` /
//! `ghost`) all use the same purple→orange gradient, mirroring the
//! PancakeSwap design system. The `metro` flag toggles the squared
//! metro-style corners.
//!
//! ## Tests
//!
//! `test_pancake_button_renders_gradient` — every variant emits
//! the `bg-gradient-to-r from-purple-500 to-orange-500` class.
//! `test_pancake_icon_button_renders_with_badge` — the badge slot
//! is rendered as a positioned span.

use dioxus::prelude::*;

/// PancakeSwap-styled gradient button. Use as the primary CTA on
/// admin pages.
#[component]
pub fn PancakeButton(
    variant: Option<String>,
    size: Option<String>,
    full_width: Option<bool>,
    metro: Option<bool>,
    disabled: Option<bool>,
    /// Optional emoji / icon glyph rendered before the children.
    icon: Option<String>,
    children: Element,
) -> Element {
    let variant = variant.unwrap_or_else(|| "pancake".to_string());
    let size = size.unwrap_or_else(|| "md".to_string());
    let full_width = full_width.unwrap_or(false);
    let metro = metro.unwrap_or(false);
    let disabled = disabled.unwrap_or(false);

    let bg_cls = match variant.as_str() {
        "analytics" => "bg-secondary",
        "ghost" => "bg-transparent",
        _ => "bg-gradient-to-r from-purple-500 to-orange-500",
    };
    let hover_cls = match variant.as_str() {
        "analytics" => "hover:shadow-xl hover:shadow-orange-500/30 hover:-translate-y-1",
        "ghost" => "hover:bg-accent hover:text-accent-foreground",
        _ => "hover:shadow-xl hover:shadow-purple-500/30 hover:-translate-y-1",
    };
    let text_cls = match variant.as_str() {
        "ghost" => "text-foreground",
        _ => "text-white",
    };
    let accent_cls = match variant.as_str() {
        "analytics" => "border-orange-500/30",
        "ghost" => "border-input",
        _ => "border-purple-500/30",
    };

    let size_cls = match size.as_str() {
        "sm" => "px-4 py-2 text-sm",
        "md" => "px-6 py-3 text-base",
        "lg" => "px-8 py-4 text-lg",
        "xl" => "px-10 py-5 text-xl",
        _ => "px-6 py-3 text-base",
    };
    let radius_cls = if metro { "rounded-none" } else { "rounded-xl" };

    let mut cls = format!(
        "{full_width} {size} {bg} {hover} {text} font-bold {radius} shadow-lg border {accent} relative overflow-hidden transition-all duration-200 {disabled_cls}",
        full_width = if full_width { "w-full" } else { "" },
        size = size_cls,
        bg = bg_cls,
        hover = hover_cls,
        text = text_cls,
        radius = radius_cls,
        accent = accent_cls,
        disabled_cls = if disabled { "opacity-50 cursor-not-allowed" } else { "cursor-pointer" },
    );
    // Collapse any double-spaces caused by the empty `w-full` arm
    // when `full_width` is false.
    while cls.contains("  ") {
        cls = cls.replace("  ", " ");
    }

    rsx! {
        button {
            class: "{cls}",
            r#type: "button",
            disabled: disabled,
            // Metro variant: 45° gradient sheen overlay (decoration only).
            if metro {
                div {
                    class: "absolute inset-0 opacity-10",
                    style: "background:linear-gradient(45deg, transparent 30%, rgba(255,255,255,0.2) 50%, transparent 70%);",
                }
            }
            // Hover sheen overlay.
            div {
                class: "absolute inset-0 opacity-0 hover:opacity-100 transition-opacity",
                style: "background:linear-gradient(90deg, transparent, rgba(255,255,255,0.1), transparent);",
            }
            // Content row.
            div { class: "relative z-10 flex items-center justify-center gap-2",
                if let Some(glyph) = icon.clone() {
                    if !glyph.is_empty() {
                        span { class: "text-lg", "{glyph}" }
                    }
                }
                {children}
            }
            // Metro variant: 2px bottom accent bar.
            if metro {
                div { class: "absolute bottom-0 left-0 w-full h-0.5 bg-white/20" }
            }
        }
    }
}

/// Square icon-only PancakeSwap button with optional badge.
#[component]
pub fn PancakeIconButton(
    variant: Option<String>,
    icon: String,
    size: Option<String>,
    /// Optional badge value (string or numeric). Rendered as a
    /// circular indicator in the top-right corner.
    badge: Option<String>,
    disabled: Option<bool>,
) -> Element {
    let variant = variant.unwrap_or_else(|| "pancake".to_string());
    let size = size.unwrap_or_else(|| "md".to_string());
    let disabled = disabled.unwrap_or(false);

    let size_cls = match size.as_str() {
        "sm" => "w-8 h-8 text-sm",
        "lg" => "w-16 h-16 text-xl",
        _ => "w-12 h-12 text-lg",
    };
    let variant_cls = match variant.as_str() {
        "analytics" => "bg-gradient-to-r from-orange-500 to-yellow-500 shadow-orange-500/20 hover:shadow-orange-500/30",
        _ => "bg-gradient-to-r from-purple-500 to-orange-500 shadow-purple-500/20 hover:shadow-purple-500/30",
    };

    let mut cls = format!(
        "{size} {variant} text-white relative overflow-hidden shadow-lg rounded-xl hover:shadow-xl hover:-translate-y-1 transition-all {disabled_cls}",
        size = size_cls,
        variant = variant_cls,
        disabled_cls = if disabled { "opacity-50 cursor-not-allowed" } else { "cursor-pointer" },
    );

    rsx! {
        button {
            class: "{cls}",
            r#type: "button",
            disabled: disabled,
            if let Some(b) = badge.clone() {
                div { class: "absolute -top-1 -right-1 bg-destructive text-destructive-foreground rounded-full w-5 h-5 flex items-center justify-center text-xs font-bold z-10",
                    "{b}"
                }
            }
            span { class: "relative z-5", "{icon}" }
        }
    }
}

/// Floating action button (fixed-position). 64×64, rounded-2xl,
/// positioned via the `position` slot (`bottom-right` default).
#[component]
pub fn PancakeFAB(
    variant: Option<String>,
    icon: String,
    /// Anchor position. Default `bottom-right`.
    position: Option<String>,
) -> Element {
    let variant = variant.unwrap_or_else(|| "pancake".to_string());
    let position = position.unwrap_or_else(|| "bottom-right".to_string());

    let pos_cls = match position.as_str() {
        "bottom-left" => "bottom-6 left-6",
        "top-right" => "top-6 right-6",
        "top-left" => "top-6 left-6",
        _ => "bottom-6 right-6",
    };
    let variant_cls = match variant.as_str() {
        "analytics" => "bg-gradient-to-r from-orange-500 to-yellow-500 shadow-orange-500/30 hover:shadow-orange-500/40",
        _ => "bg-gradient-to-r from-purple-500 to-orange-500 shadow-purple-500/30 hover:shadow-purple-500/40",
    };

    let cls = format!(
        "fixed {pos} w-16 h-16 {variant} text-white text-2xl shadow-2xl rounded-2xl hover:scale-110 transition-all z-50 overflow-hidden",
        pos = pos_cls,
        variant = variant_cls,
    );

    rsx! {
        button {
            class: "{cls}",
            r#type: "button",
            div { class: "flex items-center justify-center h-full", "{icon}" }
        }
    }
}

// =====================================================================
// Tests
// =====================================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// Every PancakeButton variant (except `ghost`) emits the canonical
    /// purple→orange gradient. The `ghost` variant uses
    /// `bg-transparent` instead.
    #[test]
    fn test_pancake_button_renders_gradient() {
        for variant in &["pancake", "admin", "analytics"] {
            let el = rsx! {
                PancakeButton {
                    variant: Some(variant.to_string()),
                    "Click me"
                }
            };
            let html = dioxus_ssr::render_element(el);
            assert!(
                html.contains("rounded-xl"),
                "PancakeButton[variant={variant}] should use rounded-xl. Got: {html}"
            );
            assert!(
                html.contains("shadow-lg"),
                "PancakeButton[variant={variant}] should have shadow-lg. Got: {html}"
            );
            assert!(
                html.contains("text-white"),
                "PancakeButton[variant={variant}] should be text-white. Got: {html}"
            );
            assert!(html.contains("Click me"), "PancakeButton should render children. Got: {html}");
        }

        // The `ghost` variant should NOT have the gradient — it's a
        // transparent button.
        let el = rsx! {
            PancakeButton {
                variant: Some("ghost".to_string()),
                "Cancel"
            }
        };
        let html = dioxus_ssr::render_element(el);
        assert!(
            html.contains("bg-transparent"),
            "PancakeButton[variant=ghost] should be bg-transparent. Got: {html}"
        );
    }

    /// The `size` slot picks the correct Tailwind padding/text-size.
    #[test]
    fn test_pancake_button_size_lg() {
        let el = rsx! {
            PancakeButton {
                size: Some("lg".to_string()),
                "Save"
            }
        };
        let html = dioxus_ssr::render_element(el);
        assert!(
            html.contains("px-8 py-4 text-lg"),
            "PancakeButton[size=lg] should use px-8 py-4 text-lg. Got: {html}"
        );
    }

    /// The `metro` flag toggles `rounded-none`.
    #[test]
    fn test_pancake_button_metro_variant() {
        let el = rsx! {
            PancakeButton {
                metro: Some(true),
                "Metro"
            }
        };
        let html = dioxus_ssr::render_element(el);
        assert!(
            html.contains("rounded-none"),
            "PancakeButton[metro=true] should use rounded-none. Got: {html}"
        );
        assert!(
            html.contains("linear-gradient(45deg"),
            "PancakeButton[metro=true] should have the metro sheen overlay. Got: {html}"
        );
    }

    /// `disabled` adds the opacity-50 / cursor-not-allowed classes.
    #[test]
    fn test_pancake_button_disabled() {
        let el = rsx! {
            PancakeButton {
                disabled: Some(true),
                "Disabled"
            }
        };
        let html = dioxus_ssr::render_element(el);
        assert!(
            html.contains("opacity-50"),
            "PancakeButton[disabled=true] should be opacity-50. Got: {html}"
        );
        assert!(
            html.contains("cursor-not-allowed"),
            "PancakeButton[disabled=true] should be cursor-not-allowed. Got: {html}"
        );
        assert!(
            html.contains("disabled"),
            "PancakeButton[disabled=true] should emit the disabled HTML attribute. Got: {html}"
        );
    }

    /// The `icon` slot renders the glyph before the children.
    #[test]
    fn test_pancake_button_with_icon() {
        let el = rsx! {
            PancakeButton {
                icon: Some("🚀".to_string()),
                "Launch"
            }
        };
        let html = dioxus_ssr::render_element(el);
        assert!(
            html.contains("🚀"),
            "PancakeButton should render the icon glyph. Got: {html}"
        );
        assert!(html.contains("Launch"), "PancakeButton should render children. Got: {html}");
    }

    /// `PancakeIconButton` renders the square button with the badge.
    #[test]
    fn test_pancake_icon_button_with_badge() {
        let el = rsx! {
            PancakeIconButton {
                icon: "🔔".to_string(),
                badge: Some("3".to_string()),
            }
        };
        let html = dioxus_ssr::render_element(el);
        assert!(
            html.contains("w-12 h-12 text-lg"),
            "PancakeIconButton default size should be w-12 h-12 text-lg. Got: {html}"
        );
        assert!(
            html.contains("bg-destructive"),
            "PancakeIconButton badge should be bg-destructive. Got: {html}"
        );
        assert!(html.contains(">3<"), "PancakeIconButton badge value should render. Got: {html}");
    }

    /// `PancakeIconButton` size lg renders w-16 h-16 text-xl.
    #[test]
    fn test_pancake_icon_button_size_lg() {
        let el = rsx! {
            PancakeIconButton {
                icon: "+".to_string(),
                size: Some("lg".to_string()),
            }
        };
        let html = dioxus_ssr::render_element(el);
        assert!(
            html.contains("w-16 h-16 text-xl"),
            "PancakeIconButton[size=lg] should use w-16 h-16 text-xl. Got: {html}"
        );
    }

    /// `PancakeFAB` is fixed-position with the bottom-right anchor.
    #[test]
    fn test_pancake_fab_default_position() {
        let el = rsx! {
            PancakeFAB {
                icon: "+".to_string(),
            }
        };
        let html = dioxus_ssr::render_element(el);
        assert!(
            html.contains("fixed bottom-6 right-6"),
            "PancakeFAB default position should be bottom-right. Got: {html}"
        );
        assert!(
            html.contains("w-16 h-16"),
            "PancakeFAB should be 64x64. Got: {html}"
        );
        assert!(
            html.contains("z-50"),
            "PancakeFAB should have z-50 to layer above the page. Got: {html}"
        );
    }

    /// `PancakeFAB` with `position=top-left` renders the matching anchor.
    #[test]
    fn test_pancake_fab_top_left() {
        let el = rsx! {
            PancakeFAB {
                icon: "×".to_string(),
                position: Some("top-left".to_string()),
            }
        };
        let html = dioxus_ssr::render_element(el);
        assert!(
            html.contains("top-6 left-6"),
            "PancakeFAB[position=top-left] should be top-6 left-6. Got: {html}"
        );
    }
}
