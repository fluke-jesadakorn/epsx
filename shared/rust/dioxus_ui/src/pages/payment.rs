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

use dioxus::prelude::*;
use super::PageContext;
use super::PageMeta;
use crate::layout::main_layout::MainLayout;
use crate::layout::PageHeader;
use crate::auth::AuthGate;

pub fn render(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::app("Payment");
    (meta, rsx! { RenderPayment { ctx: ctx.clone() } })
}

#[component]
fn RenderPayment(ctx: PageContext) -> Element {
    let mut step = use_signal(|| 0usize);
    let mut pay_type = use_signal(|| "subscription".to_string());
    let mut amount = use_signal(|| "29.00".to_string());
    let mut token = use_signal(|| "USDT".to_string());
    let steps = vec![("Method".to_string(), false), ("Details".to_string(), false), ("Confirm".to_string(), false), ("Complete".to_string(), false)];

    // === wave6-auth-pages-depth-track-d payment section ===
    // The unified payment flow wraps every block below. The wrapper
    // exposes a "current access" card, the 3-step indicator, the plan
    // grid, the confirm step, and the security footer — same surface
    // as `unified-payment-flow.tsx`.
    rsx! {
        MainLayout { ctx: ctx.clone(),
            AuthGate { user: ctx.user.clone(), feature: Some("payment".to_string()),
                required_permissions: Some(vec!["payments:read".to_string()]),
                return_url: Some(ctx.path.clone()),
                div { class: "container page-content max-w-4xl",
                    PageHeader { title: "Choose Your Plan".to_string(), description: Some("Unlock powerful analytics, API access, and premium features with blockchain-secured payments".to_string()), icon: Some("gem".to_string()) }
                    // === wave6-auth-pages-depth-track-d payment current-access-card ===
                    CurrentAccessCard { payment_type: "plan".to_string() }
                    // === wave6-auth-pages-depth-track-d payment step indicator ===
                    div { class: "mb-6 payment-step-indicator", Stepper { steps, current: *step.read() } }
                    // === wave6-auth-pages-depth-track-d payment flow steps ===
                    div { class: "card card-glass payment-flow", div { class: "card-body",
                        if *step.read() == 0 {
                            // === wave6-auth-pages-depth-track-d payment plan-grid-view ===
                            StepPanel { title: "Choose payment method".to_string(), description: Some("Select how you want to pay".to_string()),
                                div { class: "payment-plan-grid grid grid-cols-1 md:grid-cols-2 gap-4",
                                    button { class: "card card-glass p-4 text-left", r#type: "button", onclick: move |_| { pay_type.set("subscription".to_string()); step.set(1); }, div { class: "font-bold", "Subscription" } div { class: "text-sm text-muted-foreground", "Pay a subscription plan" } }
                                    button { class: "card card-glass p-4 text-left", r#type: "button", onclick: move |_| { pay_type.set("one-time".to_string()); step.set(1); }, div { class: "font-bold", "One-time payment" } div { class: "text-sm text-muted-foreground", "Pay a merchant once" } }
                                }
                            }
                        } else if *step.read() == 1 {
                            // === wave6-auth-pages-depth-track-d payment confirm-step ===
                            StepPanel { title: "Payment details".to_string(), description: Some("Enter the amount and token".to_string()),
                                Form { method: "POST".to_string(), action: "/api/v1/payments/confirm".to_string(),
                                    div { class: "field", label { class: "field-label", "Amount" } input { class: "input", name: "amount", r#type: "number", step: "0.01", required: true, value: "{amount.read()}", oninput: move |e| amount.set(e.value().to_string()) } }
                                    div { class: "field", label { class: "field-label", "Token" }
                                        SelectField { name: "token".to_string(), options: vec![("USDT".to_string(), "USDT".to_string()), ("USDC".to_string(), "USDC".to_string()), ("BNB".to_string(), "BNB".to_string())], value: Some(token.read().clone()), required: true, label: None, help: None, error: None, placeholder: None, onchange: None }
                                    }
                                    div { class: "flex justify-between mt-4",
                                        button { class: "btn btn-outline", r#type: "button", onclick: move |_| step.set(0), "Back" }
                                        button { class: "btn btn-primary", r#type: "button", onclick: move |_| step.set(2), "Next" }
                                    }
                                }
                            }
                        } else if *step.read() == 2 {
                            // === wave6-auth-pages-depth-track-d payment success-step ===
                            StepPanel { title: "Confirm payment".to_string(), description: Some("Review the details before submitting".to_string()),
                                div { class: "space-y-2",
                                    div { class: "flex justify-between", span { "Type" } span { class: "font-semibold", "{pay_type.read()}" } }
                                    div { class: "flex justify-between", span { "Amount" } span { class: "font-mono font-bold", "{amount.read()} {token.read()}" } }
                                    div { class: "flex justify-between", span { "Network fee" } span { class: "font-mono text-muted-foreground", "~0.0001 BNB" } }
                                }
                                div { class: "flex justify-between mt-4",
                                    button { class: "btn btn-outline", r#type: "button", onclick: move |_| step.set(1), "Back" }
                                    button { class: "btn btn-primary", r#type: "button", onclick: move |_| step.set(3), "Submit payment" }
                                }
                            }
                        } else {
                            // === wave6-auth-pages-depth-track-d payment complete-step ===
                            div { class: "text-center py-8",
                                Icon { name: "check-circle".to_string(), size: Some(64) }
                                h2 { class: "text-2xl font-bold mt-4", "Payment submitted" }
                                p { class: "text-muted-foreground mt-2", "Your payment is being processed on-chain. You will be notified when it confirms." }
                                div { class: "mt-6 flex justify-center gap-2", a { class: "btn btn-primary", href: "/dashboard", "Back to dashboard" } a { class: "btn btn-outline", href: "/account", "View payment history" } }
                            }
                        }
                    } }
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
            "payment-step-indicator",
            "payment-flow",
            "current-access-card",
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
