//! Alert — non-modal inline message box. Mirrors `shared/components/ui/alert.tsx`.
//!
//! The TS shadcn source uses CVA to produce variant classes. We mirror that
//! with an `AlertKind` enum and a `classes()` method, plus the
//! `AlertTitle` / `AlertDescription` / `AlertAction` slot components.

use dioxus::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AlertKind {
    Default,
    Success,
    Warning,
    Danger,
    Info,
}

impl AlertKind {
    pub fn classes(&self) -> &'static str {
        match self {
            AlertKind::Default => "alert",
            AlertKind::Success => "alert alert-success",
            AlertKind::Warning => "alert alert-warn",
            AlertKind::Danger => "alert alert-error",
            AlertKind::Info => "alert alert-info",
        }
    }

    /// Default icon name (lucide registry) for this variant. Caller can
    /// override via the `icon` prop on `<Alert>`.
    ///
    /// Icon names must exist in `epsx_templates::lucide`'s registry.
    /// We pick closest semantic matches from the registered set:
    /// - `check`        for Success
    /// - `bell`         for Warning (semantic "notify")
    /// - `x`            for Danger (visual "stop")
    /// - `info`         for Default / Info
    ///
    /// TODO: add lucide registry entries for the proper shadcn names
    /// (`check-circle`, `alert-triangle`, `alert-circle`) so the
    /// rendered icons match the TS shadcn output exactly. The
    /// current substitutes are visually distinct but semantically
    /// close.
    pub fn default_icon(&self) -> &'static str {
        match self {
            AlertKind::Default => "info",
            AlertKind::Success => "check",
            AlertKind::Warning => "bell",
            AlertKind::Danger => "x",
            AlertKind::Info => "info",
        }
    }
}

/// Inline alert / callout box.
///
/// Renders `<div role="alert">` for screen reader live region announcements.
/// Variant class is chosen via `kind`. If `icon` is `Some(name)`, an
/// `<Icon>` is rendered above the title (matching the TS shadcn
/// `[&>svg]:absolute` positioning).
#[component]
pub fn Alert(
    kind: Option<AlertKind>,
    title: Option<String>,
    description: Option<String>,
    icon: Option<String>,
    class_name: Option<String>,
    children: Element,
) -> Element {
    let k = kind.unwrap_or(AlertKind::Default);
    let base = k.classes();
    let extra = class_name.unwrap_or_default();
    let cls = if extra.is_empty() {
        base.to_string()
    } else {
        format!("{base} {extra}")
    };
    let icon_name: String = icon.unwrap_or_else(|| k.default_icon().to_string());
    rsx! {
        div {
            class: "{cls}",
            role: "alert",
            if !icon_name.is_empty() {
                crate::primitives::icon::Icon {
                    name: icon_name,
                    size: Some(16),
                }
            }
            div {
                if let Some(t) = &title {
                    h5 { class: "alert-title mb-1 font-medium leading-none tracking-tight", "{t}" }
                }
                if let Some(d) = &description {
                    div { class: "alert-description text-sm [&_p]:leading-relaxed", "{d}" }
                }
                {children}
            }
        }
    }
}

/// Alert title slot.
#[component]
pub fn AlertTitle(class_name: Option<String>, children: Element) -> Element {
    let extra = class_name.unwrap_or_default();
    let cls = if extra.is_empty() {
        "alert-title mb-1 font-medium leading-none tracking-tight".to_string()
    } else {
        format!("alert-title mb-1 font-medium leading-none tracking-tight {extra}")
    };
    rsx! { h5 { class: "{cls}", {children} } }
}

/// Alert description / body slot.
#[component]
pub fn AlertDescription(class_name: Option<String>, children: Element) -> Element {
    let extra = class_name.unwrap_or_default();
    let cls = if extra.is_empty() {
        "alert-description text-sm [&_p]:leading-relaxed".to_string()
    } else {
        format!("alert-description text-sm [&_p]:leading-relaxed {extra}")
    };
    rsx! { div { class: "{cls}", {children} } }
}

/// Right-aligned action area inside an `<Alert>`.
#[component]
pub fn AlertAction(class_name: Option<String>, children: Element) -> Element {
    let extra = class_name.unwrap_or_default();
    let cls = if extra.is_empty() {
        "alert-action".to_string()
    } else {
        format!("alert-action {extra}")
    };
    rsx! { div { class: "{cls}", {children} } }
}
