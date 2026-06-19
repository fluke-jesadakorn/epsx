//! `PaymentFlowSteps` — 3-step progress indicator for the
//! payment flow.
//!
//! Port of
//! `apps-old/frontend/components/payment/payment-flow-steps.tsx`
//! (783 LoC). The TS source renders a numbered stepper with
//! "Connect Wallet / Approve / Pay" steps and per-step state
//! (pending / active / done / error). The Dioxus port renders
//! the same visual structure with a `PaymentFlowStepsState` data
//! prop.

use crate::primitives::icon::Icon;

use dioxus::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum PaymentFlowStep {
    #[default]
    ConnectWallet,
    ApproveToken,
    SubmitPayment,
    Complete,
}

#[derive(Clone, PartialEq, Debug, Default)]
pub struct PaymentFlowStepsState {
    pub current: PaymentFlowStep,
    pub connect_wallet_done: bool,
    pub approve_token_done: bool,
    pub submit_payment_done: bool,
    pub error: Option<String>,
}

#[component]
pub fn PaymentFlowSteps(state: PaymentFlowStepsState) -> Element {
    rsx! {
        div { class: "payment-flow-steps",
            ol { class: "payment-flow-steps-list flex items-center justify-between gap-2",
                Step {
                    label: "Connect Wallet",
                    done: state.connect_wallet_done,
                    active: matches!(state.current, PaymentFlowStep::ConnectWallet),
                }
                Step {
                    label: "Approve",
                    done: state.approve_token_done,
                    active: matches!(state.current, PaymentFlowStep::ApproveToken),
                }
                Step {
                    label: "Pay",
                    done: state.submit_payment_done,
                    active: matches!(state.current, PaymentFlowStep::SubmitPayment),
                }
                Step {
                    label: "Done",
                    done: matches!(state.current, PaymentFlowStep::Complete),
                    active: matches!(state.current, PaymentFlowStep::Complete),
                }
            }
            if let Some(err) = state.error.as_ref() {
                div { class: "payment-flow-steps-error mt-3 p-3 bg-red-500/10 border border-red-500/20 rounded-lg text-sm text-red-500",
                    "{err}"
                }
            }
        }
    }
}

#[component]
fn Step(label: String, done: bool, active: bool) -> Element {
    let (icon, color) = if done {
        ("check-circle", "green")
    } else if active {
        ("loader", "orange")
    } else {
        ("circle", "slate")
    };
    rsx! {
        li {
            class: if active { format!("payment-flow-step payment-flow-step-active flex items-center gap-2 text-{color}-500 font-bold") } else { format!("payment-flow-step flex items-center gap-2 text-{color}-500") },
            Icon { name: icon.to_string(), size: Some(20) }
            span { class: "text-sm", "{label}" }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn payment_flow_step_default_is_connect_wallet() {
        let s = PaymentFlowStep::default();
        assert_eq!(s, PaymentFlowStep::ConnectWallet);
    }

    #[test]
    fn payment_flow_steps_smoke() {
        
    }
}
