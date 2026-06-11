//! /admin/notifications/manage + /admin/notifications/create.
//!
//! Wave 6C Track D — 7 sections per the design doc
//! `docs/wave6c-live-render-and-1to1/design.md` §"Track D":
//!   1. NotificationList             — list of sent/scheduled/draft
//!   2. SendForm                     — compose form
//!   3. RecipientsPicker             — targeted client vs. global broadcast
//!   4. NotificationTemplateEditor   — title + body + action URL
//!   5. NotificationPreview          — live preview pane
//!   6. NotificationScheduleDialog   — schedule-for-later
//!   7. NotificationManagementFilters— filter chips
//!
//! All 7 sub-components + the `Notification` + `NotificationStats`
//! data structs + the `NotificationStatCard` helper live in
//! `components::admin::notifications`. This page just composes
//! them inside the `AdminAuthGate` wrapper.

use crate::primitives::*;
use crate::components::admin::notifications::{
    Notification, NotificationList, NotificationManagementFilters, NotificationPreview,
    NotificationScheduleDialog, NotificationStats, NotificationStatCard,
    NotificationTemplateEditor, RecipientsPicker,
};
use crate::data_table::{Column, DataTable, Row, SortDir};

use dioxus::prelude::*;
use super::super::{PageContext, PageMeta};
use crate::auth::AdminAuthGate;

// =====================================================================
// Public entry points
// =====================================================================

/// 3 sample notifications for the SSR'd list.
fn sample_notifications() -> Vec<Notification> {
    vec![
        Notification {
            id: "n1".into(),
            title: "Welcome to the platform".into(),
            message: "We're excited to have you on board. Here's a quick tour of the key features.".into(),
            priority: "normal".into(),
            notification_type: "general".into(),
            timestamp: Some("2024-09-20T10:32:15Z".into()),
        },
        Notification {
            id: "n2".into(),
            title: "New feature: charts".into(),
            message: "Pro users can now create custom charts in the analytics dashboard.".into(),
            priority: "high".into(),
            notification_type: "system".into(),
            timestamp: Some("2024-09-18T14:15:00Z".into()),
        },
        Notification {
            id: "n3".into(),
            title: "Maintenance window".into(),
            message: "Scheduled maintenance on Sep 22, 02:00–04:00 UTC. Brief downtime expected.".into(),
            priority: "critical".into(),
            notification_type: "system".into(),
            timestamp: None,
        },
    ]
}

fn sample_stats() -> NotificationStats {
    NotificationStats { total: 1234, sent_today: 28, sent_this_week: 142 }
}

/// `/admin/notifications/manage`.
pub fn render_manage(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::admin("Notifications");
    let notifications = sample_notifications();
    let stats = sample_stats();
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

    (
        meta,
        rsx! {
            AdminAuthGate {
                user: ctx.user.clone(),
                feature: Some("notification management".to_string()),
                required_permissions: Some(vec!["notifications:manage".to_string()]),
                return_url: Some(ctx.path.clone()),
                div { class: "container page-content",
                    // Header row
                    div { class: "flex items-center justify-between mb-6",
                        div {
                            h1 { class: "text-2xl font-bold", "Notifications" }
                            p { class: "text-muted-foreground", "All sent, scheduled, and draft notifications" }
                        }
                        a { class: "btn btn-primary", href: "/notifications/create",
                            Icon { name: "plus".to_string(), size: Some(16) }
                            " New notification"
                        }
                    }
                    // Section 7.
                    NotificationManagementFilters {}
                    // Stats grid (4 cards)
                    div { class: "notification-stats-grid grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6 mt-6",
                        NotificationStatCard { title: "Total Sent".to_string(), value: stats.total.to_string(), icon: "layers".to_string(), color: "cyan".to_string() }
                        NotificationStatCard { title: "Today's Pulse".to_string(), value: stats.sent_today.to_string(), icon: "clock".to_string(), color: "amber".to_string() }
                        NotificationStatCard { title: "Weekly Volume".to_string(), value: stats.sent_this_week.to_string(), icon: "calendar".to_string(), color: "purple".to_string() }
                        NotificationStatCard { title: "System Health".to_string(), value: "Stable".to_string(), icon: "shield".to_string(), color: "green".to_string() }
                    }
                    // Action buttons row
                    div { class: "notification-action-buttons grid grid-cols-1 sm:grid-cols-2 gap-6 mt-6",
                        button {
                            class: "notification-sync-btn group relative overflow-hidden rounded-xl bg-card border border-border/20 p-8 text-left hover:border-[#1fc7d4]/30 shadow-xl",
                            r#type: "button",
                            div { class: "flex items-center space-x-6",
                                div { class: "w-16 h-16 flex items-center justify-center bg-cyan-500/10 rounded-2xl border border-cyan-500/20 text-[#1fc7d4]",
                                    Icon { name: "refresh-cw".to_string(), size: Some(32) }
                                }
                                div {
                                    h3 { class: "text-xl font-black text-foreground uppercase tracking-tight", "Synchronize" }
                                    p { class: "text-sm text-muted-foreground", "Refresh real-time telemetry" }
                                }
                            }
                        }
                        button {
                            class: "notification-analytics-btn group relative overflow-hidden rounded-xl bg-card border border-border/20 p-8 text-left hover:border-purple-500/30 shadow-xl",
                            r#type: "button",
                            div { class: "flex items-center space-x-6",
                                div { class: "w-16 h-16 flex items-center justify-center bg-purple-500/10 rounded-2xl border border-purple-500/20 text-purple-400",
                                    Icon { name: "bar-chart-2".to_string(), size: Some(32) }
                                }
                                div {
                                    h3 { class: "text-xl font-black text-foreground uppercase tracking-tight", "Analytics" }
                                    p { class: "text-sm text-muted-foreground", "Deep dive performance metrics" }
                                }
                            }
                        }
                    }
                    // Section 1.
                    NotificationList { notifications: notifications.clone() }
                    // DataTable — secondary table view.
                    div { class: "mt-8",
                        h2 { class: "text-lg font-semibold mb-3", "All broadcasts" }
                        DataTable {
                            columns,
                            rows,
                            striped: true,
                            page_size: 20,
                            filter_placeholder: Some("Filter by title, audience, channel...".to_string()),
                            initial_sort: Some(("created".to_string(), SortDir::Desc)),
                        }
                    }
                }
            }
        },
    )
}

/// `/admin/notifications/create` — the send form.
pub fn render_create(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::admin("New notification");
    (
        meta,
        rsx! {
            AdminAuthGate {
                user: ctx.user.clone(),
                feature: Some("creating notifications".to_string()),
                required_permissions: Some(vec!["notifications:manage".to_string()]),
                return_url: Some(ctx.path.clone()),
                div { class: "container page-content max-w-3xl",
                    a { class: "btn btn-sm btn-ghost mb-4", href: "/notifications/manage",
                        Icon { name: "arrow-left".to_string(), size: Some(16) }
                        " Back"
                    }
                    Form { method: "POST".to_string(), action: "/api/v1/notification/send".to_string(),
                        div { class: "card card-glass",
                            div { class: "card-header", h1 { class: "card-title", "Compose notification" } }
                            div { class: "card-body space-y-4",
                                // Section 3.
                                RecipientsPicker {}
                                // Section 4.
                                NotificationTemplateEditor {}
                                // Section 5.
                                NotificationPreview {}
                                // Section 6.
                                NotificationScheduleDialog {}
                                // Section 2 (footer actions).
                                div { class: "send-form-actions flex gap-2",
                                    a { class: "btn btn-outline", href: "/notifications/manage", "Cancel" }
                                    button { class: "btn btn-primary", r#type: "submit", "📤 Send notification" }
                                }
                            }
                        }
                    }
                }
            }
        },
    )
}

// =====================================================================
// Unit tests
// =====================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pages::PageContext;
    use crate::auth::User;

    fn test_user_admin_with(perms: &[&str]) -> User {
        User {
            id: "u1".to_string(),
            address: "0xADMIN0000000000000000000000000000000001".to_string(),
            chain_id: "56".to_string(),
            roles: vec!["admin".to_string()],
            email: Some("admin@epsx.io".to_string()),
            tier: Some("admin".to_string()),
            permissions: perms.iter().map(|s| s.to_string()).collect(),
            ..Default::default()
        }
    }

    fn render_to_string(el: Element) -> String {
        dioxus_ssr::render_element(el)
    }

    /// test_render_smoke for the manage view.
    #[test]
    fn notifications_manage_renders_smoke() {
        let ctx = PageContext {
            user: Some(test_user_admin_with(&["notifications:manage"])),
            path: "/admin/notifications/manage".to_string(),
            ..Default::default()
        };
        let (_meta, el) = render_manage(&ctx);
        let html = render_to_string(el);
        assert!(!html.trim().is_empty(), "notifications_manage should render non-empty HTML");
    }

    /// test_section_markers for the manage view.
    #[test]
    fn notifications_manage_section_markers() {
        let ctx = PageContext {
            user: Some(test_user_admin_with(&["notifications:manage"])),
            path: "/admin/notifications/manage".to_string(),
            ..Default::default()
        };
        let (_, el) = render_manage(&ctx);
        let html = render_to_string(el);
        for marker in &[
            "notification-list",
            "notification-list-row",
            "notification-management-filters",
            "notification-stats-grid",
            "All sent, scheduled, and draft notifications",
        ] {
            assert!(
                html.contains(marker),
                "notifications_manage should contain section marker `{marker}`. Got: {}",
                html
            );
        }
    }

    /// test_section_markers for the create view.
    #[test]
    fn notifications_create_section_markers() {
        let ctx = PageContext {
            user: Some(test_user_admin_with(&["notifications:manage"])),
            path: "/admin/notifications/create".to_string(),
            ..Default::default()
        };
        let (_meta, el) = render_create(&ctx);
        let html = render_to_string(el);
        for marker in &[
            "recipients-picker",
            "notification-template-editor",
            "notification-preview",
            "notification-schedule-dialog",
            "Compose notification",
        ] {
            assert!(
                html.contains(marker),
                "notifications_create should contain section marker `{marker}`. Got: {}",
                html
            );
        }
    }

    /// Non-admin user is bounced by the admin gate.
    #[test]
    fn notifications_gates_non_admin_user() {
        let u = User {
            id: "u2".to_string(),
            address: "0xUSER000000000000000000000000000000000001".to_string(),
            chain_id: "56".to_string(),
            roles: vec!["user".to_string()],
            email: Some("user@epsx.io".to_string()),
            tier: Some("free".to_string()),
            permissions: vec![],
            ..Default::default()
        };
        let ctx = PageContext {
            user: Some(u),
            path: "/admin/notifications/manage".to_string(),
            ..Default::default()
        };
        let (_, el) = render_manage(&ctx);
        let html = render_to_string(el);
        assert!(
            html.contains("auth-gate-admin"),
            "AdminAuthGate must render the admin gate panel for non-admin. Got: {}",
            html
        );
    }
}
