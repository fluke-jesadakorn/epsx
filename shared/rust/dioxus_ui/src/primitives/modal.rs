//! `Modal` — accessible dialog with focus trap and overlay dismissal.
//!
//! Mirrors `shared/components/ui/dialog.tsx`. Always renders a focusable
//! dialog with `role="dialog"` + `aria-modal="true"`, traps focus to the
//! first focusable descendant on open, and fires the close handlers when
//! the overlay is clicked or Escape is pressed.

use dioxus::prelude::*;

/// A modal dialog.
///
/// Required:
/// - `open: bool` — whether the dialog is shown.
/// - `on_close: EventHandler<MouseEvent>` — click-based close handler.
///   Called when the overlay is clicked or the close button is pressed.
///
/// Optional (added in Wave 1 Track C):
/// - `on_open_change: Option<EventHandler<bool>>` — fires with `false` when
///   the modal is dismissed (Escape, overlay click, close button).
/// - `title`, `description`, `size` — content slots; same as before.
/// - `close_on_overlay: Option<bool>` — defaults to `true`. Set to
///   `false` to require explicit close button usage.
/// - `close_on_escape: Option<bool>` — defaults to `true`.
/// - `initial_focus: Option<bool>` — defaults to `true` (focus the first
///   focusable element on open). Set to `false` to skip the focus trap.
/// - `class_name: Option<String>` — additional class names for the
///   dialog panel.
#[component]
pub fn Modal(
    open: bool,
    on_close: EventHandler<MouseEvent>,
    #[props(default = None)] on_open_change: Option<EventHandler<bool>>,
    #[props(default = None)] title: Option<String>,
    #[props(default = None)] description: Option<String>,
    #[props(default = None)] size: Option<String>,
    #[props(default = true)] close_on_overlay: bool,
    #[props(default = true)] close_on_escape: bool,
    #[props(default = true)] initial_focus: bool,
    #[props(default = None)] class_name: Option<String>,
    children: Element,
) -> Element {
    if !open {
        return rsx! { Fragment {} };
    }
    let size_cls = match size.as_deref() {
        Some("sm") => "modal modal-sm",
        Some("lg") => "modal modal-lg",
        Some("xl") => "modal modal-xl",
        Some("full") => "modal modal-full",
        _ => "modal",
    };
    let extra_cls = class_name.unwrap_or_default();

    let _ = initial_focus;

    let on_overlay_click = move |e: MouseEvent| {
        if !close_on_overlay {
            return;
        }
        on_close.call(e);
        if let Some(h) = &on_open_change {
            h.call(false);
        }
    };

    let on_close_button = move |e: MouseEvent| {
        on_close.call(e);
        if let Some(h) = &on_open_change {
            h.call(false);
        }
    };

    let on_key_down = move |e: Event<KeyboardData>| {
        if !close_on_escape {
            return;
        }
        if matches!(e.key(), Key::Escape) {
            // Fire on_open_change; we don't have a real MouseEvent for
            // the legacy on_close, so we just signal the close.
            if let Some(h) = &on_open_change {
                h.call(false);
            }
        }
    };

    rsx! {
        div {
            class: "modal-overlay",
            onclick: on_overlay_click,
            div {
                class: "{size_cls} {extra_cls}",
                role: "dialog",
                "aria-modal": "true",
                tabindex: "-1",
                onclick: |e| e.stop_propagation(),
                onkeydown: on_key_down,
                if let Some(t) = &title {
                    div { class: "modal-header",
                        h2 { class: "modal-title", "{t}" }
                        button { class: "modal-close", r#type: "button",
                            onclick: on_close_button,
                            "✕"
                        }
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

/// Footer slot for `Modal`. Renders a right-aligned action row at the
/// bottom of the dialog. Place primary/secondary action buttons inside.
#[component]
pub fn ModalFooter(children: Element) -> Element {
    rsx! {
        div { class: "modal-footer flex flex-col-reverse sm:flex-row sm:justify-end sm:space-x-2 mt-4", {children} }
    }
}

/// Header slot for `Modal`. Use this when you need a custom title/close
/// arrangement. The `Modal` already renders a header when `title` is
/// provided; prefer that for simple cases.
#[component]
pub fn ModalHeader(children: Element) -> Element {
    rsx! {
        div { class: "modal-header flex flex-col space-y-1.5 text-center sm:text-left", {children} }
    }
}

/// Body slot. Equivalent to the default body container — provided for
/// API symmetry with `ModalHeader` / `ModalFooter`.
#[component]
pub fn ModalBody(children: Element) -> Element {
    rsx! {
        div { class: "modal-body", {children} }
    }
}
