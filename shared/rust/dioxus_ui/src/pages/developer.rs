//! /developer + /developer/usage + /developer/docs — developer portal.
//!
//! Wave 6C Track E — the 10 developer sub-components were extracted
//! to `crate::components::user::developer`. The `ApiKey` data type
//! and the sample data helpers are also lifted and `pub`.

use crate::components::user::developer::{DeveloperOverviewBody, DeveloperUsageBody};
use crate::data_table::{Column, DataTable, Row};
use crate::feedback::*;
use crate::primitives::*;
use crate::layout::main_layout::MainLayout;
use crate::layout::{PageHeader, DeveloperShell};

use dioxus::prelude::*;
use super::PageContext;
use super::PageMeta;

/// `/developer` — overview.
pub fn render_overview(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::app("Developer");
    let body = rsx! { DeveloperOverviewBody { ctx: ctx.clone() } };
    (meta, body)
}

/// `/developer/usage` — usage monitor.
pub fn render_usage(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::app("API usage");
    let body = rsx! { DeveloperUsageBody { ctx: ctx.clone() } };
    (meta, body)
}

/// `/developer/docs` — endpoints sidebar + endpoint cards.
pub fn render_docs(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::app("API documentation");

    (meta, rsx! {
        MainLayout { ctx: ctx.clone(),
            DeveloperShell { current_path: ctx.path.clone(),
                div { class: "container page-content",
                    PageHeader {
                        title: "API documentation".to_string(),
                        description: Some("REST endpoints, request/response schemas, and examples".to_string()),
                        icon: Some("book".to_string()),
                    }
                    div { class: "developer-docs grid grid-cols-1 md:grid-cols-3 gap-4",
                        "data-section": "developer-docs",
                        div { class: "card card-glass md:col-span-1",
                            div { class: "card-header", h3 { class: "card-title", "Endpoints" } }
                            div { class: "card-body",
                                ul { class: "docs-nav space-y-1",
                                    li { a { class: "block p-2 rounded hover:bg-muted", href: "#auth", "Auth" } }
                                    li { a { class: "block p-2 rounded hover:bg-muted", href: "#payments", "Payments" } }
                                    li { a { class: "block p-2 rounded hover:bg-muted", href: "#subscriptions", "Subscriptions" } }
                                    li { a { class: "block p-2 rounded hover:bg-muted", href: "#analytics", "Analytics" } }
                                    li { a { class: "block p-2 rounded hover:bg-muted", href: "#notifications", "Notifications" } }
                                }
                            }
                        }
                        div { class: "md:col-span-2 space-y-4",
                            div { class: "card card-glass",
                                div { class: "card-body",
                                    h2 { id: "auth", class: "text-xl font-bold", "Auth" }
                                    p { class: "text-muted-foreground mt-2", "All API calls require a Bearer token. Get one via the SIWE flow." }
                                    pre { class: "code-block mt-3", "POST /api/v1/auth/challenge\nPOST /api/v1/auth/siwe" }
                                }
                            }
                            div { class: "card card-glass",
                                div { class: "card-body",
                                    h2 { id: "payments", class: "text-xl font-bold", "Payments" }
                                    p { class: "text-muted-foreground mt-2", "Create and confirm payment intents." }
                                    pre { class: "code-block mt-3", "POST /api/v1/payment/intents\nPOST /api/v1/payment/intents/[id]/confirm" }
                                }
                            }
                            div { class: "card card-glass",
                                div { class: "card-body",
                                    h2 { id: "subscriptions", class: "text-xl font-bold", "Subscriptions" }
                                    p { class: "text-muted-foreground mt-2", "Create plans and subscribe." }
                                    pre { class: "code-block mt-3", "POST /api/v1/subscription/plans\nPOST /api/v1/subscription/subscribe" }
                                }
                            }
                        }
                    }
                }
            }
        }
    })
}

// === wave6-auth-pages-depth-track-b ===
// Unit tests for the developer page.
#[cfg(test)]
mod tests {
    use super::*;

    fn authed_ctx() -> PageContext {
        PageContext {
            user: Some(crate::auth::User {
                id: "test-user".to_string(),
                address: "0xtest".to_string(),
                chain_id: "56".to_string(),
                roles: vec!["user".to_string()],
                email: None,
                tier: Some("Pro".to_string()),
                permissions: vec!["developer:read".to_string()],
                last_login_at: None,
                auth_method: crate::auth::AuthMethod::Wallet,
                display_name: Some("Test".to_string()),
            }),
            path: "/developer".to_string(),
            ..Default::default()
        }
    }

    fn render_overview_to_string(ctx: &PageContext) -> String {
        let (_meta, el) = render_overview(ctx);
        dioxus_ssr::render_element(el)
    }

    /// Wave 6A — `test_render_smoke`. The `render_overview` function
    /// returns a non-empty HTML string.
    #[test]
    fn test_render_smoke() {
        let ctx = authed_ctx();
        let (_meta, el) = render_overview(&ctx);
        let html = dioxus_ssr::render_element(el);
        assert!(
            !html.trim().is_empty(),
            "developer overview should render non-empty HTML"
        );
    }

    /// Wave 6A — `test_section_markers`. The rendered HTML must
    /// contain each of the 6 overview section markers.
    #[test]
    fn test_section_markers() {
        let html = render_overview_to_string(&authed_ctx());
        for marker in &[
            "developer-stats-cards",
            "api-keys-list",
            "api-key-create-form",
            "plan-transfer-list",
            "permission-list",
            "docs-quick-links",
        ] {
            assert!(
                html.contains(marker),
                "developer overview should contain section marker `{marker}`. Got: {html}"
            );
        }
    }
}
