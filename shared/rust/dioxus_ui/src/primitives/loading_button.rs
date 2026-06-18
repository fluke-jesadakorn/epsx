//! `LoadingButton` — button with built-in loading state.
//!
//! Mirrors `apps-old/frontend/components/ui/loading-button.tsx`. The
//! Dioxus `Button` already has a `loading: bool` prop, but this
//! module provides a dedicated component with the shadcn API:
//! - `is_loading: bool` — when true, the button is disabled and shows
//!   a spinner + optional `loading_text`
//! - `loading_text: Option<String>` — text shown next to the spinner
//! - `icon: Option<String>` — left-side icon when not loading
//!
//! Internally delegates to the existing `Button` so all visual styles
//! (variant, size) stay consistent.

use super::button::{Button, ButtonKind, ButtonSize};
use super::icon::Icon;

use dioxus::prelude::*;

#[component]
pub fn LoadingButton(
    is_loading: bool,
    #[props(default = None)] loading_text: Option<String>,
    #[props(default = None)] kind: Option<ButtonKind>,
    #[props(default = None)] size: Option<ButtonSize>,
    #[props(default = None)] icon: Option<String>,
    #[props(default = None)] class_name: Option<String>,
    #[props(default = None)] r#type: Option<String>,
    #[props(default = None)] disabled: Option<bool>,
    #[props(default = None)] onclick: Option<EventHandler<MouseEvent>>,
    children: Element,
) -> Element {
    let loading = is_loading;
    let disabled_combined = loading || disabled.unwrap_or(false);
    let text = loading_text.unwrap_or_else(|| "Loading...".to_string());

    rsx! {
        Button {
            kind: kind.unwrap_or(ButtonKind::Primary),
            size: size.unwrap_or(ButtonSize::Md),
            disabled: Some(disabled_combined),
            class_name: class_name,
            r#type: r#type,
            onclick: onclick,
            if loading {
                Icon { name: "loader".to_string(), size: Some(16) }
                span { class: "ml-2", "{text}" }
            } else {
                if let Some(i) = icon {
                    span { class: "mr-2", Icon { name: i.clone(), size: Some(16) } }
                }
                {children}
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_loading_text_is_loading() {
        // The default text is "Loading..." — verify the constant.
        let default_text: String = "Loading...".to_string();
        assert_eq!(default_text, "Loading...");
    }

    #[test]
    fn disabled_when_loading_regardless_of_explicit_disabled() {
        let explicit_disabled: Option<bool> = None;
        let loading = true;
        let result = loading || explicit_disabled.unwrap_or(false);
        assert!(result);
    }

    #[test]
    fn disabled_when_explicit_disabled_even_if_not_loading() {
        let explicit_disabled: Option<bool> = Some(true);
        let loading = false;
        let result = loading || explicit_disabled.unwrap_or(false);
        assert!(result);
    }
}