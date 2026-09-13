use super::super::{PageContext, PageMeta};
use dioxus::prelude::*;

pub fn render(_ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::admin("Notifications");
    (
        meta,
        rsx! {
            div { "Redirecting… " a { href: "/notifications/manage", "Continue" } }
        },
    )
}
