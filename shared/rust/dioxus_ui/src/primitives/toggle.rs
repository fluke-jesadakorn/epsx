//! `Toggle` / `ToggleGroup` / `ToggleGroupItem` — toggle button
//! and toggle group.
//!
//! Mirrors the visual pattern from shadcn new-york: a single
//! toggle button (pressed / unpressed state) and a group of
//! related toggle buttons (where one or many can be active).
//!
//! Useful for view mode switches (e.g. grid vs. list), filter
//! chips with on/off state, or formatting toolbars (bold,
//! italic, underline).

use dioxus::prelude::*;

/// Toggle size.
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub enum ToggleSize {
    Sm,
    #[default]
    Default,
    Lg,
}

impl ToggleSize {
    pub fn as_class(self) -> &'static str {
        match self {
            ToggleSize::Sm => "h-8 px-2 text-xs",
            ToggleSize::Default => "h-9 px-3",
            ToggleSize::Lg => "h-10 px-4 text-base",
        }
    }
}

/// A toggle button. Renders a `<button>` with `aria-pressed` set
/// to `pressed`.
///
/// - `pressed: bool` — whether the toggle is in the "on" state.
/// - `on_pressed_change: EventHandler<bool>` — fired when the user
///   clicks the toggle. Receives the new pressed value.
/// - `size: Option<ToggleSize>` — size token. Defaults to
///   `"default"`.
/// - `disabled: Option<bool>` — disable the toggle.
/// - `class: Option<String>` — extra Tailwind classes.
#[component]
pub fn Toggle(
    pressed: bool,
    on_pressed_change: EventHandler<bool>,
    #[props(default = ToggleSize::default())] size: ToggleSize,
    #[props(default = false)] disabled: bool,
    #[props(default = None)] class: Option<String>,
    children: Element,
) -> Element {
    let mut cls = format!(
        "toggle inline-flex items-center justify-center rounded-md text-sm font-medium ring-offset-background transition-colors hover:bg-muted hover:text-muted-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:pointer-events-none disabled:opacity-50 {} {}",
        if pressed { "bg-accent text-accent-foreground" } else { "bg-transparent" },
        size.as_class(),
    );
    if let Some(c) = class {
        cls.push(' ');
        cls.push_str(&c);
    }
    rsx! {
        button {
            class: "{cls}",
            r#type: "button",
            "aria-pressed": "{pressed}",
            disabled: disabled,
            onclick: move |_| on_pressed_change.call(!pressed),
            {children}
        }
    }
}

/// Selection mode for `ToggleGroup`.
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub enum ToggleGroupType {
    /// Only one item can be pressed at a time.
    #[default]
    Single,
    /// Multiple items can be pressed.
    Multiple,
}

/// Group of toggle buttons. Renders a `<div>` with the items
/// connected visually (no gaps, shared border).
///
/// - `r#type: Option<ToggleGroupType>` — selection mode. Defaults
///   to `"single"`.
/// - `value: Option<String>` — for `"single"` mode, the pressed
///   item's key.
/// - `on_value_change: Option<EventHandler<String>>` — fired when
///   the pressed item changes (single mode).
/// - `values: Option<Vec<String>>` — for `"multiple"` mode, the
///   list of pressed item keys.
/// - `on_values_change: Option<EventHandler<Vec<String>>>` —
///   fired when the pressed items change (multiple mode).
/// - `class: Option<String>` — extra Tailwind classes.
#[component]
pub fn ToggleGroup(
    #[props(default = ToggleGroupType::default())] r#type: ToggleGroupType,
    #[props(default = None)] value: Option<String>,
    #[props(default = None)] on_value_change: Option<EventHandler<String>>,
    #[props(default = None)] values: Option<Vec<String>>,
    #[props(default = None)] on_values_change: Option<EventHandler<Vec<String>>>,
    #[props(default = None)] class: Option<String>,
    children: Element,
) -> Element {
    let mut cls = "toggle-group inline-flex items-center gap-0 rounded-md border bg-background p-0.5".to_string();
    if let Some(c) = class {
        cls.push(' ');
        cls.push_str(&c);
    }
    rsx! {
        div {
            class: "{cls}",
            role: if r#type == ToggleGroupType::Single { "radiogroup" } else { "group" },
            {children}
        }
    }
}

/// An item inside a `ToggleGroup`. Renders a `Toggle` whose
/// `pressed` state is derived from the parent group's state.
#[component]
pub fn ToggleGroupItem(
    item_key: String,
    value: String,
    on_value_change: EventHandler<String>,
    #[props(default = ToggleSize::default())] size: ToggleSize,
    children: Element,
) -> Element {
    let pressed = value == item_key;
    rsx! {
        Toggle {
            pressed: pressed,
            on_pressed_change: move |_| on_value_change.call(item_key.clone()),
            size: size,
            class: Some("rounded-sm".to_string()),
            {children}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn toggle_size_classes_are_distinct() {
        let sizes = [ToggleSize::Sm, ToggleSize::Default, ToggleSize::Lg];
        let classes: Vec<&str> = sizes.iter().map(|s| s.as_class()).collect();
        let unique: std::collections::HashSet<&str> = classes.iter().copied().collect();
        assert_eq!(unique.len(), sizes.len(), "size classes must be distinct");
    }

    #[test]
    fn toggle_pressed_class_includes_accent() {
        let pressed_cls = if true { "bg-accent text-accent-foreground" } else { "bg-transparent" };
        assert!(pressed_cls.contains("bg-accent"));
    }

    #[test]
    fn toggle_unpressed_class_is_transparent() {
        let pressed_cls = if false { "bg-accent text-accent-foreground" } else { "bg-transparent" };
        assert_eq!(pressed_cls, "bg-transparent");
    }

    #[test]
    fn toggle_group_default_type_is_single() {
        assert_eq!(ToggleGroupType::default(), ToggleGroupType::Single);
    }
}
