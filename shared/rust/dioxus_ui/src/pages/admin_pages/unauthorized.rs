use crate::primitives::*;

use super::super::{PageContext, PageMeta};
use crate::auth::AccessDenied;
use dioxus::prelude::*;

pub fn render(_ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::admin("Unauthorized");
    (
        meta,
        rsx! {
            AccessDenied { reason: Some("You are not authorized to view this resource.".to_string()), required_permissions: Some(vec!["admin:*".to_string()]) }
        },
    )
}
