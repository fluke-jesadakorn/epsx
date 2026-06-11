//! Sub-components for `/admin/developer-portal` — Wave 6C Track D.
//!
//! 1:1 mirror of `apps-old/admin-frontend/components/admin/developer-portal/*.tsx`:
//!   1. `DeveloperPortalOverview`  — 4 stat cards + recent keys + modules
//!   2. `ApiKeysTab`                — API key DataTable
//!   3. `ApiKeyCreateForm`          — create-key form (211 LoC in source)
//!   4. `ApiKeyRevokeModal`         — revoke confirmation modal
//!   5. `UsageAnalyticsTab`         — 7-day chart + per-key breakdown
//!   6. `DocumentationTab`          — API docs viewer
//!   7. `DeveloperPortalStats`      — 4 stat cards
//!
//! Helper components (`ApiKeyListItem`, `ModuleCard`, `UsageKeyCard`,
//! `DocModuleItem`) stay private.

use crate::primitives::*;
use crate::primitives::admin_metric_card::{AdminMetricCard, MetricTrend};
use crate::data_table::{Column, DataTable, Row, SortDir};

use dioxus::prelude::*;

// ============================================================================
// Section 7: DeveloperPortalStats
// ============================================================================

#[component]
pub fn DeveloperPortalStats(
    total_keys: u32,
    active_keys: u32,
    total_requests: u32,
    available_modules: u32,
) -> Element {
    rsx! {
        div { class: "grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6 developer-portal-stats",
            AdminMetricCard {
                label: "Total API keys".to_string(),
                value: format!("{}", total_keys),
                trend: Some(MetricTrend::Up(2.0)),
                sparkline_data: Some(vec![1.0, 2.0, 2.0, 3.0, 3.0, 4.0, total_keys as f32]),
                icon: Some("key".to_string()),
            }
            AdminMetricCard {
                label: "Active keys".to_string(),
                value: format!("{}", active_keys),
                trend: Some(MetricTrend::Up(1.5)),
                sparkline_data: Some(vec![2.0, 2.0, 2.5, 2.5, 3.0, 3.0, active_keys as f32]),
                icon: Some("activity".to_string()),
            }
            AdminMetricCard {
                label: "Total requests".to_string(),
                value: format!("{}", total_requests),
                trend: Some(MetricTrend::Up(8.2)),
                sparkline_data: Some(vec![800.0, 900.0, 950.0, 1000.0, 1100.0, 1150.0, total_requests as f32]),
                icon: Some("bar-chart-3".to_string()),
            }
            AdminMetricCard {
                label: "Available modules".to_string(),
                value: format!("{}", available_modules),
                trend: Some(MetricTrend::Flat),
                sparkline_data: Some(vec![12.0, 12.0, 12.0, 12.0, 12.0, 12.0, available_modules as f32]),
                icon: Some("settings".to_string()),
            }
        }
    }
}

// ============================================================================
// Section 1: DeveloperPortalOverview
// ============================================================================

#[component]
pub fn DeveloperPortalOverview() -> Element {
    rsx! {
        div { class: "space-y-6 developer-portal-overview",
            div { class: "flex items-center justify-between",
                div {
                    h2 { class: "text-lg font-semibold text-foreground", "Developer overview" }
                    p { class: "text-sm text-muted-foreground", "API keys, requests, and modules" }
                }
                a { class: "btn btn-primary", href: "/developer-portal/api-keys/create", Icon { name: "plus".to_string(), size: Some(16) } " Create API key" }
            }
            DeveloperPortalStats {
                total_keys: 5,
                active_keys: 4,
                total_requests: 12450,
                available_modules: 12,
            }
            div { class: "card card-glass overflow-hidden",
                div { class: "h-[3px] bg-gradient-to-r from-[#1fc7d4] to-[#7645d9]" }
                div { class: "p-6 border-b border-border/20",
                    div { class: "flex items-center justify-between",
                        h3 { class: "text-lg font-medium text-foreground", "Recent API keys" }
                        a { class: "btn btn-sm btn-outline", href: "/developer-portal", "View all" }
                    }
                }
                div { class: "divide-y divide-border/20 text-sm",
                    ApiKeyListItem { client_name: "Production".to_string(), key_prefix: "epsx_live_abc".to_string(), status: "active".to_string(), requests: 12345 }
                    ApiKeyListItem { client_name: "Staging".to_string(), key_prefix: "epsx_test_xyz".to_string(), status: "active".to_string(), requests: 2345 }
                    ApiKeyListItem { client_name: "Dev".to_string(), key_prefix: "epsx_dev_qqq".to_string(), status: "active".to_string(), requests: 123 }
                    ApiKeyListItem { client_name: "Old staging".to_string(), key_prefix: "epsx_test_old".to_string(), status: "revoked".to_string(), requests: 0 }
                }
            }
            div { class: "card card-glass overflow-hidden",
                div { class: "h-[3px] bg-gradient-to-r from-[#7645d9] to-[#ed4b9e]" }
                div { class: "p-6 border-b border-border/20",
                    h3 { class: "text-lg font-medium text-foreground", "Available modules" }
                    p { class: "text-sm text-muted-foreground", "Choose from these modules when creating API keys" }
                }
                div { class: "grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6 p-6",
                    ModuleCard { display_name: "Portfolio".to_string(), name: "portfolio".to_string(), category: "Analytics".to_string(), status: "active".to_string(), description: "Read wallet portfolios and balances".to_string() }
                    ModuleCard { display_name: "Payments".to_string(), name: "payments".to_string(), category: "Pay".to_string(), status: "active".to_string(), description: "Initiate and verify payments".to_string() }
                    ModuleCard { display_name: "Notifications".to_string(), name: "notifications".to_string(), category: "Communication".to_string(), status: "active".to_string(), description: "Send and read notifications".to_string() }
                    ModuleCard { display_name: "Analytics".to_string(), name: "analytics".to_string(), category: "Telemetry".to_string(), status: "active".to_string(), description: "Track custom analytics events".to_string() }
                    ModuleCard { display_name: "Plans".to_string(), name: "plans".to_string(), category: "Subscriptions".to_string(), status: "active".to_string(), description: "Read subscription plans".to_string() }
                    ModuleCard { display_name: "News".to_string(), name: "news".to_string(), category: "Content".to_string(), status: "active".to_string(), description: "Read news articles".to_string() }
                }
            }
        }
    }
}

#[component]
fn ApiKeyListItem(client_name: String, key_prefix: String, status: String, requests: u32) -> Element {
    let status_cls = match status.as_str() {
        "active" => "bg-emerald-500/10 text-emerald-400 border-emerald-500/20",
        "revoked" => "bg-red-500/10 text-red-400 border-red-500/20",
        _ => "bg-amber-500/10 text-amber-400 border-amber-500/20",
    };
    rsx! {
        div { class: "p-6 flex items-center justify-between",
            div { class: "flex items-center gap-3",
                div { class: "p-2 bg-muted/30 rounded-xl",
                    Icon { name: "key".to_string(), size: Some(18) }
                }
                div {
                    h4 { class: "font-medium text-foreground", "{client_name}" }
                    p { class: "text-sm text-muted-foreground", "Key: {key_prefix}\u{2026}" }
                }
            }
            div { class: "flex items-center gap-3",
                span { class: "px-2 py-1 rounded-full text-xs font-medium {status_cls}", "{status}" }
                div { class: "text-sm text-muted-foreground", "{requests} requests" }
            }
        }
    }
}

#[component]
fn ModuleCard(display_name: String, name: String, category: String, status: String, description: String) -> Element {
    rsx! {
        div { class: "border border-border/20 rounded-xl p-4",
            div { class: "flex items-start justify-between mb-3",
                div {
                    h4 { class: "font-medium text-foreground", "{display_name}" }
                    p { class: "text-sm text-muted-foreground", "{name}" }
                }
                span { class: "px-2 py-1 bg-emerald-500/10 text-emerald-400 border border-emerald-500/20 rounded-full text-xs font-medium", "{status}" }
            }
            p { class: "text-sm text-muted-foreground mb-3", "{description}" }
            div { class: "text-xs text-muted-foreground",
                span { class: "font-medium", "Category:" }
                " {category}"
            }
        }
    }
}

// ============================================================================
// Section 2: ApiKeysTab
// ============================================================================

#[component]
pub fn ApiKeysTab() -> Element {
    let columns = vec![
        Column { key: "name".into(), label: "Name".into(), sortable: true, align: crate::primitives::data_table::Align::Left, width: Some("30%".into()), class_name: None },
        Column { key: "key".into(), label: "Key".into(), sortable: false, align: crate::primitives::data_table::Align::Left, width: Some("40%".into()), class_name: None },
        Column { key: "created".into(), label: "Created".into(), sortable: true, align: crate::primitives::data_table::Align::Left, width: Some("15%".into()), class_name: None },
        Column { key: "last_used".into(), label: "Last used".into(), sortable: true, align: crate::primitives::data_table::Align::Right, width: Some("15%".into()), class_name: None },
    ];
    let rows = vec![
        Row { id: "1".into(), cells: vec!["Production".into(), "epsx_live_xxxxxxxxxxxxx".into(), "2024-08-01".into(), "5 min ago".into()] },
        Row { id: "2".into(), cells: vec!["Staging".into(), "epsx_test_xxxxxxxxxxxxx".into(), "2024-08-15".into(), "1 hour ago".into()] },
        Row { id: "3".into(), cells: vec!["Dev".into(), "epsx_dev_xxxxxxxxxxxxxxxx".into(), "2024-09-01".into(), "Just now".into()] },
    ];
    rsx! {
        div { class: "space-y-4 api-keys-tab",
            div { class: "flex items-center justify-between",
                div {
                    h2 { class: "text-lg font-semibold text-foreground", "API key management" }
                    p { class: "text-sm text-muted-foreground", "Create and manage API keys for third-party integrations" }
                }
                a { class: "btn btn-primary", href: "/developer-portal/api-keys/create", Icon { name: "plus".to_string(), size: Some(16) } " Create API key" }
            }
            DataTable {
                columns,
                rows,
                striped: true,
                page_size: 10,
                filter_placeholder: Some("Filter by name...".to_string()),
                initial_sort: Some(("created".to_string(), SortDir::Desc)),
            }
        }
    }
}

// ============================================================================
// Section 3: ApiKeyCreateForm
// ============================================================================

#[component]
pub fn ApiKeyCreateForm() -> Element {
    let mut name = use_signal(String::new);
    let mut description = use_signal(String::new);
    let mut email = use_signal(String::new);
    let mut ip_restrictions = use_signal(String::new);
    let mut created_key = use_signal(|| None::<String>);

    rsx! {
        div { class: "api-key-create-form max-w-4xl",
            div { class: "flex items-center gap-3 mb-6",
                div { class: "p-2 bg-blue-500/10 rounded-lg",
                    Icon { name: "shield".to_string(), size: Some(24), class_name: Some("text-blue-400".to_string()) }
                }
                div {
                    h1 { class: "text-2xl font-bold", "Create API key" }
                    p { class: "text-muted-foreground", "Generate a new API key for third-party integration" }
                }
            }
            if let Some(k) = created_key.read().clone() {
                div { class: "alert alert-success",
                    p { class: "font-semibold", "API key created" }
                    p { class: "text-sm mt-2", "Save this key now. You will not be able to see it again." }
                    div { class: "mt-3 flex gap-2",
                        code { class: "flex-1 p-3 bg-background rounded font-mono text-sm", "{k}" }
                        CopyButton { text: k.clone(), label: "Copy".to_string() }
                    }
                }
            } else {
                Form { method: "POST".to_string(), action: "/api/v1/developer/api-keys".to_string(),
                    div { class: "grid grid-cols-1 md:grid-cols-2 gap-6",
                        div { class: "field",
                            label { class: "field-label", "Client name" }
                            input { class: "input", name: "client_name", required: true, placeholder: "My Application", value: "{name.read()}", oninput: move |e| name.set(e.value().to_string()) }
                        }
                        div { class: "field",
                            label { class: "field-label", "Contact email" }
                            input { class: "input", name: "client_contact_email", r#type: "email", placeholder: "contact@example.com", value: "{email.read()}", oninput: move |e| email.set(e.value().to_string()) }
                        }
                    }
                    div { class: "field mt-4",
                        label { class: "field-label", "Description" }
                        textarea { class: "input", name: "client_description", placeholder: "Brief description of your application and use case", rows: "3", value: "{description.read()}", oninput: move |e| description.set(e.value().to_string()) }
                    }
                    div { class: "grid grid-cols-1 md:grid-cols-2 gap-6 mt-4",
                        div { class: "field",
                            label { class: "field-label", "Expiration" }
                            select { class: "input", name: "expires_at", required: true,
                                option { value: "never", "Never" }
                                option { value: "30d", "30 days" }
                                option { value: "90d", selected: true, "90 days" }
                                option { value: "1y", "1 year" }
                            }
                        }
                        div { class: "field",
                            label { class: "field-label", "IP restrictions" }
                            textarea { class: "input", name: "ip_restrictions", placeholder: "192.168.1.0/24\n203.0.113.0/24", rows: "3", value: "{ip_restrictions.read()}", oninput: move |e| ip_restrictions.set(e.value().to_string()) }
                        }
                    }
                    div { class: "field mt-4",
                        label { class: "field-label", "Module permissions" }
                        div { class: "space-y-1",
                            CheckboxField { name: "perm_portfolio".to_string(), label: "Portfolio — read wallet portfolios and balances".to_string(), checked: true }
                            CheckboxField { name: "perm_payments".to_string(), label: "Payments — initiate and verify payments".to_string() }
                            CheckboxField { name: "perm_notifications".to_string(), label: "Notifications — send and read notifications".to_string() }
                            CheckboxField { name: "perm_analytics".to_string(), label: "Analytics — track custom events".to_string() }
                            CheckboxField { name: "perm_admin".to_string(), label: "Admin — full admin access".to_string() }
                        }
                    }
                    div { class: "mt-6 p-4 rounded-lg bg-amber-500/10 border border-amber-500/20",
                        div { class: "flex items-start gap-3",
                            span { class: "text-amber-400 mt-0.5", "\u{26a0}\u{fe0f}" }
                            div {
                                h3 { class: "font-medium text-amber-200 mb-1", "Module configuration required" }
                                p { class: "text-sm text-amber-300/80", "After creating the API key, you'll need to configure module permissions and access levels in the main developer portal." }
                            }
                        }
                    }
                    FormActions {
                        a { class: "btn btn-outline", href: "/developer-portal", "Cancel" }
                        button { class: "btn btn-primary", r#type: "submit", Icon { name: "plus".to_string(), size: Some(16) } " Create API key" }
                    }
                }
            }
        }
    }
}

// ============================================================================
// Section 4: ApiKeyRevokeModal
// ============================================================================

#[component]
pub fn ApiKeyRevokeModal(client_name: String) -> Element {
    rsx! {
        div { class: "alert-dialog api-key-revoke-modal",
            div { class: "alert-dialog-content",
                div { class: "alert-dialog-header",
                    h2 { class: "alert-dialog-title", "Revoke API key?" }
                    p { class: "alert-dialog-description",
                        "You are about to revoke the API key "
                        strong { "{client_name}" }
                        ". This action cannot be undone — all clients using this key will lose access immediately."
                    }
                }
                Form { method: "POST".to_string(), action: "/api/v1/developer/api-keys/revoke".to_string(),
                    input { r#type: "hidden", name: "client_name", value: "{client_name}" }
                    div { class: "field",
                        label { class: "field-label", "Reason (optional)" }
                        input { class: "input", name: "reason", placeholder: "e.g. key leaked" }
                    }
                    FormActions {
                        a { class: "btn btn-outline", href: "/developer-portal", "Cancel" }
                        button { class: "btn btn-danger", r#type: "submit", "Revoke key" }
                    }
                }
            }
        }
    }
}

// ============================================================================
// Section 5: UsageAnalyticsTab
// ============================================================================

#[component]
pub fn UsageAnalyticsTab() -> Element {
    rsx! {
        div { class: "space-y-4 usage-analytics-tab",
            div {
                h2 { class: "text-lg font-semibold text-foreground", "Usage analytics" }
                p { class: "text-sm text-muted-foreground", "API calls and quota usage over the last 7 days" }
            }
            div { class: "card card-glass",
                div { class: "card-header",
                    h3 { class: "card-title", "API calls (7d)" }
                    Badge { kind: BadgeKind::Success, "Live" }
                }
                div { class: "card-body",
                    crate::charts::ChartLine {
                        series: vec![
                            crate::charts::Series {
                                name: "Calls".to_string(),
                                color: "#22d3ee".to_string(),
                                points: (0..7).map(|i| crate::charts::DataPoint { x: i as f64, y: 800.0 + (i as f64 * 50.0) + (i as f64 * 23.0).sin() * 100.0, label: None }).collect(),
                            }
                        ],
                        width: 720, height: 220,
                    }
                }
            }
            div { class: "grid grid-cols-1 md:grid-cols-3 gap-4",
                UsageKeyCard { name: "Production".to_string(), calls: 12345, quota_pct: 12.0, color: "text-emerald-400".to_string() }
                UsageKeyCard { name: "Staging".to_string(), calls: 2345, quota_pct: 2.0, color: "text-blue-400".to_string() }
                UsageKeyCard { name: "Dev".to_string(), calls: 123, quota_pct: 0.1, color: "text-purple-400".to_string() }
            }
        }
    }
}

#[component]
fn UsageKeyCard(name: String, calls: u32, quota_pct: f32, color: String) -> Element {
    rsx! {
        div { class: "card card-glass",
            div { class: "card-body",
                p { class: "text-sm text-muted-foreground", "{name}" }
                p { class: "text-2xl font-bold mt-1 {color}", "{calls}" }
                p { class: "text-xs text-muted-foreground mt-1", "{quota_pct:.1}% of quota" }
            }
        }
    }
}

// ============================================================================
// Section 6: DocumentationTab
// ============================================================================

#[component]
pub fn DocumentationTab() -> Element {
    rsx! {
        div { class: "space-y-4 documentation-tab",
            div {
                h2 { class: "text-lg font-semibold text-foreground", "Documentation" }
                p { class: "text-sm text-muted-foreground", "API reference and integration guides" }
            }
            div { class: "card card-glass",
                div { class: "card-body",
                    h3 { class: "text-lg font-semibold mb-2", "Authentication" }
                    p { "All API calls require a Bearer token. Get one via the SIWE flow." }
                    pre { class: "code-block mt-2", "Authorization: Bearer epsx_live_xxx" }
                }
            }
            div { class: "card card-glass",
                div { class: "card-body",
                    h3 { class: "text-lg font-semibold mb-2", "Endpoints" }
                    ul { class: "list-disc list-inside space-y-1",
                        li { code { "POST /v1/auth/challenge" } " — get a SIWE challenge" }
                        li { code { "POST /v1/auth/siwe" } " — verify signature" }
                        li { code { "GET  /v1/portfolio/[address]" } " — get wallet portfolio" }
                        li { code { "GET  /v1/notifications" } " — list notifications" }
                        li { code { "POST /v1/analytics/track" } " " "— track an event" }
                        li { code { "POST /v1/payments/initiate" } " " "— initiate a payment" }
                        li { code { "GET  /v1/plans" } " — list subscription plans" }
                    }
                }
            }
            div { class: "card card-glass",
                div { class: "card-body",
                    h3 { class: "text-lg font-semibold mb-2", "Modules" }
                    p { "Each API key can be granted access to one or more modules. Module permissions are configured when the key is created." }
                    div { class: "mt-3 grid grid-cols-1 md:grid-cols-2 gap-3",
                        DocModuleItem { name: "portfolio".to_string(), description: "Read wallet portfolios and balances".to_string() }
                        DocModuleItem { name: "payments".to_string(), description: "Initiate and verify payments".to_string() }
                        DocModuleItem { name: "notifications".to_string(), description: "Send and read notifications".to_string() }
                        DocModuleItem { name: "analytics".to_string(), description: "Track custom analytics events".to_string() }
                    }
                }
            }
            div { class: "card card-glass",
                div { class: "card-body",
                    h3 { class: "text-lg font-semibold mb-2", "Errors" }
                    p { "API errors return a JSON envelope with the following shape:" }
                    pre { class: "code-block mt-2", "{{ \"success\": false, \"error\": {{ \"code\": \"unauthorized\", \"message\": \"...\" }} }}" }
                    ul { class: "list-disc list-inside mt-2 space-y-1",
                        li { code { "400" } " — invalid request" }
                        li { code { "401" } " — unauthorized / invalid token" }
                        li { code { "403" } " — forbidden / missing permission" }
                        li { code { "404" } " — resource not found" }
                        li { code { "429" } " — rate limit exceeded" }
                        li { code { "500" } " — internal server error" }
                    }
                }
            }
        }
    }
}

#[component]
fn DocModuleItem(name: String, description: String) -> Element {
    rsx! {
        div { class: "border border-border/20 rounded-lg p-3",
            div { class: "flex items-center gap-2",
                code { class: "text-sm font-bold", "{name}" }
            }
            p { class: "text-sm text-muted-foreground mt-1", "{description}" }
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn render_to_string(el: Element) -> String {
        dioxus_ssr::render_element(el)
    }

    #[test]
    fn test_render_smoke_developer_portal_stats() {
        let el = rsx! {
            DeveloperPortalStats { total_keys: 5, active_keys: 4, total_requests: 12450, available_modules: 12 }
        };
        let html = render_to_string(el);
        assert!(html.contains("Total API keys"), "DeveloperPortalStats must render the 'Total API keys' card. Got: {}", html);
        assert!(html.contains("developer-portal-stats"), "DeveloperPortalStats must render its class. Got: {}", html);
    }

    #[test]
    fn test_render_smoke_developer_portal_overview() {
        let el = rsx! { DeveloperPortalOverview {} };
        let html = render_to_string(el);
        assert!(html.contains("Developer overview"), "DeveloperPortalOverview must render the title. Got: {}", html);
        assert!(html.contains("Recent API keys"), "DeveloperPortalOverview must render the 'Recent API keys' card. Got: {}", html);
        assert!(html.contains("Available modules"), "DeveloperPortalOverview must render the 'Available modules' card. Got: {}", html);
    }

    #[test]
    fn test_render_smoke_api_keys_tab() {
        let el = rsx! { ApiKeysTab {} };
        let html = render_to_string(el);
        assert!(html.contains("API key management"), "ApiKeysTab must render the title. Got: {}", html);
        assert!(html.contains("api-keys-tab"), "ApiKeysTab must render its class. Got: {}", html);
    }

    #[test]
    fn test_render_smoke_api_key_create_form() {
        let el = rsx! { ApiKeyCreateForm {} };
        let html = render_to_string(el);
        assert!(html.contains("Create API key"), "ApiKeyCreateForm must render the title. Got: {}", html);
        assert!(html.contains("Module permissions"), "ApiKeyCreateForm must render the module permissions field. Got: {}", html);
        assert!(html.contains("api-key-create-form"), "ApiKeyCreateForm must render its class. Got: {}", html);
    }

    #[test]
    fn test_render_smoke_api_key_revoke_modal() {
        let el = rsx! { ApiKeyRevokeModal { client_name: "Production".to_string() } };
        let html = render_to_string(el);
        assert!(html.contains("Revoke API key?"), "ApiKeyRevokeModal must render the title. Got: {}", html);
        assert!(html.contains("api-key-revoke-modal"), "ApiKeyRevokeModal must render its class. Got: {}", html);
    }

    #[test]
    fn test_render_smoke_usage_analytics_tab() {
        let el = rsx! { UsageAnalyticsTab {} };
        let html = render_to_string(el);
        assert!(html.contains("Usage analytics"), "UsageAnalyticsTab must render the title. Got: {}", html);
        assert!(html.contains("API calls (7d)"), "UsageAnalyticsTab must render the chart header. Got: {}", html);
    }

    #[test]
    fn test_render_smoke_documentation_tab() {
        let el = rsx! { DocumentationTab {} };
        let html = render_to_string(el);
        assert!(html.contains("Documentation"), "DocumentationTab must render the title. Got: {}", html);
        assert!(html.contains("Authentication"), "DocumentationTab must render the Authentication section. Got: {}", html);
        assert!(html.contains("Endpoints"), "DocumentationTab must render the Endpoints section. Got: {}", html);
    }
}
