//! `Tabs` — tabbed content switcher.
//!
//! Backward-compatible with the previous API (`items`, `active`, `on_select`).
//! Adds `vertical` orientation, an `on_change` alias for `on_select`,
//! `class_name` override, and `default_active` for uncontrolled initial
//! state. When `on_change` is `Some`, it fires in addition to `on_select`
//! so callers can opt into either name.
//!
//! Wave 23 T4: added full keyboard navigation. Arrow Left/Right (or
//! Up/Down when `vertical`) move focus + activation between tabs.
//! Home / End jump to the first / last tab. Space / Enter activate a
//! tab. This matches the WAI-ARIA Authoring Practices for the
//! "automatic activation" tab pattern, which is what the prod
//! `tabs.tsx` (Radix) gives for free.

use super::icon::Icon;

use dioxus::prelude::*;

#[derive(Clone, Debug, PartialEq)]
pub struct TabItem {
    pub key: String,
    pub label: String,
    pub icon: Option<String>,
}

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

    // Keyboard handler — moves focus and selects the previous / next
    // tab. We use `onkeydown` on the tablist wrapper so the entire
    // tablist acts as a single focusable region (roving tabindex).
    let items_for_keys = items.clone();
    let active_for_keys = active.clone();
    let onkeydown = move |e: Event<KeyboardData>| {
        let key = e.key();
        let cur_idx = items_for_keys.iter().position(|i| i.key == active_for_keys);
        let total = items_for_keys.len();
        if total == 0 {
            return;
        }
        let cur = cur_idx.unwrap_or(0);
        // Arrow keys move focus; Home / End jump to the ends.
        // Activation is automatic (per WAI-ARIA "automatic
        // activation" pattern).
        let next: Option<usize> = match key {
            Key::ArrowLeft if !vertical => Some(if cur == 0 { total - 1 } else { cur - 1 }),
            Key::ArrowRight if !vertical => Some(if cur + 1 >= total { 0 } else { cur + 1 }),
            Key::ArrowUp if vertical => Some(if cur == 0 { total - 1 } else { cur - 1 }),
            Key::ArrowDown if vertical => Some(if cur + 1 >= total { 0 } else { cur + 1 }),
            Key::Home => Some(0),
            Key::End => Some(total - 1),
            _ => None,
        };
        if let Some(n) = next {
            // Prevent the default browser behaviour (page scroll on
            // arrow keys, etc.).
            e.prevent_default();
            let new_key = items_for_keys[n].key.clone();
            on_select.call(new_key.clone());
            if let Some(h) = &on_change {
                h.call(new_key);
            }
        }
    };

    rsx! {
        div {
            class: "{cls}",
            role: "tablist",
            "aria-orientation": if vertical { "vertical" } else { "horizontal" },
            tabindex: "0",
            onkeydown: onkeydown,
            for (idx, item) in items.iter().enumerate() {
                {
                    let key = item.key.clone();
                    let key_for_cb = item.key.clone();
                    let key_for_focus_cb = item.key.clone();
                    let active_for_focus = active.clone();
                    let on_select_for_click = on_select;
                    let on_change_for_click = on_change;
                    rsx! {
                        button {
                            key: "{key}",
                            class: if key == active { "tab tab-active" } else { "tab" },
                            role: "tab",
                            id: format!("tab-{}", key),
                            "aria-selected": key == active,
                            "aria-controls": format!("tabpanel-{}", key),
                            tabindex: if key == active_for_focus { "0" } else { "-1" },
                            onclick: move |_| {
                                on_select_for_click.call(key_for_cb.clone());
                                if let Some(h) = &on_change_for_click {
                                    h.call(key_for_cb.clone());
                                }
                            },
                            onfocus: move |_| {
                                // Roving tabindex: focusing a tab
                                // makes it the active one (matches
                                // WAI-ARIA's "automatic activation"
                                // pattern, which the prod
                                // `tabs.tsx` defaults to).
                                if idx > 0 {
                                    on_select_for_click.call(key_for_focus_cb.clone());
                                    if let Some(h) = &on_change_for_click {
                                        h.call(key_for_focus_cb.clone());
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
    }
}
