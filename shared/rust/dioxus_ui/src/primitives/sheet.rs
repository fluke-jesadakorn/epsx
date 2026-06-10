//! Sheet — slide-in side panel. Mirrors `shared/components/ui/sheet.tsx`.

use dioxus::prelude::*;

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
    rsx! {
        div { class: "sheet-overlay", onclick: move |e| on_close.call(e),
            div {
                class: "{cls}",
                role: "dialog",
                "aria-modal": "true",
                onclick: |e| e.stop_propagation(),
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

/// Title slot.
#[component]
pub fn SheetTitle(class_name: Option<String>, children: Element) -> Element {
    let extra = class_name.unwrap_or_default();
    let cls = if extra.is_empty() {
        "text-lg font-semibold text-foreground".to_string()
    } else {
        format!("text-lg font-semibold text-foreground {extra}")
    };
    rsx! { h2 { class: "{cls}", {children} } }
}

/// Description slot.
#[component]
pub fn SheetDescription(class_name: Option<String>, children: Element) -> Element {
    let extra = class_name.unwrap_or_default();
    let cls = if extra.is_empty() {
        "text-sm text-muted-foreground".to_string()
    } else {
        format!("text-sm text-muted-foreground {extra}")
    };
    rsx! { p { class: "{cls}", {children} } }
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
