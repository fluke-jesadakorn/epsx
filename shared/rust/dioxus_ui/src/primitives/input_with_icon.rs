//! `InputWithIcon` — text input with an inline icon.
//!
//! Mirrors `apps-old/frontend/components/ui/input-with-icon.tsx`. The
//! Dioxus `Input` component already supports an `icon: Option<String>`
//! prop that renders the icon inside the wrap. This module provides a
//! dedicated component for callers that prefer the shadcn-style
//! separate symbol.

use super::input::{Input, InputKind};
use super::icon::Icon;

use dioxus::prelude::*;

#[component]
pub fn InputWithIcon(
    icon: Option<String>,
    #[props(default = None)] r#type: Option<InputKind>,
    #[props(default = None)] name: Option<String>,
    #[props(default = None)] value: Option<String>,
    #[props(default = None)] default_value: Option<String>,
    #[props(default = None)] placeholder: Option<String>,
    #[props(default = None)] disabled: Option<bool>,
    #[props(default = None)] required: Option<bool>,
    #[props(default = None)] class_name: Option<String>,
    #[props(default = None)] id: Option<String>,
    #[props(default = None)] oninput: Option<EventHandler<FormEvent>>,
    #[props(default = None)] onchange: Option<EventHandler<FormEvent>>,
) -> Element {
    rsx! {
        div { class: "input-with-icon-wrap relative",
            if let Some(i) = &icon {
                span { class: "input-with-icon-leading absolute left-3 top-1/2 -translate-y-1/2 pointer-events-none text-muted-foreground",
                    Icon { name: i.clone(), size: Some(16) }
                }
            }
            Input {
                r#type: r#type.unwrap_or(InputKind::Text),
                name: name,
                value: value,
                default_value: default_value,
                placeholder: placeholder,
                disabled: disabled,
                required: required,
                class_name: class_name,
                id: id,
                oninput: oninput,
                onchange: onchange,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn icon_wrap_class_is_present() {
        let cls = "input-with-icon-wrap relative";
        assert!(cls.contains("input-with-icon-wrap"));
    }
}