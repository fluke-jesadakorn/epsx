use crate::primitives::*;

use dioxus::prelude::*;
use super::PageContext;
use super::PageMeta;
use crate::layout::{Navbar, Footer};

pub fn render(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::marketing("Not found");
    (meta, rsx! {
        Navbar { user: ctx.user.clone(), current_path: Some(ctx.path.clone()) }
        div { class: "container page-content",
            div { class: "not-found",
                div { class: "not-found-code", "404" }
                h1 { class: "not-found-title", "Page not found" }
                p { class: "not-found-description text-muted-foreground", "The page you are looking for does not exist." }
                div { class: "not-found-actions",
                    a { class: "btn btn-primary", href: "/", "Back to home" }
                    a { class: "btn btn-outline", href: "/contact", "Contact support" }
                }
            }
        }
        Footer {}
    })
}
