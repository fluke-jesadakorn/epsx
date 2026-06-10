use dioxus::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ButtonKind {
    Primary, Gradient, Brand, Cool, Outline, Ghost, Glass, Danger, Link,
    Destructive, Secondary, Default,
}

impl ButtonKind {
    pub fn classes(&self) -> &'static str {
        match self {
            ButtonKind::Primary => "btn btn-primary",
            ButtonKind::Gradient => "btn btn-gradient",
            ButtonKind::Brand => "btn btn-brand",
            ButtonKind::Cool => "btn btn-cool",
            ButtonKind::Outline => "btn btn-outline",
            ButtonKind::Ghost => "btn btn-ghost",
            ButtonKind::Glass => "btn btn-glass",
            ButtonKind::Danger => "btn btn-danger",
            ButtonKind::Link => "btn btn-link",
            ButtonKind::Destructive => "btn btn-danger",
            ButtonKind::Secondary => "btn btn-outline",
            ButtonKind::Default => "btn",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ButtonSize { Sm, Md, Lg, Xl, Icon }

impl ButtonSize {
    pub fn classes(&self) -> &'static str {
        match self {
            ButtonSize::Sm => "btn-sm",
            ButtonSize::Md => "",
            ButtonSize::Lg => "btn-lg",
            ButtonSize::Xl => "btn-xl",
            ButtonSize::Icon => "btn-icon",
        }
    }
}

#[component]
pub fn Button(
    kind: Option<ButtonKind>,
    size: Option<ButtonSize>,
    href: Option<String>,
    r#type: Option<String>,
    disabled: Option<bool>,
    block: Option<bool>,
    loading: Option<bool>,
    class_name: Option<String>,
    id: Option<String>,
    onclick: Option<EventHandler<MouseEvent>>,
    children: Element,
) -> Element {
    let kind = kind.unwrap_or(ButtonKind::Primary);
    let size = size.unwrap_or(ButtonSize::Md);
    let disabled = disabled.unwrap_or(false);
    let loading = loading.unwrap_or(false);
    let block = block.unwrap_or(false);
    let mut cls = format!("{} {}", kind.classes(), size.classes());
    if block { cls.push_str(" btn-block"); }
    if loading { cls.push_str(" btn-loading"); }
    if let Some(c) = class_name { cls.push(' '); cls.push_str(&c); }

    if let Some(url) = href {
        rsx! {
            a { class: "{cls}", href: "{url}", id: id.clone(), {children} }
        }
    } else {
        rsx! {
            button {
                class: "{cls}",
                r#type: r#type.unwrap_or_else(|| "button".to_string()),
                disabled: disabled,
                id: id.clone(),
                onclick: move |e| if let Some(h) = &onclick { h.call(e); },
                {children}
            }
        }
    }
}
