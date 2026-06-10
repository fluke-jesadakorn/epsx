use crate::primitives::*;

use dioxus::prelude::*;
use super::PageContext;
use super::PageMeta;
use crate::layout::{Navbar, Footer};

pub fn render(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::marketing("Error");
    (meta, rsx! {
        Navbar { user: ctx.user.clone(), current_path: Some(ctx.path.clone()) }
        div { class: "container page-content",
            div { class: "error-page",
                h1 { class: "error-page-title", "Something went wrong" }
                p { class: "text-muted-foreground", "An unexpected error occurred. Please try again." }
                div { class: "error-page-actions",
                    a { class: "btn btn-primary", href: "/", "Back to home" }
                    a { class: "btn btn-outline", href: "javascript:location.reload()", "Reload" }
                }
            }
        }
        Footer {}
    })
}
