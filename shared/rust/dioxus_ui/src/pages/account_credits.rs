//! /account/credits — credit balance + transaction history (the
//! "credit ledger" widget).
//!
//! Wave 6C Track E — the 6 account_credits sub-components
//! (`RenderAccountCredits`, `CreditBalance`, `CreditTopUp`,
//! `QuickAmount`, `CreditTransactionList`, `CreditTransactionRow`)
//! were extracted to `crate::components::user::account_credits`.
//! This page file keeps the `render()` entry point and delegates.

use crate::components::user::account_credits::*;
use crate::primitives::*;

use dioxus::prelude::*;
use super::PageContext;
use super::PageMeta;

pub fn render(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::app("Credits");
    (meta, rsx! { RenderAccountCredits { ctx: ctx.clone() } })
}

// =============================================================================
// Tests
// =============================================================================
//
// - `test_render_smoke` — render(&empty_ctx()) returns non-empty Element.
// - `test_section_markers` — SSR'd HTML contains every section-marker
//   class the design doc claims.

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::user::User;
    use crate::auth::user::AuthMethod;

    fn authed_ctx() -> PageContext {
        PageContext {
            user: Some(User {
                id: "u-1".to_string(),
                address: "0x1234abcd".to_string(),
                chain_id: "1".to_string(),
                roles: vec!["user".to_string()],
                email: Some("test@epsx.io".to_string()),
                tier: Some("pro".to_string()),
                permissions: vec!["profile:read".to_string()],
                last_login_at: None,
                auth_method: AuthMethod::default(),
                display_name: Some("EPSX tester".to_string()),
            }),
            path: "/account/credits".to_string(),
            ..Default::default()
        }
    }

    #[test]
    fn test_render_smoke() {
        let (_meta, el) = render(&authed_ctx());
        let html = dioxus_ssr::render_element(el);
        assert!(!html.is_empty(), "account/credits must render non-empty HTML. Got: {}", html);
        assert!(html.len() > 100, "account/credits HTML is suspiciously short ({} bytes).", html.len());
    }

    #[test]
    fn test_section_markers() {
        let (_meta, el) = render(&authed_ctx());
        let html = dioxus_ssr::render_element(el);
        for marker in &[
            "credits-ledger-page",
            "credits-balance-row",
            "credits-balance-available",
            "credits-balance-earned",
            "credits-balance-spent",
            "credits-topup",
            "credits-transaction-list",
            "credits-ledger-row",
        ] {
            let needle_a = format!("class=\"{}\"", marker);
            let needle_b = format!("class=\"{mark} ", mark = marker);
            let needle_c = format!(" {}\"", marker);
            let needle_d = format!(" {} ", marker);
            assert!(
                html.contains(&needle_a)
                    || html.contains(&needle_b)
                    || html.contains(&needle_c)
                    || html.contains(&needle_d),
                "account/credits must contain section marker '{}'. Got: {}",
                marker, html
            );
        }
    }
}
