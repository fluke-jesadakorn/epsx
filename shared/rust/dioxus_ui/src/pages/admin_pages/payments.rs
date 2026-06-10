//! /admin/payments — payment management.

use crate::primitives::*;
use crate::data_table::{Column, DataTable, Row, SortDir};

use dioxus::prelude::*;
use super::super::{PageContext, PageMeta};
use crate::layout::DashboardShell;
use crate::auth::AuthGate;

pub fn render(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::admin("Payments");
    let columns = vec![
        Column { key: "id".into(), label: "Intent ID".into(), sortable: true, align: crate::primitives::data_table::Align::Left, width: Some("20%".into()), class_name: None },
        Column { key: "amount".into(), label: "Amount".into(), sortable: true, align: crate::primitives::data_table::Align::Right, width: Some("15%".into()), class_name: None },
        Column { key: "token".into(), label: "Token".into(), sortable: false, align: crate::primitives::data_table::Align::Left, width: Some("10%".into()), class_name: None },
        Column { key: "status".into(), label: "Status".into(), sortable: true, align: crate::primitives::data_table::Align::Center, width: Some("15%".into()), class_name: None },
        Column { key: "created".into(), label: "Created".into(), sortable: true, align: crate::primitives::data_table::Align::Right, width: Some("20%".into()), class_name: None },
        Column { key: "actions".into(), label: "Actions".into(), sortable: false, align: crate::primitives::data_table::Align::Right, width: Some("20%".into()), class_name: None },
    ];
    let rows = vec![
        Row { id: "pi_1".into(), cells: vec!["pi_abc123".into(), "$29.00".into(), "USDT".into(), "Confirmed".into(), "2024-09-20 10:32".into(), "View".into()] },
        Row { id: "pi_2".into(), cells: vec!["pi_def456".into(), "$29.00".into(), "USDT".into(), "Pending".into(), "2024-09-20 11:14".into(), "Confirm / Cancel".into()] },
        Row { id: "pi_3".into(), cells: vec!["pi_ghi789".into(), "$99.00".into(), "USDC".into(), "Failed".into(), "2024-09-19 09:21".into(), "View".into()] },
    ];
    (meta, rsx! {
        AuthGate { user: ctx.user.clone(), feature: Some("payment management".to_string()),
            DashboardShell { current_path: ctx.path.clone(), page_title: "Payments".to_string(), user: ctx.user.clone(),
                div { class: "container page-content",
                    div { class: "flex items-center justify-between mb-6",
                        div {
                            h1 { class: "text-2xl font-bold", "Payments" }
                            p { class: "text-muted-foreground", "All payment intents and their status" }
                        }
                        div { class: "flex gap-2",
                            button { class: "btn btn-sm btn-outline", r#type: "button", "Export" }
                            button { class: "btn btn-sm btn-outline", r#type: "button", "Refresh" }
                        }
                    }
                    div { class: "grid grid-cols-1 md:grid-cols-4 gap-4 mb-6",
                        StatCard { label: "Confirmed".to_string(), value: "1,234".to_string(), icon: Some("check".to_string()) }
                        StatCard { label: "Pending".to_string(), value: "12".to_string(), icon: Some("clock".to_string()) }
                        StatCard { label: "Failed".to_string(), value: "3".to_string(), icon: Some("x".to_string()) }
                        StatCard { label: "Total volume".to_string(), value: "$45,231".to_string(), icon: Some("trending-up".to_string()) }
                    }
                    DataTable { columns, rows, striped: true, page_size: 25, filter_placeholder: Some("Filter by ID, status, token...".to_string()), initial_sort: Some(("created".to_string(), SortDir::Desc)) }
                }
            }
        }
    })
}
