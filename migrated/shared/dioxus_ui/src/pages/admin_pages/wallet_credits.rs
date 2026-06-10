//! /admin/wallet-management/credits — credits management.

use crate::primitives::*;
use crate::feedback::*;
use crate::data_table::{Column, DataTable, Row, SortDir};

use dioxus::prelude::*;
use super::super::{PageContext, PageMeta};
use crate::layout::DashboardShell;
use crate::auth::AuthGate;

pub fn render(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::admin("Credits");
    let columns = vec![
        Column { key: "wallet".into(), label: "Wallet".into(), sortable: true, align: crate::primitives::data_table::Align::Left, width: Some("35%".into()), class_name: None },
        Column { key: "available".into(), label: "Available".into(), sortable: true, align: crate::primitives::data_table::Align::Right, width: Some("20%".into()), class_name: None },
        Column { key: "used".into(), label: "Used".into(), sortable: true, align: crate::primitives::data_table::Align::Right, width: Some("20%".into()), class_name: None },
        Column { key: "earned".into(), label: "Earned".into(), sortable: true, align: crate::primitives::data_table::Align::Right, width: Some("25%".into()), class_name: None },
    ];
    let rows = vec![
        Row { id: "0x1234…5678".into(), cells: vec!["0x1234…5678".into(), "1,250".into(), "340".into(), "1,590".into()] },
        Row { id: "0xabcd…ef12".into(), cells: vec!["0xabcd…ef12".into(), "850".into(), "120".into(), "970".into()] },
        Row { id: "0x9876…5432".into(), cells: vec!["0x9876…5432".into(), "5,200".into(), "1,400".into(), "6,600".into()] },
    ];
    (meta, rsx! {
        AuthGate { user: ctx.user.clone(), feature: Some("credits management".to_string()),
            DashboardShell { current_path: ctx.path.clone(), page_title: "Credits".to_string(), user: ctx.user.clone(),
                div { class: "container page-content",
                    div { class: "flex items-center justify-between mb-6",
                        div {
                            h1 { class: "text-2xl font-bold", "Credits" }
                            p { class: "text-muted-foreground", "Manage platform credits per wallet" }
                        }
                        button { class: "btn btn-primary", r#type: "button", Icon { name: "plus".to_string(), size: Some(16) } " Award credits" }
                    }
                    div { class: "grid grid-cols-1 md:grid-cols-3 gap-4 mb-6",
                        StatCard { label: "Total in circulation".to_string(), value: "45,200".to_string(), icon: Some("coins".to_string()) }
                        StatCard { label: "Awarded this month".to_string(), value: "8,400".to_string(), icon: Some("trending-up".to_string()) }
                        StatCard { label: "Spent this month".to_string(), value: "3,250".to_string(), icon: Some("trending-down".to_string()) }
                    }
                    DataTable {
                        columns, rows,
                        striped: true,
                        page_size: 20,
                        filter_placeholder: Some("Filter by wallet...".to_string()),
                        initial_sort: Some(("available".to_string(), SortDir::Desc)),
                    }
                }
            }
        }
    })
}
