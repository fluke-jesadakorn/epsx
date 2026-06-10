//! /admin/audit-log — audit log table with filters.

use crate::primitives::*;
use crate::data_table::{Column, DataTable, Row, SortDir};

use dioxus::prelude::*;
use super::super::{PageContext, PageMeta};
use crate::layout::DashboardShell;
use crate::auth::AuthGate;

pub fn render(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::admin("Audit log");
    let columns = vec![
        Column { key: "time".into(), label: "Time".into(), sortable: true, align: crate::primitives::data_table::Align::Left, width: Some("20%".into()), class_name: None },
        Column { key: "actor".into(), label: "Actor".into(), sortable: true, align: crate::primitives::data_table::Align::Left, width: Some("25%".into()), class_name: None },
        Column { key: "action".into(), label: "Action".into(), sortable: true, align: crate::primitives::data_table::Align::Left, width: Some("20%".into()), class_name: None },
        Column { key: "resource".into(), label: "Resource".into(), sortable: false, align: crate::primitives::data_table::Align::Left, width: Some("20%".into()), class_name: None },
        Column { key: "ip".into(), label: "IP".into(), sortable: false, align: crate::primitives::data_table::Align::Right, width: Some("15%".into()), class_name: None },
    ];
    let rows = vec![
        Row { id: "1".into(), cells: vec!["2024-09-20 10:32:15".into(), "admin@epsx.io".into(), "user.create".into(), "user/0xabc".into(), "192.168.1.1".into()] },
        Row { id: "2".into(), cells: vec!["2024-09-20 10:30:01".into(), "admin@epsx.io".into(), "plan.update".into(), "plan/pro".into(), "192.168.1.1".into()] },
        Row { id: "3".into(), cells: vec!["2024-09-20 10:25:42".into(), "0x1234…5678".into(), "wallet.connect".into(), "wallet/0x1234".into(), "10.0.0.1".into()] },
        Row { id: "4".into(), cells: vec!["2024-09-20 10:20:00".into(), "admin@epsx.io".into(), "news.publish".into(), "news/welcome".into(), "192.168.1.1".into()] },
    ];
    (meta, rsx! {
        AuthGate { user: ctx.user.clone(), feature: Some("the audit log".to_string()),
            DashboardShell { current_path: ctx.path.clone(), page_title: "Audit log".to_string(), user: ctx.user.clone(),
                div { class: "container page-content",
                    div { class: "flex items-center justify-between mb-6",
                        div {
                            h1 { class: "text-2xl font-bold", "Audit log" }
                            p { class: "text-muted-foreground", "All platform actions by admin users and authenticated wallets" }
                        }
                        div { class: "flex gap-2",
                            button { class: "btn btn-sm btn-outline", r#type: "button", "Export CSV" }
                            button { class: "btn btn-sm btn-outline", r#type: "button", "Export JSON" }
                        }
                    }
                    DataTable {
                        columns: columns.clone(),
                        rows: rows.clone(),
                        striped: true,
                        page_size: 25,
                        filter_placeholder: Some("Filter by actor, action, resource...".to_string()),
                        initial_sort: Some(("time".to_string(), SortDir::Desc)),
                    }
                }
            }
        }
    })
}
