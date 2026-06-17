//! Stepper / Step — multi-step flow (payment, plan create, etc.)
//!
//! Two stepper variants are provided:
//! - **Tuple-based** (`Stepper { steps: Vec<(String, bool)>, current }`) —
//!   matches the original API used by `pages/payment.rs`. Each step is a
//!   `(label, complete)` pair. The optional `progress: bool` flag renders a
//!   linear progress bar above the row of step circles.
//! - **Struct-based** (`StepperSteps { steps: Vec<Step>, current }`) — richer
//!   shape that includes an optional Lucide `icon` next to the step number.

use dioxus::prelude::*;

/// One step in a struct-based stepper.
#[derive(Clone, Debug, PartialEq)]
pub struct Step {
    pub label: String,
    pub complete: bool,
    /// Optional Lucide icon name. Rendered next to the step number.
    pub icon: Option<String>,
}

impl Step {
    /// Convenience constructor for `Step { label, complete: false, icon: None }`.
    pub fn new(label: impl Into<String>) -> Self {
        Self { label: label.into(), complete: false, icon: None }
    }
    pub fn complete(mut self) -> Self { self.complete = true; self }
    pub fn with_icon(mut self, icon: impl Into<String>) -> Self { self.icon = Some(icon.into()); self }
}

/// Tuple-based stepper (preserved for the existing API used by `pages/payment.rs`).
///
/// Renders a horizontal row of step circles connected by lines. The current
/// step is highlighted. The optional `progress: bool` flag (default `false`)
/// also renders a linear progress bar above the row, showing the fraction
/// of completed steps.
#[component]
pub fn Stepper(
    steps: Vec<(String, bool)>,
    current: usize,
    #[props(default = false)] progress: bool,
) -> Element {
    let total = steps.len();
    let progress_pct = if total == 0 {
        0u32
    } else {
        let completed = steps.iter().filter(|(_, c)| *c).count()
            + if current < total && steps.get(current).map(|(_, c)| !*c).unwrap_or(false) { 1 } else { 0 };
        ((completed as f64 / total as f64) * 100.0).round() as u32
    };
    rsx! {
        div { class: "stepper-wrap",
            if progress {
                div {
                    class: "stepper-progress progress",
                    role: "progressbar",
                    "aria-valuenow": progress_pct.to_string(),
                    "aria-valuemin": "0",
                    "aria-valuemax": "100",
                    div { class: "progress-bar", style: format!("width: {}%", progress_pct) }
                }
            }
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
}

/// Struct-based stepper with optional per-step icon.
#[component]
pub fn StepperSteps(steps: Vec<Step>, current: usize) -> Element {
    let total = steps.len();
    let progress_pct = if total == 0 {
        0u32
    } else {
        let completed = steps.iter().filter(|s| s.complete).count()
            + if current < total && steps.get(current).map(|s| !s.complete).unwrap_or(false) { 1 } else { 0 };
        ((completed as f64 / total as f64) * 100.0).round() as u32
    };
    rsx! {
        div { class: "stepper-wrap",
            div {
                class: "stepper-progress progress",
                role: "progressbar",
                "aria-valuenow": progress_pct.to_string(),
                "aria-valuemin": "0",
                "aria-valuemax": "100",
                div { class: "progress-bar", style: format!("width: {}%", progress_pct) }
            }
            div { class: "stepper flex items-center",
                for (i, step) in steps.iter().enumerate() {
                    div {
                        class: if i == current { "step step-active" } else if step.complete { "step step-complete" } else { "step" },
                        div { class: "step-circle flex items-center justify-center",
                            if let Some(icon) = &step.icon {
                                super::icon::Icon { name: icon.clone(), size: Some(14) }
                            } else {
                                span { "{i + 1}" }
                            }
                        }
                        span { class: "step-label ml-2", "{step.label}" }
                    }
                    if i + 1 < steps.len() {
                        div { class: "step-line" }
                    }
                }
            }
        }
    }
}

/// Panel for one step's content. Renders a title + optional description +
/// the body children.
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

/// Back / Next navigation row for a stepper.
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
