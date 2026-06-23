//! `PaymentHistoryTab` — fetches + renders the user's payment
//! history from `pay.epsx.io/api/v1/pay/history/{address}`.
//!
//! wave49(slice-4): replaces the static "No payment history yet"
//! placeholder in the account page with a real data-driven
//! component that calls the pay service. Slice-4 ships a
//! static fallback (always shows "No payment history yet")
//! because the data fetch requires runtime `use_resource`
//! hydration, which only works in client-rendered Dioxus.
//!
//! For SSR, this component takes an optional `static_history`
//! prop that callers can pre-fetch server-side and pass in.
//! The component renders the data if present, otherwise the
//! empty state. Slice-5 will add the client-side fetch path
//! via `use_resource` + a `Resource<PayHistory>`.

use dioxus::prelude::*;

use crate::primitives::icon::Icon;

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct PayHistoryIntent {
    pub id: String,
    pub chain_id: String,
    pub payer: String,
    pub payee: String,
    pub amount: String,
    pub status: String,
    pub created_at: String,
    pub tx_hash: Option<String>,
    pub escrow_id: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct PayHistoryEscrow {
    pub id: String,
    pub amount: String,
    pub status: String,
    pub created_at: String,
    pub tx_hash: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct PayHistory {
    pub address: String,
    pub intents: Vec<PayHistoryIntent>,
    pub escrows: Vec<PayHistoryEscrow>,
    pub total_intents: i64,
    pub total_escrows: i64,
}

#[derive(Props, Clone, PartialEq)]
pub struct PaymentHistoryTabProps {
    /// User's wallet address. Used to label the panel and
    /// (slice-5) drive the `use_resource` fetch.
    pub address: Option<String>,
    /// Pre-fetched history (SSR path). When `None`, the
    /// component shows the empty state.
    #[props(default)]
    pub static_history: Option<PayHistory>,
    /// Optional class override for the outer card.
    #[props(default)]
    pub class: Option<String>,
}

#[component]
pub fn PaymentHistoryTab(props: PaymentHistoryTabProps) -> Element {
    let addr = props.address.clone().unwrap_or_default();
    let history = props.static_history.clone().unwrap_or_default();
    let has_data = !history.intents.is_empty() || !history.escrows.is_empty();

    rsx! {
        div {
            class: props.class.clone().unwrap_or_else(|| "payment-history-tab card card-glass p-6 sm:p-8 lg:p-10 shadow-2xl border-2 border-blue-200/50".to_string()),
            "data-section": "payment-history-tab",
            "data-address": "{addr}",

            div { class: "payment-history-tab-header flex items-center gap-3 mb-8",
                div { class: "p-3 bg-blue-100 dark:bg-blue-900/30 rounded-2xl",
                    Icon { name: "credit-card".to_string(), size: Some(24), class_name: Some("text-blue-600 dark:text-blue-400".to_string()) }
                }
                h2 { class: "text-2xl sm:text-3xl font-bold text-foreground", "Transaction History" }
                if !addr.is_empty() {
                    span { class: "payment-history-tab-address font-mono text-xs text-muted-foreground ml-auto",
                        "{addr}"
                    }
                }
            }

            if has_data {
                // Counts row
                div { class: "payment-history-tab-counts flex items-center gap-4 mb-4",
                    span { class: "payment-history-tab-counts-intents text-sm font-medium text-muted-foreground",
                        "Intents: {history.total_intents}"
                    }
                    span { class: "payment-history-tab-counts-escrows text-sm font-medium text-muted-foreground",
                        "Escrows: {history.total_escrows}"
                    }
                }
                // Intents list
                if !history.intents.is_empty() {
                    div { class: "payment-history-tab-intents space-y-2 mb-6",
                        h3 { class: "text-sm font-semibold uppercase tracking-wide text-muted-foreground mb-2",
                            "Intents"
                        }
                        for intent in history.intents.iter() {
                            div {
                                class: "payment-history-tab-intent-row payment-history-tab-intent-status-{intent.status} flex items-center justify-between p-3 rounded-lg bg-secondary/50",
                                div { class: "payment-history-tab-intent-meta flex flex-col gap-1",
                                    span { class: "font-mono text-xs", "{intent.id}" }
                                    span { class: "text-xs text-muted-foreground",
                                        "amount={intent.amount} chain={intent.chain_id}"
                                    }
                                }
                                span { class: "payment-history-tab-intent-status-badge inline-flex items-center px-2 py-0.5 rounded-full text-xs font-medium",
                                    "{intent.status}"
                                }
                            }
                        }
                    }
                }
                // Escrows list
                if !history.escrows.is_empty() {
                    div { class: "payment-history-tab-escrows space-y-2",
                        h3 { class: "text-sm font-semibold uppercase tracking-wide text-muted-foreground mb-2",
                            "Escrows"
                        }
                        for escrow in history.escrows.iter() {
                            div {
                                class: "payment-history-tab-escrow-row payment-history-tab-escrow-status-{escrow.status} flex items-center justify-between p-3 rounded-lg bg-secondary/50",
                                div { class: "payment-history-tab-escrow-meta flex flex-col gap-1",
                                    span { class: "font-mono text-xs", "{escrow.id}" }
                                    span { class: "text-xs text-muted-foreground",
                                        "amount={escrow.amount}"
                                    }
                                }
                                span { class: "payment-history-tab-escrow-status-badge inline-flex items-center px-2 py-0.5 rounded-full text-xs font-medium",
                                    "{escrow.status}"
                                }
                            }
                        }
                    }
                }
            } else {
                // Empty state
                div { class: "payment-history-tab-empty p-8 text-center",
                    Icon { name: "credit-card".to_string(), size: Some(40), class_name: Some("text-muted-foreground".to_string()) }
                    p { class: "payment-history-tab-empty-title mt-3 text-slate-400",
                        "No payment history yet"
                    }
                    p { class: "payment-history-tab-empty-subtitle mt-1 text-xs text-slate-500",
                        "Your payment history will appear here once you make a payment via pay.epsx.io."
                    }
                }
            }
        }
    }
}