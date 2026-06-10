//! /admin/notifications/manage + /admin/notifications/create.

use crate::primitives::*;
use crate::data_table::{Column, DataTable, Row, SortDir};

use dioxus::prelude::*;
use super::super::{PageContext, PageMeta};
use crate::layout::DashboardShell;
use crate::auth::AuthGate;

pub fn render_manage(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::admin("Notifications");
    let columns = vec![
        Column { key: "title".into(), label: "Title".into(), sortable: true, align: crate::primitives::data_table::Align::Left, width: Some("30%".into()), class_name: None },
        Column { key: "audience".into(), label: "Audience".into(), sortable: true, align: crate::primitives::data_table::Align::Left, width: Some("20%".into()), class_name: None },
        Column { key: "channel".into(), label: "Channel".into(), sortable: true, align: crate::primitives::data_table::Align::Center, width: Some("15%".into()), class_name: None },
        Column { key: "status".into(), label: "Status".into(), sortable: true, align: crate::primitives::data_table::Align::Center, width: Some("15%".into()), class_name: None },
        Column { key: "sent".into(), label: "Sent".into(), sortable: true, align: crate::primitives::data_table::Align::Right, width: Some("10%".into()), class_name: None },
        Column { key: "created".into(), label: "Created".into(), sortable: true, align: crate::primitives::data_table::Align::Right, width: Some("10%".into()), class_name: None },
    ];
    let rows = vec![
        Row { id: "n1".into(), cells: vec!["Welcome to the platform".into(), "All users".into(), "Email".into(), "Sent".into(), "1,234".into(), "2024-09-20".into()] },
        Row { id: "n2".into(), cells: vec!["New feature: charts".into(), "Pro plan".into(), "In-app".into(), "Sent".into(), "432".into(), "2024-09-18".into()] },
        Row { id: "n3".into(), cells: vec!["Maintenance window".into(), "All users".into(), "Email + Push".into(), "Scheduled".into(), "—".into(), "2024-09-15".into()] },
    ];
    (meta, rsx! {
        AuthGate { user: ctx.user.clone(), feature: Some("notification management".to_string()),
            DashboardShell { current_path: ctx.path.clone(), page_title: "Notifications".to_string(), user: ctx.user.clone(),
                div { class: "container page-content",
                    div { class: "flex items-center justify-between mb-6",
                        div {
                            h1 { class: "text-2xl font-bold", "Notifications" }
                            p { class: "text-muted-foreground", "All sent, scheduled, and draft notifications" }
                        }
                        a { class: "btn btn-primary", href: "/notifications/create", Icon { name: "plus".to_string(), size: Some(16) } " New notification" }
                    }
                    DataTable { columns, rows, striped: true, page_size: 20, filter_placeholder: Some("Filter by title, audience, channel...".to_string()), initial_sort: Some(("created".to_string(), SortDir::Desc)) }
                }
            }
        }
    })
}

pub fn render_create(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::admin("New notification");
    (meta, rsx! {
        AuthGate { user: ctx.user.clone(), feature: Some("creating notifications".to_string()),
            DashboardShell { current_path: ctx.path.clone(), page_title: "New notification".to_string(), user: ctx.user.clone(),
                div { class: "container page-content max-w-3xl",
                    a { class: "btn btn-sm btn-ghost mb-4", href: "/notifications/manage", Icon { name: "arrow-left".to_string(), size: Some(16) } " Back" }
                    Form { method: "POST".to_string(), action: "/api/v1/notification/send".to_string(),
                        div { class: "card card-glass",
                            div { class: "card-header", h1 { class: "card-title", "Compose notification" } }
                            div { class: "card-body space-y-4",
                                div { class: "field",
                                    label { class: "field-label", "Title" }
                                    input { class: "input", name: "title", required: true, placeholder: "Notification title" }
                                }
                                div { class: "field",
                                    label { class: "field-label", "Body" }
                                    textarea { class: "input", name: "body", rows: "4", required: true, placeholder: "Message body..." }
                                }
                                div { class: "grid grid-cols-1 md:grid-cols-2 gap-4",
                                    div { class: "field",
                                        label { class: "field-label", "Audience" }
                                        SelectField { name: "audience".to_string(), options: vec![("all".to_string(), "All users".to_string()), ("pro".to_string(), "Pro plan".to_string()), ("enterprise".to_string(), "Enterprise".to_string()), ("admins".to_string(), "Admins only".to_string())], value: Some("all".to_string()), required: true, label: None, help: None, error: None, placeholder: None, onchange: None }
                                    }
                                    div { class: "field",
                                        label { class: "field-label", "Channel" }
                                        SelectField { name: "channel".to_string(), options: vec![("inapp".to_string(), "In-app".to_string()), ("email".to_string(), "Email".to_string()), ("push".to_string(), "Push".to_string())], value: Some("inapp".to_string()), required: true, label: None, help: None, error: None, placeholder: None, onchange: None }
                                    }
                                }
                                div { class: "field",
                                    label { class: "field-label", "Action URL (optional)" }
                                    input { class: "input", name: "action_url", placeholder: "/path/to/resource" }
                                }
                                div { class: "field",
                                    CheckboxField { name: "schedule".to_string(), label: "Schedule for later".to_string() }
                                }
                                FormActions {
                                    a { class: "btn btn-outline", href: "/notifications/manage", "Cancel" }
                                    button { class: "btn btn-primary", r#type: "submit", "Send notification" }
                                }
                            }
                        }
                    }
                }
            }
        }
    })
}
