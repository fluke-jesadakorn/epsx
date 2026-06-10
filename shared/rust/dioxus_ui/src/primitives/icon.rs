use dioxus::prelude::*;

use super::button::{ButtonKind, ButtonSize};

#[component]
pub fn Icon(name: String, size: Option<u32>, class_name: Option<String>) -> Element {
    let s = size.unwrap_or(20);
    let cls = class_name.unwrap_or_default();
    let sz_str = s.to_string();
    let svg = epsx_templates::lucide(&name, &sz_str, &cls);
    rsx! { span { class: "epsx-icon", dangerous_inner_html: "{svg}" } }
}

/// Square icon-only button. Wraps a single lucide icon in a button
/// with the `btn-icon` sizing class. Use for toolbars, row actions,
/// modal close buttons, etc. Set `aria_label` (and optionally `title`)
/// for accessibility — the visible label is only the icon.
#[component]
pub fn IconButton(
    name: String,
    kind: Option<ButtonKind>,
    /// Override the icon size in pixels. Defaults to 16.
    icon_size: Option<u32>,
    href: Option<String>,
    r#type: Option<String>,
    disabled: Option<bool>,
    loading: Option<bool>,
    /// Accessible label for the button (e.g. "Delete", "Close").
    /// Rendered as `aria-label` (and `title` unless `title` is set
    /// explicitly).
    aria_label: Option<String>,
    /// Tooltip text shown on hover. Defaults to `aria_label` when not
    /// provided.
    title: Option<String>,
    class_name: Option<String>,
    id: Option<String>,
    onclick: Option<EventHandler<MouseEvent>>,
) -> Element {
    let kind = kind.unwrap_or(ButtonKind::Ghost);
    let ic_size = icon_size.unwrap_or(16);
    let aria = aria_label.clone().unwrap_or_else(|| name.clone());
    let title_attr = title.clone().or_else(|| aria_label.clone()).unwrap_or_default();

    let mut cls = "btn btn-icon".to_string();
    cls.push(' ');
    cls.push_str(match kind {
        ButtonKind::Primary => "btn-primary",
        ButtonKind::Gradient => "btn-gradient",
        ButtonKind::Brand => "btn-brand",
        ButtonKind::Cool => "btn-cool",
        ButtonKind::Outline => "btn-outline",
        ButtonKind::Ghost => "btn-ghost",
        ButtonKind::Glass => "btn-glass",
        ButtonKind::Danger => "btn-danger",
        ButtonKind::Link => "btn-link",
        ButtonKind::Destructive => "btn-danger",
        ButtonKind::Secondary => "btn-outline",
        ButtonKind::Default => "",
    });
    // size-class for Icon (sized by .btn-icon; do nothing here)
    let _ = ButtonSize::Icon;
    if loading.unwrap_or(false) { cls.push_str(" btn-loading"); }
    if disabled.unwrap_or(false) { cls.push_str(" disabled"); }
    if let Some(c) = class_name { cls.push(' '); cls.push_str(&c); }

    if let Some(url) = href {
        rsx! {
            a { class: "{cls}", href: "{url}", id: id.clone(), title: "{title_attr}", "aria-label": "{aria}",
                Icon { name: name, size: Some(ic_size) }
            }
        }
    } else {
        rsx! {
            button {
                class: "{cls}",
                r#type: r#type.unwrap_or_else(|| "button".to_string()),
                disabled: disabled.unwrap_or(false),
                id: id.clone(),
                title: "{title_attr}",
                "aria-label": "{aria}",
                onclick: move |e| if let Some(h) = &onclick { h.call(e); },
                Icon { name: name, size: Some(ic_size) }
            }
        }
    }
}
