//! Switch primitive — a Radix-style toggle with optional size variants.

use dioxus::prelude::*;

/// Switch size variants. The corresponding CSS class is added to the root
/// element so the design-system CSS can size the track + thumb.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SwitchSize { Sm, Md, Lg }

impl SwitchSize {
    pub fn as_str(self) -> &'static str {
        match self {
            SwitchSize::Sm => "sm",
            SwitchSize::Md => "md",
            SwitchSize::Lg => "lg",
        }
    }
}

impl Default for SwitchSize {
    fn default() -> Self { SwitchSize::Md }
}

/// A toggle switch with optional label and size variant.
#[component]
pub fn Switch(
    checked: bool,
    #[props(default = None)] label: Option<String>,
    #[props(default = None)] disabled: Option<bool>,
    /// Size variant — `Sm`, `Md` (default), or `Lg`.
    #[props(default = None)] size: Option<SwitchSize>,
    #[props(default = None)] name: Option<String>,
    #[props(default = None)] id: Option<String>,
    onchange: Option<EventHandler<FormEvent>>,
) -> Element {
    let size = size.unwrap_or(SwitchSize::Md);
    let size_str = size.as_str();
    let class_with_size = format!("SwitchRoot switch-{} state-{}", size_str, if checked { "checked" } else { "unchecked" });
    rsx! {
        label { class: "{class_with_size}",
            input {
                r#type: "checkbox",
                role: "switch",
                "aria-checked": checked.to_string(),
                class: "SwitchInput",
                checked: checked,
                disabled: disabled.unwrap_or(false),
                name: name.clone(),
                id: id.clone(),
                onchange: move |e| if let Some(h) = &onchange { h.call(e); },
            }
            span { class: "SwitchThumb" }
            if let Some(l) = label {
                span { class: "SwitchLabel", "{l}" }
            }
        }
    }
}
