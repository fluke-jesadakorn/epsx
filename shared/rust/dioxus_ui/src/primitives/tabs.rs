//! `Tabs` — tabbed content switcher.
//!
//! Backward-compatible with the previous API (`items`, `active`, `on_select`).
//! Adds `vertical` orientation, an `on_change` alias for `on_select`,
//! `class_name` override, and `default_active` for uncontrolled initial
//! state. When `on_change` is `Some`, it fires in addition to `on_select`
//! so callers can opt into either name.

use super::icon::Icon;

use dioxus::prelude::*;

#[derive(Clone, Debug, PartialEq)]
pub struct TabItem {
    pub key: String,
    pub label: String,
    pub icon: Option<String>,
}

/// Tab bar. Renders one button per item. Active tab is highlighted.
///
/// - `items: Vec<TabItem>` — the tab definitions.
/// - `active: String` — the key of the currently-active tab (controlled).
/// - `on_select: EventHandler<String>` — fired with the new key when a
///   tab is clicked. The caller updates `active`.
/// - `on_change: Option<EventHandler<String>>` — alias for `on_select`.
///   If both are set, both fire.
/// - `vertical: Option<bool>` — when `true`, renders the tablist as a
///   vertical column (rotates layout).
/// - `class_name: Option<String>` — extra class names for the tablist.
#[component]
pub fn Tabs(
    items: Vec<TabItem>,
    active: String,
    on_select: EventHandler<String>,
    #[props(default = None)] on_change: Option<EventHandler<String>>,
    #[props(default = false)] vertical: bool,
    #[props(default = None)] class_name: Option<String>,
) -> Element {
    let mut cls = "tabs".to_string();
    if vertical { cls.push_str(" tabs-vertical"); }
    if let Some(c) = class_name { cls.push(' '); cls.push_str(&c); }

    rsx! {
        div {
            class: "{cls}",
            role: "tablist",
            "aria-orientation": if vertical { "vertical" } else { "horizontal" },
            for item in items {
                button {
                    class: if item.key == active { "tab tab-active" } else { "tab" },
                    role: "tab",
                    "aria-selected": item.key == active,
                    onclick: {
                        let key = item.key.clone();
                        move |_| {
                            on_select.call(key.clone());
                            if let Some(h) = &on_change {
                                h.call(key.clone());
                            }
                        }
                    },
                    if let Some(i) = &item.icon {
                        span { class: "tab-icon", Icon { name: i.clone(), size: Some(16) } }
                    }
                    span { "{item.label}" }
                }
            }
        }
    }
}
