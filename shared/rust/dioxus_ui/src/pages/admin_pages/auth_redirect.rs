use crate::primitives::*;

use super::super::{PageContext, PageMeta};
use dioxus::prelude::*;

pub fn render(_ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::admin("Sign in");
    (
        meta,
        rsx! {
            script { "window.location.replace('/');" }
            div { "Redirecting…" }
        },
    )
}
