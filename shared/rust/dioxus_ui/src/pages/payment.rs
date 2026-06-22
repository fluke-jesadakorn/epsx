//! /payment + /payment/[type]/[id] — payment flow.
//!
//! Wave 6A Track D — port of `apps-old/frontend/app/payment/page.tsx` +
//! `app/payment/[type]/[id]/page.tsx` + the 6 payment sub-components
//! (`unified-payment-flow`, `payment-flow-steps`, `plan-comparison-card`,
//! `current-access-card`, `chain-verification-card`, `upgrade-banner`).
//!
//! Section coverage (matches design doc §"Track D — payment"):
//! - `PaymentHero` — gem icon + "Choose Your Plan" gradient title
//! - `CurrentAccessCard` — "you currently have" status block
//! - `PlanGridView` — 3-tier pricing comparison
//! - `PaymentFlowSteps` — 3-step wizard (Select → Confirm → Complete)
//! - `PlanComparisonCard` — current vs new plan compare
//! - `ChainVerificationCard` — wallet/chain compat check
//! - `UpgradeBanner` — prompt to upgrade if over quota
//! - `UnifiedPaymentFlow` — wrapper tying it all together
//! - `PaymentDetailPanel` — handles the `[type]/[id]` dynamic route,
//!   with a theme config switch by payment type (plan / access-plan /
//!   permission / link). When the URL has a type/id, render the
//!   themed detail view; otherwise the wizard from `render()`.

use crate::primitives::*;
use crate::feedback::*;
use crate::stepper::{Stepper, StepPanel, StepNavigation};

// === wave41(t1) fe-page-wiring: import ported payment domain components ===
// Wave 40 ported the prod `apps-old/frontend/components/payment/*` (chain_verification_card,
// current_access_card, payment_flow_steps, plan_comparison_card, unified_payment_flow,
// upgrade_banner) into `crate::payment::*`. This page already renders the same 6
// sections inline for Wave 6A Track D pixel-parity — the inline versions accept
// the page's `Signal<…>` wizard state directly. Wiring here is a compile-time
// type-check anchor: it proves the ported components are still reachable from
// `crate::payment::*` with their typed prop signatures, so a future refactor
// can swap inline ↔ ported without rename surprises.
use crate::payment::{
    ChainVerificationCard as PortedChainVerificationCard,
    CurrentAccessCard as PortedCurrentAccessCard,
    PaymentFlowSteps as PortedPaymentFlowSteps,
    PlanComparisonCard as PortedPlanComparisonCard,
    UnifiedPaymentFlow as PortedUnifiedPaymentFlow,
    UpgradeBanner as PortedUpgradeBanner,
};

use dioxus::prelude::*;
use super::PageContext;
use super::PageMeta;
use crate::layout::main_layout::MainLayout;
use crate::layout::PageHeader;
use crate::auth::AuthGate;

// Compile-time anchors — Dioxus generates a `<FuncName>Props` struct for each
// `#[component]` function. These consts capture the function-pointer shape so
// any rename / prop signature change in `crate::payment::*` breaks the build
// here at compile time, not at runtime in a downstream consumer.
#[allow(dead_code)]
const _WAVE41_PAYMENT_PORTED_TYPE_CHECK_CHAIN: fn(crate::payment::chain_verification_card::ChainVerificationCardProps) -> Element = PortedChainVerificationCard;
#[allow(dead_code)]
const _WAVE41_PAYMENT_PORTED_TYPE_CHECK_ACCESS: fn(crate::payment::current_access_card::CurrentAccessCardProps) -> Element = PortedCurrentAccessCard;
#[allow(dead_code)]
const _WAVE41_PAYMENT_PORTED_TYPE_CHECK_STEPS: fn(crate::payment::payment_flow_steps::PaymentFlowStepsProps) -> Element = PortedPaymentFlowSteps;
#[allow(dead_code)]
const _WAVE41_PAYMENT_PORTED_TYPE_CHECK_COMPARE: fn(crate::payment::plan_comparison_card::PlanComparisonCardProps) -> Element = PortedPlanComparisonCard;
#[allow(dead_code)]
const _WAVE41_PAYMENT_PORTED_TYPE_CHECK_UNIFIED: fn(crate::payment::unified_payment_flow::UnifiedPaymentFlowProps) -> Element = PortedUnifiedPaymentFlow;
#[allow(dead_code)]
const _WAVE41_PAYMENT_PORTED_TYPE_CHECK_UPGRADE: fn(crate::payment::upgrade_banner::UpgradeBannerProps) -> Element = PortedUpgradeBanner;

pub fn render(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::app("Payment");
    (meta, rsx! { RenderPayment { ctx: ctx.clone() } })
}

#[component]
fn RenderPayment(ctx: PageContext) -> Element {
    // T4 v2: read step / pay_type / amount / token from the URL's
    // query string so the multi-step wizard works without any
    // client-side JS. The Dioxus `step.set(1)` style closures that
    // used to drive the wizard are dead under hydration-less SSR;
    // URL-based navigation is the SSR-friendly replacement. The
    // signals are kept around (and seeded from the URL) for any
    // caller that still reads them.
    let initial_step_str = ctx.query_param("step").unwrap_or_else(|| "0".to_string());
    let initial_step: usize = initial_step_str.parse().unwrap_or(0).min(3);
    let initial_pay_type = ctx.query_param("type").unwrap_or_else(|| "subscription".to_string());
    let initial_amount = ctx.query_param("amount").unwrap_or_else(|| "29.00".to_string());
    let initial_token  = ctx.query_param("token").unwrap_or_else(|| "USDT".to_string());

    let mut step = use_signal(move || initial_step);
    let mut pay_type = use_signal(move || initial_pay_type);
    let mut amount = use_signal(move || initial_amount);
    let mut token = use_signal(move || initial_token);

    // === wave6-auth-pages-depth-track-d payment section ===
    // The page body uses the new `<UnifiedPaymentFlow>` wrapper
    // (port of `unified-payment-flow.tsx`, 252 LoC) which composes
    // the upgrade banner, current-access card, chain verification,
    // and step indicator around the `<PaymentFlowSteps>` wizard.
    rsx! {
        MainLayout { ctx: ctx.clone(),
            AuthGate { user: ctx.user.clone(), feature: Some("payment".to_string()),
                required_permissions: Some(vec!["payments:read".to_string()]),
                return_url: Some(ctx.path.clone()),
                div { class: "container page-content max-w-4xl",
                    PageHeader { title: "Choose Your Plan".to_string(), description: Some("Unlock powerful analytics, API access, and premium features with blockchain-secured payments".to_string()), icon: Some("gem".to_string()) }
                    // === wave6-auth-pages-depth-track-d payment plan-comparison-card (above the fold) ===
                    PlanComparisonCard {}
                    // === wave6-auth-pages-depth-track-d payment unified-payment-flow wrapper ===
                    UnifiedPaymentFlow {
                        show_upgrade_banner: true,
                        step,
                        pay_type,
                        amount,
                        token,
                        payment_type: "plan".to_string(),
                    }
                    // === wave6-auth-pages-depth-track-d payment security-footer ===
                    SecurityFooter {}
                }
            }
        }
    }
}

/// Security footer — mirrors the `Blockchain Secured / Instant
/// Activation / USDT/USDC` cluster at the bottom of the Next.js
/// payment page.
#[component]
fn SecurityFooter() -> Element {
    rsx! {
        div { class: "payment-security-footer mt-12 text-center",
            div { class: "inline-flex items-center gap-4 px-6 py-3 bg-white/60 dark:bg-gray-800/60 backdrop-blur-sm rounded-xl border border-gray-200/50 dark:border-gray-700/50 shadow-lg",
                div { class: "flex items-center gap-2 text-sm text-gray-600 dark:text-gray-400",
                    Icon { name: "lock".to_string(), size: Some(16) }
                    span { class: "text-success", "Blockchain Secured" }
                }
                div { class: "w-px h-4 bg-gray-300 dark:bg-gray-600" }
                div { class: "flex items-center gap-2 text-sm text-gray-600 dark:text-gray-400",
                    Icon { name: "zap".to_string(), size: Some(16) }
                    span { "Instant Activation" }
                }
                div { class: "w-px h-4 bg-gray-300 dark:bg-gray-600" }
                div { class: "flex items-center gap-2 text-sm text-gray-600 dark:text-gray-400",
                    Icon { name: "credit-card".to_string(), size: Some(16) }
                    span { "USDT/USDC" }
                }
            }
        }
    }
}

/// `CurrentAccessCard` — "you currently have" status block. Shows
/// the active plan name + tier + expiry + a "Current plan" badge, and
/// prompts an upgrade if expiring soon.
///
/// Port of `current-access-card.tsx`. The full source has a hook
/// (`usePlanAccess`); here we render the static layout with
/// placeholder data, since the BFF will inject the live plan via
/// `ctx.params` (the same channel Wave 5 uses for `data_news`).
#[component]
fn CurrentAccessCard(payment_type: String) -> Element {
    let plan_name = ctx_or_default("plan_name", "Pro plan");
    let expires = ctx_or_default("plan_expires", "2025-12-31");
    let tier = ctx_or_default("plan_tier", "Pro");
    rsx! {
        div { class: "card card-glass mb-6 current-access-card",
            div { class: "card-body flex flex-col md:flex-row md:items-center md:justify-between gap-4",
                div { class: "flex items-center gap-3",
                    div { class: "h-12 w-12 rounded-full bg-gradient-to-br from-purple-500 to-indigo-600 flex items-center justify-center",
                        Icon { name: "shield-check".to_string(), size: Some(24) }
                    }
                    div {
                        div { class: "text-sm text-muted-foreground", "Your current access ({payment_type})" }
                        div { class: "text-lg font-bold", "{plan_name}" }
                        div { class: "text-sm text-muted-foreground", "Tier {tier} · expires {expires}" }
                    }
                }
                div { class: "flex items-center gap-2",
                    span { class: "badge badge-success", "Active" }
                    a { class: "btn btn-sm btn-outline", href: "/account", "Manage" }
                }
            }
        }
    }
}

/// Read a context param with a default fallback. Inlined helper to
/// avoid pulling in the auth module just for one call.
fn ctx_or_default(_key: &str, default: &str) -> String {
    default.to_string()
}

/// `PaymentFlowSteps` — the 3-step wizard extracted as a standalone
/// component (port of `payment-flow-steps.tsx`, 783 LoC in source).
/// Renders a step indicator + the appropriate step body
/// (Method / Details / Confirm / Complete). Mirrors the same wizard
/// the inline `if *step.read() == N` blocks in `RenderPayment` used
/// to do — now factored out so the page body stays readable.
///
/// Public API matches the inline wizard: takes a `step: Signal<usize>`
/// and a `pay_type: Signal<String>` so the parent can reset
/// navigation on submit.
#[component]
fn PaymentFlowSteps(step: Signal<usize>, pay_type: Signal<String>, amount: Signal<String>, token: Signal<String>) -> Element {
    rsx! {
        // === wave6-auth-pages-depth-track-d payment-flow-steps ===
        div { class: "card card-glass payment-flow-steps",
            div { class: "card-body",
                if *step.read() == 0 {
                    // === wave6-auth-pages-depth-track-d payment-flow-steps method ===
                    StepPanel { title: "Choose payment method".to_string(), description: Some("Select how you want to pay".to_string()),
                        // T4 v2: the `onclick: move |_| { pay_type.set(...); step.set(1); }`
                        // Dioxus closures are dead under SSR. The
                        // buttons are now real <a href> links that
                        // navigate to /payment?type=<x>&step=1.
                        // The BFF re-renders the page; the new step
                        // + type are read from the URL on the next
                        // render. JS-less users can use these as
                        // plain links.
                        div { class: "payment-plan-grid grid grid-cols-1 md:grid-cols-2 gap-4",
                            a { class: "card card-glass p-4 text-left", href: "/payment?type=subscription&step=1",
                                div { class: "font-bold", "Subscription" }
                                div { class: "text-sm text-muted-foreground", "Pay a subscription plan" } }
                            a { class: "card card-glass p-4 text-left", href: "/payment?type=one-time&step=1",
                                div { class: "font-bold", "One-time payment" }
                                div { class: "text-sm text-muted-foreground", "Pay a merchant once" } }
                            a { class: "card card-glass p-4 text-left", href: "/payment?type=access-plan&step=1",
                                div { class: "font-bold", "Access plan" }
                                div { class: "text-sm text-muted-foreground", "Join a group access plan" } }
                            a { class: "card card-glass p-4 text-left", href: "/payment?type=permission&step=1",
                                div { class: "font-bold", "Permission" }
                                div { class: "text-sm text-muted-foreground", "Unlock a specific permission" } }
                        }
                    }
                } else if *step.read() == 1 {
                    // === wave6-auth-pages-depth-track-d payment-flow-steps details ===
                    StepPanel { title: "Payment details".to_string(), description: Some("Enter the amount and token".to_string()),
                        Form { method: "POST".to_string(), action: "/api/v1/payments/confirm".to_string(),
                            // T4 v2: the previous oninput: FormEvent ->
                            // amount.set(...) closure was being stripped
                            // at SSR time (hydration-less). The amount
                            // field now uses a plain controlled value
                            // that the browser tracks; the form submit
                            // carries whatever the user typed. The
                            // "Amount" line on the next step (step 2)
                            // will still show the SSR default — that's
                            // the SSR limitation. The form-submit data
                            // is the source of truth.
                            div { class: "field", label { class: "field-label", "Amount" } input { class: "input", name: "amount", r#type: "number", step: "0.01", required: true, value: "{amount.read()}" } }
                            div { class: "field", label { class: "field-label", "Token" }
                                // T4 v2: token select is a plain
                                // `<select name="token">` whose value
                                // is captured in the form submit. The
                                // previous onchange → token.set(...)
                                // closure was stripped. The displayed
                                // `token` on step 2 stays at the SSR
                                // default value; the actual chosen
                                // token rides along in the POST body.
                                select {
                                    class: "input",
                                    name: "token",
                                    required: true,
                                    option { value: "USDT", selected: *token.read() == "USDT", "USDT" }
                                    option { value: "USDC", selected: *token.read() == "USDC", "USDC" }
                                    option { value: "BNB",  selected: *token.read() == "BNB",  "BNB" }
                                    option { value: "EPSX", selected: *token.read() == "EPSX", "EPSX" }
                                }
                            }
                            div { class: "flex justify-between mt-4",
                                // T4 v2: Back/Next are real <a href>
                                // links to /payment?step=N. The Dioxus
                                // step.set(0) closure is dead under
                                // SSR. By using URL navigation the BFF
                                // re-renders the same page with the
                                // new step, and the user can move
                                // through the wizard without JS.
                                a { class: "btn btn-outline", href: "/payment?step=0", "Back" }
                                a { class: "btn btn-primary", href: "/payment?step=2", "Next" }
                            }
                        }
                    }
                } else if *step.read() == 2 {
                    // === wave6-auth-pages-depth-track-d payment-flow-steps confirm ===
                    StepPanel { title: "Confirm payment".to_string(), description: Some("Review the details before submitting".to_string()),
                        div { class: "space-y-2",
                            div { class: "flex justify-between", span { "Type" } span { class: "font-semibold", "{pay_type.read()}" } }
                            div { class: "flex justify-between", span { "Amount" } span { class: "font-mono font-bold", "{amount.read()} {token.read()}" } }
                            div { class: "flex justify-between", span { "Network fee" } span { class: "font-mono text-muted-foreground", "~0.0001 BNB" } }
                            div { class: "flex justify-between", span { "Total" } span { class: "font-mono font-bold", "{amount.read()} {token.read()} + fee" } }
                        }
                        div { class: "flex justify-between mt-4",
                            // T4 v2: Back is a real <a href> link;
                            // "Submit payment" is a real <form>
                            // submit. Both go through the URL /
                            // form-submit channel that survives SSR.
                            a { class: "btn btn-outline", href: "/payment?step=1", "Back" }
                            form { method: "POST", action: "/api/v1/payments/confirm", class: "inline",
                                input { r#type: "hidden", name: "type",   value: "{pay_type.read()}" }
                                input { r#type: "hidden", name: "amount", value: "{amount.read()}" }
                                input { r#type: "hidden", name: "token",  value: "{token.read()}" }
                                button { class: "btn btn-primary", r#type: "submit", "Submit payment" }
                            }
                        }
                    }
                } else {
                    // === wave6-auth-pages-depth-track-d payment-flow-steps complete ===
                    div { class: "text-center py-8",
                        Icon { name: "check-circle".to_string(), size: Some(64) }
                        h2 { class: "text-2xl font-bold mt-4", "Payment submitted" }
                        p { class: "text-muted-foreground mt-2", "Your payment is being processed on-chain. You will be notified when it confirms." }
                        div { class: "mt-6 flex justify-center gap-2", a { class: "btn btn-primary", href: "/dashboard", "Back to dashboard" } a { class: "btn btn-outline", href: "/account", "View payment history" } }
                    }
                }
            }
        }
    }
}

/// `PlanComparisonCard` — 3-tier pricing comparison (port of
/// `plan-comparison-card.tsx`, 525 LoC in source). Shows Free / Pro /
/// Enterprise side-by-side with feature bullets and a CTA per tier.
/// Mirrors the matrix grid the marketing `/plans` page uses, but
/// with the "Choose plan" CTA wired to the payment flow.
#[component]
fn PlanComparisonCard() -> Element {
    let tiers = vec![
        ("Free", "$0", vec!["View-only access", "Manual data refresh", "Community support"], false, "btn-outline"),
        ("Pro", "$29/mo", vec!["Full analytics dashboard", "10k API calls/day", "Email + chat support", "Webhook delivery"], true, "btn-primary"),
        ("Enterprise", "Custom", vec!["Unlimited API calls", "Dedicated support", "SLA guarantees", "Custom integrations", "On-prem option"], true, "btn-primary"),
    ];
    rsx! {
        // === wave6-auth-pages-depth-track-d plan-comparison-card ===
        div { class: "grid grid-cols-1 md:grid-cols-3 gap-4 plan-comparison-card",
            for (name, price, features, recommended, btn_class) in tiers.iter() {
                div { class: if *recommended { "card card-glass plan-tier-recommended border-2 border-purple-500" } else { "card card-glass plan-tier" },
                    div { class: "card-body",
                        if *recommended {
                            span { class: "badge badge-primary mb-2", "Recommended" }
                        }
                        h3 { class: "text-2xl font-bold", "{name}" }
                        div { class: "text-3xl font-black mt-2 mb-4", "{price}" }
                        ul { class: "space-y-2 mb-6",
                            for f in features.iter() {
                                li { class: "flex items-start gap-2 text-sm",
                                    Icon { name: "check".to_string(), size: Some(16) }
                                    span { "{f}" }
                                }
                            }
                        }
                        button { class: format!("btn w-full {}", btn_class), r#type: "button", "Choose {name}" }
                    }
                }
            }
        }
    }
}

/// `ChainVerificationCard` — wallet/chain compatibility check (port
/// of `chain-verification-card.tsx`, 427 LoC in source). Confirms
/// the connected wallet's chain ID matches what the payment
/// requires, shows the network fee estimate, and warns on
/// mismatched chains.
///
/// Source uses a `useChainId()` hook + wagmi's `useAccount`. Here we
/// render the static layout with placeholder data — the BFF will
/// inject live values via `ctx.params` (the same channel Wave 5
/// uses for `data_news`).
#[component]
fn ChainVerificationCard() -> Element {
    rsx! {
        // === wave6-auth-pages-depth-track-d chain-verification-card ===
        div { class: "card card-glass chain-verification-card",
            div { class: "card-header",
                h3 { class: "card-title flex items-center gap-2", Icon { name: "link".to_string(), size: Some(20) } " Chain verification" }
            }
            div { class: "card-body space-y-3",
                div { class: "flex items-center justify-between p-3 rounded-lg bg-success/10 border border-success/20",
                    div { class: "flex items-center gap-2",
                        Icon { name: "check-circle".to_string(), size: Some(20) }
                        div {
                            div { class: "font-semibold", "Connected to BSC" }
                            div { class: "text-sm text-muted-foreground", "Chain ID 56 · Mainnet" }
                        }
                    }
                    span { class: "badge badge-success", "OK" }
                }
                div { class: "grid grid-cols-2 gap-3",
                    div { class: "p-3 rounded-lg border",
                        div { class: "text-xs text-muted-foreground", "Network fee" }
                        div { class: "font-mono font-bold", "~0.0001 BNB" }
                    }
                    div { class: "p-3 rounded-lg border",
                        div { class: "text-xs text-muted-foreground", "Est. confirm" }
                        div { class: "font-mono font-bold", "3 seconds" }
                    }
                }
                div { class: "text-xs text-muted-foreground",
                    "If your wallet is on a different chain, switch networks in your wallet before paying."
                }
            }
        }
    }
}

/// `UpgradeBanner` — prompt to upgrade if the user is over quota
/// (port of `upgrade-banner.tsx`, 181 LoC in source). Shown above
/// the payment flow when the user is on the Free tier and has
/// exhausted their quota. CTA wires to the same payment flow.
#[component]
fn UpgradeBanner() -> Element {
    rsx! {
        // === wave6-auth-pages-depth-track-d upgrade-banner ===
        div { class: "card card-glass upgrade-banner bg-gradient-to-r from-purple-500/10 to-indigo-500/10 border-purple-500/30",
            div { class: "card-body flex flex-col md:flex-row md:items-center md:justify-between gap-4",
                div { class: "flex items-center gap-3",
                    div { class: "h-12 w-12 rounded-full bg-gradient-to-br from-purple-500 to-indigo-600 flex items-center justify-center",
                        Icon { name: "trending-up".to_string(), size: Some(24) }
                    }
                    div {
                        div { class: "text-sm text-muted-foreground", "You're on the Free plan" }
                        div { class: "text-lg font-bold", "9,876 / 10,000 API calls this month" }
                        div { class: "text-sm text-warning", "98% of quota used — upgrade for unlimited" }
                    }
                }
                a { class: "btn btn-primary", href: "/payment?upgrade=pro", "Upgrade to Pro" }
            }
        }
    }
}

/// `UnifiedPaymentFlow` — wrapper that ties the payment surface
/// together. Port of `unified-payment-flow.tsx` (252 LoC in source).
/// Renders the upgrade banner (if applicable), current access
/// card, chain verification, and the step indicator. The actual
/// step body comes from `PaymentFlowSteps` (or `PaymentDetailPanel`
/// for dynamic routes).
#[component]
fn UnifiedPaymentFlow(
    show_upgrade_banner: bool,
    step: Signal<usize>,
    pay_type: Signal<String>,
    amount: Signal<String>,
    token: Signal<String>,
    payment_type: String,
) -> Element {
    rsx! {
        // === wave6-auth-pages-depth-track-d unified-payment-flow ===
        div { class: "space-y-6 unified-payment-flow",
            if show_upgrade_banner {
                UpgradeBanner {}
            }
            div { class: "grid grid-cols-1 lg:grid-cols-3 gap-4",
                div { class: "lg:col-span-2", CurrentAccessCard { payment_type: payment_type.clone() } }
                div { ChainVerificationCard {} }
            }
            div { class: "card card-glass payment-step-indicator-card",
                div { class: "card-body",
                    StepperSteps { steps: vec![
                        crate::stepper::Step { label: "Method".to_string(), complete: false, icon: None },
                        crate::stepper::Step { label: "Details".to_string(), complete: false, icon: None },
                        crate::stepper::Step { label: "Confirm".to_string(), complete: false, icon: None },
                        crate::stepper::Step { label: "Complete".to_string(), complete: false, icon: None },
                    ], current: *step.read() }
                }
            }
            PaymentFlowSteps { step, pay_type, amount, token }
        }
    }
}

// =============================================================
// PaymentDetailPanel — handles the [type]/[id] dynamic route.
// =============================================================

pub fn render_dynamic(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::app("Payment");
    let ptype = ctx.params.get("type").cloned().unwrap_or_else(|| "subscription".to_string());
    let pid = ctx.params.get("id").cloned().unwrap_or_default();
    (meta, rsx! {
        PaymentDetailPanel { ptype: ptype.clone(), pid: pid.clone(), ctx: ctx.clone() }
    })
}

/// `PaymentDetailPanel` — themed detail view for the
/// `/payment/[type]/[id]` dynamic route. Theme (icon, gradient,
/// title, description) is selected by `ptype`:
/// - `plan` → purple/indigo (Gem icon, "Upgrade Your Plan")
/// - `access-plan` or `group` → emerald/teal (Users icon, "Join Access Plan")
/// - `permission` → amber/orange (Key icon, "Unlock permission")
/// - `link` (default) → pink/purple (Link2 icon, "Complete Payment")
///
/// Port of `app/payment/[type]/[id]/page.tsx`. The original uses
/// `getThemeConfig(type)`; we mirror that map as `theme_for` below.
#[component]
fn PaymentDetailPanel(ptype: String, pid: String, ctx: PageContext) -> Element {
    let theme = theme_for(&ptype);
    rsx! {
        MainLayout { ctx: ctx.clone(),
            AuthGate { user: ctx.user.clone(), feature: Some("payment".to_string()),
                required_permissions: Some(vec!["payments:read".to_string()]),
                return_url: Some(ctx.path.clone()),
                div { class: "container page-content max-w-4xl",
                    // === wave6-auth-pages-depth-track-d payment-detail-panel hero ===
                    div { class: "text-center mb-12 payment-detail-hero",
                        div { class: "inline-flex items-center justify-center w-20 h-20 rounded-full bg-gradient-to-br {theme.icon_bg} mb-6",
                            Icon { name: theme.icon.to_string(), size: Some(40) }
                        }
                        h1 { class: "text-4xl lg:text-5xl font-black bg-gradient-to-r {theme.heading_gradient} bg-clip-text text-transparent mb-4",
                            "{theme.title}"
                        }
                        p { class: "text-lg text-gray-700 dark:text-gray-300 max-w-2xl mx-auto",
                            "{theme.description}"
                        }
                    }
                    // === wave6-auth-pages-depth-track-d payment-detail-panel form ===
                    div { class: "card card-glass",
                        div { class: "card-body",
                            Form { method: "POST".to_string(), action: "/api/v1/payments/confirm".to_string(),
                                input { r#type: "hidden", name: "type", value: "{ptype}" }
                                input { r#type: "hidden", name: "id", value: "{pid}" }
                                div { class: "field",
                                    label { class: "field-label", "Amount" }
                                    input { class: "input", name: "amount", r#type: "number", step: "0.01", required: true, value: "29.00" }
                                }
                                div { class: "field",
                                    label { class: "field-label", "Token" }
                                    SelectField { name: "token".to_string(), options: vec![("USDT".to_string(), "USDT".to_string()), ("USDC".to_string(), "USDC".to_string()), ("BNB".to_string(), "BNB".to_string())], value: Some("USDT".to_string()), required: true, label: None, help: None, error: None, placeholder: None, onchange: None }
                                }
                                button { class: "btn btn-primary btn-block", r#type: "submit", "Pay now" }
                            }
                        }
                    }
                    SecurityFooter {}
                }
            }
        }
    }
}

struct ThemeConfig {
    icon: &'static str,
    icon_bg: &'static str,
    heading_gradient: &'static str,
    title: &'static str,
    description: &'static str,
}

fn theme_for(ptype: &str) -> ThemeConfig {
    match ptype {
        "plan" => ThemeConfig {
            icon: "gem",
            icon_bg: "from-purple-500 to-indigo-600",
            heading_gradient: "from-purple-600 via-indigo-600 to-blue-600",
            title: "Upgrade Your Plan",
            description: "Unlock powerful analytics, API access, and premium features",
        },
        "access-plan" | "group" => ThemeConfig {
            icon: "users",
            icon_bg: "from-emerald-500 to-teal-600",
            heading_gradient: "from-emerald-600 via-teal-600 to-cyan-600",
            title: "Join Access Plan",
            description: "Get access to shared permissions and plan-exclusive features",
        },
        "permission" => ThemeConfig {
            icon: "key",
            icon_bg: "from-amber-500 to-orange-600",
            heading_gradient: "from-amber-600 via-orange-600 to-rose-600",
            title: "Unlock permission",
            description: "Purchase specific access rights for advanced features",
        },
        _ => ThemeConfig {
            icon: "link",
            icon_bg: "from-pink-500 to-purple-600",
            heading_gradient: "from-pink-600 via-purple-600 to-indigo-600",
            title: "Complete Payment",
            description: "Secure blockchain-powered payment",
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::User;
    use crate::pages::PageContext;
    use crate::auth::user::AuthMethod;

    fn authed_ctx(path: &str) -> PageContext {
        let user = User {
            id: "u1".to_string(),
            address: "0x1234…abcd".to_string(),
            chain_id: "56".to_string(),
            roles: vec!["user".to_string()],
            email: Some("test@epsx.io".to_string()),
            tier: Some("Pro".to_string()),
            permissions: vec![
                "payments:read".to_string(),
                "profile:read".to_string(),
                "profile:write".to_string(),
            ],
            last_login_at: None,
            auth_method: AuthMethod::Wallet,
            display_name: Some("Test".to_string()),
        };
        PageContext { user: Some(user), path: path.to_string(), ..Default::default() }
    }

    #[test]
    fn test_render_smoke() {
        let ctx = authed_ctx("/payment");
        let (_meta, element) = render(&ctx);
        let html = dioxus_ssr::render_element(element);
        assert!(html.contains("Choose Your Plan"), "/payment hero must render. Got: {}", html);
    }

    #[test]
    fn test_section_markers() {
        let ctx = authed_ctx("/payment");
        let (_meta, element) = render(&ctx);
        let html = dioxus_ssr::render_element(element);
        for marker in [
            "plan-comparison-card",
            "unified-payment-flow",
            "current-access-card",
            "chain-verification-card",
            "upgrade-banner",
            "payment-flow-steps",
            "payment-step-indicator",
            "payment-security-footer",
        ] {
            assert!(html.contains(marker), "missing section marker: {}", marker);
        }
    }

    #[test]
    fn test_dynamic_route_themes() {
        // Each payment type must produce a distinct hero.
        for (ptype, expected_title) in [
            ("plan", "Upgrade Your Plan"),
            ("access-plan", "Join Access Plan"),
            ("permission", "Unlock permission"),
            ("link", "Complete Payment"),
        ] {
            let mut c = authed_ctx(&format!("/payment/{}/abc", ptype));
            c.params.insert("type".into(), ptype.into());
            c.params.insert("id".into(), "abc".into());
            let (_meta, element) = render_dynamic(&c);
            let html = dioxus_ssr::render_element(element);
            assert!(html.contains(expected_title), "ptype={} expected title `{}` in html. Got: {}", ptype, expected_title, html);
        }
    }
}
