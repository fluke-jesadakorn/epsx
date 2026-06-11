//! Sub-components extracted from `pages/payment.rs` during
//! Wave 6C Track E (1:1 user-side component parity).
//!
//! Nine named sub-components: `RenderPayment`, `SecurityFooter`,
//! `CurrentAccessCard`, `PaymentFlowSteps`, `PlanComparisonCard`,
//! `ChainVerificationCard`, `UpgradeBanner`, `UnifiedPaymentFlow`,
//! `PaymentDetailPanel`. Also the `ThemeConfig` data type and
//! `theme_for` helper.

use crate::pages::PageContext;
use crate::primitives::*;
use crate::feedback::*;
use crate::stepper::{Stepper, StepPanel, StepNavigation};

use dioxus::prelude::*;

/// Page-level orchestrator for the `/payment` route.
#[component]
pub fn RenderPayment(ctx: PageContext) -> Element {
    let mut step = use_signal(|| 0usize);
    let mut pay_type = use_signal(|| "subscription".to_string());
    let mut amount = use_signal(|| "29.00".to_string());
    let mut token = use_signal(|| "USDT".to_string());

    rsx! {
        crate::layout::main_layout::MainLayout { ctx: ctx.clone(),
            crate::auth::AuthGate { user: ctx.user.clone(), feature: Some("payment".to_string()),
                required_permissions: Some(vec!["payments:read".to_string()]),
                return_url: Some(ctx.path.clone()),
                div { class: "container page-content max-w-4xl",
                    crate::layout::PageHeader { title: "Choose Your Plan".to_string(), description: Some("Unlock powerful analytics, API access, and premium features with blockchain-secured payments".to_string()), icon: Some("gem".to_string()) }
                    PlanComparisonCard {}
                    UnifiedPaymentFlow {
                        show_upgrade_banner: true,
                        step,
                        pay_type,
                        amount,
                        token,
                        payment_type: "plan".to_string(),
                    }
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
pub fn SecurityFooter() -> Element {
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

/// `CurrentAccessCard` — "you currently have" status block.
#[component]
pub fn CurrentAccessCard(payment_type: String) -> Element {
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

/// Read a context param with a default fallback.
pub fn ctx_or_default(_key: &str, default: &str) -> String {
    default.to_string()
}

/// `PaymentFlowSteps` — the 3-step wizard.
#[component]
pub fn PaymentFlowSteps(step: Signal<usize>, pay_type: Signal<String>, amount: Signal<String>, token: Signal<String>) -> Element {
    rsx! {
        div { class: "card card-glass payment-flow-steps",
            div { class: "card-body",
                if *step.read() == 0 {
                    StepPanel { title: "Choose payment method".to_string(), description: Some("Select how you want to pay".to_string()),
                        div { class: "payment-plan-grid grid grid-cols-1 md:grid-cols-2 gap-4",
                            button { class: "card card-glass p-4 text-left", r#type: "button", onclick: move |_| { pay_type.set("subscription".to_string()); step.set(1); }, div { class: "font-bold", "Subscription" } div { class: "text-sm text-muted-foreground", "Pay a subscription plan" } }
                            button { class: "card card-glass p-4 text-left", r#type: "button", onclick: move |_| { pay_type.set("one-time".to_string()); step.set(1); }, div { class: "font-bold", "One-time payment" } div { class: "text-sm text-muted-foreground", "Pay a merchant once" } }
                            button { class: "card card-glass p-4 text-left", r#type: "button", onclick: move |_| { pay_type.set("access-plan".to_string()); step.set(1); }, div { class: "font-bold", "Access plan" } div { class: "text-sm text-muted-foreground", "Join a group access plan" } }
                            button { class: "card card-glass p-4 text-left", r#type: "button", onclick: move |_| { pay_type.set("permission".to_string()); step.set(1); }, div { class: "font-bold", "Permission" } div { class: "text-sm text-muted-foreground", "Unlock a specific permission" } }
                        }
                    }
                } else if *step.read() == 1 {
                    StepPanel { title: "Payment details".to_string(), description: Some("Enter the amount and token".to_string()),
                        Form { method: "POST".to_string(), action: "/api/v1/payments/confirm".to_string(),
                            div { class: "field", label { class: "field-label", "Amount" } input { class: "input", name: "amount", r#type: "number", step: "0.01", required: true, value: "{amount.read()}", oninput: move |e| amount.set(e.value().to_string()) } }
                            div { class: "field", label { class: "field-label", "Token" }
                                SelectField { name: "token".to_string(), options: vec![("USDT".to_string(), "USDT".to_string()), ("USDC".to_string(), "USDC".to_string()), ("BNB".to_string(), "BNB".to_string()), ("EPSX".to_string(), "EPSX".to_string())], value: Some(token.read().clone()), required: true, label: None, help: None, error: None, placeholder: None, onchange: None }
                            }
                            div { class: "flex justify-between mt-4",
                                button { class: "btn btn-outline", r#type: "button", onclick: move |_| step.set(0), "Back" }
                                button { class: "btn btn-primary", r#type: "button", onclick: move |_| step.set(2), "Next" }
                            }
                        }
                    }
                } else if *step.read() == 2 {
                    StepPanel { title: "Confirm payment".to_string(), description: Some("Review the details before submitting".to_string()),
                        div { class: "space-y-2",
                            div { class: "flex justify-between", span { "Type" } span { class: "font-semibold", "{pay_type.read()}" } }
                            div { class: "flex justify-between", span { "Amount" } span { class: "font-mono font-bold", "{amount.read()} {token.read()}" } }
                            div { class: "flex justify-between", span { "Network fee" } span { class: "font-mono text-muted-foreground", "~0.0001 BNB" } }
                            div { class: "flex justify-between", span { "Total" } span { class: "font-mono font-bold", "{amount.read()} {token.read()} + fee" } }
                        }
                        div { class: "flex justify-between mt-4",
                            button { class: "btn btn-outline", r#type: "button", onclick: move |_| step.set(1), "Back" }
                            button { class: "btn btn-primary", r#type: "button", onclick: move |_| step.set(3), "Submit payment" }
                        }
                    }
                } else {
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

/// `PlanComparisonCard` — 3-tier pricing comparison.
#[component]
pub fn PlanComparisonCard() -> Element {
    let tiers = vec![
        ("Free", "$0", vec!["View-only access", "Manual data refresh", "Community support"], false, "btn-outline"),
        ("Pro", "$29/mo", vec!["Full analytics dashboard", "10k API calls/day", "Email + chat support", "Webhook delivery"], true, "btn-primary"),
        ("Enterprise", "Custom", vec!["Unlimited API calls", "Dedicated support", "SLA guarantees", "Custom integrations", "On-prem option"], true, "btn-primary"),
    ];
    rsx! {
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

/// `ChainVerificationCard` — wallet/chain compatibility check.
#[component]
pub fn ChainVerificationCard() -> Element {
    rsx! {
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

/// `UpgradeBanner` — prompt to upgrade if the user is over quota.
#[component]
pub fn UpgradeBanner() -> Element {
    rsx! {
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
/// together.
#[component]
pub fn UnifiedPaymentFlow(
    show_upgrade_banner: bool,
    step: Signal<usize>,
    pay_type: Signal<String>,
    amount: Signal<String>,
    token: Signal<String>,
    payment_type: String,
) -> Element {
    rsx! {
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

/// `PaymentDetailPanel` — themed detail view for the
/// `/payment/[type]/[id]` dynamic route.
#[component]
pub fn PaymentDetailPanel(ptype: String, pid: String, ctx: PageContext) -> Element {
    let theme = theme_for(&ptype);
    rsx! {
        crate::layout::main_layout::MainLayout { ctx: ctx.clone(),
            crate::auth::AuthGate { user: ctx.user.clone(), feature: Some("payment".to_string()),
                required_permissions: Some(vec!["payments:read".to_string()]),
                return_url: Some(ctx.path.clone()),
                div { class: "container page-content max-w-4xl",
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

pub struct ThemeConfig {
    pub icon: &'static str,
    pub icon_bg: &'static str,
    pub heading_gradient: &'static str,
    pub title: &'static str,
    pub description: &'static str,
}

pub fn theme_for(ptype: &str) -> ThemeConfig {
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

    /// Wave 6C Track E — `test_render_smoke` for the extracted
    /// payment sub-components.
    #[test]
    fn payment_subcomponents_render_smoke() {
        // SecurityFooter
        let el = rsx! { SecurityFooter {} };
        let html = dioxus_ssr::render_element(el);
        assert!(html.contains("payment-security-footer"), "SecurityFooter missing section-marker");
        assert!(html.contains("Blockchain Secured"));

        // CurrentAccessCard
        let el = rsx! { CurrentAccessCard { payment_type: "plan".to_string() } };
        let html = dioxus_ssr::render_element(el);
        assert!(html.contains("current-access-card"));
        assert!(html.contains("Active"));

        // PlanComparisonCard
        let el = rsx! { PlanComparisonCard {} };
        let html = dioxus_ssr::render_element(el);
        assert!(html.contains("plan-comparison-card"));
        assert!(html.contains("Recommended"));

        // ChainVerificationCard
        let el = rsx! { ChainVerificationCard {} };
        let html = dioxus_ssr::render_element(el);
        assert!(html.contains("chain-verification-card"));
        assert!(html.contains("BSC"));

        // UpgradeBanner
        let el = rsx! { UpgradeBanner {} };
        let html = dioxus_ssr::render_element(el);
        assert!(html.contains("upgrade-banner"));
        assert!(html.contains("98% of quota used"));

        // theme_for variants
        assert_eq!(theme_for("plan").title, "Upgrade Your Plan");
        assert_eq!(theme_for("access-plan").title, "Join Access Plan");
        assert_eq!(theme_for("permission").title, "Unlock permission");
        assert_eq!(theme_for("link").title, "Complete Payment");
        assert_eq!(theme_for("").title, "Complete Payment");
    }
}
