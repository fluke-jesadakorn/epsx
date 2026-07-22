//! `/policies` — intentionally unavailable.
//!
//! The legacy development application has no policies route and the deployed
//! admin application returns HTTP 404 here. Keep this leaf as a direct
//! delegation to the shared not-found page so it cannot imply that policy
//! data, evaluation telemetry, or mutations are wired to a live backend.

use dioxus::prelude::*;

use super::super::{not_found, PageContext, PageMeta};

/// Preserve the public admin-page render contract while returning an explicit
/// not-found status for every session state.
pub fn render(ctx: &PageContext) -> (PageMeta, Element) {
    not_found::render(ctx)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::user::{AuthMethod, User};
    use crate::pages::PageStatus;

    fn context(signed_in: bool) -> PageContext {
        PageContext {
            path: "/policies".to_string(),
            user: signed_in.then(|| User {
                id: "policy-test-user".to_string(),
                address: "0x1234abcd5678ef90".to_string(),
                chain_id: "56".to_string(),
                roles: vec!["admin".to_string()],
                email: None,
                tier: None,
                permissions: Vec::new(),
                last_login_at: None,
                auth_method: AuthMethod::Wallet,
                display_name: None,
            }),
            ..Default::default()
        }
    }

    fn rendered(ctx: &PageContext) -> (PageMeta, String) {
        let (meta, element) = render(ctx);
        (meta, dioxus_ssr::render_element(element))
    }

    #[test]
    fn signed_out_and_signed_in_requests_are_explicit_not_found() {
        for signed_in in [false, true] {
            let (meta, html) = rendered(&context(signed_in));

            assert_eq!(meta.status, PageStatus::NotFound);
            assert!(html.contains("404"), "missing 404 content: {html}");
            assert!(
                html.contains("Page not found"),
                "missing not-found title: {html}"
            );
        }
    }

    #[test]
    fn policy_samples_controls_and_frontend_gate_are_absent() {
        for signed_in in [false, true] {
            let (_, html) = rendered(&context(signed_in));

            for forbidden in [
                "policies:manage",
                "Total Policies",
                "Active Members",
                "$24,512",
                "Admin full access",
                "New policy",
                "Run test",
                "Filter by name, type, effect, or status",
                "data-section=\"policy-builder\"",
                "data-section=\"policy-monitor\"",
            ] {
                assert!(
                    !html.contains(forbidden),
                    "not-found response exposed `{forbidden}`: {html}"
                );
            }
        }
    }

    #[test]
    fn request_query_and_params_are_not_reflected() {
        let mut ctx = context(true);
        ctx.query = "draft=QUERY_SECRET_7f08d1".to_string();
        ctx.params
            .insert("policyId".to_string(), "PARAM_SECRET_913ea2".to_string());

        let (_, html) = rendered(&ctx);
        assert!(
            !html.contains("QUERY_SECRET_7f08d1"),
            "query leaked: {html}"
        );
        assert!(
            !html.contains("PARAM_SECRET_913ea2"),
            "param leaked: {html}"
        );
    }

    #[test]
    fn leaf_does_not_render_admin_or_page_shells() {
        let (_, html) = rendered(&context(true));

        for forbidden in [
            "admin-shell",
            "admin-sidebar",
            "admin-footer",
            "<header",
            "<footer",
            "<main",
        ] {
            assert!(
                !html.contains(forbidden),
                "not-found leaf rendered nested shell marker `{forbidden}`: {html}"
            );
        }
    }
}
