//! `UnifiedPaymentFlow` — full payment flow component (combines
//! steps + connect modal + confirmation).
//!
//! Port of
//! `apps-old/frontend/components/payment/unified-payment-flow.tsx`
//! (252 LoC). The TS source is a client component that combines
//! the stepper + connect modal + confirm dialog. The Dioxus port
//! renders the same shell as a static element with the data
//! provided by the caller.

use crate::payment::payment_flow_steps::{PaymentFlowStep, PaymentFlowSteps, PaymentFlowStepsState};

use dioxus::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum UnifiedPaymentStep {
    #[default]
    Idle,
    ConnectWallet,
    SelectPlan,
    Approve,
    Pay,
    Done,
}

#[component]
pub fn UnifiedPaymentFlow(
    /// Current step in the unified flow.
    step: UnifiedPaymentStep,
    /// Optional error message.
    #[props(default = None)] error: Option<String>,
) -> Element {
    let flow_state = match step {
        UnifiedPaymentStep::Idle => PaymentFlowStepsState::default(),
        UnifiedPaymentStep::ConnectWallet => PaymentFlowStepsState {
            current: PaymentFlowStep::ConnectWallet,
            ..Default::default()
        },
        UnifiedPaymentStep::SelectPlan => PaymentFlowStepsState {
            current: PaymentFlowStep::ApproveToken,
            connect_wallet_done: true,
            ..Default::default()
        },
        UnifiedPaymentStep::Approve => PaymentFlowStepsState {
            current: PaymentFlowStep::ApproveToken,
            connect_wallet_done: true,
            ..Default::default()
        },
        UnifiedPaymentStep::Pay => PaymentFlowStepsState {
            current: PaymentFlowStep::SubmitPayment,
            connect_wallet_done: true,
            approve_token_done: true,
            ..Default::default()
        },
        UnifiedPaymentStep::Done => PaymentFlowStepsState {
            current: PaymentFlowStep::Complete,
            connect_wallet_done: true,
            approve_token_done: true,
            submit_payment_done: true,
            ..Default::default()
        },
    };
    rsx! {
        div { class: "unified-payment-flow card card-glass p-6",
            div { class: "unified-payment-flow-header mb-4",
                h2 { class: "text-lg font-bold", "Complete your payment" }
                p { class: "text-sm text-slate-500", "Follow the steps below to upgrade your plan." }
            }
            PaymentFlowSteps { state: PaymentFlowStepsState { error, ..flow_state } }
            div { class: "unified-payment-flow-cta mt-6",
                button { class: "w-full px-4 py-2 bg-orange-500 text-white rounded-lg font-bold hover:bg-orange-600",
                    "Continue"
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unified_payment_step_default_is_idle() {
        let s = UnifiedPaymentStep::default();
        assert_eq!(s, UnifiedPaymentStep::Idle);
    }

    #[test]
    fn unified_payment_flow_smoke() {
        
    }
}
