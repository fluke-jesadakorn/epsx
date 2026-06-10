use crate::primitives::*;

use dioxus::prelude::*;
use super::PageContext;
use super::PageMeta;
use crate::layout::{Navbar, Footer};

pub fn render(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::marketing("Privacy policy");
    (meta, rsx! {
        Navbar { user: ctx.user.clone(), current_path: Some(ctx.path.clone()) }
        div { class: "container page-content prose",
            h1 { "Privacy policy" }
            p { "Last updated: today." }
            h2 { "What we collect" }
            p { "EPSX stores wallet addresses, signatures, and (optionally) email addresses. We do not collect passwords." }
            h2 { "How we use it" }
            p { "We use your data to authenticate you, process payments, and surface analytics. We do not sell your data to third parties." }
            h2 { "Cookies" }
            p { "We use first-party cookies for session management (auth_token, refresh_token, user). No third-party tracking cookies." }
            h2 { "Contact" }
            p { "Questions? Email info@epsx.io." }
        }
        Footer {}
    })
}
