//! `PayCheckoutForm` — main `/checkout` page for pay.epsx.io.
//!
//! wave49(slice-2): replaces the inline `pay_checkout_body()`
//! HTML string from the previous apps/pay/src/main.rs. The page
//! composes 3 ported payment components:
//!
//!   - `UpgradeBanner` — "Upgrade to {plan}" full-bleed CTA
//!   - `CurrentAccessCard` — "Your current access" status
//!   - `UnifiedPaymentFlow` — stepper + Continue button
//!
//! Plus 2 ported primitives from `epsx_dioxus_ui::primitives`:
//!
//!   - `Card` — the outer card wrapping the form
//!   - `Button` — the primary CTA
//!
//! The page accepts a `PaymentWizardState` prop so the BFF SSR
//! fallback can pass URL params + initial step without round-trip
//! through the wallet. Slice-3 will hydrate this state on the
//! client (via `use_signal` + `use_effect`) so the user can
//! click "Continue" to advance the stepper.

use dioxus::prelude::*;

use epsx_dioxus_ui::payment::{
    CurrentAccessCard, CurrentAccessInfo, UnifiedPaymentFlow,
    UpgradeBanner, UpgradeUrgency,
};

use crate::state::payment_wizard_state::PaymentWizardState;

#[component]
pub fn PayCheckoutForm(state: PaymentWizardState) -> Element {
    // Derive display values from the URL params (with safe defaults).
    let amount = state.params.amount.clone().unwrap_or_else(|| "0.00".to_string());
    let currency = state.params.currency.clone().unwrap_or_else(|| "USDT".to_string());
    let chain_label = match state.params.chain_id.as_deref() {
        Some("56") | None => "BSC (BEP-20)",
        Some("97") => "BSC Testnet",
        Some(other) => other,
    };
    let description = state.params.description.clone()
        .unwrap_or_else(|| "Complete your payment on BSC".to_string());

    // Map the wizard state into the visual stepper.
    let unified_step = state.unified_step();
    let error_msg = match &state.step {
        crate::state::payment_wizard_state::WizardStep::Failed { reason } => Some(reason.clone()),
        _ => None,
    };

    rsx! {
        div { class: "pay-checkout-page page-bg",
            section { class: "section",
                // Top: UpgradeBanner CTA (matches prod)
                UpgradeBanner {
                    plan_name: "EPSX Pay".to_string(),
                    urgency: UpgradeUrgency::Medium,
                    message: Some("Complete your payment securely via BSC escrow.".to_string()),
                    href: "/".to_string(),
                }
                // Body: card with amount + chain + CTA
                div { class: "pay-checkout-card-wrap",
                    style: "display:flex;align-items:center;justify-content:center;margin-top:2rem;",
                    div {
                        class: "pay-checkout-card card card-glass",
                        style: "width:100%;max-width:28rem;padding:2.5rem;",
                        div { class: "pay-checkout-header",
                            style: "text-align:center;margin-bottom:2rem;",
                            span { class: "pay-checkout-badge badge-pill",
                                style: "display:inline-flex;align-items:center;gap:0.5rem;",
                                i { class: "epsx-icon", dangerous_inner_html: epsx_templates::lucide("credit-card", "16", "var(--epsx-orange)") }
                                " EPSX Pay"
                            }
                            h1 { class: "pay-checkout-title",
                                style: "font-size:1.75rem;font-weight:800;margin-top:0.75rem;",
                                "Complete Payment"
                            }
                            p { class: "pay-checkout-subtitle",
                                style: "color:var(--text-muted);font-size:0.875rem;margin-top:0.25rem;",
                                "{description}"
                            }
                        }
                        // Amount + chain summary
                        div { class: "pay-checkout-summary",
                            style: "background:var(--bg-secondary);border-radius:0.75rem;padding:1.25rem;margin-bottom:1.5rem;",
                            div { class: "pay-checkout-amount-row",
                                style: "display:flex;justify-content:space-between;align-items:center;margin-bottom:0.75rem;",
                                span { class: "pay-checkout-amount-label",
                                    style: "font-size:0.875rem;color:var(--text-muted);",
                                    "Amount"
                                }
                                span { class: "pay-checkout-amount-value gradient-text",
                                    style: "font-size:1.5rem;font-weight:800;",
                                    "{amount} {currency}"
                                }
                            }
                            div { class: "pay-checkout-chain-row",
                                style: "display:flex;justify-content:space-between;align-items:center;",
                                span { class: "pay-checkout-chain-label",
                                    style: "font-size:0.875rem;color:var(--text-muted);",
                                    "Chain"
                                }
                                span { class: "pay-checkout-chain-value",
                                    style: "font-size:0.875rem;font-weight:500;display:inline-flex;align-items:center;gap:0.25rem;",
                                    i { class: "epsx-icon", dangerous_inner_html: epsx_templates::lucide("link", "16", "var(--epsx-orange)") }
                                    "{chain_label}"
                                }
                            }
                        }
                        // UnifiedPaymentFlow — the stepper + Continue
                        UnifiedPaymentFlow {
                            step: unified_step,
                            error: error_msg,
                        }
                        // Footer: secured by escrow
                        p { class: "pay-checkout-footer",
                            style: "font-size:0.75rem;color:var(--text-subtle);text-align:center;margin-top:1rem;display:inline-flex;align-items:center;gap:0.5rem;justify-content:center;width:100%;",
                            i { class: "epsx-icon", dangerous_inner_html: epsx_templates::lucide("shield", "16", "var(--epsx-green)") }
                            " Secured by EPSX escrow"
                        }
                    }
                }
                // Bottom: CurrentAccessCard (matches prod layout — shows current plan status)
                div { class: "pay-checkout-current-access-wrap",
                    style: "max-width:28rem;margin:2rem auto 0;",
                    CurrentAccessCard {
                        info: CurrentAccessInfo {
                            plan_name: "EPSX Pay".to_string(),
                            tier: "Pay-as-you-go".to_string(),
                            expires_at: None,
                            renews_at: None,
                            is_active: true,
                        }
                    }
                }
            }
        }
    }
}