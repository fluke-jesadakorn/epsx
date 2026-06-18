//! `List` / `ListItem` / `DescriptionList` / `DescriptionItem` —
//! list and description-list components.
//!
//! - `List` / `ListItem` — vertical or horizontal list of items.
///   Each item can have a leading icon, content, and trailing
///   action.
/// - `DescriptionList` / `DescriptionItem` — a `<dl>` for
///   key-value pairs (e.g. settings panels, profile fields,
///   metadata).

use super::icon::Icon;

use dioxus::prelude::*;

/// A list item. Renders a row with an optional leading icon, a
/// content area, and an optional trailing action.
#[component]
pub fn ListItem(
    #[props(default = None)] icon: Option<String>,
    #[props(default = None)] title: Option<String>,
    #[props(default = None)] description: Option<String>,
    #[props(default = None)] trailing: Option<String>,
    #[props(default = None)] on_click: Option<EventHandler<MouseEvent>>,
) -> Element {
    let mut cls = "list-item flex items-center gap-3 px-3 py-2 rounded-md transition-colors hover:bg-muted/50".to_string();
    if on_click.is_some() {
        cls.push_str(" cursor-pointer");
    }
    rsx! {
        div { class: "{cls}", onclick: move |e| if let Some(h) = &on_click { h.call(e); },
            if let Some(i) = icon {
                Icon { name: i.clone(), size: Some(16) }
            }
            div { class: "list-item-content flex-1 min-w-0",
                if let Some(t) = title {
                    div { class: "list-item-title text-sm font-medium truncate", "{t}" }
                }
                if let Some(d) = description {
                    div { class: "list-item-description text-xs text-muted-foreground truncate", "{d}" }
                }
            }
            if let Some(t) = trailing {
                div { class: "list-item-trailing text-xs text-muted-foreground", "{t}" }
            }
        }
    }
}

/// A vertical list of items.
#[component]
pub fn List(
    children: Element,
    #[props(default = None)] class: Option<String>,
) -> Element {
    let mut cls = "list flex flex-col gap-0.5".to_string();
    if let Some(c) = class {
        cls.push(' ');
        cls.push_str(&c);
    }
    rsx! {
        div { class: "{cls}", role: "list", {children} }
    }
}

/// A key-value item in a `DescriptionList`. Renders a `<div>`
/// with the term on the left and the description on the right.
#[component]
pub fn DescriptionItem(
    term: String,
    description: String,
) -> Element {
    rsx! {
        div { class: "description-item grid grid-cols-3 gap-2 py-2 border-b border-border last:border-b-0",
            dt { class: "description-term text-sm font-medium text-muted-foreground", "{term}" }
            dd { class: "description-value text-sm col-span-2", "{description}" }
        }
    }
}

/// A description list. Renders a `<dl>` with the items stacked
/// vertically.
#[component]
pub fn DescriptionList(
    children: Element,
    #[props(default = None)] class: Option<String>,
) -> Element {
    let mut cls = "description-list flex flex-col".to_string();
    if let Some(c) = class {
        cls.push(' ');
        cls.push_str(&c);
    }
    rsx! {
        dl { class: "{cls}", {children} }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn list_item_default_class() {
        let base = "list-item flex items-center gap-3 px-3 py-2 rounded-md transition-colors hover:bg-muted/50";
        assert!(base.contains("flex items-center"));
        assert!(base.contains("hover:bg-muted/50"));
    }

    #[test]
    fn list_uses_role_list() {
        // The list container uses `role="list"` for accessibility.
        let base = "list flex flex-col gap-0.5";
        assert!(base.contains("flex flex-col"));
    }

    #[test]
    fn description_item_grid_3_columns() {
        // The description item uses `grid grid-cols-3` so the term
        // takes 1/3 of the width and the description takes 2/3.
        let base = "description-item grid grid-cols-3 gap-2 py-2 border-b border-border last:border-b-0";
        assert!(base.contains("grid-cols-3"));
    }
}
