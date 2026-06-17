use crate::primitives::*;

use dioxus::prelude::*;
use super::super::{PageContext, PageMeta};
use crate::auth::AccessDenied;

pub fn render(_ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::admin("Unauthorized");
    (meta, rsx! {
        AccessDenied { reason: Some("You are not authorized to view this resource.".to_string()), required_permissions: Some(vec!["admin:*".to_string()]) }
    })
}
