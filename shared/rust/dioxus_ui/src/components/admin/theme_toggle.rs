//! Admin theme-toggle component — Wave 37 T1 admin primitives port.
//!
//! Mirrors `apps-old/admin-frontend/components/ui/theme-toggle.tsx`,
//! which is a re-export from `@/shared/components/ui/theme-toggle`.
//! The shared module exposes 9 variants (`AdminThemeToggle`,
//! `AnimatedThemeToggle`, `GradientThemeToggle`, `MinimalThemeToggle`,
//! `OptimizedThemeToggle`, `SimpleThemeToggle`, `ThemeToggle`,
//! `ThemeToggleCSS`, `UnifiedThemeToggle`).
//!
//! ## Source-of-truth mapping
//!
//! | apps-old `theme-toggle.tsx` | Dioxus |
//! | --- | --- |
//! | `UnifiedThemeToggle` (canonical hydration-less button) | `crate::theme::UnifiedThemeToggle` (re-exported) |
//! | `AdminThemeToggle` (admin-side default)             | `AdminThemeToggle` (thin re-export of the canonical one) |
//! | `SimpleThemeToggle` / `GradientThemeToggle` / etc.   | `SimpleThemeToggle` / `GradientThemeToggle` (thin re-exports) |
//!
//! The Dioxus port delegates to the existing `epsx_dioxus_ui::theme`
//! module, which already has the SSR-friendly inline-onclick
//! pattern (`epsx.toggleTheme()`). The re-exports here exist so
//! admin pages can do `use crate::components::admin::AdminThemeToggle`
//! without reaching into the generic `theme` module.
//!
//! ## Tests
//!
//! `test_admin_theme_toggle_renders_button` — the re-exported
//! `UnifiedThemeToggle` renders a `<button>` with the
//! `theme-toggle` class.

pub use crate::theme::{
    UnifiedThemeToggle as AdminThemeToggle, UnifiedThemeToggle as ThemeToggle,
    UnifiedThemeToggle as AnimatedThemeToggle, UnifiedThemeToggle as GradientThemeToggle,
    UnifiedThemeToggle as MinimalThemeToggle, UnifiedThemeToggle as OptimizedThemeToggle,
    UnifiedThemeToggle as SimpleThemeToggle,
};

/// CSS-only theme toggle (no JS hydration required). For the Dioxus
/// port this is the same as `UnifiedThemeToggle` because Dioxus
/// SSR is hydration-less — the toggle's click handler is already
/// attached via the inline `onclick="epsx.toggleTheme()"` attribute.
pub use crate::theme::UnifiedThemeToggle as ThemeToggleCSS;

// =====================================================================
// Tests
// =====================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use dioxus::prelude::*;

    /// `AdminThemeToggle` is a re-export of `UnifiedThemeToggle`. The
    /// rendered HTML must contain the canonical `theme-toggle` class
    /// and the inline `onclick="epsx.toggleTheme()"` hydration hook.
    #[test]
    fn test_admin_theme_toggle_renders_button() {
        let el = rsx! { AdminThemeToggle {} };
        let html = dioxus_ssr::render_element(el);
        assert!(
            html.contains("theme-toggle"),
            "AdminThemeToggle should expose the theme-toggle class. Got: {html}"
        );
        assert!(
            html.contains("epsx.toggleTheme"),
            "AdminThemeToggle should emit the epsx.toggleTheme hydration hook. Got: {html}"
        );
    }

    /// `ThemeToggle` alias resolves to the same component.
    #[test]
    fn test_theme_toggle_alias() {
        let el = rsx! { ThemeToggle {} };
        let html = dioxus_ssr::render_element(el);
        assert!(html.contains("theme-toggle"), "ThemeToggle alias should render the canonical toggle");
    }
}
