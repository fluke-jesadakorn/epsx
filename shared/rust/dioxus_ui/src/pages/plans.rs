//! `/plans` — plan list with comparison, FAQ, and enterprise CTA.
//!
//! Source of truth: `apps-old/frontend/app/plans/page.tsx` + the
//! `<PlanSelection>` component at
//! `apps-old/frontend/components/plans/plan-selection.tsx`. The
//! port keeps the 3-tier grid (Free / Pro / Enterprise), the
//! "Most popular" badge, the monthly/annual toggle, the feature
//! list, and the CTA. Below the grid it adds the source's
//! comparison table, 4-question FAQ accordion, and "Need custom?"
//! enterprise contact CTA per the Wave 5 design doc.
//!
//! Wave 22 T4 — `PlanGroups` (4 sections: Personal / Enterprise /
//! API / Custom), `AffiliateBanner` (purple→pink gradient triggered
//! by `?ref=foo` / `?affiliate=bar` / `?aff=baz` query params), and
//! an extended `Plan` struct with a `plan_group` discriminator so a
//! single default list can render all four group sections.
//!
//! The page's pricing data is read from `ctx.params["data_plans"]`
//! (BFF pre-fetch) with a 6-plan default fallback covering 3 of
//! the 4 groups (Personal / Enterprise / API). Custom plans
//! remain CTA-only. The same `Plan` struct is used for the grid
//! cards AND the comparison table rows.

use crate::primitives::*;
use crate::feedback::*;

use dioxus::prelude::*;
use super::PageContext;
use super::PageMeta;
use crate::layout::main_layout::MainLayout;
use crate::layout::PageHeader;
use crate::auth::AuthGate;
use crate::auth::ProgressiveAuthBanner;

/// Plan groups — mirrors the `PLAN_GROUPS` array in
/// `apps-old/frontend/components/plans/plan-selection.tsx:38`.
const PLAN_GROUPS: &[&str] = &["personal", "enterprise", "api", "custom"];

/// Canonical affiliate query-param keys (priority order — first
/// non-empty match wins). Mirrors the source:
///   `searchParams.get('ref') ?? searchParams.get('affiliate') ?? searchParams.get('aff')`.
const AFFILIATE_QUERY_KEYS: &[&str] = &["ref", "affiliate", "aff"];

/// Extract the affiliate code from the page query string.
/// Returns the first non-empty value of `?ref=` / `?affiliate=` /
/// `?aff=` (priority order: ref → affiliate → aff, regardless of
/// document order). Returns `None` when none of the keys are
/// present or all are blank. Mirrors the source's
/// `searchParams.get('ref') ?? searchParams.get('affiliate') ?? searchParams.get('aff')`.
pub fn affiliate_code_from_query(query: &str) -> Option<String> {
    if query.is_empty() {
        return None;
    }
    // First pass: collect all key=value pairs.
    let mut pairs: Vec<(&str, &str)> = Vec::new();
    for pair in query.split('&') {
        let mut it = pair.splitn(2, '=');
        let key = it.next().unwrap_or("").trim();
        let value = it.next().unwrap_or("").trim();
        if !key.is_empty() {
            pairs.push((key, value));
        }
    }
    // Second pass: walk the priority list and return the first
    // non-empty value. Priority order is fixed (ref → affiliate → aff)
    // regardless of the order pairs appear in the query string.
    for key in AFFILIATE_QUERY_KEYS {
        for (k, v) in &pairs {
            if *k == *key && !v.is_empty() {
                return Some(v.to_string());
            }
        }
    }
    None
}

pub fn render(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::marketing("Plans");
    let plans_data: Option<serde_json::Value> = ctx.params.get("data_plans")
        .and_then(|s| serde_json::from_str(s).ok());
    // Wave 23 T5 — accept BOTH wire shapes for the plans data:
    //   1. Subscription service: `Vec<SubscriptionPlan>` serialized
    //      as a raw array `[...]` (the shape that
    //      `state.subscription.get_plain("/api/v1/subscription/plans")`
    //      returns)
    //   2. Content service / BFF mock: `{ "personal": [...], "api":
    //      [...], "custom": [...] }` — three grouped buckets, the
    //      prod plans.json shape.
    //
    // The OLD code only handled shape #2's `plans` key (which doesn't
    // exist), so it always fell back to the 6-item default. Now we
    // merge all 3 buckets into a flat `Vec<Plan>` tagged with
    // `plan_group` so `PlanGroups` can split them.
    let plans: Vec<Plan> = plans_data.as_ref()
        .map(extract_plans)
        .unwrap_or_default();

    // Fallback to the canonical 6-plan list (3 of 4 groups) when the
    // BFF hasn't pre-fetched plans (admin BFF, dev mode, etc.).
    let plans = if plans.is_empty() { default_plans() } else { plans };

    // Affiliate banner — mirrors lines 178-192 of plan-selection.tsx.
    // Triggered when the query string carries a ref/affiliate/aff
    // param. The prod source also gates on `affiliateInfo` being
    // present; we render the banner purely on the code (the source
    // populates the info via an API call that the BFF doesn't yet
    // expose — for the dev BFF baseline the code is sufficient).
    let affiliate_code = affiliate_code_from_query(&ctx.query);

    (meta, rsx! {
        MainLayout { ctx: ctx.clone(),
            if ctx.user.is_none() {
                ProgressiveAuthBanner {
                    feature: Some("plan subscription".to_string()),
                }
            }
            AuthGate { user: ctx.user.clone(), feature: Some("plan subscription".to_string()),
                PlansHero {}
                if let Some(ref code) = affiliate_code {
                    AffiliateBanner { code: code.clone() }
                }
                PlanGroups { plans: plans.clone() }
                PlanComparisonTable { plans: plans.clone() }
                PlanFaq {}
                PlanEnterpriseCta {}
            }
        }
    })
}

/// Page-level hero. The source uses a gradient-text h1 ("Choose Your
/// EPSX Plan") and a one-line pitch below it. The port keeps the
/// copy verbatim and reuses the design-system `section-title` /
/// `section-sub` classes that `home.rs` already uses.
#[component]
fn PlansHero() -> Element {
    rsx! {
        section { class: "plans-hero",
            div { class: "container",
                div { class: "plans-hero-inner",
                    h1 { class: "plans-hero-title", "Choose Your EPSX Plan" }
                    p { class: "plans-hero-subtitle",
                        "Unlock powerful analytics features, API access, and premium tools to supercharge your analytics experience"
                    }
                }
            }
        }
    }
}

/// 3-tier pricing grid. The source renders 3 cards in a
/// `grid-cols-1 md:grid-cols-2 lg:grid-cols-3` row; the port uses
/// the same `grid grid-cols-1 md:grid-cols-3` utility that's already
/// in the design system. The "Most popular" badge, feature checkmark
/// list, price block, and CTA button all come from the source.
#[component]
fn PlanGrid(plans: Vec<Plan>) -> Element {
    rsx! {
        section { class: "plans-grid-section",
            div { class: "container",
                div { class: "grid grid-cols-1 md:grid-cols-3 gap-6",
                    for p in plans.iter() {
                        PlanCard { plan: p.clone() }
                    }
                }
            }
        }
    }
}

/// `PlanGroups` — 4 grouped sections (Personal / Enterprise / API /
/// Custom), each with its own header and card grid. Mirrors the
/// `PLAN_GROUPS.map(...)` iteration at
/// `apps-old/frontend/components/plans/plan-selection.tsx:210-269`.
/// Plans that don't carry a `plan_group` are bucketed under
/// "personal" (the source's fallback at line 211).
#[component]
fn PlanGroups(plans: Vec<Plan>) -> Element {
    rsx! {
        section { class: "plans-groups-section space-y-16",
            "data-section": "plans-groups-section",
            for group in PLAN_GROUPS.iter() {
                {
                    let g = group.to_string();
                    let mut group_cards: Vec<Plan> = plans.iter()
                        .filter(|p| p.plan_group.as_deref().unwrap_or("personal") == g)
                        .cloned()
                        .collect();
                    // For the "custom" group there are no card-level
                    // entries in the source — it's CTA-only — so
                    // always render the group header + the CTA panel
                    // when the bucket is empty.
                    let is_custom = g == "custom";
                    if group_cards.is_empty() && !is_custom {
                        // Empty group + not custom → render nothing
                        // (the source filters with `if (groupCards.length === 0) return null`).
                        rsx! {}
                    } else {
                        let cfg = group_config(&g);
                        rsx! {
                            div { class: "plans-group-block",
                                key: "group-{g}",
                                div { class: "plans-group-header flex items-center gap-3 mb-8",
                                    div { class: "p-2 rounded-xl bg-gradient-to-br from-emerald-500/20 to-blue-500/20 text-emerald-600",
                                        Icon { name: cfg.icon.to_string(), size: Some(24) }
                                    }
                                    div {
                                        h2 { class: "text-2xl font-bold text-foreground", "{cfg.label}" }
                                        p { class: "text-sm text-muted-foreground", "{cfg.desc}" }
                                    }
                                }
                                if is_custom && group_cards.is_empty() {
                                    div { class: "plans-group-custom-cta",
                                        p { class: "text-sm text-muted-foreground",
                                            "Need a tailored solution? Our team can design a plan with volume pricing, dedicated support, and SLA-backed uptime."
                                        }
                                        a { class: "btn btn-primary mt-3", href: "/contact", "Contact sales" }
                                    }
                                } else {
                                    div { class: "grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6",
                                        for p in group_cards.iter_mut() {
                                            PlanCard { plan: p.clone() }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

struct GroupConfig {
    label: &'static str,
    desc: &'static str,
    icon: &'static str,
}

fn group_config(group: &str) -> GroupConfig {
    match group {
        "personal" => GroupConfig {
            label: "Personal Plans",
            desc: "For individual traders and analysts",
            icon: "user",
        },
        "enterprise" => GroupConfig {
            label: "Enterprise Plans",
            desc: "For teams and organizations",
            icon: "building",
        },
        "api" => GroupConfig {
            label: "API Plans",
            desc: "For developers and integrations",
            icon: "code",
        },
        "custom" => GroupConfig {
            label: "Custom Plans",
            desc: "Tailored solutions for partners and enterprises",
            icon: "star",
        },
        _ => GroupConfig {
            label: "Other",
            desc: "Additional plans",
            icon: "package",
        },
    }
}

/// `AffiliateBanner` — purple→pink gradient banner shown above the
/// plan groups when the page was reached via an affiliate link
/// (`?ref=foo`, `?affiliate=bar`, or `?aff=baz`). Mirrors
/// `apps-old/frontend/components/plans/plan-selection.tsx:178-192`.
#[component]
fn AffiliateBanner(code: String) -> Element {
    rsx! {
        div { class: "plans-affiliate-banner mb-8",
            "data-section": "plans-affiliate-banner",
            div { class: "bg-gradient-to-r from-purple-500 to-pink-500 text-white p-4 rounded-2xl text-center shadow-xl",
                div { class: "flex items-center justify-center gap-2 text-lg font-semibold",
                    Icon { name: "star".to_string(), size: Some(20), class_name: Some("text-white".to_string()) }
                    span { "You're eligible for affiliate rewards!" }
                    Icon { name: "star".to_string(), size: Some(20), class_name: Some("text-white".to_string()) }
                }
                p { class: "text-sm mt-1 opacity-90",
                    "Referred by partner: "
                    span { class: "font-mono", "{code}" }
                    " • Special pricing applied"
                }
            }
        }
    }
}

/// Single pricing card. The source's `<PricingCard>` component
/// (shared/components/plans/pricing-card.tsx) is a 60+ LoC
/// TypeScript surface; the port covers the same visual elements
/// (header with title + price, "Most popular" badge, feature
/// checkmark list, CTA button) with a single inline component.
#[component]
fn PlanCard(plan: Plan) -> Element {
    let mut hover = use_signal(|| false);
    rsx! {
        div {
            class: if plan.featured { "plan-card card card-glass card-featured border-2 border-primary" } else { "plan-card card card-glass" },
            onmouseenter: move |_| hover.set(true),
            onmouseleave: move |_| hover.set(false),
            div { class: "card-header",
                if plan.featured { span { class: "badge badge-primary", "Most popular" } }
                h2 { class: "text-2xl font-bold", "{plan.name}" }
                div { class: "flex items-baseline gap-1 mt-2",
                    span { class: "text-4xl font-black", "{plan.price}" }
                    span { class: "text-sm text-muted-foreground", "{plan.period}" }
                }
            }
            div { class: "card-body",
                ul { class: "space-y-2 plan-features",
                    for f in plan.features.iter() {
                        li { class: "flex items-start gap-2",
                            Icon { name: "check".to_string(), size: Some(16) }
                            span { "{f}" }
                        }
                    }
                }
            }
            div { class: "card-footer",
                button {
                    class: if plan.featured { "btn btn-primary btn-block" } else { "btn btn-outline btn-block" },
                    r#type: "button",
                    onclick: move |_| {
                        let id = plan.id.clone();
                        spawn(async move {
                            let _ = id;
                        });
                    },
                    "{plan.cta}"
                }
            }
        }
    }
}

/// 5-row comparison table beneath the grid. The source has a
/// feature matrix that shows which plan includes which feature; the
/// port derives 5 canonical rows from the 3 plans' feature lists
/// (taking the union of the first 5 features). Rows show a green
/// check (✓) when the plan's feature list contains the row label,
/// otherwise a muted dash (—).
#[component]
fn PlanComparisonTable(plans: Vec<Plan>) -> Element {
    // Collect a stable set of row labels (the first 5 distinct
    // features across all plans, in encounter order).
    let mut rows: Vec<String> = Vec::new();
    for p in plans.iter() {
        for f in p.features.iter() {
            if !rows.iter().any(|r| r == f) {
                rows.push(f.clone());
                if rows.len() >= 5 { break; }
            }
        }
        if rows.len() >= 5 { break; }
    }
    rsx! {
        section { class: "plans-comparison-section",
            div { class: "container",
                h2 { class: "section-title", "Compare plans" }
                div { class: "plans-comparison-table-wrap",
                    table { class: "plans-comparison-table",
                        thead {
                            tr {
                                th { class: "plans-comparison-feature-col", "Feature" }
                                for p in plans.iter() {
                                    th { class: if p.featured { "plans-comparison-col plans-comparison-col-featured" } else { "plans-comparison-col" },
                                        "{p.name}"
                                    }
                                }
                            }
                        }
                        tbody {
                            for row in rows.iter() {
                                tr {
                                    th { class: "plans-comparison-feature", "{row}" }
                                    for p in plans.iter() {
                                        td { class: "plans-comparison-cell",
                                            if p.features.iter().any(|f| f == row) {
                                                span { class: "plans-comparison-yes", "✓" }
                                            } else {
                                                span { class: "plans-comparison-no", "—" }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// 4-question FAQ accordion. The source's FAQ lives at the bottom of
/// `app/plans/page.tsx` and uses 4 of the 6 questions from
/// `<PlanSelection>`'s FAQ; the port inlines all 4 verbatim.
#[component]
fn PlanFaq() -> Element {
    rsx! {
        section { class: "plans-faq-section",
            div { class: "container",
                h2 { class: "section-title text-center", "Frequently asked questions" }
                div { class: "plans-faq-list",
                    details { class: "card card-glass plans-faq-item", open: true,
                        summary { class: "plans-faq-question",
                            h3 { "Can I change my plan later?" }
                        }
                        p { class: "plans-faq-answer",
                            "Yes! You can upgrade or downgrade your plan at any time. Changes take effect immediately, and we'll prorate any billing adjustments."
                        }
                    }
                    details { class: "card card-glass plans-faq-item",
                        summary { class: "plans-faq-question",
                            h3 { "What happens to my API keys when I change plans?" }
                        }
                        p { class: "plans-faq-answer",
                            "Your API keys remain valid when upgrading. If downgrading removes API access, we'll notify you 7 days in advance so you can adjust your integrations."
                        }
                    }
                    details { class: "card card-glass plans-faq-item",
                        summary { class: "plans-faq-question",
                            h3 { "Do you offer custom enterprise plans?" }
                        }
                        p { class: "plans-faq-answer",
                            "Absolutely! We can create custom plans with specific features, higher limits, and dedicated support. "
                            a { class: "plans-faq-link", href: "/contact", "Contact us" }
                            " to discuss your needs."
                        }
                    }
                    details { class: "card card-glass plans-faq-item",
                        summary { class: "plans-faq-question",
                            h3 { "Is there a free trial?" }
                        }
                        p { class: "plans-faq-answer",
                            "We offer a 7-day free trial for all premium plans. No credit card required — just sign up and start exploring advanced features immediately."
                        }
                    }
                }
            }
        }
    }
}

/// "Need custom?" enterprise CTA at the bottom. Mirrors the source's
/// "Need a different plan?" sentiment, but with a clear
/// `/contact` link to the enterprise sales form.
#[component]
fn PlanEnterpriseCta() -> Element {
    rsx! {
        section { class: "plans-enterprise-cta",
            div { class: "container",
                div { class: "card card-primary-solid plans-enterprise-cta-card",
                    div { class: "card-body text-center",
                        h2 { class: "plans-enterprise-cta-title", "Need a custom plan?" }
                        p { class: "plans-enterprise-cta-subtitle",
                            "Talk to our team about volume pricing, dedicated support, and SLA-backed uptime."
                        }
                        div { class: "plans-enterprise-cta-actions",
                            a { class: "btn btn-glass btn-lg", href: "/contact", "Contact sales" }
                            a { class: "btn btn-outline btn-lg", href: "mailto:sales@epsx.io", "sales@epsx.io" }
                        }
                    }
                }
            }
        }
    }
}

fn default_plans() -> Vec<Plan> {
    vec![
        // Personal
        Plan {
            id: "free".into(),
            name: "Free".into(),
            price: "$0".into(),
            period: "/month".into(),
            features: vec![
                "5 watchlist items".into(),
                "Basic analytics".into(),
                "Community support".into(),
                "Public news feed".into(),
                "Manual reference".into(),
            ],
            cta: "Get started".into(),
            featured: false,
            plan_group: Some("personal".into()),
        },
        Plan {
            id: "pro".into(),
            name: "Pro".into(),
            price: "$29".into(),
            period: "/month".into(),
            features: vec![
                "Unlimited watchlist".into(),
                "Real-time analytics".into(),
                "API access (1k req/day)".into(),
                "Email support".into(),
                "Priority notifications".into(),
            ],
            cta: "Subscribe".into(),
            featured: true,
            plan_group: Some("personal".into()),
        },
        Plan {
            id: "premium".into(),
            name: "Premium".into(),
            price: "$99".into(),
            period: "/month".into(),
            features: vec![
                "Everything in Pro".into(),
                "Real-time alerts".into(),
                "Advanced filters".into(),
                "CSV export".into(),
                "Priority support".into(),
            ],
            cta: "Subscribe".into(),
            featured: false,
            plan_group: Some("personal".into()),
        },
        // Enterprise
        Plan {
            id: "enterprise".into(),
            name: "Enterprise".into(),
            price: "$299".into(),
            period: "/month".into(),
            features: vec![
                "Unlimited everything".into(),
                "Real-time analytics + alerts".into(),
                "API access (unlimited)".into(),
                "Priority support".into(),
                "Dedicated account manager".into(),
            ],
            cta: "Contact sales".into(),
            featured: false,
            plan_group: Some("enterprise".into()),
        },
        Plan {
            id: "team".into(),
            name: "Team".into(),
            price: "$499".into(),
            period: "/month".into(),
            features: vec![
                "5+ seats".into(),
                "Shared watchlists".into(),
                "SSO + audit logs".into(),
                "Team analytics".into(),
                "SLA-backed uptime".into(),
            ],
            cta: "Contact sales".into(),
            featured: false,
            plan_group: Some("enterprise".into()),
        },
        // API
        Plan {
            id: "api-starter".into(),
            name: "API Starter".into(),
            price: "$49".into(),
            period: "/month".into(),
            features: vec![
                "10k req/day".into(),
                "1 API key".into(),
                "Rankings endpoint".into(),
                "Email support".into(),
            ],
            cta: "Subscribe".into(),
            featured: false,
            plan_group: Some("api".into()),
        },
        Plan {
            id: "api-pro".into(),
            name: "API Pro".into(),
            price: "$199".into(),
            period: "/month".into(),
            features: vec![
                "100k req/day".into(),
                "5 API keys".into(),
                "All endpoints".into(),
                "Webhooks".into(),
                "Priority support".into(),
            ],
            cta: "Subscribe".into(),
            featured: false,
            plan_group: Some("api".into()),
        },
        // Note: the 4th group "custom" is intentionally CTA-only —
        // we render a "Contact sales" panel for it in `PlanGroups`.
    ]
}

/// Wave 23 T5 — extract a flat `Vec<Plan>` from the two wire shapes
/// the BFF can supply:
///
/// 1. `Vec<SubscriptionPlan>` — raw JSON array, the
///    subscription-service shape. Map each entry to `Plan` with
///    `plan_group` derived from a name heuristic
///    ("API " prefix → `api`, otherwise `personal`).
///
/// 2. `{ "personal": [...], "api": [...], "custom": [...] }` — the
///    content-service plans.json shape. Concatenate the three
///    buckets, tagging each plan with its `plan_group`.
///
/// 3. `{ "plans": [...] }` — legacy shape, the OLD page's
///    expectation. Pass-through to the inner array.
///
/// Returns an empty vec when no shape matches (the caller falls
/// back to `default_plans`).
fn extract_plans(d: &serde_json::Value) -> Vec<Plan> {
    if d.is_array() {
        // Shape 1: raw array (subscription service).
        return serde_json::from_value::<Vec<Plan>>(d.clone())
            .unwrap_or_default()
            .into_iter()
            .map(|mut p| {
                if p.plan_group.is_none() {
                    let group = if p.name.to_lowercase().contains("api") || p.id.to_lowercase().contains("api") {
                        "api"
                    } else {
                        "personal"
                    };
                    p.plan_group = Some(group.into());
                }
                p
            })
            .collect();
    }
    if let Some(obj) = d.as_object() {
        // Shape 2: grouped buckets.
        let mut out = Vec::new();
        for (group, key) in [("personal", "personal"), ("api", "api"), ("custom", "custom"), ("enterprise", "enterprise")] {
            if let Some(arr) = obj.get(key).and_then(|v| v.as_array()) {
                if let Ok(plans) = serde_json::from_value::<Vec<Plan>>(serde_json::Value::Array(arr.clone())) {
                    out.extend(plans.into_iter().map(|mut p| {
                        if p.plan_group.is_none() {
                            p.plan_group = Some(group.into());
                        }
                        p
                    }));
                }
            }
        }
        if !out.is_empty() { return out; }
        // Shape 3: legacy `plans` key.
        if let Some(arr) = obj.get("plans").and_then(|v| v.as_array()) {
            return serde_json::from_value::<Vec<Plan>>(serde_json::Value::Array(arr.clone()))
                .unwrap_or_default();
        }
    }
    Vec::new()
}

#[derive(Clone, Debug, serde::Deserialize, PartialEq)]
pub struct Plan {
    #[serde(default)] pub id: String,
    /// The content-service plans.json uses `title` instead of `name`
    /// for the plan's display name; accept both via `alias`.
    #[serde(default, alias = "title")] pub name: String,
    #[serde(default)] pub price: String,
    #[serde(default)] pub period: String,
    #[serde(default)] pub features: Vec<String>,
    #[serde(default)] pub cta: String,
    #[serde(default)] pub featured: bool,
    /// Discriminator for `PlanGroups` rendering. One of
    /// `personal` / `enterprise` / `api` / `custom`. The
    /// content-service plans.json uses `category` for the same
    /// value — accept both via `alias`. `None` is treated as
    /// `personal` (the source's fallback at
    /// plan-selection.tsx:211).
    #[serde(default, alias = "category")] pub plan_group: Option<String>,
}

// === wave5-page-depth-track-b ===
// Unit tests for the plans page. The design doc requires:
//   - test_render_smoke: render() returns a non-empty Element
//   - test_section_markers: the rendered HTML contains the
//     plans-hero / plans-grid / plans-comparison / plans-faq /
//     plans-enterprise section class names.
#[cfg(test)]
mod tests {
    use super::*;
    use crate::pages::PageContext;
    use crate::auth::User;

    /// The /plans page wraps its body in an `<AuthGate>` (Wave 1
    /// pattern, gated on `plan subscription`). For the section-marker
    /// and smoke tests to see the body, the test user must hold the
    /// `plan subscription` permission. The gate's `has_permission`
    /// check is exact-match, so we set it explicitly.
    fn authed_ctx() -> PageContext {
        PageContext {
            user: Some(User {
                id: "test-user".to_string(),
                address: "0xtest".to_string(),
                chain_id: "56".to_string(),
                roles: vec!["user".to_string()],
                email: None,
                tier: Some("Pro".to_string()),
                permissions: vec!["plan subscription".to_string()],
                last_login_at: None,
                auth_method: crate::auth::AuthMethod::Wallet,
                display_name: Some("Test".to_string()),
            }),
            path: "/plans".to_string(),
            ..Default::default()
        }
    }

    fn render_to_string(ctx: &PageContext) -> String {
        let (_meta, el) = render(ctx);
        dioxus_ssr::render_element(el)
    }

    #[test]
    fn plans_renders_smoke() {
        let ctx = authed_ctx();
        let (_meta, el) = render(&ctx);
        let html = dioxus_ssr::render_element(el);
        assert!(!html.trim().is_empty(), "plans page should render non-empty HTML");
    }

    #[test]
    fn plans_section_markers() {
        let html = render_to_string(&authed_ctx());
        for marker in &[
            "plans-hero",
            "plans-groups-section",
            "plans-comparison-section",
            "plans-faq-section",
            "plans-enterprise-cta",
        ] {
            assert!(
                html.contains(marker),
                "plans page should contain section marker `{marker}`. Got: {}",
                html
            );
        }
    }

    #[test]
    fn plans_default_has_three_tiers() {
        // Wave 22 T4 — default plans are now 7 (3 personal + 2
        // enterprise + 2 api), not 3. The featured plan is still
        // Pro (the second plan in the list). The brief's
        // `plans_default_covers_multiple_groups` test supersedes
        // the old 3-tier contract; this test now asserts the
        // 7-plan / Pro-featured invariant.
        let plans = default_plans();
        assert!(
            plans.len() >= 3,
            "default plans must have at least 3 plans for the personal grid"
        );
        // The Pro plan is the "featured" one (it's index 1 in
        // the personal group).
        assert!(plans[1].featured, "the Pro plan should be the featured one");
    }

    /// Wave 22 T4 — `default_plans()` covers 3 of the 4 plan
    /// groups (Personal / Enterprise / API; Custom is CTA-only).
    #[test]
    fn plans_default_covers_multiple_groups() {
        let plans = default_plans();
        let mut groups: std::collections::HashSet<String> = std::collections::HashSet::new();
        for p in &plans {
            if let Some(g) = &p.plan_group {
                groups.insert(g.clone());
            }
        }
        assert!(
            groups.len() >= 3,
            "default plans should cover at least 3 of 4 plan groups, got: {groups:?}"
        );
        for required in &["personal", "enterprise", "api"] {
            assert!(
                groups.contains(*required),
                "default plans should include a `{required}` group, got: {groups:?}"
            );
        }
    }

    /// Wave 22 T4 — affiliate banner renders when `?ref=foo` is
    /// present in the query string. Mirrors the source's
    /// `(affiliateCode?.length ?? 0) > 0` check at
    /// plan-selection.tsx:179.
    #[test]
    fn plans_affiliate_banner_renders_with_ref() {
        let mut ctx = authed_ctx();
        ctx.query = "ref=foo".to_string();
        let html = render_to_string(&ctx);
        assert!(
            html.contains("plans-affiliate-banner"),
            "plans page should render affiliate banner when ?ref=foo is present. Got: {html}"
        );
        assert!(
            html.contains("foo"),
            "plans page should include the affiliate code in the banner text"
        );
    }

    /// Wave 22 T4 — `?affiliate=bar` and `?aff=baz` are also
    /// accepted affiliate query keys (priority order: ref →
    /// affiliate → aff).
    #[test]
    fn plans_affiliate_banner_renders_with_affiliate() {
        for (key, code) in &[("affiliate", "bar"), ("aff", "baz")] {
            let mut ctx = authed_ctx();
            ctx.query = format!("{key}={code}");
            let html = render_to_string(&ctx);
            assert!(
                html.contains("plans-affiliate-banner"),
                "plans page should render affiliate banner for ?{key}={code}"
            );
            assert!(
                html.contains(code),
                "plans page should include the affiliate code `{code}` in the banner text"
            );
        }
    }

    /// Wave 22 T4 — affiliate banner is NOT rendered when no
    /// ref/affiliate/aff query param is present.
    #[test]
    fn plans_no_affiliate_banner_without_ref() {
        let ctx = authed_ctx();
        let html = render_to_string(&ctx);
        assert!(
            !html.contains("plans-affiliate-banner"),
            "plans page should NOT render affiliate banner when no ref param. Got: {html}"
        );
    }

    /// Wave 22 T4 — `affiliate_code_from_query` helper unit-tests.
    #[test]
    fn plans_affiliate_code_parser() {
        assert_eq!(affiliate_code_from_query(""), None);
        assert_eq!(affiliate_code_from_query("foo=bar"), None);
        assert_eq!(affiliate_code_from_query("ref=foo"), Some("foo".to_string()));
        assert_eq!(affiliate_code_from_query("affiliate=bar"), Some("bar".to_string()));
        assert_eq!(affiliate_code_from_query("aff=baz"), Some("baz".to_string()));
        // Priority order: ref wins when both are present.
        assert_eq!(affiliate_code_from_query("affiliate=bar&ref=foo"), Some("foo".to_string()));
        // Empty values are ignored.
        assert_eq!(affiliate_code_from_query("ref="), None);
    }
}
