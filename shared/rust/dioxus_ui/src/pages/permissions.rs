//! /permissions — user permissions with matrix + feature list.
//!
//! Wave 6C Track E — the 4 permissions sub-components
//! (PermissionsMatrix, FeatureList, RequestAccessCTA,
//! PermissionCategoryBreakdown) were extracted to
//! `crate::components::user::permissions`. The page file's
//! `RenderPermissions` wrapper orchestrates them.

use crate::primitives::*;
use crate::feedback::*;

use dioxus::prelude::*;
use super::PageContext;
use super::PageMeta;
use crate::layout::main_layout::MainLayout;
use crate::layout::PageHeader;
use crate::auth::AuthGate;
use crate::components::user::permissions::{
    FeatureList, PermissionCategoryBreakdown, PermissionsMatrix, RequestAccessCTA,
};

pub fn render(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::app("Permissions");
    (meta, rsx! { RenderPermissions { ctx: ctx.clone() } })
}

#[component]
fn RenderPermissions(ctx: PageContext) -> Element {
    let mut tab = use_signal(|| "permissions".to_string());
    rsx! {
        MainLayout { ctx: ctx.clone(),
            AuthGate { user: ctx.user.clone(), feature: Some("your permissions".to_string()),
                required_permissions: Some(vec!["permissions:read".to_string()]),
                return_url: Some(ctx.path.clone()),
                div { class: "container page-content max-w-6xl",
                    PageHeader { title: "My permissions".to_string(), description: Some("Active permissions, history, and analytics".to_string()), icon: Some("shield".to_string()) }
                    div { class: "grid grid-cols-1 md:grid-cols-3 gap-4 mb-4 permissions-stats",
                        StatCard { label: "Total permissions".to_string(), value: "12".to_string(), icon: Some("key".to_string()) }
                        StatCard { label: "Active".to_string(), value: "10".to_string(), icon: Some("check".to_string()) }
                        StatCard { label: "Expiring soon".to_string(), value: "2".to_string(), icon: Some("clock".to_string()) }
                    }
                    div { class: "card card-glass",
                        div { class: "card-header",
                            h2 { class: "card-title", "Permission details" }
                            div { class: "tabs permissions-tab-nav",
                                button { class: if *tab.read() == "permissions" { "btn btn-sm btn-primary" } else { "btn btn-sm btn-outline" }, onclick: move |_| tab.set("permissions".to_string()), "Permissions" }
                                button { class: if *tab.read() == "analytics" { "btn btn-sm btn-primary" } else { "btn btn-sm btn-outline" }, onclick: move |_| tab.set("analytics".to_string()), "Analytics" }
                                button { class: if *tab.read() == "history" { "btn btn-sm btn-primary" } else { "btn btn-sm btn-outline" }, onclick: move |_| tab.set("history".to_string()), "History" }
                            }
                        }
                        div { class: "card-body",
                            if *tab.read() == "permissions" {
                                div { class: "space-y-6 permissions-matrix-panel",
                                    PermissionsMatrix {}
                                    PermissionCategoryBreakdown {}
                                    FeatureList {}
                                    RequestAccessCTA {}
                                }
                            } else if *tab.read() == "analytics" {
                                div { class: "grid grid-cols-1 md:grid-cols-2 gap-4",
                                    div { class: "card card-glass", div { class: "card-header", h4 { "By category" } } div { class: "card-body",
                                        crate::charts::ChartDonut { data: vec![("Trade".to_string(), 4.0, "#22d3ee".to_string()), ("View".to_string(), 3.0, "#22c55e".to_string()), ("Pay".to_string(), 3.0, "#f59e0b".to_string())], size: 180, thickness: 28 } } }
                                    div { class: "card card-glass", div { class: "card-header", h4 { "By source" } } div { class: "card-body",
                                        crate::charts::ChartDonut { data: vec![("Plan".to_string(), 8.0, "#3b82f6".to_string()), ("Direct".to_string(), 2.0, "#9ca3af".to_string())], size: 180, thickness: 28 } } }
                                }
                            } else {
                                div { class: "table-wrap permissions-history-table",
                                    table { class: "table", thead { tr { th { "Date" } th { "Action" } th { "Permission" } th { "Source" } } } tbody {
                                        tr { td { "2024-09-15" } td { span { class: "badge badge-success", "Granted" } } td { "trade" } td { "Pro plan" } }
                                        tr { td { "2024-08-01" } td { span { class: "badge badge-info", "Updated" } } td { "view" } td { "Pro plan" } }
                                        tr { td { "2024-07-20" } td { span { class: "badge badge-danger", "Revoked" } } td { "admin" } td { "— " } }
                                    } } }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::User;
    use crate::pages::PageContext;
    use crate::auth::user::AuthMethod;

    fn ctx(path: &str) -> PageContext {
        let user = User {
            id: "u1".to_string(),
            address: "0x1234…abcd".to_string(),
            chain_id: "56".to_string(),
            roles: vec!["user".to_string()],
            email: Some("test@epsx.io".to_string()),
            tier: Some("Pro".to_string()),
            permissions: vec!["permissions:read".to_string()],
            last_login_at: None,
            auth_method: AuthMethod::Wallet,
            display_name: Some("Test".to_string()),
        };
        PageContext { user: Some(user), path: path.to_string(), ..Default::default() }
    }

    #[test]
    fn test_render_smoke() {
        let (_meta, element) = render(&ctx("/permissions"));
        let html = dioxus_ssr::render_element(element);
        assert!(html.contains("My permissions"), "/permissions header must render. Got: {}", html);
    }

    #[test]
    fn test_section_markers() {
        let (_meta, element) = render(&ctx("/permissions"));
        let html = dioxus_ssr::render_element(element);
        for marker in [
            "permissions-stats",
            "permissions-tab-nav",
            "permissions-matrix",
            "permissions-feature-list",
            "permissions-request-access",
        ] {
            assert!(html.contains(marker), "missing section marker: {}", marker);
        }
    }
}
