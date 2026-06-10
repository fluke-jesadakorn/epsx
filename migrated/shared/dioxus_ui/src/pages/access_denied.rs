use crate::primitives::*;

use dioxus::prelude::*;
use super::PageContext;
use super::PageMeta;
use crate::layout::{Navbar, Footer};
use crate::auth::AccessDenied;

pub fn render(ctx: &PageContext) -> (PageMeta, Element) {
    let reason = ctx.query_param("reason");
    let meta = PageMeta::marketing("Access denied");
    (meta, rsx! {
        Navbar { user: ctx.user.clone(), current_path: Some(ctx.path.clone()) }
        div { class: "container page-content",
            AccessDenied { reason: reason, required_permissions: None }
        }
        Footer {}
    })
}
