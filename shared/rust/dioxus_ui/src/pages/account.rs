//! /account + /account/credits — account overview, payment history, credits
//! balance.

use crate::primitives::*;
use crate::feedback::*;

use dioxus::prelude::*;
use super::PageContext;
use super::PageMeta;
use crate::layout::main_layout::MainLayout;
use crate::layout::PageHeader;
use crate::auth::AuthGate;

pub fn render(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::app("Account");
    (meta, rsx! { RenderAccount { ctx: ctx.clone() } })
}

#[component]
fn RenderAccount(ctx: PageContext) -> Element {
    let mut tab = use_signal(|| "overview".to_string());
    rsx! {
        MainLayout { ctx: ctx.clone(),
            AuthGate { user: ctx.user.clone(), feature: Some("your account".to_string()),
                required_permissions: Some(vec!["profile:read".to_string()]),
                return_url: Some(ctx.path.clone()),
                div { class: "container page-content",
                    PageHeader { title: "Account".to_string(), description: Some("Manage your profile, payment methods, and credits".to_string()), icon: Some("user".to_string()) }
                    div { class: "tabs mb-4",
                        button { class: if *tab.read() == "overview" { "btn btn-primary" } else { "btn btn-outline" }, onclick: move |_| tab.set("overview".to_string()), "Overview" }
                        button { class: if *tab.read() == "payment" { "btn btn-primary" } else { "btn btn-outline" }, onclick: move |_| tab.set("payment".to_string()), "Payment history" }
                        button { class: if *tab.read() == "credits" { "btn btn-primary" } else { "btn btn-outline" }, onclick: move |_| tab.set("credits".to_string()), "Credits" }
                    }
                    if *tab.read() == "payment" { PaymentHistory {} }
                    else if *tab.read() == "credits" {
                        div { class: "card card-glass", div { class: "card-body", a { class: "btn btn-primary", href: "/account/credits", "View credits details" } } }
                    }
                    else { Overview { user: ctx.user.clone() } }
                }
            }
        }
    }
}

#[component]
fn Overview(user: Option<crate::auth::User>) -> Element {
    rsx! {
        div { class: "grid grid-cols-1 md:grid-cols-2 gap-4",
            div { class: "card card-glass",
                div { class: "card-header", h3 { class: "card-title", "Profile" } }
                div { class: "card-body",
                    if let Some(u) = &user {
                        div { class: "space-y-2",
                            div { span { class: "text-muted-foreground", "Wallet: " } span { class: "font-mono text-sm", "{u.address}" } }
                            div { span { class: "text-muted-foreground", "Chain: " } span { "{u.chain_id}" } }
                            div { span { class: "text-muted-foreground", "Roles: " } span { "{u.roles.join(\", \")}" } }
                        }
                    } else {
                        p { "Not signed in" }
                    }
                }
            }
            div { class: "card card-glass",
                div { class: "card-header", h3 { class: "card-title", "Quick actions" } }
                div { class: "card-body flex flex-col gap-2",
                    a { class: "btn btn-outline btn-block", href: "/account/credits", "View credits" }
                    a { class: "btn btn-outline btn-block", href: "/plans", "Manage plan" }
                    a { class: "btn btn-outline btn-block", href: "/permissions", "Permissions" }
                }
            }
        }
    }
}

#[component]
fn PaymentHistory() -> Element {
    let rows = vec![
        ("2024-09-15", "$29.00", "Pro plan", "Confirmed"),
        ("2024-08-15", "$29.00", "Pro plan", "Confirmed"),
        ("2024-07-15", "$29.00", "Pro plan", "Confirmed"),
        ("2024-06-15", "$0.00", "Free plan", "Active"),
    ];
    rsx! {
        div { class: "card card-glass",
            div { class: "card-header", h3 { class: "card-title", "Payment history" } }
            div { class: "card-body p-0",
                div { class: "table-wrap",
                    table { class: "table",
                        thead { tr { th { "Date" } th { "Amount" } th { "Description" } th { "Status" } } }
                        tbody {
                            for (d, a, desc, status) in rows {
                                tr {
                                    td { class: "text-sm", "{d}" }
                                    td { class: "font-mono font-semibold", "{a}" }
                                    td { "{desc}" }
                                    td {
                                        span { class: if status == "Confirmed" { "badge badge-success" } else { "badge badge-info" }, "{status}" }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
