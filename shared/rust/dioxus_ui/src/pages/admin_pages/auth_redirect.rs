use super::super::{PageContext, PageMeta};
use dioxus::prelude::*;

pub fn render(_ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::admin("Sign in");
    (
        meta,
        rsx! {
            div { "Redirecting… " a { href: "/", "Continue" } }
        },
    )
}
