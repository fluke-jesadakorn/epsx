//! /payment + /payment/[type]/[id] — payment flow.
//!
//! Wave 6C Track E — the 9 payment sub-components
//! (`RenderPayment`, `SecurityFooter`, `CurrentAccessCard`,
//! `PaymentFlowSteps`, `PlanComparisonCard`, `ChainVerificationCard`,
//! `UpgradeBanner`, `UnifiedPaymentFlow`, `PaymentDetailPanel`)
//! were extracted to `crate::components::user::payment`. The
//! `ThemeConfig` data type and `theme_for` helper are also lifted
//! and `pub`.

use crate::components::user::payment::{PaymentDetailPanel, RenderPayment, theme_for};
use crate::primitives::*;

use dioxus::prelude::*;
use super::PageContext;
use super::PageMeta;

pub fn render(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::app("Payment");
    (meta, rsx! { RenderPayment { ctx: ctx.clone() } })
}

pub fn render_dynamic(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::app("Payment");
    let ptype = ctx.params.get("type").cloned().unwrap_or_else(|| "subscription".to_string());
    let pid = ctx.params.get("id").cloned().unwrap_or_default();
    (meta, rsx! {
        PaymentDetailPanel { ptype: ptype.clone(), pid: pid.clone(), ctx: ctx.clone() }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::User;
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
