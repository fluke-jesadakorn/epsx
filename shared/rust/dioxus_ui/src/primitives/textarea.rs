//! `Textarea` — shadcn new-york multiline input with variants.
//!
//! Mirrors `apps-old/frontend/components/ui/textarea.tsx`. The
//! Next.js version defines a `cva` with 7 variants × 5 sizes × 4
//! states (140+ unique combinations).
//!
//! The Dioxus version supports the same surface but with a simpler
//! prop API:
//! - `variant` — `"default" | "wp" | "pancake" | "ghost" | "tile" | "outlined" | "admin"`
//! - `size` — `"sm" | "default" | "lg" | "xl" | "tile"`
//! - `state` — `"default" | "error" | "success" | "warning"`
//!
//! Coexists with `form::Textarea` (a simpler form-field-aware
//! wrapper). Use this version when you need a shadcn-namespace
//! textarea with the full variant set.

use dioxus::prelude::*;

/// Visual variant — controls the border, background, and corner
/// radius tokens. Defaults to `"default"` (standard shadcn).
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub enum TextareaVariant {
    #[default]
    Default,
    /// Windows Phone–style secondary border + rounded-none.
    Wp,
    /// PancakeSwap–style with primary gradient.
    Pancake,
    /// Underline-only (no full border).
    Ghost,
    /// Live-tile: large card with primary/foreground gradient.
    Tile,
    /// Outlined with primary color, no background.
    Outlined,
    /// Admin form variant: gray-300/gray-600 border + rounded-lg.
    Admin,
}

impl TextareaVariant {
    /// Map a variant to its Tailwind class fragment. Returns
    /// `&'static str` so the value can be embedded directly in
    /// the rsx! literal.
    pub fn as_class(self) -> &'static str {
        match self {
            TextareaVariant::Default => "border-input bg-background",
            TextareaVariant::Wp => "border-2 border-secondary/30 bg-card text-foreground focus-visible:border-secondary focus-visible:ring-2 focus-visible:ring-secondary/20 hover:border-secondary/50 shadow-sm hover:shadow-md rounded-none",
            TextareaVariant::Pancake => "border-2 border-primary/30 bg-gradient-to-r from-background to-primary/5 text-foreground focus-visible:border-primary focus-visible:ring-2 focus-visible:ring-primary/20 hover:border-primary/50 shadow-lg hover:shadow-xl rounded-xl",
            TextareaVariant::Ghost => "border-0 border-b-2 border-muted-foreground/30 bg-transparent rounded-none text-foreground focus-visible:border-primary focus-visible:ring-0 hover:border-muted-foreground/50 px-0",
            TextareaVariant::Tile => "border-0 bg-gradient-to-br from-card to-muted rounded-xl text-foreground focus-visible:ring-2 focus-visible:ring-primary shadow-lg hover:shadow-xl",
            TextareaVariant::Outlined => "border-2 border-primary bg-transparent text-foreground focus-visible:bg-primary/5 focus-visible:border-primary focus-visible:ring-2 focus-visible:ring-primary/30 hover:border-primary/80",
            TextareaVariant::Admin => "border-2 border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-800 text-gray-900 dark:text-gray-100 focus-visible:border-primary focus-visible:ring-2 focus-visible:ring-primary/20 hover:border-gray-400 dark:hover:border-gray-500 rounded-lg",
        }
    }
}

/// Size token — controls the minimum height and padding.
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub enum TextareaSize {
    Sm,
    #[default]
    Default,
    Lg,
    Xl,
    /// Windows Phone live-tile size.
    Tile,
}

impl TextareaSize {
    pub fn as_class(self) -> &'static str {
        match self {
            TextareaSize::Sm => "min-h-[60px] px-3 py-2 text-sm rounded-md",
            TextareaSize::Default => "min-h-[80px] px-3 py-2",
            TextareaSize::Lg => "min-h-[120px] px-6 py-4 text-base rounded-xl",
            TextareaSize::Xl => "min-h-[160px] px-8 py-5 text-lg rounded-2xl",
            TextareaSize::Tile => "min-h-[140px] px-6 py-5 text-lg rounded-2xl",
        }
    }
}

/// Validation state — controls the border / focus-ring color.
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub enum TextareaState {
    #[default]
    Default,
    Error,
    Success,
    Warning,
}

impl TextareaState {
    pub fn as_class(self) -> &'static str {
        match self {
            TextareaState::Default => "",
            TextareaState::Error => "border-destructive focus-visible:border-destructive focus-visible:ring-destructive/20 text-destructive-foreground",
            TextareaState::Success => "border-success focus-visible:border-success focus-visible:ring-success/20",
            TextareaState::Warning => "border-warning focus-visible:border-warning focus-visible:ring-warning/20",
        }
    }
}

/// Shadcn-namespace textarea with full variant support.
///
/// - `name: Option<String>` — the form field name.
/// - `placeholder: Option<String>` — placeholder text.
/// - `value: Option<String>` — controlled value.
/// - `rows: Option<u32>` — explicit row count. Defaults to size-based
///   minimum height.
/// - `variant: Option<TextareaVariant>` — visual variant.
/// - `size: Option<TextareaSize>` — size token.
/// - `state: Option<TextareaState>` — validation state.
/// - `class: Option<String>` — extra Tailwind classes.
/// - `disabled: Option<bool>` — disable the field.
/// - `required: Option<bool>` — mark as required.
/// - `oninput: Option<EventHandler<FormEvent>>` — input handler.
/// - `onchange: Option<EventHandler<FormEvent>>` — change handler.
#[component]
pub fn Textarea(
    #[props(default = None)] name: Option<String>,
    #[props(default = None)] placeholder: Option<String>,
    #[props(default = None)] value: Option<String>,
    #[props(default = None)] rows: Option<u32>,
    #[props(default = TextareaVariant::default())] variant: TextareaVariant,
    #[props(default = TextareaSize::default())] size: TextareaSize,
    #[props(default = TextareaState::default())] state: TextareaState,
    #[props(default = None)] class: Option<String>,
    #[props(default = false)] disabled: bool,
    #[props(default = false)] required: bool,
    #[props(default = None)] oninput: Option<EventHandler<FormEvent>>,
    #[props(default = None)] onchange: Option<EventHandler<FormEvent>>,
) -> Element {
    let mut cls = "flex w-full resize-y rounded-md border text-sm ring-offset-background placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50".to_string();
    cls.push(' ');
    cls.push_str(variant.as_class());
    cls.push(' ');
    cls.push_str(size.as_class());
    let state_cls = state.as_class();
    if !state_cls.is_empty() {
        cls.push(' ');
        cls.push_str(state_cls);
    }
    if let Some(c) = class {
        cls.push(' ');
        cls.push_str(&c);
    }
    let value_str = value.as_deref().unwrap_or("");
    let placeholder_str = placeholder.as_deref().unwrap_or("");
    let rows_attr = rows.unwrap_or(0);
    rsx! {
        div { class: "textarea-wrap relative w-full",
            textarea {
                class: "{cls}",
                name: name.as_deref().unwrap_or(""),
                placeholder: "{placeholder_str}",
                value: "{value_str}",
                rows: "{rows_attr}",
                disabled: disabled,
                required: required,
                oninput: move |e| if let Some(h) = &oninput { h.call(e); },
                onchange: move |e| if let Some(h) = &onchange { h.call(e); },
            }
            // Windows Phone accent dot for pancake + tile variants.
            if variant == TextareaVariant::Pancake || variant == TextareaVariant::Tile {
                div { class: "absolute bottom-2 right-2 w-1 h-1 bg-primary/60 rounded-full pointer-events-none" }
            }
            // PancakeSwap corner accent.
            if variant == TextareaVariant::Pancake {
                div { class: "absolute top-0 right-0 w-3 h-3 bg-gradient-to-bl from-primary/30 to-transparent rounded-tr-lg pointer-events-none" }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_variant_class() {
        assert_eq!(TextareaVariant::Default.as_class(), "border-input bg-background");
    }

    #[test]
    fn wp_variant_class() {
        assert!(TextareaVariant::Wp.as_class().contains("border-secondary"));
        assert!(TextareaVariant::Wp.as_class().contains("rounded-none"));
    }

    #[test]
    fn pancake_variant_class() {
        assert!(TextareaVariant::Pancake.as_class().contains("bg-gradient-to-r"));
        assert!(TextareaVariant::Pancake.as_class().contains("rounded-xl"));
    }

    #[test]
    fn ghost_variant_has_no_full_border() {
        let cls = TextareaVariant::Ghost.as_class();
        assert!(cls.contains("border-0"));
        assert!(cls.contains("border-b-2"));
    }

    #[test]
    fn tile_variant_class() {
        assert!(TextareaVariant::Tile.as_class().contains("from-card"));
        assert!(TextareaVariant::Tile.as_class().contains("rounded-xl"));
    }

    #[test]
    fn outlined_variant_class() {
        assert!(TextareaVariant::Outlined.as_class().contains("border-primary"));
        assert!(TextareaVariant::Outlined.as_class().contains("bg-transparent"));
    }

    #[test]
    fn admin_variant_class() {
        let cls = TextareaVariant::Admin.as_class();
        assert!(cls.contains("border-gray-300"));
        assert!(cls.contains("dark:border-gray-600"));
        assert!(cls.contains("rounded-lg"));
    }

    #[test]
    fn size_classes_have_distinct_min_heights() {
        let sm_min = TextareaSize::Sm.as_class();
        let default_min = TextareaSize::Default.as_class();
        let lg_min = TextareaSize::Lg.as_class();
        let xl_min = TextareaSize::Xl.as_class();
        let tile_min = TextareaSize::Tile.as_class();
        // Each size maps to a different min-h value.
        assert!(sm_min.contains("min-h-[60px]"));
        assert!(default_min.contains("min-h-[80px]"));
        assert!(lg_min.contains("min-h-[120px]"));
        assert!(xl_min.contains("min-h-[160px]"));
        assert!(tile_min.contains("min-h-[140px]"));
    }

    #[test]
    fn state_default_class_is_empty() {
        assert_eq!(TextareaState::Default.as_class(), "");
    }

    #[test]
    fn state_error_class_has_destructive() {
        assert!(TextareaState::Error.as_class().contains("border-destructive"));
    }

    #[test]
    fn state_success_class_has_green() {
        assert!(TextareaState::Success.as_class().contains("border-success"));
    }

    #[test]
    fn state_warning_class_has_yellow() {
        assert!(TextareaState::Warning.as_class().contains("border-warning"));
    }

    #[test]
    fn all_variants_distinct() {
        let variants = [
            TextareaVariant::Default,
            TextareaVariant::Wp,
            TextareaVariant::Pancake,
            TextareaVariant::Ghost,
            TextareaVariant::Tile,
            TextareaVariant::Outlined,
            TextareaVariant::Admin,
        ];
        let classes: Vec<&str> = variants.iter().map(|v| v.as_class()).collect();
        let unique: std::collections::HashSet<&str> = classes.iter().copied().collect();
        assert_eq!(unique.len(), variants.len(), "variants must have distinct class strings");
    }
}
