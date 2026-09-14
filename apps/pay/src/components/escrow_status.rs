//! `PayEscrowStatus` — `/intent/:id` page for pay.epsx.io.
//!
//! Live polling is owned by the generated Rust/WASM browser runtime. The
//! server-rendered page exposes a same-origin status endpoint through data
//! attributes; no executable script is embedded in the document.
//!
//! | upstream status | rendered stepper           | chain card                |
//! |-----------------|-----------------------------|---------------------------|
//! | `pending`       | ConnectWallet               | Idle                      |
//! | `escrowed`      | SubmitPayment (active)      | Confirming { 1/12 }       |
//! | `released`      | Complete (success badge)    | Verified (green check)     |
//! | `refunded`      | Complete (warning badge)    | Failed { "refunded" }      |
//! | `cancelled`     | Complete (muted badge)      | Failed { "cancelled" }    |
//!
//! The status text + colored dot are rendered server-side from a static
//! `pending` placeholder and progressively updated by Rust/WASM.

use dioxus::prelude::*;

use epsx_dioxus_ui::payment::{
    ChainVerificationCard, ChainVerificationStatus, PaymentFlowStep, PaymentFlowSteps,
    PaymentFlowStepsState,
};

#[component]
pub fn PayEscrowStatus(intent_id: String) -> Element {
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
                    "Intent: "
                    span { id: "pay-escrow-intent-id-value", "{intent_id.clone()}" }
                }
                // Status pill — the JS poll mutates the
                // `data-status` attribute + text content of the
                // status label every 5s. SSR renders "pending".
                div {
                    id: "pay-escrow-status-pill",
                    class: "pay-escrow-status-pill inline-flex items-center gap-2 px-3 py-1 rounded-full bg-secondary/50 mb-4",
                    "data-status": "pending",
                    "data-payment-status-endpoint": format!("/api/v1/pay/intent/{intent_id}"),
                    span { id: "pay-escrow-status-dot",
                        class: "pay-escrow-status-dot h-2 w-2 rounded-full bg-orange-500"
                    }
                    span { id: "pay-escrow-status-label",
                        class: "pay-escrow-status-label text-sm font-medium text-orange-500",
                        "pending"
                    }
                }
                // Stepper — the JS poll mutates the inner
                // data-active attribute on each step.
                div { id: "pay-escrow-flow-steps",
                    PaymentFlowSteps {
                        state: PaymentFlowStepsState {
                            current: PaymentFlowStep::ConnectWallet,
                            connect_wallet_done: false,
                            approve_token_done: false,
                            submit_payment_done: false,
                            error: None,
                        }
                    }
                }
                // Chain verification card — the JS poll mutates
                // the status enum via class swaps on the inner
                // icon + label.
                div { id: "pay-escrow-chain-card",
                    ChainVerificationCard {
                        status: ChainVerificationStatus::Idle,
                        tx_hash: None,
                        network: "BNB Smart Chain".to_string(),
                    }
                }
                // Polling indicator (visible while a poll is in
                // flight — JS adds/removes the .is-loading class).
                p { id: "pay-escrow-polling-indicator",
                    class: "pay-escrow-polling-indicator text-xs text-muted-foreground text-center mt-2 hidden",
                    "Polling pay.epsx.io every 5s..."
                }
            }
        }
    }
}
