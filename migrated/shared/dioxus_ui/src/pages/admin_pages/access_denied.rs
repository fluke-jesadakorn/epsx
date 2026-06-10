use crate::primitives::*;

use dioxus::prelude::*;
use super::super::{PageContext, PageMeta};
use crate::auth::AccessDenied;

pub fn render(ctx: &PageContext) -> (PageMeta, Element) {
    let reason = ctx.query_param("reason");
    let detail = ctx.query_param("detail");
    let mut r = reason.unwrap_or_else(|| "Access denied".to_string());
    if let Some(d) = detail { r.push_str(": "); r.push_str(&d); }
    let meta = PageMeta::admin("Access denied");
    (meta, rsx! { AccessDenied { reason: Some(r), required_permissions: None } })
}
