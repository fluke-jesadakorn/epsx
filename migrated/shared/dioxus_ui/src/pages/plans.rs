//! /plans — plan list with comparison, subscribe button.

use crate::primitives::*;
use crate::feedback::*;

use dioxus::prelude::*;
use super::PageContext;
use super::PageMeta;
use crate::layout::{Navbar, Footer, PageHeader};
use crate::auth::AuthGate;

pub fn render(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::marketing("Plans");
    let plans_data: Option<serde_json::Value> = ctx.params.get("data_plans")
        .and_then(|s| serde_json::from_str(s).ok());
    let plans: Vec<Plan> = plans_data.as_ref()
        .and_then(|d| serde_json::from_value(d.get("plans").cloned().or_else(|| Some(d.clone())).unwrap_or(serde_json::json!([]))).ok())
        .unwrap_or_else(default_plans);

    (meta, rsx! {
        Navbar { user: ctx.user.clone(), current_path: Some(ctx.path.clone()) }
        AuthGate { user: ctx.user.clone(), feature: Some("plan subscription".to_string()),
            div { class: "container page-content",
                PageHeader {
                    title: "Plans".to_string(),
                    description: Some("Pick a plan that fits your trading volume".to_string()),
                    icon: Some("zap".to_string()),
                }
                div { class: "grid grid-cols-1 md:grid-cols-3 gap-6",
                    for p in plans.iter() {
                        PlanCard { plan: p.clone() }
                    }
                }
            }
        }
        Footer {}
    })
}

fn default_plans() -> Vec<Plan> {
    vec![
        Plan { id: "free".into(), name: "Free".into(), price: "$0".into(), period: "/month".into(), features: vec!["5 watchlist items".into(), "Basic analytics".into(), "Community support".into()], cta: "Get started".into(), featured: false },
        Plan { id: "pro".into(), name: "Pro".into(), price: "$29".into(), period: "/month".into(), features: vec!["Unlimited watchlist".into(), "Real-time analytics".into(), "API access (1k req/day)".into(), "Email support".into()], cta: "Subscribe".into(), featured: true },
        Plan { id: "enterprise".into(), name: "Enterprise".into(), price: "$299".into(), period: "/month".into(), features: vec!["Unlimited everything".into(), "Real-time analytics + alerts".into(), "API access (unlimited)".into(), "Priority support".into(), "Dedicated account manager".into()], cta: "Contact sales".into(), featured: false },
    ]
}

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
                ul { class: "space-y-2",
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
