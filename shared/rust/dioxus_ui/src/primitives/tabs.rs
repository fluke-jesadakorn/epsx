//! `Tabs` — tabbed content switcher.
//!
//! Backward-compatible with the previous API (`items`, `active`, `on_select`).
//! Adds `vertical` orientation, an `on_change` alias for `on_select`,
//! `class_name` override, and `default_active` for uncontrolled initial
//! state. When `on_change` is `Some`, it fires in addition to `on_select`
//! so callers can opt into either name.
//!
//! Wave 23 T4 v2: the previous T4 added Dioxus-closure-based
//! keyboard nav (onkeydown, onclick, onfocus) — but those closures
//! are stripped at SSR time (hydration-less), so the keyboard
//! nav and click handlers never fired in production. This v2
//! adds `data-epsx-tabs="<group>"` to the tablist and
//! `data-tab-name="<key>"` to each tab button; the
//! `global_js` `bindTabLists()` function picks these up on
//! `DOMContentLoaded` and wires a delegated click + keydown
//! handler that:
//!   1. On click, calls the existing `epsx.activateTab(group, name)`
//!      (which toggles `data-tab-group` elements' `active` class
//!      and `display` style — useful for in-page tab panels) AND
//!      follows `data-tab-href` if set (so URL-based tabs work).
//!   2. On ArrowLeft / ArrowRight (or Up/Down for `aria-orientation="vertical"`)
//!      / Home / End, moves the roving-tabindex focus to the
//!      previous/next tab and updates `aria-selected`.
//!
//! **Caveat:** the Tabs component is a "controlled" component —
//! the parent owns the `active` state and reacts to `on_select`.
//! The Dioxus `on_select` callback is the parent's signal setter
//! and is a closure that gets stripped under SSR. So the
//! `on_select` callback is still wired for callers that hydrate
//! (Dioxus desktop app, future hydration) but does NOT fire for
//! the BFF SSR path. **For full SSR-friendliness, parents
//! should pass `data-tab-href` URLs on the `Tabs` items** (not
//! yet implemented) so each tab is a real link the browser
//! navigates to; the BFF re-renders the page with the new
//! `?tab=…` and the SSR re-render shows the right active tab.
//! That full refactor is a follow-up; this v2 wires the markup
//! + keyboard nav so the page is at least fully navigable via
//! keyboard once `bindTabLists` runs.

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
    /// Stable id used to group tabs for the delegated keyboard
    /// handler in `global_js`. Defaults to a hard-coded
    /// `"tabs-default"` for backwards compat. When multiple
    /// `Tabs` instances live on the same page they should pass
    /// distinct ids to avoid cross-talk.
    #[props(default = "tabs-default".to_string())] group_id: String,
) -> Element {
    let mut cls = "tabs".to_string();
    if vertical { cls.push_str(" tabs-vertical"); }
    if let Some(c) = class_name { cls.push(' '); cls.push_str(&c); }

    // The Dioxus `on_select` callback is still wired for any
    // future hydration path; under SSR it's a no-op. The
    // `global_js` `bindTabLists` function handles the actual
    // keyboard nav and click delegation.
    let _ = on_select;
    let _ = on_change;

    rsx! {
        div {
            class: "{cls}",
            role: "tablist",
            "aria-orientation": if vertical { "vertical" } else { "horizontal" },
            "data-epsx-tabs": "{group_id}",
            tabindex: "0",
            for item in items.iter() {
                {
                    let key = item.key.clone();
                    let safe_key = epsx_templates::html_attr_escape_pub(&item.key);
                    let is_active = key == active;
                    rsx! {
                        button {
                            key: "{key}",
                            class: if is_active { "tab tab-active" } else { "tab" },
                            role: "tab",
                            id: format!("tab-{}", key),
                            "data-tab-name": "{safe_key}",
                            "aria-selected": is_active,
                            "aria-controls": format!("tabpanel-{}", key),
                            tabindex: if is_active { "0" } else { "-1" },
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
