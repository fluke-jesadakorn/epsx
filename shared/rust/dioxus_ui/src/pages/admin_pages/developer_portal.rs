//! /admin/developer-portal — API key management, usage, docs.
//!
//! Wave 6C Track D — 7 sections per the design doc
//! `docs/wave6c-live-render-and-1to1/design.md` §"Track D":
//! 1. `DeveloperPortalOverview`
//! 2. `ApiKeysTab`
//! 3. `ApiKeyCreateForm`
//! 4. `ApiKeyRevokeModal`
//! 5. `UsageAnalyticsTab`
//! 6. `DocumentationTab`
//! 7. `DeveloperPortalStats`
//!
//! All 7 sub-components live in `components::admin::developer`. This
//! page just composes them inside the `AdminAuthGate` wrapper.

use crate::primitives::*;
use crate::components::admin::developer::{
    ApiKeyCreateForm, ApiKeysTab, DeveloperPortalOverview, DeveloperPortalStats,
    DocumentationTab, UsageAnalyticsTab,
};

use dioxus::prelude::*;
use super::super::{PageContext, PageMeta};
use crate::auth::AdminAuthGate;

// ============================================================================
// Top-level page entry points
// ============================================================================

pub fn render(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::admin("Developer portal");
    (meta, rsx! { RenderDevPortal { ctx: ctx.clone() } })
}

#[component]
fn RenderDevPortal(ctx: PageContext) -> Element {
    let mut tab = use_signal(|| "overview".to_string());
    rsx! {
        AdminAuthGate {
            user: ctx.user.clone(),
            feature: Some("the developer portal".to_string()),
            required_permissions: Some(vec!["developer:manage".to_string()]),
            return_url: Some(ctx.path.clone()),
            div { class: "container page-content",
                div { class: "flex items-center justify-between mb-4",
                    h1 { class: "text-2xl font-bold", "Developer portal" }
                    a { class: "btn btn-primary", href: "/developer-portal/api-keys/create", Icon { name: "plus".to_string(), size: Some(16) } " Create API key" }
                }
                div { class: "tabs mb-4",
                    button { class: if *tab.read() == "overview" { "btn btn-primary" } else { "btn btn-outline" }, onclick: move |_| tab.set("overview".to_string()), "Overview" }
                    button { class: if *tab.read() == "keys" { "btn btn-primary" } else { "btn btn-outline" }, onclick: move |_| tab.set("keys".to_string()), "API keys" }
                    button { class: if *tab.read() == "usage" { "btn btn-primary" } else { "btn btn-outline" }, onclick: move |_| tab.set("usage".to_string()), "Usage" }
                    button { class: if *tab.read() == "docs" { "btn btn-primary" } else { "btn btn-outline" }, onclick: move |_| tab.set("docs".to_string()), "Documentation" }
                }
                if *tab.read() == "keys" {
                    // Section 2.
                    ApiKeysTab {}
                } else if *tab.read() == "usage" {
                    // Section 5.
                    UsageAnalyticsTab {}
                } else if *tab.read() == "docs" {
                    // Section 6.
                    DocumentationTab {}
                } else {
                    // Section 1.
                    DeveloperPortalOverview {}
                }
                // Section 7 — stats row is rendered inside the
                // overview, but the page-level stats bar is also
                // available for direct invocation.
                div { class: "hidden",
                    DeveloperPortalStats { total_keys: 5, active_keys: 4, total_requests: 12450, available_modules: 12 }
                }
            }
        }
    }
}

pub fn render_create_key(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::admin("Create API key");
    (meta, rsx! { RenderCreateKey { ctx: ctx.clone() } })
}

#[component]
fn RenderCreateKey(ctx: PageContext) -> Element {
    rsx! {
        AdminAuthGate {
            user: ctx.user.clone(),
            feature: Some("creating API keys".to_string()),
            required_permissions: Some(vec!["developer:manage".to_string()]),
            return_url: Some(ctx.path.clone()),
            div { class: "container page-content",
                a { class: "btn btn-sm btn-ghost mb-4", href: "/developer-portal", Icon { name: "arrow-left".to_string(), size: Some(16) } " Back" }
                // Section 3.
                ApiKeyCreateForm {}
            }
        }
    }
}

// ============================================================================
// Section markers (used by `tests::test_section_markers`):
//
//   1. "Developer portal overview" → "Developer overview" + "Recent API keys"
//   2. "API keys tab"              → "API key management" + DataTable
//   3. "API key create form"       → "Create API key" + "Module permissions"
//   4. "API key revoke modal"      → "Revoke API key?"
//   5. "Usage analytics tab"      → "API calls (7d)" + per-key cards
//   6. "Documentation tab"         → "Authentication" + "Endpoints" + "Modules"
//   7. "Developer portal stats"    → "Total API keys" / "Active keys" / "Total requests" / "Available modules"
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pages::PageContext;
    use crate::auth::User;

    /// Build an admin `User` with the `developer:manage` permission.
    fn test_user_admin() -> User {
        User {
            id: "test-admin".to_string(),
            address: "0xADMIN0000000000000000000000000000000001".to_string(),
            chain_id: "56".to_string(),
            roles: vec!["admin".to_string()],
            email: Some("admin@epsx.io".to_string()),
            tier: Some("admin".to_string()),
            permissions: vec!["developer:manage".to_string()],
            ..Default::default()
        }
    }

    /// Render the admin page's `Element` to an HTML string.
    fn render_to_string(el: Element) -> String {
        dioxus_ssr::render_element(el)
    }

    /// `test_render_smoke` — the page body header is rendered for an
    /// admin user with the right permission.
    #[test]
    fn test_render_smoke() {
        let ctx = PageContext {
            user: Some(test_user_admin()),
            path: "/developer-portal".to_string(),
            ..Default::default()
        };
        let (_, el) = render(&ctx);
        let html = render_to_string(el);
        assert!(
            html.contains("Developer portal"),
            "Developer portal page must render the title for an admin. Got: {}",
            html
        );
        // The default tab is the overview; assert section 1 marker.
        assert!(
            html.contains("Developer overview"),
            "Section 1 (DeveloperPortalOverview) marker missing. Got: {}",
            html
        );
    }

    /// `test_section_markers` — assert each of the 7 design-doc
    /// sections renders its section-marker text. We exercise each
    /// tab by calling the page render with a `path` that
    /// pre-selects the tab (the actual tab state is in a
    /// `use_signal`, so the default is always `overview` — we
    /// assert the overview sections directly via component
    /// invocations).
    #[test]
    fn test_section_markers() {
        // Section 1: DeveloperPortalOverview.
        let el = rsx! { DeveloperPortalOverview {} };
        let html = render_to_string(el);
        assert!(html.contains("Developer overview"), "section 1 marker missing");
        assert!(html.contains("Recent API keys"), "section 1 'Recent API keys' list missing");
        assert!(html.contains("Available modules"), "section 1 'Available modules' grid missing");
        // Section 7: DeveloperPortalStats (rendered inside Section 1).
        assert!(html.contains("Total API keys"), "section 7 (DeveloperPortalStats) marker missing");
        assert!(html.contains("Active keys"), "section 7 'Active keys' card missing");
        assert!(html.contains("Total requests"), "section 7 'Total requests' card missing");
        assert!(html.contains("Available modules"), "section 7 'Available modules' card missing");

        // Section 2: ApiKeysTab.
        let el = rsx! { ApiKeysTab {} };
        let html = render_to_string(el);
        assert!(html.contains("API key management"), "section 2 (ApiKeysTab) marker missing");
        assert!(html.contains("Filter by name..."), "section 2 (ApiKeysTab) filter placeholder missing");

        // Section 3: ApiKeyCreateForm.
        let el = rsx! { ApiKeyCreateForm {} };
        let html = render_to_string(el);
        assert!(html.contains("Create API key"), "section 3 (ApiKeyCreateForm) marker missing");
        assert!(html.contains("Module permissions"), "section 3 'Module permissions' label missing");
        assert!(html.contains("Module configuration required"), "section 3 amber callout missing");

        // Section 4: ApiKeyRevokeModal.
        let el = rsx! { crate::components::admin::developer::ApiKeyRevokeModal { client_name: "Production".to_string() } };
        let html = render_to_string(el);
        assert!(html.contains("Revoke API key?"), "section 4 (ApiKeyRevokeModal) marker missing");

        // Section 5: UsageAnalyticsTab.
        let el = rsx! { UsageAnalyticsTab {} };
        let html = render_to_string(el);
        assert!(html.contains("Usage analytics"), "section 5 (UsageAnalyticsTab) marker missing");
        assert!(html.contains("API calls (7d)"), "section 5 'API calls (7d)' chart missing");

        // Section 6: DocumentationTab.
        let el = rsx! { DocumentationTab {} };
        let html = render_to_string(el);
        assert!(html.contains("Documentation"), "section 6 (DocumentationTab) marker missing");
        assert!(html.contains("Authentication"), "section 6 'Authentication' section missing");
        assert!(html.contains("Endpoints"), "section 6 'Endpoints' section missing");
        assert!(html.contains("Modules"), "section 6 'Modules' section missing");
    }

    /// The create-key page renders the form, not the gate panel,
    /// for an admin user with the right permission.
    #[test]
    fn test_render_create_key_smoke() {
        let ctx = PageContext {
            user: Some(test_user_admin()),
            path: "/developer-portal/api-keys/create".to_string(),
            ..Default::default()
        };
        let (_, el) = render_create_key(&ctx);
        let html = render_to_string(el);
        assert!(
            html.contains("Create API key"),
            "Create-key page must render the form header. Got: {}",
            html
        );
        assert!(
            !html.contains("Admin access required"),
            "Create-key page must NOT render the admin gate panel for an admin. Got: {}",
            html
        );
    }
}
