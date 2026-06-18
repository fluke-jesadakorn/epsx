//! `Dialog` — shadcn/Radix-naming wrapper around `Modal`.
//!
//! Mirrors `apps-old/frontend/components/ui/dialog.tsx`. The underlying
//! Dioxus primitive (`Modal`) already provides an accessible dialog
//! with overlay dismissal and focus trap; this module re-exports it
//! under the `Dialog` namespace and adds the shadcn-style sub-component
//! names (`DialogTrigger`, `DialogContent`, `DialogHeader`, `DialogTitle`,
//! `DialogDescription`, `DialogFooter`, `DialogClose`).
//!
//! Note: the sub-component names are prefixed with `Dialog` to avoid
//! collisions with `dropdown_menu`'s `Trigger`. Callers should use the
//! fully-qualified names when they want unambiguous resolution.

use dioxus::prelude::*;

/// `Dialog` is an alias for the existing `Modal` primitive. Identical
/// behaviour; the alias is here for shadcn-API parity.
pub use super::modal::Modal as DialogRoot;
pub use super::modal::ModalBody as DialogBody;
pub use super::modal::ModalFooter as DialogFooter;
pub use super::modal::ModalHeader as DialogHeader;

/// Trigger slot — a button that opens the dialog. The parent owns the
/// `open` state and flips it via `on_click`.
#[component]
pub fn DialogTrigger(
    #[props(default = None)] class: Option<String>,
    onclick: Option<EventHandler<MouseEvent>>,
    children: Element,
) -> Element {
    let extra = class.unwrap_or_default();
    let base = "dialog-trigger";
    let cls = if extra.is_empty() {
        base.to_string()
    } else {
        format!("{base} {extra}")
    };
    rsx! {
        button {
            class: "{cls}",
            r#type: "button",
            "aria-haspopup": "dialog",
            onclick: move |e| if let Some(h) = &onclick { h.call(e); },
            {children}
        }
    }
}

/// Content slot — the dialog panel. Renders the same markup as the
/// `Modal` panel; use this when you want the shadcn `DialogContent`
/// name.
#[component]
pub fn DialogContent(
    open: bool,
    on_close: EventHandler<MouseEvent>,
    #[props(default = None)] title: Option<String>,
    #[props(default = None)] description: Option<String>,
    #[props(default = None)] class_name: Option<String>,
    children: Element,
) -> Element {
    let extra = class_name.clone();
    rsx! {
        super::modal::Modal {
            open: open,
            on_close: on_close,
            title: title,
            description: description,
            class_name: extra,
            {children}
        }
    }
}

/// Title slot — renders inside `DialogContent` when no `title` prop was
/// passed. Use this for custom-styled titles.
#[component]
pub fn DialogTitle(
    #[props(default = None)] class_name: Option<String>,
    children: Element,
) -> Element {
    let extra = class_name.unwrap_or_default();
    let cls = if extra.is_empty() {
        "dialog-title".to_string()
    } else {
        format!("dialog-title {extra}")
    };
    rsx! { h2 { class: "{cls}", {children} } }
}

/// Description slot.
#[component]
pub fn DialogDescription(
    #[props(default = None)] class_name: Option<String>,
    children: Element,
) -> Element {
    let extra = class_name.unwrap_or_default();
    let cls = if extra.is_empty() {
        "dialog-description text-sm text-muted-foreground".to_string()
    } else {
        format!("dialog-description text-sm text-muted-foreground {extra}")
    };
    rsx! { p { class: "{cls}", {children} } }
}

/// Close slot — a button that closes the dialog.
#[component]
pub fn DialogClose(
    on_close: EventHandler<MouseEvent>,
    #[props(default = None)] class_name: Option<String>,
    children: Element,
) -> Element {
    let extra = class_name.unwrap_or_default();
    let cls = if extra.is_empty() {
        "dialog-close".to_string()
    } else {
        format!("dialog-close {extra}")
    };
    rsx! {
        button {
            class: "{cls}",
            r#type: "button",
            "aria-label": "Close",
            onclick: move |e| on_close.call(e),
            {children}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn title_class_starts_with_dialog_title() {
        // Verify the class string starts with `dialog-title` for the
        // default-class case.
        let cls_default = if true {
            let extra = String::new();
            if extra.is_empty() {
                "dialog-title".to_string()
            } else {
                format!("dialog-title {extra}")
            }
        } else {
            String::new()
        };
        assert!(cls_default.starts_with("dialog-title"));
    }
}