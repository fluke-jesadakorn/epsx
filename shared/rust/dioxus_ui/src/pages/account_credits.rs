//! /account/credits — credits balance + history.

use crate::primitives::*;
use crate::feedback::*;

use dioxus::prelude::*;
use super::PageContext;
use super::PageMeta;
use crate::layout::main_layout::MainLayout;
use crate::layout::PageHeader;
use crate::auth::AuthGate;

pub fn render(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::app("Credits");
    (meta, rsx! {
        MainLayout { ctx: ctx.clone(),
            AuthGate { user: ctx.user.clone(), feature: Some("your credits".to_string()),
                div { class: "container page-content",
                    PageHeader {
                        title: "Credits".to_string(),
                        description: Some("Your EPSX credits balance and usage".to_string()),
                        icon: Some("coins".to_string()),
                    }
                    div { class: "grid grid-cols-1 md:grid-cols-3 gap-4 mb-6",
                        StatCard { label: "Available".to_string(), value: "1,250".to_string(), icon: Some("coins".to_string()) }
                        StatCard { label: "Used this month".to_string(), value: "340".to_string(), icon: Some("trending-down".to_string()) }
                        StatCard { label: "Earned".to_string(), value: "1,590".to_string(), icon: Some("trending-up".to_string()) }
                    }
                    div { class: "card card-glass",
                        div { class: "card-header", h3 { class: "card-title", "Recent activity" } }
                        div { class: "card-body p-0",
                            div { class: "table-wrap",
                                table { class: "table",
                                    thead { tr { th { "Date" } th { "Type" } th { "Description" } th { "Amount" } } }
                                    tbody {
                                        tr { td { "2024-09-20" } td { "Earn" } td { "Trade reward" } td { class: "text-success font-semibold", "+50" } }
                                        tr { td { "2024-09-19" } td { "Spend" } td { "API call" } td { class: "text-danger font-semibold", "-1" } }
                                        tr { td { "2024-09-18" } td { "Spend" } td { "Premium feature" } td { class: "text-danger font-semibold", "-100" } }
                                        tr { td { "2024-09-15" } td { "Earn" } td { "Subscription bonus" } td { class: "text-success font-semibold", "+500" } }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    })
}
