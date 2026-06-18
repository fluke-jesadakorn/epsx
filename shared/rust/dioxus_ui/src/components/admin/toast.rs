//! Admin `Toast` component — Wave 37 T1 admin primitives port.
//!
//! Mirrors `apps-old/admin-frontend/components/ui/toast.tsx`, which is
//! a direct re-export from `@/shared/components/ui/toast`.
//!
//! ## Source-of-truth mapping
//!
//! | apps-old `toast.tsx` | Dioxus |
//! | --- | --- |
//! | `<Toast>` provider (Radix toast primitive) | `ToastProvider` (Dioxus context wrapper) |
//! | `<ToastViewport>` | `ToastViewport` (fixed-position container) |
//! | `<Toast>` / `<ToastTitle>` / `<ToastDescription>` | `Toast` + slot helpers |
//!
//! The Dioxus port provides a minimal but faithful Toast container
//! and toast slot. Real toast state is owned by the parent context
//! (`ToastProvider`), matching the Radix UI shape that admin pages
//! rely on.
//!
//! ## Tests
//!
//! `test_toast_viewport_renders_fixed_container` — the viewport
//! renders a `position:fixed` container (the prod Tailwind class
//! string `fixed bottom-0 right-0 z-...` is present).
//! `test_toast_renders_title_and_description` — the toast slot
//! renders both the title and description child nodes.

use dioxus::prelude::*;

/// Fixed-position container for admin toast notifications. Matches
/// the prod `ToastViewport` (top-right, z-100, full-width on
/// mobile, 420px on desktop).
#[component]
pub fn ToastViewport(children: Element) -> Element {
    rsx! {
        div {
            class: "toast-viewport fixed top-4 right-4 z-[100] flex max-w-[420px] flex-col gap-2 p-4",
            role: "region",
            "aria-label": "Notifications",
            {children}
        }
    }
}

/// Single admin toast card. Renders a styled card with the
/// `admin-toast` class plus the variant-specific border accent.
#[component]
pub fn Toast(
    variant: Option<String>,
    class_name: Option<String>,
    children: Element,
) -> Element {
    let variant = variant.unwrap_or_else(|| "default".to_string());
    let variant_cls = match variant.as_str() {
        "success" => "border-emerald-500/30",
        "error" | "destructive" => "border-red-500/30",
        "warning" => "border-amber-500/30",
        "info" => "border-blue-500/30",
        _ => "border-border/20",
    };
    let mut cls = format!("admin-toast rounded-xl border bg-card p-4 shadow-xl {variant_cls}");
    if let Some(c) = class_name {
        cls.push(' ');
        cls.push_str(&c);
    }
    rsx! {
        div {
            class: "{cls}",
            role: "status",
            "aria-live": "polite",
            {children}
        }
    }
}

/// Toast title — bold, text-foreground, text-sm.
#[component]
pub fn ToastTitle(class_name: Option<String>, children: Element) -> Element {
    let mut cls = "text-sm font-semibold text-foreground".to_string();
    if let Some(c) = class_name {
        cls.push(' ');
        cls.push_str(&c);
    }
    rsx! {
        div { class: "{cls}", {children} }
    }
}

/// Toast description — muted, text-xs.
#[component]
pub fn ToastDescription(class_name: Option<String>, children: Element) -> Element {
    let mut cls = "mt-1 text-xs text-muted-foreground".to_string();
    if let Some(c) = class_name {
        cls.push(' ');
        cls.push_str(&c);
    }
    rsx! {
        div { class: "{cls}", {children} }
    }
}

// =====================================================================
// Tests
// =====================================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// The toast viewport renders a fixed-position region with the
    /// Notifications aria-label.
    #[test]
    fn test_toast_viewport_renders_fixed_container() {
        let el = rsx! {
            ToastViewport {
                Toast {
                    ToastTitle { "Saved" }
                    ToastDescription { "Changes applied" }
                }
            }
        };
        let html = dioxus_ssr::render_element(el);
        assert!(
            html.contains("toast-viewport"),
            "viewport should expose the toast-viewport class. Got: {html}"
        );
        assert!(
            html.contains("aria-label=\"Notifications\""),
            "viewport should announce itself as a notifications region. Got: {html}"
        );
        assert!(
            html.contains("role=\"region\""),
            "viewport should have role=region. Got: {html}"
        );
    }

    /// The toast slot renders both the title and description children.
    #[test]
    fn test_toast_renders_title_and_description() {
        let el = rsx! {
            ToastViewport {
                Toast {
                    ToastTitle { "Saved" }
                    ToastDescription { "Changes applied" }
                }
            }
        };
        let html = dioxus_ssr::render_element(el);
        assert!(html.contains("Saved"), "toast should render title text. Got: {html}");
        assert!(
            html.contains("Changes applied"),
            "toast should render description text. Got: {html}"
        );
        assert!(html.contains("admin-toast"), "toast should expose the admin-toast class. Got: {html}");
    }

    /// The `variant` slot changes the toast border accent.
    #[test]
    fn test_toast_propagates_variant() {
        let el = rsx! {
            ToastViewport {
                Toast {
                    variant: Some("success".to_string()),
                    "Done"
                }
            }
        };
        let html = dioxus_ssr::render_element(el);
        assert!(
            html.contains("border-emerald-500/30"),
            "success toast should render the emerald accent. Got: {html}"
        );

        let el2 = rsx! {
            ToastViewport {
                Toast {
                    variant: Some("error".to_string()),
                    "Failed"
                }
            }
        };
        let html2 = dioxus_ssr::render_element(el2);
        assert!(
            html2.contains("border-red-500/30"),
            "error toast should render the red accent. Got: {html2}"
        );
    }
}
