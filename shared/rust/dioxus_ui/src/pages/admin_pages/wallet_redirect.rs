use super::super::{PageContext, PageMeta};
use dioxus::prelude::*;

pub fn render(_ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::admin("Wallet management");
    (
        meta,
        rsx! {
            script { "window.location.replace('/wallet-management/wallets');" }
            div { "Redirecting…" }
        },
    )
}
