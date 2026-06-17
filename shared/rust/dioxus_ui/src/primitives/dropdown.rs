//! Dropdown menu primitive — 1:1 mirror of `shared/components/ui/dropdown-menu.tsx`.
//!
//! Backward-compatible with the previous API (`open: bool`, `on_toggle`).
//! Adds controlled state, `align`/`side` positioning, a `DropdownTrigger`
//! slot, and `DropdownLabel` / `DropdownCheckboxItem` slot components.

use super::icon::Icon;

use dioxus::prelude::*;

/// Container for a trigger + menu. The menu is rendered conditionally
/// based on the `open` value.
///
/// - `open` (required, legacy) — the current open state.
/// - `on_toggle` (required, legacy) — fired when the trigger is clicked.
///   Prefer `on_open_change` in new code; the two are equivalent.
/// - `on_open_change` (optional) — fired with the next open value when
///   the trigger is clicked, mirroring the Radix `onOpenChange` API.
/// - `align` (optional) — `"start" | "center" | "end"` — horizontal
///   alignment of the menu relative to the trigger.
/// - `side` (optional) — `"top" | "bottom"` — which side the menu
///   renders on.
/// - `menu_class` (optional) — additional class names for the menu
///   container.
#[component]
pub fn Dropdown(
    open: bool,
    on_toggle: EventHandler<MouseEvent>,
    #[props(default = None)] on_open_change: Option<EventHandler<bool>>,
    #[props(default = None)] align: Option<String>,
    #[props(default = None)] side: Option<String>,
    #[props(default = None)] menu_class: Option<String>,
    trigger: Element,
    children: Element,
) -> Element {
    let align_cls = match align.as_deref() {
        Some("start") => " dropdown-menu-align-start",
        Some("end") => " dropdown-menu-align-end",
        _ => " dropdown-menu-align-center",
    };
    let side_cls = match side.as_deref() {
        Some("top") => " dropdown-menu-side-top",
        _ => " dropdown-menu-side-bottom",
    };
    let extra_menu = menu_class.unwrap_or_default();

    let on_trigger = move |e: MouseEvent| {
        on_toggle.call(e);
        if let Some(h) = &on_open_change {
            h.call(!open);
        }
    };

    rsx! {
        div { class: if open { "dropdown dropdown-open" } else { "dropdown" },
            div {
                class: "dropdown-trigger",
                role: "button",
                tabindex: "0",
                "aria-haspopup": "menu",
                "aria-expanded": "{open}",
                onclick: on_trigger,
                {trigger}
            }
            if open {
                div {
                    class: "dropdown-menu{align_cls}{side_cls} {extra_menu}",
                    role: "menu",
                    {children}
                }
            }
        }
    }
}

/// A non-interactive label inside a dropdown menu. Use for headings like
/// "My account" or grouped section titles.
#[component]
pub fn DropdownLabel(children: Element) -> Element {
    rsx! {
        div { class: "dropdown-label", role: "presentation", {children} }
    }
}

/// Clickable item inside a dropdown menu.
#[component]
pub fn DropdownItem(
    href: Option<String>,
    icon: Option<String>,
    danger: Option<bool>,
    inset: Option<bool>,
    onclick: Option<EventHandler<MouseEvent>>,
    children: Element,
) -> Element {
    let mut cls = "dropdown-item".to_string();
    if danger.unwrap_or(false) { cls.push_str(" dropdown-item-danger"); }
    if inset.unwrap_or(false) { cls.push_str(" dropdown-item-inset"); }
    if let Some(h) = href {
        rsx! { a { class: "{cls}", href: "{h}", role: "menuitem",
            if let Some(i) = icon {
                span { class: "dropdown-item-icon", Icon { name: i.clone(), size: Some(16) } }
            }
            {children}
        } }
    } else {
        rsx! { div {
            class: "{cls}", role: "menuitem",
            onclick: move |e| if let Some(h) = &onclick { h.call(e); },
            if let Some(i) = icon {
                span { class: "dropdown-item-icon", Icon { name: i.clone(), size: Some(16) } }
            }
            {children}
        } }
    }
}

/// Checkbox-style item inside a dropdown menu. Fires `onchange` with the
/// new `checked` value when clicked.
#[component]
pub fn DropdownCheckboxItem(
    checked: bool,
    onchange: Option<EventHandler<bool>>,
    icon: Option<String>,
    children: Element,
) -> Element {
    let mut cls = "dropdown-item dropdown-checkbox-item".to_string();
    if checked { cls.push_str(" dropdown-item-checked"); }
    rsx! { div {
        class: "{cls}", role: "menuitemcheckbox",
        "aria-checked": "{checked}",
        onclick: move |_| if let Some(h) = &onchange { h.call(!checked); },
        span { class: "dropdown-item-check",
            if checked {
                Icon { name: "check".to_string(), size: Some(16) }
            }
        }
        if let Some(i) = icon {
            span { class: "dropdown-item-icon", Icon { name: i.clone(), size: Some(16) } }
        }
        {children}
    } }
}

/// Visual separator inside a dropdown menu.
#[component]
pub fn DropdownSeparator() -> Element { rsx! { div { class: "dropdown-separator" } } }
