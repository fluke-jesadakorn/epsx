//! Compatibility renderer for retired guide bookmarks. The native BFF redirects first.
use super::{PageContext, PageMeta};
use dioxus::prelude::*;

pub fn render(ctx: &PageContext) -> (PageMeta, Element) {
    let mut ctx = ctx.clone();
    ctx.path = "/analytics".into();
    super::analytics::render(&ctx)
}
