//! Sub-components extracted from `pages/plans.rs` during
//! Wave 6C Track E (1:1 user-side component parity).
//!
//! Six named sub-components + the `Plan` data struct +
//! `default_plans()` helper are lifted into this module. The page
//! file's `render()` becomes a 1:1 composition of these.
//!
//! Source: `apps-old/frontend/app/plans/page.tsx` (74 LoC) +
//! `components/plans/plan-selection.tsx`.

use crate::primitives::*;
use crate::feedback::*;

use dioxus::prelude::*;

/// `Plan` — the data shape the plans grid + comparison table
/// consume. Mirrors the source's plan payload (id, name, price,
/// period, features, cta, featured).
#[derive(Clone, Debug, serde::Deserialize, PartialEq)]
pub struct Plan {
    #[serde(default)] pub id: String,
    #[serde(default)] pub name: String,
    #[serde(default)] pub price: String,
    #[serde(default)] pub period: String,
    #[serde(default)] pub features: Vec<String>,
    #[serde(default)] pub cta: String,
    #[serde(default)] pub featured: bool,
}

/// Page-level hero. The source uses a gradient-text h1 ("Choose
/// Your EPSX Plan") and a one-line pitch below it. The port
/// keeps the copy verbatim and reuses the design-system
/// `section-title` / `section-sub` classes.
#[component]
pub fn PlansHero() -> Element {
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

/// 3-tier pricing grid. Iterates the plans vec and renders one
/// `PlanCard` per plan.
#[component]
pub fn PlanGrid(plans: Vec<Plan>) -> Element {
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

/// Single pricing card. The source's `<PricingCard>` component
/// is a 60+ LoC TypeScript surface; the port covers the same
/// visual elements (header with title + price, "Most popular"
/// badge, feature checkmark list, CTA button) with a single
/// inline component.
#[component]
pub fn PlanCard(plan: Plan) -> Element {
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
/// feature matrix that shows which plan includes which feature;
/// the port derives 5 canonical rows from the 3 plans' feature
/// lists (taking the union of the first 5 features). Rows show
/// a green check (✓) when the plan's feature list contains the
/// row label, otherwise a muted dash (—).
#[component]
pub fn PlanComparisonTable(plans: Vec<Plan>) -> Element {
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

/// 4-question FAQ accordion. Source FAQ is inlined at the bottom
/// of `app/plans/page.tsx`; the port keeps all 4 questions.
#[component]
pub fn PlanFaq() -> Element {
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

/// "Need custom?" enterprise CTA at the bottom. Mirrors the
/// source's "Need a different plan?" sentiment, but with a clear
/// `/contact` link to the enterprise sales form.
#[component]
pub fn PlanEnterpriseCta() -> Element {
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

/// Default 3-tier plan list used when the BFF hasn't pre-fetched
/// plans (admin BFF, dev mode, etc.). The middle plan (Pro) is
/// the "featured" one.
pub fn default_plans() -> Vec<Plan> {
    vec![
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
        },
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
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Wave 6C Track E — `test_render_smoke` for the extracted
    /// plan sub-components. Asserts each one carries its
    /// section-marker class + sample data.
    #[test]
    fn plans_subcomponents_render_smoke() {
        let plans = default_plans();

        // PlansHero
        let el = rsx! { PlansHero {} };
        let html = dioxus_ssr::render_element(el);
        assert!(html.contains("plans-hero"), "PlansHero missing section-marker");
        assert!(html.contains("Choose Your EPSX Plan"));

        // PlanGrid + PlanCard
        let el = rsx! { PlanGrid { plans: plans.clone() } };
        let html = dioxus_ssr::render_element(el);
        assert!(html.contains("plans-grid-section"), "PlanGrid missing section-marker");
        let card_count = html.matches("plan-card").count();
        assert!(card_count >= 3, "PlanGrid must render 3 cards, got {} plan-card markers", card_count);
        assert!(html.contains("Most popular"), "PlanGrid must surface the Most popular badge");
        assert!(html.contains("Featured") || html.contains("card-featured"), "PlanGrid must apply featured class");

        // PlanComparisonTable
        let el = rsx! { PlanComparisonTable { plans: plans.clone() } };
        let html = dioxus_ssr::render_element(el);
        assert!(html.contains("plans-comparison-section"), "PlanComparisonTable missing section-marker");
        assert!(html.contains("Compare plans"));

        // PlanFaq
        let el = rsx! { PlanFaq {} };
        let html = dioxus_ssr::render_element(el);
        assert!(html.contains("plans-faq-section"), "PlanFaq missing section-marker");
        for q in &["Can I change my plan later?", "What happens to my API keys", "Do you offer custom enterprise", "Is there a free trial?"] {
            assert!(html.contains(q), "PlanFaq missing question '{}'", q);
        }

        // PlanEnterpriseCta
        let el = rsx! { PlanEnterpriseCta {} };
        let html = dioxus_ssr::render_element(el);
        assert!(html.contains("plans-enterprise-cta"), "PlanEnterpriseCta missing section-marker");
        assert!(html.contains("Need a custom plan?"));

        // Default plans shape: 3 tiers, Pro featured.
        let default = default_plans();
        assert_eq!(default.len(), 3, "default plans must be 3 tiers");
        assert!(default[1].featured, "Pro (middle) must be featured");
    }
}
