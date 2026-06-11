use crate::primitives::*;

use dioxus::prelude::*;
use super::PageContext;
use super::PageMeta;
use crate::layout::main_layout::MainLayout;
use crate::auth::AccessDenied;

pub fn render(ctx: &PageContext) -> (PageMeta, Element) {
    let reason = ctx.query_param("reason");
    let meta = PageMeta::marketing("Access denied");
    (meta, rsx! {
        MainLayout { ctx: ctx.clone(),
            div { class: "container page-content",
                AccessDenied { reason: reason, required_permissions: None }
            }
        }
    })
}
