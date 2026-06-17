use crate::primitives::*;

use dioxus::prelude::*;
use super::super::{PageContext, PageMeta};

pub fn render(_ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::admin("Wallet management");
    (meta, rsx! {
        script { "window.location.replace('/wallet-management/wallets');" }
        div { "Redirecting…" }
    })
}
