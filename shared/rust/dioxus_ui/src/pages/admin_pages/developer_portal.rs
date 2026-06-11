//! /admin/developer-portal — API key management, usage, docs.

use crate::primitives::*;
use crate::feedback::*;
use crate::data_table::{Column, DataTable, Row, SortDir};

use dioxus::prelude::*;
use super::super::{PageContext, PageMeta};
use crate::auth::AdminAuthGate;

const QUICK_START_CURL: &str = "curl -X POST https://api.epsx.io/v1/auth/siwe \\\n  -H 'Content-Type: application/json' \\\n  -d '{ \"message\": \"...\", \"signature\": \"0x...\" }'";

pub fn render(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::admin("Developer portal");
    (meta, rsx! { RenderDevPortal { ctx: ctx.clone() } })
}

#[component]
fn RenderDevPortal(ctx: PageContext) -> Element {
    let mut tab = use_signal(|| "overview".to_string());
    rsx! {
        AdminAuthGate { user: ctx.user.clone(), feature: Some("the developer portal".to_string()), required_permissions: Some(vec!["developer:manage".to_string()]), return_url: Some(ctx.path.clone()),
            div { class: "container page-content",
                div { class: "flex items-center justify-between mb-4", h1 { class: "text-2xl font-bold", "Developer portal" } a { class: "btn btn-primary", href: "/developer-portal/api-keys/create", Icon { name: "plus".to_string(), size: Some(16) } " Create API key" } }
                div { class: "tabs mb-4",
                    button { class: if *tab.read() == "overview" { "btn btn-primary" } else { "btn btn-outline" }, onclick: move |_| tab.set("overview".to_string()), "Overview" }
                    button { class: if *tab.read() == "keys" { "btn btn-primary" } else { "btn btn-outline" }, onclick: move |_| tab.set("keys".to_string()), "API keys" }
                    button { class: if *tab.read() == "usage" { "btn btn-primary" } else { "btn btn-outline" }, onclick: move |_| tab.set("usage".to_string()), "Usage" }
                    button { class: if *tab.read() == "docs" { "btn btn-primary" } else { "btn btn-outline" }, onclick: move |_| tab.set("docs".to_string()), "Documentation" }
                }
                if *tab.read() == "keys" { KeysView {} } else if *tab.read() == "usage" { UsageView {} } else if *tab.read() == "docs" { DocsView {} } else { OverviewView {} }
            }
        }
    }
}

#[component]
fn OverviewView() -> Element {
    rsx! {
        div { class: "grid grid-cols-1 md:grid-cols-3 gap-4",
            StatCard { label: "Active keys".to_string(), value: "3".to_string(), icon: Some("key".to_string()) }
            StatCard { label: "Calls today".to_string(), value: "1,234".to_string(), icon: Some("zap".to_string()) }
            StatCard { label: "Quota used".to_string(), value: "12%".to_string(), icon: Some("chart-line".to_string()) }
        }
        div { class: "card card-glass mt-4",
            div { class: "card-header", h3 { class: "card-title", "Quick start" } }
            div { class: "card-body",
                pre { class: "code-block", "{QUICK_START_CURL}" }
            }
        }
    }
}

#[component]
fn KeysView() -> Element {
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

#[component]
fn UsageView() -> Element {
    rsx! {
        div { class: "card card-glass",
            div { class: "card-header", h3 { class: "card-title", "API calls (7d)" } }
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
    }
}

#[component]
fn DocsView() -> Element {
    rsx! {
        div { class: "card card-glass",
            div { class: "card-body",
                h2 { "Authentication" }
                p { "All API calls require a Bearer token. Get one via the SIWE flow." }
                pre { class: "code-block mt-2", "Authorization: Bearer epsx_live_xxx" }
                h2 { class: "mt-4", "Endpoints" }
                ul { class: "list-disc list-inside space-y-1",
                    li { code { "POST /v1/auth/challenge" } " — get a SIWE challenge" }
                    li { code { "POST /v1/auth/siwe" } " — verify signature" }
                    li { code { "GET  /v1/portfolio/[address]" } " — get wallet portfolio" }
                    li { code { "GET  /v1/notifications" } " — list notifications" }
                    li { code { "POST /v1/analytics/track" } " " "— track an event" }
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
    let mut name = use_signal(String::new);
    let mut created_key = use_signal(|| None::<String>);
    rsx! {
        AdminAuthGate { user: ctx.user.clone(), feature: Some("creating API keys".to_string()), required_permissions: Some(vec!["developer:manage".to_string()]), return_url: Some(ctx.path.clone()),
            div { class: "container page-content max-w-2xl",
                a { class: "btn btn-sm btn-ghost mb-4", href: "/developer-portal", Icon { name: "arrow-left".to_string(), size: Some(16) } " Back" }
                div { class: "card card-glass",
                    div { class: "card-header", h1 { class: "card-title", "Create API key" } }
                    div { class: "card-body",
                        if let Some(k) = created_key.read().clone() {
                            div { class: "alert alert-success",
                                p { class: "font-semibold", "API key created" }
                                p { class: "text-sm mt-2", "Save this key now. You will not be able to see it again." }
                                div { class: "mt-3 flex gap-2", code { class: "flex-1 p-3 bg-background rounded font-mono text-sm", "{k}" } CopyButton { text: k.clone(), label: "Copy".to_string() } }
                            }
                        } else {
                            Form { method: "POST".to_string(), action: "/api/v1/developer/api-keys".to_string(),
                                div { class: "field", label { class: "field-label", "Key name" } input { class: "input", name: "name", required: true, placeholder: "e.g. Production, Staging, Dev", value: "{name.read()}", oninput: move |e| name.set(e.value().to_string()) } }
                                div { class: "field", label { class: "field-label", "Permissions" } div { class: "space-y-1",
                                    CheckboxField { name: "perm_read".to_string(), label: "Read — read-only access to data".to_string(), checked: true }
                                    CheckboxField { name: "perm_trade".to_string(), label: "Trade — execute trades".to_string() }
                                    CheckboxField { name: "perm_admin".to_string(), label: "Admin — full access".to_string() }
                                } }
                                div { class: "field", label { class: "field-label", "Expires" }
                                    SelectField { name: "expires".to_string(), options: vec![("never".to_string(), "Never".to_string()), ("30d".to_string(), "30 days".to_string()), ("90d".to_string(), "90 days".to_string()), ("1y".to_string(), "1 year".to_string())], value: Some("90d".to_string()), required: true, help: None, error: None, label: None, placeholder: None, onchange: None }
                                }
                                FormActions { a { class: "btn btn-outline", href: "/developer-portal", "Cancel" } button { class: "btn btn-primary", r#type: "submit", "Create key" } }
                            }
                        }
                    }
                }
            }
        }
    }
}
