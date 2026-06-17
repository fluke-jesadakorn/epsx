//! Input primitive — labelled text/password/email/number/url/tel/search/date/time/file/color/hidden/textarea.
//!
//! Mirrors `shared/components/ui/input.tsx` (53 lines) + `textarea.tsx` (72 lines) from the
//! Next.js shadcn set. The Dioxus component also provides a built-in label + help + error
//! layout so that callers can render a one-shot field.
//!
//! Modes:
//! - **Controlled** — caller passes `value: Some("…")` and reacts to `oninput` / `onchange`.
//! - **Uncontrolled** — caller passes `default_value: Some("…")`; the component manages its
//!   own internal signal. `oninput` / `onchange` are still fired so the caller can observe.
//!
//! Icon support: when `icon` is set, the icon is rendered **inside** the input wrap
//! (absolute-positioned). The previous implementation rendered the wrap as a sibling, which
//! broke the visual layout because the input had its own `input-with-icon` class without
//! the wrap providing the absolute positioning context.

use super::icon::Icon;

use dioxus::prelude::*;

/// All HTML5 input kinds supported. `Textarea` renders a `<textarea>` instead of `<input>`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum InputKind {
    Text,
    Email,
    Password,
    Number,
    Url,
    Tel,
    Search,
    Textarea,
    Select,
    Date,
    Time,
    File,
    Color,
    Hidden,
}

impl InputKind {
    /// The lowercase string used as the `type` attribute on the rendered element.
    pub fn as_str(self) -> &'static str {
        match self {
            InputKind::Text => "text",
            InputKind::Email => "email",
            InputKind::Password => "password",
            InputKind::Number => "number",
            InputKind::Url => "url",
            InputKind::Tel => "tel",
            InputKind::Search => "search",
            InputKind::Date => "date",
            InputKind::Time => "time",
            InputKind::File => "file",
            InputKind::Color => "color",
            InputKind::Hidden => "hidden",
            // `Textarea` and `Select` are not real input types — fall back to text.
            // They are handled by the caller via the dedicated components.
            InputKind::Textarea | InputKind::Select => "text",
        }
    }
}

/// Generic, single-field form input. Supports every HTML5 `type` plus a built-in
/// label/help/error layout. See module docs for controlled vs. uncontrolled mode.
#[component]
pub fn Input(
    r#type: Option<InputKind>,
    name: Option<String>,
    /// Controlled value. When `None`, the component is uncontrolled (uses internal signal).
    value: Option<String>,
    /// Initial value for uncontrolled mode.
    default_value: Option<String>,
    placeholder: Option<String>,
    label: Option<String>,
    help: Option<String>,
    error: Option<String>,
    required: Option<bool>,
    disabled: Option<bool>,
    /// Lucide icon name. When set, the icon is rendered inside the input wrap
    /// (positioned absolutely on the left).
    icon: Option<String>,
    /// Textarea row count. Only used when `r#type` is `Textarea`.
    rows: Option<u32>,
    /// When true, expose `aria-invalid="true"` on the input even if `error` is not set.
    /// Useful for forms that receive validation state from a parent context.
    aria_invalid: Option<bool>,
    class_name: Option<String>,
    id: Option<String>,
    oninput: Option<EventHandler<FormEvent>>,
    onchange: Option<EventHandler<FormEvent>>,
) -> Element {
    let r#type = r#type.unwrap_or(InputKind::Text);
    let type_str = r#type.as_str();
    let required = required.unwrap_or(false);
    let disabled = disabled.unwrap_or(false);

    // Controlled vs. uncontrolled state. When `value` is None, fall back to internal signal.
    let mut internal_value = use_signal(|| default_value.clone().unwrap_or_default());
    let is_controlled = value.is_some();
    let current_value: String = if is_controlled {
        value.clone().unwrap_or_default()
    } else {
        internal_value.read().clone()
    };

    // Build class list using only the existing design-system classes.
    let mut input_cls = "input".to_string();
    if let Some(c) = &class_name { input_cls.push(' '); input_cls.push_str(c); }
    if error.is_some() { input_cls.push_str(" input-error"); }
    if icon.is_some() { input_cls.push_str(" input-with-icon"); }

    // `aria-invalid` is set when (a) the caller passed `aria_invalid: Some(true)`, or
    // (b) an error message is present.
    let invalid = aria_invalid.unwrap_or(false) || error.is_some();

    let id_for = id.clone();
    let name_for_input = name.clone();
    let placeholder_str = placeholder.clone().unwrap_or_default();
    let value_for_input = current_value.clone();

    rsx! {
        div { class: "form-field",
            if let Some(l) = &label {
                label { class: "form-label", r#for: id_for.clone().unwrap_or_default(),
                    "{l}"
                    if required { span { class: "text-red-500 ml-1", "*" } }
                }
            }
            // Icon wrap: the input lives inside so that `position: relative` on the wrap
            // positions the icon absolutely on top of the input.
            div { class: if icon.is_some() { "input-icon-wrap" } else { "" },
                if let Some(i) = &icon {
                    span { class: "input-icon", Icon { name: i.clone(), size: Some(16) } }
                }
                if r#type == InputKind::Textarea {
                    textarea {
                        class: "{input_cls}",
                        name: name_for_input.clone(),
                        id: id_for.clone(),
                        placeholder: placeholder_str.clone(),
                        required: required,
                        disabled: disabled,
                        rows: rows.unwrap_or(4),
                        value: value_for_input.clone(),
                        "aria-invalid": if invalid { "true" } else { "false" },
                        oninput: move |e| {
                            if !is_controlled {
                                internal_value.set(e.value().to_string());
                            }
                            if let Some(h) = &oninput { h.call(e); }
                        },
                        onchange: move |e| if let Some(h) = &onchange { h.call(e); },
                    }
                } else {
                    input {
                        class: "{input_cls}",
                        r#type: "{type_str}",
                        name: name_for_input.clone(),
                        id: id_for.clone(),
                        placeholder: placeholder_str.clone(),
                        required: required,
                        disabled: disabled,
                        value: value_for_input.clone(),
                        "aria-invalid": if invalid { "true" } else { "false" },
                        oninput: move |e| {
                            if !is_controlled {
                                internal_value.set(e.value().to_string());
                            }
                            if let Some(h) = &oninput { h.call(e); }
                        },
                        onchange: move |e| if let Some(h) = &onchange { h.call(e); },
                    }
                }
            }
            if let Some(h) = &help {
                p { class: "form-help text-xs text-muted-foreground mt-1", "{h}" }
            }
            if let Some(e) = &error {
                p { class: "form-error text-xs text-red-500 mt-1", "{e}" }
            }
        }
    }
}
