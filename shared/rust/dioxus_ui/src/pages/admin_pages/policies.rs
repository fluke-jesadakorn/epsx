//! /admin/policies — NEW page (was missing in the original port).

use crate::primitives::*;
use crate::data_table::{Column, DataTable, Row, SortDir};

use dioxus::prelude::*;
use super::super::{PageContext, PageMeta};
use crate::layout::DashboardShell;
use crate::auth::AuthGate;

pub fn render(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::admin("Policies");
    (meta, rsx! { RenderPolicies { ctx: ctx.clone() } })
}

#[component]
fn RenderPolicies(ctx: PageContext) -> Element {
    let mut tab = use_signal(|| "list".to_string());
    rsx! {
        AuthGate { user: ctx.user.clone(), feature: Some("policy management".to_string()),
            DashboardShell { current_path: ctx.path.clone(), page_title: "Policies".to_string(), user: ctx.user.clone(),
                div { class: "container page-content",
                    div { class: "flex items-center justify-between mb-6",
                        div { h1 { class: "text-2xl font-bold", "Policies" } p { class: "text-muted-foreground", "Access control policies: rules, conditions, and decisions" } }
                    button { class: "btn btn-primary", r#type: "button", Icon { name: "plus".to_string(), size: Some(16) } " New policy" }
                    }
                    div { class: "tabs mb-4",
                        button { class: if *tab.read() == "list" { "btn btn-primary" } else { "btn btn-outline" }, onclick: move |_| tab.set("list".to_string()), "List" }
                        button { class: if *tab.read() == "monitor" { "btn btn-primary" } else { "btn btn-outline" }, onclick: move |_| tab.set("monitor".to_string()), "Monitor" }
                        button { class: if *tab.read() == "stats" { "btn btn-primary" } else { "btn btn-outline" }, onclick: move |_| tab.set("stats".to_string()), "Stats" }
                    }
                    if *tab.read() == "list" { ListView {} } else if *tab.read() == "monitor" { MonitorView {} } else { StatsView {} }
                }
            }
        }
    }
}

#[component]
fn ListView() -> Element {
    let columns = vec![
        Column { key: "name".into(), label: "Name".into(), sortable: true, align: crate::primitives::data_table::Align::Left, width: Some("30%".into()), class_name: None },
        Column { key: "type".into(), label: "Type".into(), sortable: true, align: crate::primitives::data_table::Align::Left, width: Some("15%".into()), class_name: None },
        Column { key: "effect".into(), label: "Effect".into(), sortable: true, align: crate::primitives::data_table::Align::Center, width: Some("15%".into()), class_name: None },
        Column { key: "status".into(), label: "Status".into(), sortable: true, align: crate::primitives::data_table::Align::Center, width: Some("15%".into()), class_name: None },
        Column { key: "updated".into(), label: "Updated".into(), sortable: true, align: crate::primitives::data_table::Align::Right, width: Some("25%".into()), class_name: None },
    ];
    let rows = vec![
        Row { id: "p1".into(), cells: vec!["Admin full access".into(), "RBAC".into(), "Allow".into(), "Active".into(), "2024-09-15".into()] },
        Row { id: "p2".into(), cells: vec!["Pro plan trade".into(), "Subscription".into(), "Allow".into(), "Active".into(), "2024-09-10".into()] },
        Row { id: "p3".into(), cells: vec!["Rate limit 100/min".into(), "Rate".into(), "Limit".into(), "Active".into(), "2024-09-05".into()] },
        Row { id: "p4".into(), cells: vec!["Block sanctioned addresses".into(), "Compliance".into(), "Deny".into(), "Active".into(), "2024-08-20".into()] },
    ];
    rsx! { DataTable { columns, rows, striped: true, page_size: 20, filter_placeholder: Some("Filter by name, type, effect...".to_string()), initial_sort: Some(("updated".to_string(), SortDir::Desc)) } }
}

#[component]
fn MonitorView() -> Element {
    rsx! {
        div { class: "grid grid-cols-1 md:grid-cols-3 gap-4 mb-6",
            StatCard { label: "Evaluations (24h)".to_string(), value: "12,345".to_string(), icon: Some("activity".to_string()) }
            StatCard { label: "Avg eval time".to_string(), value: "0.4ms".to_string(), icon: Some("clock".to_string()) }
            StatCard { label: "Allow / Deny".to_string(), value: "10,234 / 1,234".to_string(), icon: Some("check-x".to_string()) }
        }
        div { class: "card card-glass",
            div { class: "card-header", h3 { class: "card-title", "Recent evaluations" } }
            div { class: "card-body p-0",
                div { class: "table-wrap",
                    table { class: "table",
                        thead { tr { th { "Time" } th { "Policy" } th { "Subject" } th { "Decision" } } }
                        tbody {
                            tr { td { "10:32:15" } td { "Admin full access" } td { code { class: "text-xs", "0x1234" } } td { span { class: "badge badge-success", "Allow" } } }
                            tr { td { "10:32:14" } td { "Pro plan trade" } td { code { class: "text-xs", "0xabcd" } } td { span { class: "badge badge-success", "Allow" } } }
                            tr { td { "10:32:12" } td { "Block sanctioned" } td { code { class: "text-xs", "0xdead" } } td { span { class: "badge badge-danger", "Deny" } } }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn StatsView() -> Element {
    rsx! {
        div { class: "card card-glass",
            div { class: "card-header", h3 { class: "card-title", "Decision breakdown" } }
            div { class: "card-body",
                crate::charts::ChartDonut {
                    data: vec![
                        ("Allow".to_string(), 10234.0, "#22c55e".to_string()),
                        ("Deny".to_string(), 1234.0, "#ef4444".to_string()),
                        ("Limit".to_string(), 877.0, "#f59e0b".to_string()),
                    ],
                    size: 200, thickness: 32,
                }
            }
        }
    }
}
