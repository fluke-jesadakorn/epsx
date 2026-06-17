use crate::primitives::*;

use dioxus::prelude::*;
use super::super::{PageContext, PageMeta};
use crate::auth::AccessDenied;

pub fn render(ctx: &PageContext) -> (PageMeta, Element) {
    let reason = ctx.query_param("reason");
    let detail = ctx.query_param("detail");
    let route = ctx.query_param("route");
    let context = ctx.query_param("context");
    let permission = ctx.query_param("permission");
    let mut r = reason.unwrap_or_else(|| "Access denied".to_string());
    if let Some(d) = detail { r.push_str(": "); r.push_str(&d); }

    // Append the additional context fields (route / context /
    // permission) to the reason text, mirroring the OLD's
    // `<AccessDeniedContent>` display. The shared `AccessDenied`
    // component doesn't have dedicated props for these, so we fold
    // them into the reason string. Wave 21 admin-recheck port.
    if let Some(p) = permission {
        r.push_str(" [permission=");
        r.push_str(&p);
        r.push(']');
    }
    if let Some(c) = context {
        r.push_str(" [context=");
        r.push_str(&c);
        r.push(']');
    }
    if let Some(rr) = route {
        r.push_str(" [route=");
        r.push_str(&rr);
        r.push(']');
    }

    let meta = PageMeta::admin("Access denied");
    (meta, rsx! { AccessDenied { reason: Some(r), required_permissions: None } })
}
