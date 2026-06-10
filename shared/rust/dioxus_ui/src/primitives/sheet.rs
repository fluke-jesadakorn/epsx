//! Sheet — slide-in side panel. Mirrors `shared/components/ui/sheet.tsx`.
//!
//! A11y: the sheet container has `role="dialog"`, `aria-modal="true"`,
//! `aria-labelledby` pointing at the title's auto-generated ID, and
//! `aria-describedby` pointing at the description's auto-generated
//! ID. This matches the Radix Dialog (which Sheet uses under the
//! hood in the TS shadcn source) and WAI-ARIA dialog spec.

use dioxus::prelude::*;
use std::sync::atomic::{AtomicU64, Ordering};

/// Per-component-instance nonce used to generate stable IDs for
/// `aria-labelledby` and `aria-describedby` wiring.
static SHEET_NEXT_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SheetSide {
    Left,
    Right,
    Top,
    Bottom,
}

impl SheetSide {
    pub fn classes(&self) -> &'static str {
        match self {
            SheetSide::Left => "sheet sheet-left",
            SheetSide::Right => "sheet sheet-right",
            SheetSide::Top => "sheet sheet-top",
            SheetSide::Bottom => "sheet sheet-bottom",
        }
    }

    /// Accept the TS shadcn string form ("left" | "right" | "top" |
    /// "bottom"); anything else falls back to Right.
    pub fn from_str(s: Option<&str>) -> Self {
        match s.unwrap_or("right") {
            "left" => SheetSide::Left,
            "top" => SheetSide::Top,
            "bottom" => SheetSide::Bottom,
            _ => SheetSide::Right,
        }
    }
}

/// Side panel. Renders nothing when `open` is false.
#[component]
pub fn Sheet(
    open: bool,
    on_close: EventHandler<MouseEvent>,
    side: Option<String>,
    class_name: Option<String>,
    children: Element,
) -> Element {
    if !open {
        return rsx! { Fragment {} };
    }
    let base = SheetSide::from_str(side.as_deref()).classes();
    let extra = class_name.unwrap_or_default();
    let cls = if extra.is_empty() {
        base.to_string()
    } else {
        format!("{base} {extra}")
    };
    // Stable per-instance IDs for the title and description elements
    // so screen readers can announce the dialog's accessible name.
    let nonce = SHEET_NEXT_ID.fetch_add(1, Ordering::SeqCst);
    let title_id = format!("sheet-title-{nonce}");
    let desc_id = format!("sheet-desc-{nonce}");
    rsx! {
        div { class: "sheet-overlay", onclick: move |e| on_close.call(e),
            div {
                class: "{cls}",
                role: "dialog",
                "aria-modal": "true",
                "aria-labelledby": "{title_id}",
                "aria-describedby": "{desc_id}",
                onclick: |e| e.stop_propagation(),
                // Visually-hidden title + description, like the TS
                // shadcn source's VisuallyHidden.Root pattern. They
                // get auto-generated IDs so they wire up to the
                // `aria-labelledby` / `aria-describedby` attributes
                // above; screen readers will read them as the
                // dialog's accessible name and description.
                h2 { id: "{title_id}", class: "sr-only", "Sheet panel" }
                p { id: "{desc_id}", class: "sr-only", "Slide-in side panel" }
                {children}
                button {
                    class: "sheet-close absolute right-4 top-4 rounded-sm opacity-70 hover:opacity-100",
                    r#type: "button",
                    "aria-label": "Close",
                    onclick: move |e| on_close.call(e),
                    "✕"
                }
            }
        }
    }
}

/// Trigger slot for `<Sheet>`.
#[component]
pub fn SheetTrigger(
    onclick: Option<EventHandler<MouseEvent>>,
    class_name: Option<String>,
    children: Element,
) -> Element {
    let extra = class_name.unwrap_or_default();
    let cls = if extra.is_empty() {
        "sheet-trigger".to_string()
    } else {
        format!("sheet-trigger {extra}")
    };
    rsx! {
        span {
            class: "{cls}",
            onclick: move |e| if let Some(h) = &onclick { h.call(e); },
            {children}
        }
    }
}

/// Sheet content body.
#[component]
pub fn SheetContent(
    side: Option<String>,
    class_name: Option<String>,
    children: Element,
) -> Element {
    let base = SheetSide::from_str(side.as_deref()).classes();
    let extra = class_name.unwrap_or_default();
    let cls = if extra.is_empty() {
        base.to_string()
    } else {
        format!("{base} {extra}")
    };
    rsx! { div { class: "{cls}", {children} } }
}

/// Header row inside a sheet.
#[component]
pub fn SheetHeader(class_name: Option<String>, children: Element) -> Element {
    let extra = class_name.unwrap_or_default();
    let cls = if extra.is_empty() {
        "flex flex-col space-y-2 text-center sm:text-left".to_string()
    } else {
        format!("flex flex-col space-y-2 text-center sm:text-left {extra}")
    };
    rsx! { div { class: "{cls}", {children} } }
}

/// Footer row inside a sheet.
#[component]
pub fn SheetFooter(class_name: Option<String>, children: Element) -> Element {
    let extra = class_name.unwrap_or_default();
    let cls = if extra.is_empty() {
        "flex flex-col-reverse sm:flex-row sm:justify-end sm:space-x-2".to_string()
    } else {
        format!("flex flex-col-reverse sm:flex-row sm:justify-end sm:space-x-2 {extra}")
    };
    rsx! { div { class: "{cls}", {children} } }
}

/// Title slot. Renders an `<h2 id="sheet-title-{n}">` for use
/// inside `<SheetHeader>`.
#[component]
pub fn SheetTitle(class_name: Option<String>, children: Element) -> Element {
    let extra = class_name.unwrap_or_default();
    let cls = if extra.is_empty() {
        "text-lg font-semibold text-foreground".to_string()
    } else {
        format!("text-lg font-semibold text-foreground {extra}")
    };
    let id = format!("sheet-title-{}", SHEET_NEXT_ID.fetch_add(1, Ordering::SeqCst));
    rsx! { h2 { class: "{cls}", id: "{id}", {children} } }
}

/// Description slot. Renders a `<p id="sheet-desc-{n}">` for use
/// inside `<SheetHeader>`.
#[component]
pub fn SheetDescription(class_name: Option<String>, children: Element) -> Element {
    let extra = class_name.unwrap_or_default();
    let cls = if extra.is_empty() {
        "text-sm text-muted-foreground".to_string()
    } else {
        format!("text-sm text-muted-foreground {extra}")
    };
    let id = format!("sheet-desc-{}", SHEET_NEXT_ID.fetch_add(1, Ordering::SeqCst));
    rsx! { p { class: "{cls}", id: "{id}", {children} } }
}

/// Close button slot.
#[component]
pub fn SheetClose(
    onclick: Option<EventHandler<MouseEvent>>,
    class_name: Option<String>,
    children: Element,
) -> Element {
    let extra = class_name.unwrap_or_default();
    let cls = if extra.is_empty() {
        "sheet-close-trigger".to_string()
    } else {
        format!("sheet-close-trigger {extra}")
    };
    rsx! {
        span {
            class: "{cls}",
            onclick: move |e| if let Some(h) = &onclick { h.call(e); },
            {children}
        }
    }
}
