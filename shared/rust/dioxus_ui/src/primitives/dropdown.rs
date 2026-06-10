use super::icon::Icon;

use dioxus::prelude::*;

#[component]
pub fn Dropdown(
    trigger: Element,
    open: bool,
    on_toggle: EventHandler<MouseEvent>,
    children: Element,
) -> Element {
    rsx! {
        div { class: if open { "dropdown dropdown-open" } else { "dropdown" },
            div { class: "dropdown-trigger", onclick: move |e| on_toggle.call(e), {trigger} }
            if open {
                div { class: "dropdown-menu", role: "menu", {children} }
            }
        }
    }
}

#[component]
pub fn DropdownItem(
    href: Option<String>,
    icon: Option<String>,
    danger: Option<bool>,
    onclick: Option<EventHandler<MouseEvent>>,
    children: Element,
) -> Element {
    let mut cls = "dropdown-item".to_string();
    if danger.unwrap_or(false) { cls.push_str(" dropdown-item-danger"); }
    if let Some(h) = href {
        rsx! { a { class: "{cls}", href: "{h}",
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

#[component]
pub fn DropdownSeparator() -> Element { rsx! { div { class: "dropdown-separator" } } }
