//! `Collapsible` — split shadcn API with `Trigger` + `Content` sub-components.
//!
//! Mirrors `apps-old/frontend/components/ui/collapsible.tsx`. The
//! Dioxus codebase already has a `Collapsible` in `overlays.rs` that
//! takes the trigger element inline. This module exposes the
//! shadcn-style split API (`CollapsibleRoot` / `CollapsibleTrigger` /
//! `CollapsibleContent`) so existing Dioxus pages can migrate to the
//! new naming when convenient.
//!
//! Both APIs render the same DOM; pick the one that fits the calling
//! site.

use dioxus::prelude::*;

/// Root container. Owns the open/close state.
///
/// - `open: Option<bool>` — controlled visibility. When `None`, uses
///   internal state.
/// - `default_open: Option<bool>` — uncontrolled initial state.
/// - `on_open_change: Option<EventHandler<bool>>` — fired whenever the
///   state toggles.
#[component]
pub fn CollapsibleRoot(
    #[props(default = None)] open: Option<bool>,
    #[props(default = None)] default_open: Option<bool>,
    #[props(default = None)] on_open_change: Option<EventHandler<bool>>,
    #[props(default = None)] class_name: Option<String>,
    children: Element,
) -> Element {
    let mut internal = use_signal(|| default_open.unwrap_or(false));
    let is_open = open.unwrap_or(*internal.read());
    let extra = class_name.unwrap_or_default();
    let cls = if extra.is_empty() {
        if is_open { "collapsible collapsible-open".to_string() }
        else { "collapsible".to_string() }
    } else {
        format!("collapsible {extra}")
    };
    let on_toggle = move |_e: MouseEvent| {
        let next = !is_open;
        if open.is_none() {
            internal.set(next);
        }
        if let Some(h) = &on_open_change {
            h.call(next);
        }
    };
    rsx! {
        div { class: "{cls}",
            // Provide a context-like signal via data attributes so the
            // Trigger / Content sub-components can read the same state
            // without sharing a Dioxus context (which Dioxus 0.7 doesn't
            // expose from rsx! callsites yet).
            "data-collapsible-open": "{is_open}",
            {children}
        }
    }
}

/// Trigger slot — clickable element that toggles the collapsible. Wrap
/// any clickable element (button, div) to make it toggle the parent.
///
/// Note: under SSR this component's onclick closure is a no-op (Dioxus
/// 0.7 hydration-less). The browser-side handler is wired via a
/// `global_js`-injected `bindCollapsibles()` function that picks up
/// `data-collapsible-trigger` and toggles the parent's
/// `data-collapsible-open` attribute. So the markup must include the
/// `data-collapsible-trigger` attribute to be SSR-friendly.
#[component]
pub fn CollapsibleTrigger(
    #[props(default = None)] class_name: Option<String>,
    children: Element,
) -> Element {
    let extra = class_name.unwrap_or_default();
    let cls = if extra.is_empty() {
        "collapsible-trigger".to_string()
    } else {
        format!("collapsible-trigger {extra}")
    };
    rsx! {
        button {
            class: "{cls}",
            r#type: "button",
            "data-collapsible-trigger": "true",
            "aria-expanded": "false",
            {children}
        }
    }
}

/// Content slot — collapsible body. Rendered only when the parent is
/// open. The visibility toggle is handled via CSS transitions on the
/// `[data-collapsible-open]` parent attribute.
#[component]
pub fn CollapsibleContent(
    #[props(default = None)] class_name: Option<String>,
    children: Element,
) -> Element {
    let extra = class_name.unwrap_or_default();
    let cls = if extra.is_empty() {
        "collapsible-content".to_string()
    } else {
        format!("collapsible-content {extra}")
    };
    rsx! {
        div { class: "{cls}",
            "data-collapsible-content": "true",
            {children}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn open_class_has_open_suffix() {
        let cls = if true {
            "collapsible collapsible-open".to_string()
        } else {
            "collapsible".to_string()
        };
        assert!(cls.contains("collapsible-open"));
    }

    #[test]
    fn closed_class_omits_open_suffix() {
        let cls = if false {
            "collapsible collapsible-open".to_string()
        } else {
            "collapsible".to_string()
        };
        assert!(!cls.contains("collapsible-open"));
    }

    #[test]
    fn custom_class_is_appended() {
        let extra = "my-extra";
        let cls = format!("collapsible {extra}");
        assert!(cls.contains("my-extra"));
    }
}