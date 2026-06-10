use crate::primitives::*;

use dioxus::prelude::*;
use super::super::{PageContext, PageMeta};

pub fn render(_ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::admin("Notifications");
    (meta, rsx! {
        script { "window.location.replace('/notifications/manage');" }
        div { "Redirecting…" }
    })
}
