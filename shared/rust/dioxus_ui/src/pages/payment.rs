//! `/payment` and `/payment/plan/:id` checkout surfaces.
//!
//! Pricing and entitlement inputs are rendered only from the backend-owned
//! public plan projection. The browser can submit only the selected plan ID
//! and a wallet-produced transaction hash; confirmation remains server-owned.

use super::{plans::PublicPlan, PageContext, PageMeta};
use crate::auth::GlobalAuthGuard;
use crate::layout::main_layout::MainLayout;
use crate::primitives::Icon;
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

pub const PAYMENT_CHECKOUT_DATA_PARAM: &str = "data_payment_checkout";

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct PlanCheckoutData {
    pub plan: PublicPlan,
    pub chain_id: u64,
    pub network: String,
    pub token_address: String,
    pub receiver_address: String,
    pub token_decimals: u8,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum PaymentCheckoutLoadOutcome {
    Ready { checkout: PlanCheckoutData },
    NotFound,
    Error { code: String },
}

pub fn render(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::app("Payment");
    (
        meta,
        rsx! { PaymentPage { ctx: ctx.clone(), outcome: None } },
    )
}

pub fn render_dynamic(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::app("Checkout");
    let outcome = ctx
        .param(PAYMENT_CHECKOUT_DATA_PARAM)
        .and_then(|value| serde_json::from_str::<PaymentCheckoutLoadOutcome>(value).ok());
    (meta, rsx! { PaymentPage { ctx: ctx.clone(), outcome } })
}

#[component]
fn PaymentPage(ctx: PageContext, outcome: Option<PaymentCheckoutLoadOutcome>) -> Element {
    let user_authenticated = ctx.user.is_some();
    rsx! {
        MainLayout { ctx: ctx.clone(),
            if !user_authenticated {
                PaymentAuthRequiredContent {}
            } else if let Some(outcome) = outcome {
                match outcome {
                    PaymentCheckoutLoadOutcome::Ready { checkout } => rsx! {
                        CheckoutContent {
                            checkout,
                            session_wallet: ctx.user.as_ref().map(|user| user.address.clone()).unwrap_or_default()
                        }
                    },
                    PaymentCheckoutLoadOutcome::NotFound => rsx! {
                        CheckoutErrorContent {
                            title: "Plan not found".to_string(),
                            body: "This plan is no longer available. Choose another current plan.".to_string()
                        }
                    },
                    PaymentCheckoutLoadOutcome::Error { .. } => rsx! {
                        CheckoutErrorContent {
                            title: "Checkout unavailable".to_string(),
                            body: "Payment configuration could not be verified. Please try again shortly.".to_string()
                        }
                    },
                }
            } else {
                PaymentEntryContent {}
            }
        }
    }
}

#[component]
fn PaymentAuthRequiredContent() -> Element {
    rsx! {
        div {
            class: "payment-auth-required relative flex min-h-[70vh] items-center justify-center overflow-hidden bg-gradient-to-br from-purple-50 via-indigo-50 to-blue-50 dark:from-gray-900 dark:via-purple-900/20 dark:to-gray-800",
            "data-payment-state": "auth-required",
            div { class: "relative z-10 mx-auto w-full max-w-6xl p-6",
                GlobalAuthGuard { user_authenticated: false }
            }
        }
    }
}

#[component]
fn PaymentEntryContent() -> Element {
    rsx! {
        div { class: "mx-auto flex min-h-[70vh] max-w-3xl items-center justify-center px-4 py-16",
            section { class: "w-full rounded-3xl border border-slate-200 bg-white p-8 text-center shadow-xl dark:border-slate-700 dark:bg-slate-900",
                Icon { name: "wallet".to_string(), size: Some(44) }
                h1 { class: "mt-4 text-3xl font-black text-slate-900 dark:text-white", "Choose a plan first" }
                p { class: "mt-3 text-slate-600 dark:text-slate-300", "Checkout starts from a current backend-verified plan and price." }
                a { class: "mt-6 inline-flex rounded-xl bg-blue-600 px-6 py-3 font-bold text-white hover:bg-blue-700", href: "/plans", "View plans" }
            }
        }
    }
}

fn first_visible_rank(offset: i32) -> i32 {
    offset.max(0).saturating_add(1)
}

fn duration_label(plan: &PublicPlan) -> String {
    match plan.duration_days {
        None => "Lifetime access".to_string(),
        Some(1) => "1 day of access".to_string(),
        Some(days) => format!("{days} days of access"),
    }
}

fn ranking_limit_label(plan: &PublicPlan) -> String {
    match plan.rankings_limit {
        -1 => "Unlimited ranking inventory".to_string(),
        limit => format!("Up to {limit} ranking results"),
    }
}

fn ranking_range_label(plan: &PublicPlan) -> String {
    let first_rank = first_visible_rank(plan.ranking_offset);
    match plan.rankings_limit {
        -1 => format!("Ranks {first_rank}+"),
        limit => {
            let last_rank = first_rank.saturating_add(limit.max(1)).saturating_sub(1);
            format!("Ranks {first_rank}-{last_rank}")
        }
    }
}

#[component]
fn CheckoutContent(checkout: PlanCheckoutData, session_wallet: String) -> Element {
    let plan = checkout.plan.clone();
    let chain_hex = format!("0x{:x}", checkout.chain_id);
    let billing_cycle = plan.billing_cycle.replace('_', " ");
    rsx! {
        div {
            class: "payment-checkout min-h-screen bg-gradient-to-br from-slate-50 via-violet-50 to-blue-50 px-4 py-12 dark:from-slate-950 dark:via-violet-950/30 dark:to-slate-900 sm:px-6",
            "data-payment-state": "ready",
            div { class: "mx-auto max-w-5xl",
                header { class: "mb-8 text-center",
                    div { class: "mx-auto flex h-16 w-16 items-center justify-center rounded-2xl bg-gradient-to-br from-violet-600 to-blue-600 text-white shadow-lg",
                        Icon { name: "gem".to_string(), size: Some(32) }
                    }
                    h1 { class: "mt-5 text-4xl font-black text-slate-950 dark:text-white", "Complete your plan purchase" }
                    p { class: "mt-2 text-slate-600 dark:text-slate-300", "Confirm one stablecoin transfer in your wallet. Access activates only after on-chain verification." }
                }

                div { class: "grid gap-6 lg:grid-cols-[1.1fr_0.9fr]",
                    section { class: "rounded-3xl border border-slate-200 bg-white p-7 shadow-xl dark:border-slate-700 dark:bg-slate-900",
                        div { class: "flex flex-wrap items-start justify-between gap-4",
                            div {
                                p { class: "text-sm font-bold uppercase tracking-wider text-violet-600", "Selected plan" }
                                h2 { class: "mt-1 text-3xl font-black text-slate-950 dark:text-white", "{plan.name}" }
                                p { class: "mt-1 capitalize text-slate-500", "{billing_cycle}" }
                            }
                            div { class: "text-right",
                                p { class: "text-4xl font-black text-slate-950 dark:text-white", "{plan.checkout_price}" }
                                p { class: "font-bold text-violet-600", "{plan.settlement_currency}" }
                            }
                        }

                        div { class: "mt-6 grid gap-3 sm:grid-cols-3",
                            AccessFact { icon: "bar-chart-2".to_string(), title: ranking_range_label(&plan), body: "Backend-enforced rank range".to_string() }
                            AccessFact { icon: "list".to_string(), title: ranking_limit_label(&plan), body: "Backend-enforced ranking range".to_string() }
                            AccessFact { icon: "clock".to_string(), title: duration_label(&plan), body: "Starts after confirmation".to_string() }
                        }

                        ul { class: "mt-6 grid gap-3 text-sm text-slate-700 dark:text-slate-200 sm:grid-cols-2",
                            for feature in plan.features.iter().take(8) {
                                li { class: "flex items-start gap-2",
                                    span { class: "mt-0.5 text-emerald-500", "✓" }
                                    span { "{feature}" }
                                }
                            }
                        }
                    }

                    aside { class: "rounded-3xl border border-slate-200 bg-white p-7 shadow-xl dark:border-slate-700 dark:bg-slate-900",
                        h2 { class: "text-xl font-black text-slate-950 dark:text-white", "Pay with MetaMask" }
                        div { class: "mt-5 space-y-3 rounded-2xl bg-slate-50 p-4 text-sm dark:bg-slate-800",
                            PaymentDetail { label: "Network".to_string(), value: checkout.network.clone() }
                            PaymentDetail { label: "Wallet".to_string(), value: abbreviated(&session_wallet) }
                            PaymentDetail { label: "Token".to_string(), value: plan.settlement_currency.clone() }
                            PaymentDetail { label: "Amount".to_string(), value: plan.checkout_price.clone() }
                        }

                        button {
                            class: "mt-6 inline-flex w-full items-center justify-center gap-2 rounded-xl bg-gradient-to-r from-violet-600 to-blue-600 px-5 py-4 text-base font-black text-white shadow-lg transition hover:from-violet-700 hover:to-blue-700 disabled:cursor-not-allowed disabled:opacity-60",
                            r#type: "button",
                            "data-action": "submit-plan-payment",
                            "data-plan-id": plan.id.clone(),
                            "data-amount": plan.checkout_price.clone(),
                            "data-currency": plan.settlement_currency.clone(),
                            "data-chain-id": "{checkout.chain_id}",
                            "data-chain-hex": chain_hex,
                            "data-network": checkout.network.clone(),
                            "data-token-address": checkout.token_address.clone(),
                            "data-receiver-address": checkout.receiver_address.clone(),
                            "data-token-decimals": "{checkout.token_decimals}",
                            "data-session-wallet": session_wallet,
                            Icon { name: "wallet".to_string(), size: Some(20) }
                            span { "Confirm {plan.checkout_price} {plan.settlement_currency}" }
                        }
                        div {
                            id: "plan-payment-status",
                            class: "mt-4 min-h-12 rounded-xl bg-blue-50 px-4 py-3 text-sm text-blue-800 dark:bg-blue-950/50 dark:text-blue-200",
                            role: "status",
                            "aria-live": "polite",
                            "data-epsx-runtime-status": "",
                            "Review the amount and confirm the transfer in MetaMask."
                        }
                        a { class: "mt-4 inline-flex w-full items-center justify-center rounded-xl px-4 py-2 font-semibold text-slate-600 hover:bg-slate-100 dark:text-slate-300 dark:hover:bg-slate-800", href: "/plans", "Choose another plan" }
                    }
                }

                section { class: "mt-6 grid gap-3 rounded-2xl border border-slate-200 bg-white/80 p-5 text-sm text-slate-600 dark:border-slate-700 dark:bg-slate-900/80 dark:text-slate-300 sm:grid-cols-3",
                    PaymentBoundaryItem { icon: "shield".to_string(), title: "Verified amount".to_string(), body: "The BFF reloads the current plan price before submission.".to_string() }
                    PaymentBoundaryItem { icon: "link".to_string(), title: "On-chain proof".to_string(), body: "The backend validates sender, recipient, token, amount, and confirmations.".to_string() }
                    PaymentBoundaryItem { icon: "check-circle".to_string(), title: "Real entitlement".to_string(), body: "Confirmation writes the plan assignment used by ranking authorization.".to_string() }
                }
            }
        }
    }
}

fn abbreviated(value: &str) -> String {
    if value.len() == 42 {
        format!("{}…{}", &value[..8], &value[36..])
    } else {
        value.to_string()
    }
}

#[component]
fn PaymentDetail(label: String, value: String) -> Element {
    rsx! {
        div { class: "flex items-center justify-between gap-3",
            span { class: "text-slate-500 dark:text-slate-400", "{label}" }
            span { class: "max-w-[70%] truncate font-bold text-slate-900 dark:text-white", "{value}" }
        }
    }
}

#[component]
fn AccessFact(icon: String, title: String, body: String) -> Element {
    rsx! {
        div { class: "rounded-2xl border border-violet-100 bg-violet-50 p-4 dark:border-violet-900 dark:bg-violet-950/40",
            Icon { name: icon, size: Some(20) }
            p { class: "mt-2 font-black text-slate-900 dark:text-white", "{title}" }
            p { class: "mt-1 text-xs text-slate-500 dark:text-slate-400", "{body}" }
        }
    }
}

#[component]
fn PaymentBoundaryItem(icon: String, title: String, body: String) -> Element {
    rsx! {
        div { class: "flex items-start gap-3",
            span { class: "text-violet-600", Icon { name: icon, size: Some(18) } }
            div {
                p { class: "font-bold text-slate-900 dark:text-white", "{title}" }
                p { class: "mt-1", "{body}" }
            }
        }
    }
}

#[component]
fn CheckoutErrorContent(title: String, body: String) -> Element {
    rsx! {
        div { class: "mx-auto flex min-h-[70vh] max-w-3xl items-center justify-center px-4 py-16",
            section { class: "w-full rounded-3xl border border-red-200 bg-white p-8 text-center shadow-xl dark:border-red-900 dark:bg-slate-900", role: "alert", "data-payment-state": "unavailable",
                Icon { name: "alert-triangle".to_string(), size: Some(44) }
                h1 { class: "mt-4 text-3xl font-black text-slate-950 dark:text-white", "{title}" }
                p { class: "mt-3 text-slate-600 dark:text-slate-300", "{body}" }
                a { class: "mt-6 inline-flex rounded-xl bg-blue-600 px-6 py-3 font-bold text-white", href: "/plans", "Back to plans" }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn plan() -> PublicPlan {
        PublicPlan {
            id: "61a62cbe-3371-41db-bd90-321c53a71e06".into(),
            name: "1 Day Package".into(),
            plan_type: "1_DAY_PACKAGE".into(),
            current_price: "5.00".into(),
            effective_price: 1.0,
            promotion_active: true,
            promotion_status: "active".into(),
            promotion_discount: 80.0,
            promotion_ends_at: None,
            currency: "USD".into(),
            billing_cycle: "one_time".into(),
            features: vec!["24-hour access".into()],
            permissions: vec!["epsx:rankings:offset:5".into()],
            is_active: true,
            tier_level: 0,
            plan_group: "personal".into(),
            ranking_offset: 5,
            rankings_limit: 5,
            checkout_price: "1.00".into(),
            settlement_currency: "USDT".into(),
            duration_days: Some(1),
        }
    }

    fn signed_in_ctx() -> PageContext {
        PageContext {
            path: "/payment/plan/61a62cbe-3371-41db-bd90-321c53a71e06".into(),
            user: Some(crate::auth::User {
                id: "user".into(),
                address: "0x1111111111111111111111111111111111111111".into(),
                chain_id: "31337".into(),
                roles: vec![],
                email: None,
                tier: None,
                permissions: vec![],
                last_login_at: None,
                auth_method: crate::auth::user::AuthMethod::Wallet,
                display_name: None,
            }),
            ..Default::default()
        }
    }

    #[test]
    fn ready_checkout_renders_only_backend_owned_terms() {
        let checkout = PlanCheckoutData {
            plan: plan(),
            chain_id: 31_337,
            network: "localhost".into(),
            token_address: "0x55d398326f99059fF775485246999027B3197955".into(),
            receiver_address: "0x2222222222222222222222222222222222222222".into(),
            token_decimals: 18,
        };
        let html = dioxus_ssr::render_element(rsx! {
            CheckoutContent {
                checkout,
                session_wallet: "0x1111111111111111111111111111111111111111".to_string()
            }
        });
        assert!(html.contains("data-action=\"submit-plan-payment\""));
        assert!(html.contains("Ranks 6-10"));
        assert!(html.contains("1.00"));
        assert!(html.contains("USDT"));
        assert!(html.contains("1 day of access"));
        assert!(!html.contains("expected_amount"));
    }

    #[test]
    fn unauthenticated_checkout_stops_at_auth_guard() {
        let ctx = PageContext {
            path: "/payment/plan/id".into(),
            ..Default::default()
        };
        let (_, element) = render_dynamic(&ctx);
        let html = dioxus_ssr::render_element(element);
        assert!(html.contains("data-payment-state=\"auth-required\""));
        assert!(!html.contains("submit-plan-payment"));
    }

    #[test]
    fn dynamic_checkout_decodes_only_server_load_outcome() {
        let checkout = PlanCheckoutData {
            plan: plan(),
            chain_id: 31_337,
            network: "localhost".into(),
            token_address: "0x55d398326f99059fF775485246999027B3197955".into(),
            receiver_address: "0x2222222222222222222222222222222222222222".into(),
            token_decimals: 18,
        };
        let mut ctx = signed_in_ctx();
        ctx.params.insert(
            PAYMENT_CHECKOUT_DATA_PARAM.into(),
            serde_json::to_string(&PaymentCheckoutLoadOutcome::Ready { checkout }).unwrap(),
        );
        let (_, element) = render_dynamic(&ctx);
        let html = dioxus_ssr::render_element(element);
        assert!(html.contains("Complete your plan purchase"));
        assert!(html.contains("data-payment-state=\"ready\""));
    }
}
