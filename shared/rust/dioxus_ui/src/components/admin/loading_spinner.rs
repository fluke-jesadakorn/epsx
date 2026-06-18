//! Admin loading-spinner component — Wave 37 T1 admin primitives port.
//!
//! Mirrors `apps-old/admin-frontend/components/ui/loading-spinner.tsx`,
//! which exports 5 spinner variants:
//!
//! | Component | Use case |
//! | --- | --- |
//! | `LoadingSpinner` | Inline indicator (button, small section) |
//! | `PageLoadingSpinner` | Full-page centered spinner + label |
//! | `ButtonLoadingSpinner` | Inline spinner inside buttons |
//! | `SectionLoading` | Section-level centered spinner + label |
//! | `InlineLoading` | Inline indicator for lists, counts, etc. |
//!
//! In Dioxus we render a CSS-only spinner (no `lucide-react`'s
//! `Loader2` runtime — instead, a Tailwind `animate-spin` div with
//! the canonical `lucide-loader-circle` SVG path baked in).
//!
//! ## Tests
//!
//! `test_loading_spinner_renders_animate_spin` — every variant
//! emits the `animate-spin` class.
//! `test_loading_spinner_propagates_label` — the `label` slot is
//! rendered next to the spinner.

use dioxus::prelude::*;

/// Inline loading indicator. The `size` and `variant` slots match
/// the prod Tailwind sizes / colors used across the admin dashboard.
#[component]
pub fn LoadingSpinner(
    size: Option<String>,
    label: Option<String>,
    class_name: Option<String>,
    label_position: Option<String>,
    variant: Option<String>,
) -> Element {
    let size = size.unwrap_or_else(|| "md".to_string());
    let label_position = label_position.unwrap_or_else(|| "inline".to_string());
    let variant = variant.unwrap_or_else(|| "default".to_string());

    let size_cls = match size.as_str() {
        "xs" => "h-3 w-3",
        "sm" => "h-4 w-4",
        "md" => "h-5 w-5",
        "lg" => "h-6 w-6",
        "xl" => "h-8 w-8",
        _ => "h-5 w-5",
    };
    let label_size_cls = match size.as_str() {
        "xs" => "text-xs",
        "sm" => "text-sm",
        "md" => "text-sm",
        "lg" => "text-base",
        "xl" => "text-lg",
        _ => "text-sm",
    };
    let variant_cls = match variant.as_str() {
        "primary" => "text-blue-600 dark:text-blue-400",
        "muted" => "text-muted-foreground",
        "white" => "text-white",
        _ => "text-muted-foreground",
    };

    let mut container_cls = "flex items-center gap-2".to_string();
    if label_position == "below" {
        container_cls.push_str(" flex-col gap-1");
    }
    if let Some(c) = class_name.clone() {
        container_cls.push(' ');
        container_cls.push_str(&c);
    }

    rsx! {
        div {
            class: "{container_cls}",
            role: "status",
            "aria-live": "polite",
            // Loader2-style spinning SVG (matches the prod
            // `lucide-react` Loader2 path data).
            svg {
                class: "animate-spin {size_cls} {variant_cls}",
                xmlns: "http://www.w3.org/2000/svg",
                width: "24",
                height: "24",
                view_box: "0 0 24 24",
                fill: "none",
                stroke: "currentColor",
                stroke_width: "2",
                stroke_linecap: "round",
                stroke_linejoin: "round",
                path { d: "M21 12a9 9 0 1 1-6.219-8.56" }
            }
            match label.clone() {
                Some(l) if !l.is_empty() => rsx! {
                    span { class: "{label_size_cls} {variant_cls}", "{l}" }
                },
                _ => rsx! {
                    span { class: "sr-only", "Loading..." }
                },
            }
        }
    }
}

/// Full-page centered loading spinner with a default
/// `Loading...` label. Use at the top of an admin page when the
/// page body is gated on a backend call.
#[component]
pub fn PageLoadingSpinner(
    label: Option<String>,
    class_name: Option<String>,
) -> Element {
    let label = label.unwrap_or_else(|| "Loading...".to_string());
    let mut cls = "min-h-[400px] flex flex-col items-center justify-center gap-4".to_string();
    if let Some(c) = class_name {
        cls.push(' ');
        cls.push_str(&c);
    }
    rsx! {
        div { class: "{cls}",
            LoadingSpinner {
                size: Some("xl".to_string()),
                variant: Some("primary".to_string()),
            }
            p { class: "text-muted-foreground text-sm", "{label}" }
        }
    }
}

/// Compact spinner sized to fit inside a button. Returns just the
/// SVG without the wrapping div so the caller can place it inside
/// a `<button>` next to the button label.
#[component]
pub fn ButtonLoadingSpinner(class_name: Option<String>) -> Element {
    let mut cls = "h-4 w-4 animate-spin".to_string();
    if let Some(c) = class_name {
        cls.push(' ');
        cls.push_str(&c);
    }
    rsx! {
        svg {
            class: "{cls}",
            xmlns: "http://www.w3.org/2000/svg",
            width: "24",
            height: "24",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            path { d: "M21 12a9 9 0 1 1-6.219-8.56" }
        }
    }
}

/// Section-level loading state. Centered spinner with a gradient
/// glow underneath and a default `Loading...` label.
#[component]
pub fn SectionLoading(
    label: Option<String>,
    class_name: Option<String>,
) -> Element {
    let label = label.unwrap_or_else(|| "Loading...".to_string());
    let mut cls = "flex flex-col items-center justify-center py-12 gap-3".to_string();
    if let Some(c) = class_name {
        cls.push(' ');
        cls.push_str(&c);
    }
    rsx! {
        div { class: "{cls}",
            div { class: "relative",
                div { class: "absolute inset-0 bg-gradient-to-r from-blue-500 to-purple-500 rounded-full blur opacity-20" }
                LoadingSpinner {
                    size: Some("lg".to_string()),
                    variant: Some("primary".to_string()),
                }
            }
            p { class: "text-muted-foreground text-sm", "{label}" }
        }
    }
}

/// Tiny inline loading indicator for lists, counts, etc. Smaller
/// than `LoadingSpinner` (h-3 instead of h-5).
#[component]
pub fn InlineLoading(class_name: Option<String>) -> Element {
    let mut cls = "inline-flex items-center gap-1.5".to_string();
    if let Some(c) = class_name {
        cls.push(' ');
        cls.push_str(&c);
    }
    rsx! {
        span { class: "{cls}",
            ButtonLoadingSpinner {
                class_name: Some("h-3 w-3".to_string()),
            }
            span { class: "text-muted-foreground text-xs", "Loading..." }
        }
    }
}

// =====================================================================
// Tests
// =====================================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// Every LoadingSpinner variant emits the `animate-spin` class so
    /// the Tailwind keyframe fires.
    #[test]
    fn test_loading_spinner_renders_animate_spin() {
        let el = rsx! { LoadingSpinner {} };
        let html = dioxus_ssr::render_element(el);
        assert!(
            html.contains("animate-spin"),
            "LoadingSpinner should emit `animate-spin`. Got: {html}"
        );
        assert!(
            html.contains("role=\"status\""),
            "LoadingSpinner should expose role=status for accessibility. Got: {html}"
        );
    }

    /// The `label` slot is rendered next to the spinner.
    #[test]
    fn test_loading_spinner_propagates_label() {
        let el = rsx! {
            LoadingSpinner {
                label: Some("Loading policies...".to_string()),
            }
        };
        let html = dioxus_ssr::render_element(el);
        assert!(
            html.contains("Loading policies..."),
            "LoadingSpinner should render the label. Got: {html}"
        );
    }

    /// `size="xl"` picks the h-8 w-8 Tailwind class.
    #[test]
    fn test_loading_spinner_size_xl() {
        let el = rsx! { LoadingSpinner { size: Some("xl".to_string()) } };
        let html = dioxus_ssr::render_element(el);
        assert!(
            html.contains("h-8 w-8"),
            "LoadingSpinner size=xl should use h-8 w-8. Got: {html}"
        );
    }

    /// `PageLoadingSpinner` renders the centered container with the
    /// default `Loading...` label.
    #[test]
    fn test_page_loading_spinner_renders_label() {
        let el = rsx! { PageLoadingSpinner {} };
        let html = dioxus_ssr::render_element(el);
        assert!(
            html.contains("min-h-[400px]"),
            "PageLoadingSpinner should be at least 400px tall. Got: {html}"
        );
        assert!(
            html.contains("Loading..."),
            "PageLoadingSpinner default label should be `Loading...`. Got: {html}"
        );
        assert!(html.contains("animate-spin"), "PageLoadingSpinner should animate. Got: {html}");
    }

    /// `PageLoadingSpinner` with a custom label.
    #[test]
    fn test_page_loading_spinner_custom_label() {
        let el = rsx! {
            PageLoadingSpinner {
                label: Some("Fetching audit log...".to_string()),
            }
        };
        let html = dioxus_ssr::render_element(el);
        assert!(
            html.contains("Fetching audit log..."),
            "PageLoadingSpinner should render custom label. Got: {html}"
        );
    }

    /// `ButtonLoadingSpinner` returns a bare SVG (no wrapper div)
    /// so it fits inline inside a `<button>`.
    #[test]
    fn test_button_loading_spinner_is_bare_svg() {
        let el = rsx! { ButtonLoadingSpinner {} };
        let html = dioxus_ssr::render_element(el);
        assert!(html.contains("<svg"), "ButtonLoadingSpinner should render an SVG");
        assert!(html.contains("animate-spin"), "ButtonLoadingSpinner should animate");
        assert!(
            !html.contains("role=\"status\""),
            "ButtonLoadingSpinner should NOT have role=status (lives inside a button). Got: {html}"
        );
    }

    /// `SectionLoading` renders the gradient glow + spinner + label.
    #[test]
    fn test_section_loading_renders_gradient_glow() {
        let el = rsx! { SectionLoading {} };
        let html = dioxus_ssr::render_element(el);
        assert!(
            html.contains("bg-gradient-to-r from-blue-500 to-purple-500"),
            "SectionLoading should render the gradient glow. Got: {html}"
        );
        assert!(
            html.contains("py-12"),
            "SectionLoading should pad vertically. Got: {html}"
        );
    }

    /// `InlineLoading` is a span-wrapped spinner + "Loading..." label.
    #[test]
    fn test_inline_loading_renders_compact() {
        let el = rsx! { InlineLoading {} };
        let html = dioxus_ssr::render_element(el);
        assert!(
            html.contains("inline-flex"),
            "InlineLoading should use inline-flex. Got: {html}"
        );
        assert!(html.contains("Loading..."), "InlineLoading label. Got: {html}");
    }
}
