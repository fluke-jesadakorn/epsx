use crate::primitives::*;

use dioxus::prelude::*;
use super::super::{PageContext, PageMeta};
use crate::layout::DashboardShell;
use crate::auth::AuthGate;

pub fn render(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::admin("Access control");
    (meta, rsx! {
        AuthGate { user: ctx.user.clone(), feature: Some("access control".to_string()),
            DashboardShell { current_path: ctx.path.clone(), page_title: "Access control".to_string(), user: ctx.user.clone(),
                div { class: "container page-content",
                    div { class: "card card-glass",
                        div { class: "card-header",
                            h3 { class: "card-title", "Plans" }
                            a { class: "btn btn-primary btn-sm", href: "/wallet-management/access/plans", "Manage plans" }
                        }
                        div { class: "card-body",
                            p { "Plans define what permissions a wallet has, billing, and API quotas." }
                        }
                    }
                }
            }
        }
    })
}
