use crate::primitives::*;

use dioxus::prelude::*;
use super::PageContext;
use super::PageMeta;
use crate::layout::main_layout::MainLayout;
use crate::layout::PageHeader;

pub fn render(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::marketing("Contact");
    (meta, rsx! {
        MainLayout { ctx: ctx.clone(),
            div { class: "container page-content",
                PageHeader { title: "Contact".to_string(), description: Some("Get in touch with the EPSX team".to_string()), icon: Some("mail".to_string()) }
                div { class: "grid grid-cols-1 md:grid-cols-3 gap-4",
                    div { class: "card card-glass",
                        div { class: "card-header", h3 { class: "card-title", "Email" } }
                        div { class: "card-body",
                            a { class: "btn btn-outline btn-block", href: "mailto:info@epsx.io", "info@epsx.io" }
                        }
                    }
                    div { class: "card card-glass",
                        div { class: "card-header", h3 { class: "card-title", "Documentation" } }
                        div { class: "card-body",
                            a { class: "btn btn-outline btn-block", href: "/manual", "Open manual" }
                        }
                    }
                    div { class: "card card-glass",
                        div { class: "card-header", h3 { class: "card-title", "Developer" } }
                        div { class: "card-body",
                            a { class: "btn btn-outline btn-block", href: "/developer/docs", "API docs" }
                        }
                    }
                }
            }
        }
    })
}
