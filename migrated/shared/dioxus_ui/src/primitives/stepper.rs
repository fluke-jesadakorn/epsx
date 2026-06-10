//! Stepper / Step — multi-step flow (payment, plan create, etc.)

use dioxus::prelude::*;

#[component]
pub fn Stepper(steps: Vec<(String, bool)>, current: usize) -> Element {
    rsx! {
        div { class: "stepper flex items-center",
            for (i, (label, complete)) in steps.iter().enumerate() {
                div {
                    class: if i == current { "step step-active" } else if *complete { "step step-complete" } else { "step" },
                    div { class: "step-circle", "{i + 1}" }
                    span { class: "step-label ml-2", "{label}" }
                }
                if i + 1 < steps.len() {
                    div { class: "step-line" }
                }
            }
        }
    }
}

#[component]
pub fn StepPanel(title: String, description: Option<String>, children: Element) -> Element {
    rsx! {
        div { class: "step-panel",
            h2 { class: "step-title", "{title}" }
            if let Some(d) = description {
                p { class: "step-description text-muted-foreground", "{d}" }
            }
            div { class: "step-body mt-4", {children} }
        }
    }
}

#[component]
pub fn StepNavigation(
    on_back: Option<EventHandler<MouseEvent>>,
    on_next: Option<EventHandler<MouseEvent>>,
    #[props(default = "Back".to_string())] back_label: String,
    #[props(default = "Next".to_string())] next_label: String,
    #[props(default = false)] hide_back: bool,
    #[props(default = false)] hide_next: bool,
) -> Element {
    rsx! {
        div { class: "step-nav flex justify-between mt-6",
            div {
                if !hide_back {
                    crate::primitives::button::Button { kind: crate::primitives::button::ButtonKind::Outline, onclick: on_back,
                        "{back_label}"
                    }
                }
            }
            div {
                if !hide_next {
                    crate::primitives::button::Button { kind: crate::primitives::button::ButtonKind::Primary, onclick: on_next,
                        "{next_label}"
                    }
                }
            }
        }
    }
}
