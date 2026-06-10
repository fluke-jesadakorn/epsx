//! AlertDialog — modal confirmation dialog. Mirrors
//! `shared/components/ui/alert-dialog.tsx`.

use dioxus::prelude::*;

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
    rsx! {
        div { class: "modal-overlay", onclick: move |e| on_close.call(e),
            div {
                class: "{cls}",
                role: "alertdialog",
                "aria-modal": "true",
                onclick: |e| e.stop_propagation(),
                if let Some(t) = &title {
                    div { class: "modal-header",
                        h2 { class: "modal-title", "{t}" }
                        button { class: "modal-close", onclick: move |e| on_close.call(e), "✕" }
                    }
                }
                if let Some(d) = &description {
                    p { class: "modal-description text-sm text-muted-foreground mb-4", "{d}" }
                }
                div { class: "modal-body", {children} }
            }
        }
    }
}

/// Header row for an `<AlertDialog>`.
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

/// Title slot.
#[component]
pub fn AlertDialogTitle(class_name: Option<String>, children: Element) -> Element {
    let extra = class_name.unwrap_or_default();
    let cls = if extra.is_empty() {
        "text-lg font-semibold".to_string()
    } else {
        format!("text-lg font-semibold {extra}")
    };
    rsx! { h2 { class: "{cls}", {children} } }
}

/// Description slot.
#[component]
pub fn AlertDialogDescription(class_name: Option<String>, children: Element) -> Element {
    let extra = class_name.unwrap_or_default();
    let cls = if extra.is_empty() {
        "text-sm text-muted-foreground".to_string()
    } else {
        format!("text-sm text-muted-foreground {extra}")
    };
    rsx! { p { class: "{cls}", {children} } }
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
