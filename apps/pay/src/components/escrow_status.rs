//! `PayEscrowStatus` — `/intent/:id` page for pay.epsx.io.
//!
//! wave49(slice-2): shows the live status of a payment intent.
//! Slice-3 will add the polling loop (calls
//! `GET /api/v1/pay/intents/{id}` every 5s) and feed status
//! changes into the `ChainVerificationCard` + `PaymentFlowSteps`.
//!
//! For slice-2 this is a static SSR shell — the page renders
//! with the `intent_id` in the URL + a placeholder "pending"
//! status. Slice-3 will add:
//!   - async data fetch via `Resource` / `use_future`
//!   - on-chain confirmations via `ChainVerificationCard`
//!   - success redirect when status transitions to "released"

use dioxus::prelude::*;

use epsx_dioxus_ui::payment::{
    ChainVerificationCard, ChainVerificationStatus, PaymentFlowSteps,
    PaymentFlowStep, PaymentFlowStepsState,
};

#[component]
pub fn PayEscrowStatus(intent_id: String) -> Element {
    // Slice-2 placeholder state. Slice-3 will replace with
    // a polled `use_resource` call to the pay service.
    let flow_state = PaymentFlowStepsState {
        current: PaymentFlowStep::ConnectWallet,
        connect_wallet_done: false,
        approve_token_done: false,
        submit_payment_done: false,
        error: None,
    };
    rsx! {
        div { class: "pay-escrow-page page-bg",
            section { class: "section",
                style: "max-width:32rem;margin:0 auto;",
                h1 { class: "pay-escrow-title",
                    style: "font-size:1.5rem;font-weight:800;margin-bottom:1rem;",
                    "Payment status"
                }
                p { class: "pay-escrow-intent-id",
                    style: "font-family:monospace;font-size:0.75rem;color:var(--text-subtle);margin-bottom:1.5rem;word-break:break-all;",
                    "Intent: {intent_id}"
                }
                // Stepper
                PaymentFlowSteps { state: flow_state }
                // Chain verification card (slice-3 will fill in real tx hash + confirmations)
                ChainVerificationCard {
                    status: ChainVerificationStatus::Idle,
                    tx_hash: None,
                    network: "BNB Smart Chain".to_string(),
                }
                // Slice-2 note — replaced in slice-3 with a polled live status
                p { class: "pay-escrow-slicenote",
                    style: "font-size:0.75rem;color:var(--text-subtle);text-align:center;margin-top:2rem;",
                    "Live polling arrives in slice-3. For now this page renders the static stepper."
                }
            }
        }
    }
}