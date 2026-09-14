use super::super::{PageContext, PageMeta};
use dioxus::prelude::*;

pub fn render(_ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::admin("Wallet management");
    (
        meta,
        rsx! {
            div { "Redirecting… " crate::navigation::AppLink { href: "/wallet-management/wallets", "Continue" } }
        },
    )
}
