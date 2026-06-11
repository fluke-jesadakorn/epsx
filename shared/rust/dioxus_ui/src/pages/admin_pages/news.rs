//! /admin/news (list) + /admin/news/create + /admin/news/[id]/edit.

use crate::primitives::*;
use crate::data_table::{Column, DataTable, Row, SortDir};
use crate::rich_text::RichTextEditor;

use dioxus::prelude::*;
use super::super::{PageContext, PageMeta};
use crate::auth::AuthGate;

pub fn render(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::admin("News");
    let columns = vec![
        Column { key: "title".into(), label: "Title".into(), sortable: true, align: crate::primitives::data_table::Align::Left, width: Some("40%".into()), class_name: None },
        Column { key: "status".into(), label: "Status".into(), sortable: true, align: crate::primitives::data_table::Align::Center, width: Some("15%".into()), class_name: None },
        Column { key: "author".into(), label: "Author".into(), sortable: true, align: crate::primitives::data_table::Align::Left, width: Some("20%".into()), class_name: None },
        Column { key: "created".into(), label: "Created".into(), sortable: true, align: crate::primitives::data_table::Align::Right, width: Some("15%".into()), class_name: None },
        Column { key: "actions".into(), label: "Actions".into(), sortable: false, align: crate::primitives::data_table::Align::Right, width: Some("10%".into()), class_name: None },
    ];
    let rows = vec![
        Row { id: "1".into(), cells: vec!["Welcome to EPSX".into(), "Published".into(), "EPSX Team".into(), "2024-09-15".into(), "Edit".into()] },
        Row { id: "2".into(), cells: vec!["BSC mainnet integration live".into(), "Published".into(), "EPSX Engineering".into(), "2024-09-10".into(), "Edit".into()] },
        Row { id: "3".into(), cells: vec!["Subscription v2: programmable plans".into(), "Draft".into(), "EPSX Product".into(), "2024-09-01".into(), "Edit".into()] },
    ];
    (meta, rsx! {
        AuthGate { user: ctx.user.clone(), feature: Some("news management".to_string()),
            div { class: "container page-content",
                div { class: "flex items-center justify-between mb-6",
                    div {
                        h1 { class: "text-2xl font-bold", "News" }
                        p { class: "text-muted-foreground", "Publish news, updates, and announcements" }
                    }
                    a { class: "btn btn-primary", href: "/news/create", Icon { name: "plus".to_string(), size: Some(16) } " New post" }
                }
                DataTable { columns, rows, striped: true, page_size: 20, filter_placeholder: Some("Filter by title, status, author...".to_string()), initial_sort: Some(("created".to_string(), SortDir::Desc)) }
            }
        }
    })
}

pub fn render_create(ctx: &PageContext) -> (PageMeta, Element) {
    render_editor(ctx, None)
}

pub fn render_edit(ctx: &PageContext) -> (PageMeta, Element) {
    let id = ctx.params.get("id").cloned();
    render_editor(ctx, id)
}

fn render_editor(ctx: &PageContext, id: Option<String>) -> (PageMeta, Element) {
    let title_str = if id.is_some() { "Edit news" } else { "New news post" };
    let meta = PageMeta::admin(title_str);
    (meta, rsx! { RenderNewsEditor { ctx: ctx.clone(), id: id.clone(), title_str: title_str.to_string() } })
}

#[component]
fn RenderNewsEditor(ctx: PageContext, id: Option<String>, title_str: String) -> Element {
    let mut title = use_signal(String::new);
    let mut body = use_signal(|| "## Introduction\n\nWrite your news article here in markdown.\n\n- Point 1\n- Point 2\n\n[Read more](https://epsx.io)".to_string());
    rsx! {
        AuthGate { user: ctx.user.clone(), feature: Some("news editing".to_string()),
            div { class: "container page-content max-w-4xl",
                a { class: "btn btn-sm btn-ghost mb-4", href: "/news", Icon { name: "arrow-left".to_string(), size: Some(16) } " Back" }
                Form { method: "POST".to_string(), action: if let Some(ref i) = id { format!("/api/v1/news/{}", i) } else { "/api/v1/news".to_string() },
                    div { class: "field",
                        label { class: "field-label", "Title" }
                        input { class: "input", name: "title", required: true, value: "{title.read()}", oninput: move |e| title.set(e.value().to_string()), placeholder: "A clear, descriptive title" }
                    }
                    div { class: "field",
                        label { class: "field-label", "Slug" }
                        input { class: "input", name: "slug", required: true, placeholder: "auto-generated-from-title" }
                    }
                    div { class: "field",
                        label { class: "field-label", "Excerpt" }
                        textarea { class: "input", name: "excerpt", rows: "2", placeholder: "Short summary for the listing page" }
                    }
                    div { class: "field",
                        label { class: "field-label", "Body" }
                        RichTextEditor { name: "body".to_string(), label: None, value: Some(body.read().clone()), rows: 16 }
                    }
                    div { class: "field",
                        label { class: "field-label", "Status" }
                        SelectField { name: "status".to_string(), options: vec![("draft".to_string(), "Draft".to_string()), ("published".to_string(), "Published".to_string())], value: Some("draft".to_string()), required: true, label: None, help: None, error: None, placeholder: None, onchange: None }
                    }
                    FormActions {
                        a { class: "btn btn-outline", href: "/news", "Cancel" }
                        button { class: "btn btn-secondary", r#type: "submit", "Save as draft" }
                        button { class: "btn btn-primary", r#type: "submit", "Publish" }
                    }
                }
            }
        }
    }
}
