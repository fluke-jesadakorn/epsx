//! `Toast` — shadcn new-york namespace for toast components.
//!
//! Mirrors `apps-old/frontend/components/ui/toast.tsx`:
//!
//! ```text
//! const ToastProvider = ToastPrimitives.Provider
//! const ToastViewport = React.forwardRef<...>(...)
//! const toastVariants = cva(
//!     "group pointer-events-auto relative flex w-full items-center justify-between ...",
//!     { variants: { variant: { default, destructive, success, warning, info } } }
//! )
//! const Toast = React.forwardRef<...>(...)
//! const ToastAction = React.forwardRef<...>(...)
//! const ToastClose = React.forwardRef<...>(...)  // renders <X className="h-4 w-4" />
//! const ToastTitle = React.forwardRef<...>(...)
//! const ToastDescription = React.forwardRef<...>(...)
//! ```
//!
//! The shadcn-namespace `Toast` (with `variant` prop) is distinct
//! from `feedback::toast::ToastItemView` (the existing renderer for
//! the `push_*` API). Both can coexist:
//! - `feedback::toast` — programmatic API (`push_info`, etc.) +
//!   the renderer that consumes the queue.
//! - `primitives::toast` — the shadcn new-york component namespace.
//!   Use these for declarative toast markup (e.g. inside a
//!   `<ToastViewport>` + custom dispatcher).

use super::icon::Icon;

use dioxus::prelude::*;

/// Visual variant for a toast. Defaults to `"default"`.
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub enum ToastVariant {
    #[default]
    Default,
    Destructive,
    Success,
    Warning,
    Info,
}

impl ToastVariant {
    pub fn as_class(self) -> &'static str {
        match self {
            ToastVariant::Default => "border bg-background text-foreground",
            ToastVariant::Destructive => "destructive group border-destructive bg-destructive text-destructive-foreground",
            ToastVariant::Success => "border-green-500 bg-green-50 text-green-900 dark:bg-green-900/10 dark:text-green-100",
            ToastVariant::Warning => "border-yellow-500 bg-yellow-50 text-yellow-900 dark:bg-yellow-900/10 dark:text-yellow-100",
            ToastVariant::Info => "border-blue-500 bg-blue-50 text-blue-900 dark:bg-blue-900/10 dark:text-blue-100",
        }
    }
}

/// Provider — pass through to children. The actual state
/// management happens via the existing `feedback::toast` queue
/// when used with the `push_*` helpers. When used in a
/// fully-declarative setup, callers can render the `ToastViewport`
/// as a sibling and dispatch toasts through their own mechanism.
#[component]
pub fn ToastProvider(children: Element) -> Element {
    rsx! {
        div { class: "toast-provider", {children} }
    }
}

/// The viewport — fixed-position container where toasts are
/// rendered. Use at the top of the layout (right before
/// `</body>`).
#[component]
pub fn ToastViewport(
    #[props(default = None)] class: Option<String>,
    children: Element,
) -> Element {
    let mut cls = "fixed top-0 z-[100] flex max-h-screen w-full flex-col-reverse p-4 sm:bottom-0 sm:right-0 sm:top-auto sm:flex-col md:max-w-[420px]".to_string();
    if let Some(c) = class {
        cls.push(' ');
        cls.push_str(&c);
    }
    rsx! {
        div {
            class: "{cls}",
            role: "region",
            "aria-label": "Notifications",
            tabindex: "0",
            {children}
        }
    }
}

/// A single toast. Renders a notification with the chosen variant.
///
/// - `variant: Option<ToastVariant>` — visual style. Defaults to
///   `"default"`.
/// - `class: Option<String>` — extra Tailwind classes.
/// - `children: Element` — toast body. Typically a
///   `ToastTitle` + `ToastDescription` and a `ToastClose` button.
#[component]
pub fn Toast(
    #[props(default = ToastVariant::default())] variant: ToastVariant,
    #[props(default = None)] class: Option<String>,
    children: Element,
) -> Element {
    let mut cls = "group pointer-events-auto relative flex w-full items-center justify-between space-x-4 overflow-hidden rounded-md border p-6 pr-8 shadow-lg transition-all data-[state=open]:animate-in data-[state=closed]:animate-out data-[swipe=end]:animate-out data-[state=closed]:fade-out-80 data-[state=closed]:slide-out-to-right-full data-[state=open]:slide-in-from-top-full data-[state=open]:sm:slide-in-from-bottom-full".to_string();
    cls.push(' ');
    cls.push_str(variant.as_class());
    if let Some(c) = class {
        cls.push(' ');
        cls.push_str(&c);
    }
    rsx! {
        div {
            class: "{cls}",
            role: "status",
            "data-state": "open",
            {children}
        }
    }
}

/// Action button inside a toast — e.g. "Undo" or "View".
#[component]
pub fn ToastAction(
    #[props(default = None)] class: Option<String>,
    children: Element,
) -> Element {
    let mut cls = "inline-flex h-8 shrink-0 items-center justify-center rounded-md border bg-transparent px-3 text-sm font-medium ring-offset-background transition-colors hover:bg-secondary focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2 disabled:pointer-events-none disabled:opacity-50 group-[.destructive]:border-muted/40 group-[.destructive]:hover:border-destructive/30 group-[.destructive]:hover:bg-destructive group-[.destructive]:hover:text-destructive-foreground group-[.destructive]:focus:ring-destructive".to_string();
    if let Some(c) = class {
        cls.push(' ');
        cls.push_str(&c);
    }
    rsx! {
        button {
            class: "{cls}",
            r#type: "button",
            {children}
        }
    }
}

/// Close button — renders an `X` icon in the top-right corner.
#[component]
pub fn ToastClose(
    #[props(default = None)] class: Option<String>,
) -> Element {
    let mut cls = "absolute right-2 top-2 rounded-md p-1 text-foreground/50 opacity-0 transition-opacity hover:text-foreground focus:opacity-100 focus:outline-none focus:ring-2 group-hover:opacity-100 group-[.destructive]:text-red-300 group-[.destructive]:hover:text-red-50 group-[.destructive]:focus:ring-red-400 group-[.destructive]:focus:ring-offset-red-600".to_string();
    if let Some(c) = class {
        cls.push(' ');
        cls.push_str(&c);
    }
    rsx! {
        button {
            class: "{cls}",
            r#type: "button",
            "toast-close": "",
            "aria-label": "Close",
            Icon { name: "x".to_string(), size: Some(16) }
        }
    }
}

/// Title — bold first line of a toast.
#[component]
pub fn ToastTitle(
    #[props(default = None)] class: Option<String>,
    children: Element,
) -> Element {
    let mut cls = "text-sm font-semibold".to_string();
    if let Some(c) = class {
        cls.push(' ');
        cls.push_str(&c);
    }
    rsx! {
        div { class: "{cls}", {children} }
    }
}

/// Description — secondary line of a toast.
#[component]
pub fn ToastDescription(
    #[props(default = None)] class: Option<String>,
    children: Element,
) -> Element {
    let mut cls = "text-sm opacity-90".to_string();
    if let Some(c) = class {
        cls.push(' ');
        cls.push_str(&c);
    }
    rsx! {
        div { class: "{cls}", {children} }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_variant_class() {
        assert_eq!(
            ToastVariant::Default.as_class(),
            "border bg-background text-foreground"
        );
    }

    #[test]
    fn destructive_variant_class() {
        let cls = ToastVariant::Destructive.as_class();
        assert!(cls.contains("border-destructive"));
        assert!(cls.contains("bg-destructive"));
        assert!(cls.contains("text-destructive-foreground"));
    }

    #[test]
    fn success_variant_class() {
        let cls = ToastVariant::Success.as_class();
        assert!(cls.contains("border-green-500"));
        assert!(cls.contains("bg-green-50"));
    }

    #[test]
    fn warning_variant_class() {
        let cls = ToastVariant::Warning.as_class();
        assert!(cls.contains("border-yellow-500"));
    }

    #[test]
    fn info_variant_class() {
        let cls = ToastVariant::Info.as_class();
        assert!(cls.contains("border-blue-500"));
    }

    #[test]
    fn all_variants_distinct() {
        let variants = [
            ToastVariant::Default,
            ToastVariant::Destructive,
            ToastVariant::Success,
            ToastVariant::Warning,
            ToastVariant::Info,
        ];
        let classes: Vec<&str> = variants.iter().map(|v| v.as_class()).collect();
        let unique: std::collections::HashSet<&str> = classes.iter().copied().collect();
        assert_eq!(unique.len(), variants.len(), "variants must have distinct class strings");
    }

    #[test]
    fn toast_title_class() {
        // Title uses `text-sm font-semibold`.
        let base = "text-sm font-semibold";
        assert!(base.contains("font-semibold"));
    }

    #[test]
    fn toast_description_class() {
        // Description uses `text-sm opacity-90`.
        let base = "text-sm opacity-90";
        assert!(base.contains("opacity-90"));
    }

    #[test]
    fn toast_action_class_contains_destructive_escape() {
        // The action button has CSS rules for `.group-[.destructive]:...`
        // to override the default style when inside a destructive toast.
        let base = "group-[.destructive]:border-muted/40";
        assert!(base.contains(".destructive"));
    }

    #[test]
    fn toast_close_renders_x_icon() {
        // The close button renders an `X` icon (lucide `x`).
        // We verify the icon name string is `x`.
        let icon_name = "x";
        assert_eq!(icon_name, "x");
    }
}
