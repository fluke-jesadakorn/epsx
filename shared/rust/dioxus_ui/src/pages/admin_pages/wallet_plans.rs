//! /admin/wallet-management/access + /admin/wallet-management/access/plans.

use crate::primitives::*;
use crate::data_table::{Column, DataTable, Row, SortDir};

use dioxus::prelude::*;
use super::super::{PageContext, PageMeta};
use crate::auth::AuthGate;

pub fn render(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::admin("Access control");
    (meta, rsx! {
        AuthGate { user: ctx.user.clone(), feature: Some("access control".to_string()),
            div { class: "container page-content",
                div { class: "mb-6",
                    h1 { class: "text-2xl font-bold", "Access control" }
                    p { class: "text-muted-foreground", "Manage wallet permissions and access plans" }
                }
                div { class: "grid grid-cols-1 md:grid-cols-3 gap-4 mb-6",
                    StatCard { label: "Total wallets".to_string(), value: "1,234".to_string(), icon: Some("users".to_string()) }
                    StatCard { label: "Active plans".to_string(), value: "5".to_string(), icon: Some("zap".to_string()) }
                    StatCard { label: "Permissions".to_string(), value: "12".to_string(), icon: Some("key".to_string()) }
                }
                div { class: "grid grid-cols-1 md:grid-cols-2 gap-4",
                    a { class: "card card-glass p-6", href: "/wallet-management/access/plans",
                        Icon { name: "list".to_string(), size: Some(32) }
                        h3 { class: "text-xl font-bold mt-3", "Plans" }
                        p { class: "text-muted-foreground mt-1", "Subscription plans with permissions, pricing, and billing intervals" }
                    }
                    a { class: "card card-glass p-6", href: "/wallet-management/wallets",
                        Icon { name: "users".to_string(), size: Some(32) }
                        h3 { class: "text-xl font-bold mt-3", "Wallets" }
                        p { class: "text-muted-foreground mt-1", "Wallet-by-wallet permission management" }
                    }
                }
            }
        }
    })
}

pub fn render_plans(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::admin("Plans");
    let columns = vec![
        Column { key: "name".into(), label: "Name".into(), sortable: true, align: crate::primitives::data_table::Align::Left, width: Some("25%".into()), class_name: None },
        Column { key: "price".into(), label: "Price".into(), sortable: true, align: crate::primitives::data_table::Align::Right, width: Some("15%".into()), class_name: None },
        Column { key: "interval".into(), label: "Interval".into(), sortable: true, align: crate::primitives::data_table::Align::Left, width: Some("15%".into()), class_name: None },
        Column { key: "subs".into(), label: "Subscribers".into(), sortable: true, align: crate::primitives::data_table::Align::Right, width: Some("15%".into()), class_name: None },
        Column { key: "status".into(), label: "Status".into(), sortable: true, align: crate::primitives::data_table::Align::Center, width: Some("15%".into()), class_name: None },
        Column { key: "actions".into(), label: "Actions".into(), sortable: false, align: crate::primitives::data_table::Align::Right, width: Some("15%".into()), class_name: None },
    ];
    let rows = vec![
        Row { id: "1".into(), cells: vec!["Free".into(), "$0".into(), "monthly".into(), "—".into(), "Active".into(), "Edit".into()] },
        Row { id: "2".into(), cells: vec!["Pro".into(), "$29".into(), "monthly".into(), "412".into(), "Active".into(), "Edit".into()] },
        Row { id: "3".into(), cells: vec!["Enterprise".into(), "$299".into(), "monthly".into(), "32".into(), "Active".into(), "Edit".into()] },
    ];
    (meta, rsx! {
        AuthGate { user: ctx.user.clone(), feature: Some("plan management".to_string()),
            div { class: "container page-content",
                div { class: "flex items-center justify-between mb-6",
                    div {
                        h1 { class: "text-2xl font-bold", "Plans" }
                        p { class: "text-muted-foreground", "Subscription plans with permissions and pricing" }
                    }
                    a { class: "btn btn-primary", href: "/wallet-management/access/plans/new", Icon { name: "plus".to_string(), size: Some(16) } " New plan" }
                }
                DataTable { columns, rows, striped: true, page_size: 20, filter_placeholder: Some("Filter by name, status...".to_string()), initial_sort: Some(("price".to_string(), SortDir::Asc)) }
            }
        }
    })
}

pub fn render_editor(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::admin("Plan editor");
    let plan_id = ctx.params.get("planId").cloned();
    (meta, rsx! {
        AuthGate { user: ctx.user.clone(), feature: Some("plan editing".to_string()),
            div { class: "container page-content max-w-3xl",
                a { class: "btn btn-sm btn-ghost mb-4", href: "/wallet-management/access/plans", Icon { name: "arrow-left".to_string(), size: Some(16) } " Back" }
                Form { method: "POST".to_string(), action: if let Some(ref p) = plan_id { format!("/api/v1/subscription/plans/{}", p) } else { "/api/v1/subscription/plans".to_string() },
                    div { class: "card card-glass",
                        div { class: "card-header", h1 { class: "card-title", "Plan details" } }
                        div { class: "card-body space-y-4",
                            div { class: "field",
                                label { class: "field-label", "Plan name" }
                                input { class: "input", name: "name", required: true, placeholder: "e.g. Pro" }
                            }
                            div { class: "field",
                                label { class: "field-label", "Description" }
                                textarea { class: "input", name: "description", rows: "2", placeholder: "Plan description" }
                            }
                            div { class: "grid grid-cols-1 md:grid-cols-2 gap-4",
                                div { class: "field",
                                    label { class: "field-label", "Price" }
                                    input { class: "input", name: "amount", r#type: "number", step: "0.01", required: true, value: "29.00" }
                                }
                                div { class: "field",
                                    label { class: "field-label", "Currency" }
                                    SelectField { name: "currency".to_string(), options: vec![("USDT".to_string(), "USDT".to_string()), ("USDC".to_string(), "USDC".to_string())], value: Some("USDT".to_string()), required: true, label: None, help: None, error: None, placeholder: None, onchange: None }
                                }
                            }
                            div { class: "field",
                                label { class: "field-label", "Billing interval" }
                                SelectField { name: "interval_secs".to_string(), options: vec![("86400".to_string(), "Daily".to_string()), ("604800".to_string(), "Weekly".to_string()), ("2592000".to_string(), "Monthly".to_string()), ("31536000".to_string(), "Yearly".to_string())], value: Some("2592000".to_string()), required: true, label: None, help: None, error: None, placeholder: None, onchange: None }
                            }
                            div { class: "field",
                                label { class: "field-label", "Permissions granted" }
                                div { class: "space-y-1",
                                    CheckboxField { name: "perm_trade".to_string(), label: "Trade".to_string(), checked: true }
                                    CheckboxField { name: "perm_view".to_string(), label: "View analytics".to_string(), checked: true }
                                    CheckboxField { name: "perm_pay".to_string(), label: "Send payments".to_string(), checked: true }
                                    CheckboxField { name: "perm_api".to_string(), label: "API access".to_string() }
                                }
                            }
                            FormActions {
                                a { class: "btn btn-outline", href: "/wallet-management/access/plans", "Cancel" }
                                button { class: "btn btn-primary", r#type: "submit", "Save plan" }
                            }
                        }
                    }
                }
            }
        }
    })
}
