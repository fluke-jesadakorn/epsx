//! /admin/wallet-management/wallets — wallet list (DataTable).
//! /admin/wallet-management/wallets/[address] — wallet detail.
//! /admin/wallet-management/wallets/[address]/disable — disable flow.

use crate::primitives::*;
use crate::data_table::{Column, DataTable, Row, SortDir};

use dioxus::prelude::*;
use super::super::{PageContext, PageMeta};
use crate::layout::DashboardShell;
use crate::auth::AuthGate;

pub fn render(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::admin("Wallet management");
    let columns = vec![
        Column { key: "address".into(), label: "Address".into(), sortable: true, align: crate::primitives::data_table::Align::Left, width: Some("30%".into()), class_name: None },
        Column { key: "chain".into(), label: "Chain".into(), sortable: true, align: crate::primitives::data_table::Align::Left, width: Some("10%".into()), class_name: None },
        Column { key: "balance".into(), label: "Balance".into(), sortable: true, align: crate::primitives::data_table::Align::Right, width: Some("15%".into()), class_name: None },
        Column { key: "status".into(), label: "Status".into(), sortable: true, align: crate::primitives::data_table::Align::Center, width: Some("10%".into()), class_name: None },
        Column { key: "permissions".into(), label: "Permissions".into(), sortable: false, align: crate::primitives::data_table::Align::Left, width: Some("20%".into()), class_name: None },
        Column { key: "last_active".into(), label: "Last active".into(), sortable: true, align: crate::primitives::data_table::Align::Right, width: Some("15%".into()), class_name: None },
    ];
    let rows = vec![
        Row { id: "0x1234567890abcdef1234567890abcdef12345678".into(), cells: vec!["0x1234…5678".into(), "BSC".into(), "1.234 BNB".into(), "Active".into(), "trade, view, pay".into(), "2 min ago".into()] },
        Row { id: "0xabcdef1234567890abcdef1234567890abcdef12".into(), cells: vec!["0xabcd…ef12".into(), "BSC".into(), "0.5 BNB".into(), "Active".into(), "trade".into(), "1 hour ago".into()] },
        Row { id: "0xdeadbeefdeadbeefdeadbeefdeadbeefdeadbeef".into(), cells: vec!["0xdead…beef".into(), "BSC".into(), "0.0 BNB".into(), "Disabled".into(), "—".into(), "1 day ago".into()] },
    ];
    (meta, rsx! {
        AuthGate { user: ctx.user.clone(), feature: Some("the wallet management page".to_string()),
            DashboardShell { current_path: ctx.path.clone(), page_title: "Wallets".to_string(), user: ctx.user.clone(),
                div { class: "container page-content",
                    div { class: "flex items-center justify-between mb-6",
                        div {
                            h1 { class: "text-2xl font-bold", "Wallets" }
                            p { class: "text-muted-foreground", "All wallets connected to the platform" }
                        }
                        a { class: "btn btn-primary", href: "/wallet-management/credits", Icon { name: "plus".to_string(), size: Some(16) } " Add wallet" }
                    }
                    DataTable {
                        columns, rows,
                        striped: true,
                        page_size: 20,
                        filter_placeholder: Some("Filter by address, status, or permission...".to_string()),
                        initial_sort: Some(("last_active".to_string(), SortDir::Desc)),
                    }
                }
            }
        }
    })
}

pub fn render_detail(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::admin("Wallet detail");
    (meta, rsx! { RenderWalletDetail { ctx: ctx.clone() } })
}

#[component]
fn RenderWalletDetail(ctx: PageContext) -> Element {
    let address = ctx.params.get("address").cloned().unwrap_or_default();
    let mut tab = use_signal(|| "overview".to_string());
    rsx! {
        AuthGate { user: ctx.user.clone(), feature: Some("wallet detail".to_string()),
            DashboardShell { current_path: ctx.path.clone(), page_title: "Wallet".to_string(), user: ctx.user.clone(),
                div { class: "container page-content",
                    a { class: "btn btn-sm btn-ghost mb-4", href: "/wallet-management/wallets", Icon { name: "arrow-left".to_string(), size: Some(16) } " Back to wallets" }
                    div { class: "card card-glass",
                        div { class: "card-header", h1 { class: "card-title", "Wallet" } code { class: "text-sm text-muted-foreground", "{address}" } }
                        div { class: "card-body grid grid-cols-1 md:grid-cols-3 gap-4",
                            DetailField { label: "Address".to_string(), value: address.clone() }
                            DetailField { label: "Chain".to_string(), value: "BSC (56)".to_string() }
                            DetailField { label: "Status".to_string(), value: "Active".to_string() }
                            DetailField { label: "Balance".to_string(), value: "1.234 BNB".to_string() }
                            DetailField { label: "Created".to_string(), value: "2024-01-15".to_string() }
                            DetailField { label: "Last active".to_string(), value: "2 min ago".to_string() }
                        }
                    }
                    div { class: "tabs mt-4 mb-4",
                        button { class: if *tab.read() == "overview" { "btn btn-primary" } else { "btn btn-outline" }, onclick: move |_| tab.set("overview".to_string()), "Overview" }
                        button { class: if *tab.read() == "tx" { "btn btn-primary" } else { "btn btn-outline" }, onclick: move |_| tab.set("tx".to_string()), "Transactions" }
                        button { class: if *tab.read() == "subs" { "btn btn-primary" } else { "btn btn-outline" }, onclick: move |_| tab.set("subs".to_string()), "Subscriptions" }
                        button { class: if *tab.read() == "perms" { "btn btn-primary" } else { "btn btn-outline" }, onclick: move |_| tab.set("perms".to_string()), "Permissions" }
                    }
                    if *tab.read() == "overview" {
                        div { class: "card card-glass", div { class: "card-body",
                            p { "Total volume: " span { class: "font-mono font-bold", "$45,231" } }
                            p { "Trades: " span { class: "font-mono", "234" } }
                            p { "Active subscriptions: " span { class: "font-mono", "2" } }
                        } }
                    } else if *tab.read() == "tx" {
                        div { class: "card card-glass", div { class: "card-body p-0",
                            div { class: "table-wrap", table { class: "table", thead { tr { th { "Time" } th { "Type" } th { "Amount" } th { "Token" } th { "Hash" } } } tbody {
                                tr { td { "2024-09-20 10:32" } td { "in" } td { class: "font-mono", "+0.5" } td { "BNB" } td { code { class: "text-xs", "0xabc...123" } } }
                                tr { td { "2024-09-19 15:21" } td { "out" } td { class: "font-mono", "-0.2" } td { "BNB" } td { code { class: "text-xs", "0xdef...456" } } }
                            } } }
                        } }
                    } else if *tab.read() == "subs" {
                        div { class: "card card-glass", div { class: "card-body", p { "Active: Pro plan ($29/mo)" } p { class: "text-muted-foreground text-sm", "Next billing: 2024-10-15" } } }
                    } else {
                        div { class: "card card-glass", div { class: "card-body flex gap-2",
                            Badge { kind: BadgeKind::Success, "trade" } Badge { kind: BadgeKind::Info, "view" } Badge { kind: BadgeKind::Warning, "pay" }
                        } }
                    }
                    div { class: "mt-4", a { class: "btn btn-danger", href: format!("/wallet-management/wallets/{}/disable", address), "Disable wallet" } }
                }
            }
        }
    }
}

pub fn render_disable(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::admin("Disable wallet");
    (meta, rsx! { RenderDisable { ctx: ctx.clone() } })
}

#[component]
fn RenderDisable(ctx: PageContext) -> Element {
    let address = ctx.params.get("address").cloned().unwrap_or_default();
    let mut confirm = use_signal(|| false);
    rsx! {
        AuthGate { user: ctx.user.clone(), feature: Some("disabling wallets".to_string()),
            DashboardShell { current_path: ctx.path.clone(), page_title: "Disable wallet".to_string(), user: ctx.user.clone(),
                div { class: "container page-content max-w-2xl",
                    a { class: "btn btn-sm btn-ghost mb-4", href: format!("/wallet-management/wallets/{}", address), Icon { name: "arrow-left".to_string(), size: Some(16) } " Back" }
                    div { class: "card card-glass border-danger",
                        div { class: "card-header", h1 { class: "card-title text-danger", "Disable wallet" } }
                        div { class: "card-body",
                            p { "You are about to disable the wallet " span { class: "font-mono", "{address}" } "." }
                            p { class: "text-muted-foreground mt-2", "This will revoke all permissions and freeze any active subscriptions." }
                            if !*confirm.read() {
                                div { class: "flex gap-2 mt-4",
                                    button { class: "btn btn-danger", r#type: "button", onclick: move |_| confirm.set(true), "Continue" }
                                    a { class: "btn btn-outline", href: format!("/wallet-management/wallets/{}", address), "Cancel" }
                                }
                            } else {
                                Form { method: "POST".to_string(), action: format!("/api/v1/wallet/wallets/{}/disable", address),
                                    input { r#type: "hidden", name: "address", value: "{address}" }
                                    div { class: "field", label { class: "field-label", "Reason" } input { class: "input", name: "reason", required: true, placeholder: "e.g. suspicious activity" } }
                                    FormActions { button { class: "btn btn-danger", r#type: "submit", "Disable wallet" } a { class: "btn btn-outline", href: format!("/wallet-management/wallets/{}", address), "Cancel" } }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn DetailField(label: String, value: String) -> Element {
    rsx! {
        div { class: "detail-field",
            div { class: "text-sm text-muted-foreground", "{label}" }
            div { class: "font-semibold", "{value}" }
        }
    }
}
