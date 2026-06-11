//! `/plans` — plan list with comparison, FAQ, and enterprise CTA.
//!
//! Wave 6C Track E — the 6 plan sub-components + the `Plan` data
//! struct + `default_plans()` helper were extracted to
//! `crate::components::user::plans`. The page file's `render()`
//! composes them.

use dioxus::prelude::*;
use super::PageContext;
use super::PageMeta;
use crate::layout::main_layout::MainLayout;
use crate::auth::AuthGate;
use crate::auth::ProgressiveAuthBanner;
use crate::components::user::plans::{
    default_plans, Plan, PlanComparisonTable, PlanEnterpriseCta, PlanFaq, PlanGrid, PlansHero,
};

pub fn render(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::marketing("Plans");
    let plans_data: Option<serde_json::Value> = ctx.params.get("data_plans")
        .and_then(|s| serde_json::from_str(s).ok());
    let plans: Vec<Plan> = plans_data.as_ref()
        .and_then(|d| serde_json::from_value(d.get("plans").cloned().or_else(|| Some(d.clone())).unwrap_or(serde_json::json!([]))).ok())
        .unwrap_or_else(default_plans);

    // Fallback to the canonical 3-tier list when the BFF hasn't
    // pre-fetched plans (admin BFF, dev mode, etc.). The first plan
    // is the "featured" one (Pro) so the "Most popular" badge lands
    // on the right card even without BFF data.
    let plans = if plans.is_empty() { default_plans() } else { plans };

    (meta, rsx! {
        MainLayout { ctx: ctx.clone(),
            if ctx.user.is_none() {
                ProgressiveAuthBanner {
                    feature: Some("plan subscription".to_string()),
                }
            }
            AuthGate { user: ctx.user.clone(), feature: Some("plan subscription".to_string()),
                PlansHero {}
                PlanGrid { plans: plans.clone() }
                PlanComparisonTable { plans: plans.clone() }
                PlanFaq {}
                PlanEnterpriseCta {}
            }
        }
    })
}

// === wave5-page-depth-track-b ===
// Unit tests for the plans page.
#[cfg(test)]
mod tests {
    use super::*;
    use crate::pages::PageContext;
    use crate::auth::User;

    /// The /plans page wraps its body in an `<AuthGate>` (Wave 1
    /// pattern, gated on `plan subscription`). For the
    /// section-marker and smoke tests to see the body, the test
    /// user must hold the `plan subscription` permission. The
    /// gate's `has_permission` check is exact-match, so we set it
    /// explicitly.
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
            "plans-grid-section",
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
        let plans = default_plans();
        assert_eq!(plans.len(), 3, "default plans must be a 3-tier grid");
        // The middle plan (Pro) is the "featured" one.
        assert!(plans[1].featured, "the Pro plan should be the featured one");
    }
}
