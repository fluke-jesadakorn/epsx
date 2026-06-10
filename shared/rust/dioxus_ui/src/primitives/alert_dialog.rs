//! AlertDialog — modal confirmation dialog. Mirrors
//! `shared/components/ui/alert-dialog.tsx`.
//!
//! A11y: the dialog container has `role="alertdialog"`,
//! `aria-modal="true"`, `aria-labelledby` pointing at the title's
//! auto-generated ID, and `aria-describedby` pointing at the
//! description's auto-generated ID. This matches the Radix
//! AlertDialog behaviour and WAI-ARIA dialog spec.

use dioxus::prelude::*;
use std::sync::atomic::{AtomicU64, Ordering};

/// Per-component-instance nonce used to generate stable IDs for
/// `aria-labelledby` and `aria-describedby` wiring.
static ALERT_DIALOG_NEXT_ID: AtomicU64 = AtomicU64::new(1);

#[component]
pub fn AlertDialog(
    open: bool,
    on_close: EventHandler<MouseEvent>,
    title: Option<String>,
    description: Option<String>,
    size: Option<String>,
    class_name: Option<String>,
    children: Element,
) -> Element {
    if !open {
        return rsx! { Fragment {} };
    }
    let base = match size.as_deref() {
        Some("sm") => "modal modal-sm",
        Some("lg") => "modal modal-lg",
        Some("xl") => "modal modal-xl",
        Some("full") => "modal modal-full",
        _ => "modal",
    };
    let extra = class_name.unwrap_or_default();
    let cls = if extra.is_empty() {
        base.to_string()
    } else {
        format!("{base} {extra}")
    };
    // Stable per-instance IDs for the title and description elements
    // so screen readers can announce the dialog's accessible name
    // and description.
    let nonce = ALERT_DIALOG_NEXT_ID.fetch_add(1, Ordering::SeqCst);
    let title_id = format!("alertdialog-title-{nonce}");
    let desc_id = format!("alertdialog-desc-{nonce}");
    rsx! {
        div { class: "modal-overlay", onclick: move |e| on_close.call(e),
            div {
                class: "{cls}",
                role: "alertdialog",
                "aria-modal": "true",
                "aria-labelledby": "{title_id}",
                "aria-describedby": "{desc_id}",
                onclick: |e| e.stop_propagation(),
                if let Some(t) = &title {
                    div { class: "modal-header",
                        h2 { class: "modal-title", id: "{title_id}", "{t}" }
                        button { class: "modal-close", onclick: move |e| on_close.call(e), "✕" }
                    }
                }
                if let Some(d) = &description {
                    p { class: "modal-description text-sm text-muted-foreground mb-4", id: "{desc_id}", "{d}" }
                }
                div { class: "modal-body", {children} }
            }
        }
    }
}

/// Header row for an `<AlertDialog>`. The header's child slot is
/// `AlertDialogTitle` / `AlertDialogDescription` — these render with
/// auto-generated IDs so a future caller-managed wiring can target
/// them with `aria-labelledby` / `aria-describedby`.
#[component]
pub fn AlertDialogHeader(class_name: Option<String>, children: Element) -> Element {
    let extra = class_name.unwrap_or_default();
    let cls = if extra.is_empty() {
        "flex flex-col space-y-2 text-center sm:text-left".to_string()
    } else {
        format!("flex flex-col space-y-2 text-center sm:text-left {extra}")
    };
    rsx! { div { class: "{cls}", {children} } }
}

/// Footer row for an `<AlertDialog>`.
#[component]
pub fn AlertDialogFooter(class_name: Option<String>, children: Element) -> Element {
    let extra = class_name.unwrap_or_default();
    let cls = if extra.is_empty() {
        "flex flex-col-reverse sm:flex-row sm:justify-end sm:space-x-2".to_string()
    } else {
        format!("flex flex-col-reverse sm:flex-row sm:justify-end sm:space-x-2 {extra}")
    };
    rsx! { div { class: "{cls}", {children} } }
}

/// Trigger element for an `<AlertDialog>`.
#[component]
pub fn AlertDialogTrigger(
    onclick: Option<EventHandler<MouseEvent>>,
    class_name: Option<String>,
    children: Element,
) -> Element {
    let extra = class_name.unwrap_or_default();
    let cls = if extra.is_empty() {
        "alert-dialog-trigger".to_string()
    } else {
        format!("alert-dialog-trigger {extra}")
    };
    rsx! {
        span {
            class: "{cls}",
            onclick: move |e| if let Some(h) = &onclick { h.call(e); },
            {children}
        }
    }
}

/// AlertDialog content wrapper.
#[component]
pub fn AlertDialogContent(class_name: Option<String>, children: Element) -> Element {
    let extra = class_name.unwrap_or_default();
    let cls = if extra.is_empty() {
        "alert-dialog-content".to_string()
    } else {
        format!("alert-dialog-content {extra}")
    };
    rsx! { div { class: "{cls}", {children} } }
}

/// Title slot. Renders an `<h2 id="alertdialog-title-{n}">` for
/// use inside `<AlertDialogHeader>`.
#[component]
pub fn AlertDialogTitle(class_name: Option<String>, children: Element) -> Element {
    let extra = class_name.unwrap_or_default();
    let cls = if extra.is_empty() {
        "text-lg font-semibold".to_string()
    } else {
        format!("text-lg font-semibold {extra}")
    };
    let id = format!("alertdialog-title-{}", ALERT_DIALOG_NEXT_ID.fetch_add(1, Ordering::SeqCst));
    rsx! { h2 { class: "{cls}", id: "{id}", {children} } }
}

/// Description slot. Renders a `<p id="alertdialog-desc-{n}">`
/// for use inside `<AlertDialogHeader>`.
#[component]
pub fn AlertDialogDescription(class_name: Option<String>, children: Element) -> Element {
    let extra = class_name.unwrap_or_default();
    let cls = if extra.is_empty() {
        "text-sm text-muted-foreground".to_string()
    } else {
        format!("text-sm text-muted-foreground {extra}")
    };
    let id = format!("alertdialog-desc-{}", ALERT_DIALOG_NEXT_ID.fetch_add(1, Ordering::SeqCst));
    rsx! { p { class: "{cls}", id: "{id}", {children} } }
}

/// Primary action button.
#[component]
pub fn AlertDialogAction(
    onclick: Option<EventHandler<MouseEvent>>,
    class_name: Option<String>,
    children: Element,
) -> Element {
    let extra = class_name.unwrap_or_default();
    let cls = if extra.is_empty() {
        "btn btn-primary".to_string()
    } else {
        format!("btn btn-primary {extra}")
    };
    rsx! {
        button {
            class: "{cls}",
            r#type: "button",
            onclick: move |e| if let Some(h) = &onclick { h.call(e); },
            {children}
        }
    }
}

/// Cancel button.
#[component]
pub fn AlertDialogCancel(
    onclick: Option<EventHandler<MouseEvent>>,
    class_name: Option<String>,
    children: Element,
) -> Element {
    let extra = class_name.unwrap_or_default();
    let cls = if extra.is_empty() {
        "btn btn-outline mt-2 sm:mt-0".to_string()
    } else {
        format!("btn btn-outline mt-2 sm:mt-0 {extra}")
    };
    rsx! {
        button {
            class: "{cls}",
            r#type: "button",
            onclick: move |e| if let Some(h) = &onclick { h.call(e); },
            {children}
        }
    }
}
