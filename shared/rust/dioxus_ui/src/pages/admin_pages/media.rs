//! /admin/media — media browser.

use crate::primitives::*;

use dioxus::prelude::*;
use super::super::{PageContext, PageMeta};
use crate::layout::DashboardShell;
use crate::auth::AuthGate;

pub fn render(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::admin("Media");
    (meta, rsx! {
        AuthGate { user: ctx.user.clone(), feature: Some("media management".to_string()),
            DashboardShell { current_path: ctx.path.clone(), page_title: "Media".to_string(), user: ctx.user.clone(),
                div { class: "container page-content",
                    div { class: "flex items-center justify-between mb-6",
                        div {
                            h1 { class: "text-2xl font-bold", "Media" }
                            p { class: "text-muted-foreground", "Images, videos, and documents" }
                        }
                        button { class: "btn btn-primary", r#type: "button", Icon { name: "upload".to_string(), size: Some(16) } " Upload" }
                    }
                    div { class: "card card-glass",
                        div { class: "card-body",
                            FileUpload { name: "media".to_string(), label: Some("Drop files here or click to upload".to_string()), multiple: true, accept: Some("image/*,video/*,.pdf".to_string()), help: Some("Max 50 MB per file".to_string()) }
                        }
                    }
                    div { class: "grid grid-cols-2 md:grid-cols-4 lg:grid-cols-6 gap-4 mt-6",
                        for i in 0..12 {
                            div { class: "card card-glass",
                                div { class: "card-body p-3",
                                    div { class: "aspect-square bg-muted rounded mb-2 flex items-center justify-center",
                                        Icon { name: "image".to_string(), size: Some(32) }
                                    }
                                    p { class: "text-xs truncate", "image_{i}.png" }
                                    p { class: "text-xs text-muted-foreground", "234 KB" }
                                }
                            }
                        }
                    }
                }
            }
        }
    })
}
