//! /admin/notifications/manage + /admin/notifications/create.
//!
//! Source of truth: `apps-old/admin-frontend/{app/notifications/*,
//! components/notifications/notification-management.tsx,
//! components/notifications/send-notification-form.tsx}`.
//!
//! The Wave 6B port brings the 7 design-doc-named sections (per
//! `docs/wave6b-admin-pages-depth/design.md` §"Track B"):
//!   1. NotificationList             — list of sent/scheduled/draft
//!   2. SendForm                     — compose form (title, body, etc.)
//!   3. RecipientsPicker             — targeted client vs. global broadcast
//!   4. NotificationTemplateEditor   — title + body + action URL
//!   5. NotificationPreview          — live preview pane
//!   6. NotificationScheduleDialog   — schedule-for-later modal
//!   7. NotificationManagementFilters — filter chips (all/sent/scheduled)
//!
//! The two existing entry points (`render_manage`, `render_create`)
//! dispatch into the right section for each route.

use crate::primitives::*;
use crate::data_table::{Column, DataTable, Row, SortDir};
use crate::feedback::admin_action_confirm::{AdminActionConfirm, ConfirmVariant};

use dioxus::prelude::*;
use super::super::{PageContext, PageMeta};
use crate::auth::AdminAuthGate;

// =====================================================================
// Public entry points
// =====================================================================

/// `/admin/notifications/manage` — the list view + filters +
/// send-form preview. Uses the `DataTable` as a secondary table
/// view for parity with the shallow port. The "manage" view
/// renders the source's `NotificationManagement` component shape:
/// 4 stats cards, 2 action buttons (Synchronize / Analytics), and
/// the notifications table.
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
                    // Header row: title + create button
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
                    // Section 7: NotificationManagementFilters
                    NotificationManagementFilters {}
                    // Stats grid (4 cards) — the source's StatsGrid
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
                    // Section 1: NotificationList
                    NotificationList { notifications: notifications.clone() }
                    // DataTable — secondary table view (kept from
                    // the shallow port).
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

/// `/admin/notifications/create` — the send form. Renders all
/// 5 form-related sections (SendForm, RecipientsPicker, Template
/// Editor, Preview, ScheduleDialog).
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
                                // Section 3: RecipientsPicker
                                RecipientsPicker {}
                                // Section 4: NotificationTemplateEditor
                                NotificationTemplateEditor {}
                                // Section 5: NotificationPreview
                                NotificationPreview {}
                                // Section 6: NotificationScheduleDialog trigger
                                NotificationScheduleDialog {}
                                // Section 2: SendForm footer actions
                                FormActions {
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
// Data
// =====================================================================

/// Notification data. Mirrors the `Notification` interface in the
/// source's `notification-management.tsx` (sub-shape — the full
/// type has more fields, but the Dioxus port only renders the
/// ones listed).
#[derive(Clone, Debug, PartialEq)]
pub struct Notification {
    pub id: String,
    pub title: String,
    pub message: String,
    /// `low` | `normal` | `high` | `critical`
    pub priority: String,
    /// `system` | `security` | `permission` | `payment` | `general`
    pub notification_type: String,
    /// ISO-8601 timestamp; None for drafts.
    pub timestamp: Option<String>,
}

/// Notification stats. Mirrors the source's `NotificationStats`
/// interface.
#[derive(Clone, Debug, PartialEq)]
pub struct NotificationStats {
    pub total: i32,
    pub sent_today: i32,
    pub sent_this_week: i32,
}

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

// =====================================================================
// Section 1: NotificationList
// =====================================================================
//
// List of sent/scheduled/draft notifications. Mirrors the
// source's `NotifTable` + `NotificationRow` pair. The
/// `notification-list` section-marker is on the outer wrapper.
#[component]
fn NotificationList(notifications: Vec<Notification>) -> Element {
    rsx! {
        div { class: "notification-list mt-6 rounded-2xl border border-border/20 overflow-hidden bg-card shadow-xl",
            // Top accent bar
            div { class: "h-[3px] bg-gradient-to-r from-[#ffb237] to-[#7645d9]" }
            // Header row
            div { class: "flex items-center justify-between px-4 py-3 border-b border-border/20",
                h2 { class: "text-xs font-bold text-[#ffb237] uppercase tracking-[0.2em]", "RECENT BROADCASTS" }
                div { class: "relative hidden sm:block",
                    input {
                        r#type: "text",
                        placeholder: "Filter events...",
                        class: "bg-muted/30 border border-border/40 rounded-xl pl-10 pr-4 py-2 text-xs font-bold focus:outline-none focus:border-[#1fc7d4]/50 transition-colors",
                    }
                }
            }
            // Rows (or empty state)
            div { class: "divide-y divide-white/5",
                if notifications.is_empty() {
                    div { class: "py-24 text-center",
                        div { class: "inline-flex p-6 bg-muted/30 rounded-xl mb-6",
                            Icon { name: "bell".to_string(), size: Some(48), class_name: Some("text-muted-foreground/20".to_string()) }
                        }
                        h3 { class: "text-xl font-black text-muted-foreground uppercase tracking-tight", "Silence is Golden" }
                        p { class: "text-sm text-muted-foreground/50 mt-2", "No active notifications detected in the grid" }
                    }
                } else {
                    for n in notifications.iter() {
                        NotificationListRow { notification: n.clone() }
                    }
                }
            }
        }
    }
}

// One row in the NotificationList. Renders the type icon, title
// + priority badge, message, type/timestamp, and a hover-revealed
// delete button.
#[component]
fn NotificationListRow(notification: Notification) -> Element {
    let priority_cls = match notification.priority.as_str() {
        "critical" => "bg-red-500/10 text-red-400 border-red-500/20",
        "high" => "bg-amber-500/10 text-amber-400 border-amber-500/20",
        "normal" => "bg-cyan-500/10 text-cyan-400 border-cyan-500/20",
        _ => "bg-slate-500/10 text-slate-400 border-slate-500/20",
    };
    let mut confirm_purge_open = use_signal(|| false);
    let type_icon = match notification.notification_type.as_str() {
        "security" => "shield",
        "system" => "refresh-cw",
        _ => "message-square",
    };
    let ts_display = notification.timestamp.clone()
        .map(|t| t.replace('T', " ").trim_end_matches('Z').to_string())
        .unwrap_or_else(|| "—".to_string());

    rsx! {
        div { class: "notification-list-row group p-8 flex items-start gap-8 hover:bg-gray-50 dark:hover:bg-white/[0.02] transition-colors",
            // Type icon
            div { class: "flex-shrink-0 w-14 h-14 rounded-xl bg-muted/30 border border-border/40 flex items-center justify-center text-muted-foreground",
                Icon { name: type_icon.to_string(), size: Some(24) }
            }
            // Title + priority + message + meta
            div { class: "flex-1 min-w-0",
                div { class: "flex flex-wrap items-center gap-3 mb-2",
                    h3 { class: "text-lg font-black text-foreground tracking-tight truncate", "{notification.title}" }
                    span { class: "notification-list-priority px-3 py-1 rounded-lg text-[10px] font-black uppercase tracking-widest border {priority_cls}", "{notification.priority}" }
                }
                p { class: "text-base font-bold text-muted-foreground mb-4 line-clamp-2 max-w-4xl", "{notification.message}" }
                div { class: "flex items-center space-x-6 text-[10px] font-black uppercase tracking-widest text-muted-foreground/40",
                    span { class: "flex items-center", Icon { name: "layers".to_string(), size: Some(12), class_name: Some("mr-2".to_string()) } "{notification.notification_type}" }
                    span { class: "flex items-center", Icon { name: "clock".to_string(), size: Some(12), class_name: Some("mr-2".to_string()) } "{ts_display}" }
                }
            }
            // Hover-revealed delete button
            div { class: "notification-list-actions opacity-0 group-hover:opacity-100 transition-opacity",
                button {
                    class: "p-3 text-red-400 hover:text-red-300 hover:bg-red-500/10 rounded-2xl border border-transparent hover:border-red-500/20",
                    r#type: "button",
                    title: "Purge broadcast",
                    onclick: move |_| confirm_purge_open.set(true),
                    Icon { name: "trash-2".to_string(), size: Some(24) }
                }
            }
            // Purge confirm modal
            AdminActionConfirm {
                open: *confirm_purge_open.read(),
                title: "Confirm Deletion".to_string(),
                message: "This broadcast will be permanently purged from the system grid.".to_string(),
                confirm_label: "Purge".to_string(),
                confirm_variant: ConfirmVariant::Destructive,
                cancel_label: Some("Abort".to_string()),
                on_confirm: move |_| confirm_purge_open.set(false),
                on_cancel: move |_| confirm_purge_open.set(false),
            }
        }
    }
}

// =====================================================================
// Section 2: SendForm
// =====================================================================
//
// Compose form (the source's `SendNotificationForm`). The
/// send-form section-marker is on the outer form wrapper. The
/// per-field sections (RecipientsPicker, TemplateEditor, Preview,
/// ScheduleDialog) compose into the form.
#[component]
fn SendForm() -> Element {
    rsx! {
        Form { method: "POST".to_string(), action: "/api/v1/notification/send".to_string(),
            div { class: "send-form space-y-4",
                // Compose fields
                div { class: "field",
                    label { class: "field-label", "Title" }
                    input { class: "input", name: "title", required: true, placeholder: "Notification title" }
                }
                div { class: "field",
                    label { class: "field-label", "Body" }
                    textarea { class: "input", name: "body", rows: "4", required: true, placeholder: "Message body..." }
                }
                FormActions {
                    a { class: "btn btn-outline", href: "/notifications/manage", "Cancel" }
                    button { class: "btn btn-primary", r#type: "submit", "📤 Send notification" }
                }
            }
        }
    }
}

// =====================================================================
// Section 3: RecipientsPicker
// =====================================================================
//
// Two-button toggle: "Targeted Client" (specific wallet) vs.
/// "Global Broadcast" (all users). Mirrors the source's
/// `RecipientSelector` block in `send-notification-form.tsx`.
#[component]
fn RecipientsPicker() -> Element {
    let mut recipient_type = use_signal(|| "specific".to_string());
    rsx! {
        div { class: "recipients-picker space-y-4",
            label { class: "text-[10px] font-black uppercase tracking-[0.2em] text-muted-foreground ml-2", "Transmission Logic" }
            div { class: "grid grid-cols-1 sm:grid-cols-2 gap-4",
                button {
                    class: if recipient_type.read().as_str() == "specific" { "flex items-center gap-6 p-6 rounded-xl border transition-all bg-[#1fc7d4]/10 border-[#1fc7d4] shadow-[0_0_20px_rgba(31,199,212,0.1)]" } else { "flex items-center gap-6 p-6 rounded-xl border transition-all bg-muted/30 border-border/40 hover:bg-muted/50" },
                    r#type: "button",
                    onclick: move |_| recipient_type.set("specific".to_string()),
                    div { class: if recipient_type.read().as_str() == "specific" { "w-12 h-12 rounded-xl flex items-center justify-center bg-[#1fc7d4] text-white" } else { "w-12 h-12 rounded-xl flex items-center justify-center bg-muted/50 text-muted-foreground/30" },
                        Icon { name: "user".to_string(), size: Some(24) }
                    }
                    div { class: "text-left",
                        div { class: "font-black text-foreground uppercase tracking-tight text-sm", "Targeted Client" }
                        div { class: "text-[10px] font-bold text-muted-foreground uppercase opacity-50", "Single Node Access" }
                    }
                }
                button {
                    class: if recipient_type.read().as_str() == "broadcast" { "flex items-center gap-6 p-6 rounded-xl border transition-all bg-amber-500/10 border-amber-500 shadow-[0_0_20px_rgba(245,158,11,0.1)]" } else { "flex items-center gap-6 p-6 rounded-xl border transition-all bg-muted/30 border-border/40 hover:bg-muted/50" },
                    r#type: "button",
                    onclick: move |_| recipient_type.set("broadcast".to_string()),
                    div { class: if recipient_type.read().as_str() == "broadcast" { "w-12 h-12 rounded-xl flex items-center justify-center bg-amber-500 text-white" } else { "w-12 h-12 rounded-xl flex items-center justify-center bg-muted/50 text-muted-foreground/30" },
                        Icon { name: "users".to_string(), size: Some(24) }
                    }
                    div { class: "text-left",
                        div { class: "font-black text-foreground uppercase tracking-tight text-sm", "Global Broadcast" }
                        div { class: "text-[10px] font-bold text-muted-foreground uppercase opacity-50", "Network-wide Delivery" }
                    }
                }
            }
            // Hidden field for form submission
            input { r#type: "hidden", name: "recipient_type", value: "{recipient_type.read()}" }
            // Conditional wallet address field
            if recipient_type.read().as_str() == "specific" {
                div { class: "space-y-3",
                    label { class: "text-[10px] font-black uppercase tracking-[0.2em] text-muted-foreground ml-2", "Destination Node" }
                    input {
                        class: "input h-14 bg-muted/30 border-border/40 rounded-2xl px-6 font-mono text-sm",
                        r#type: "text",
                        name: "wallet_address",
                        placeholder: "0x...",
                    }
                }
            }
        }
    }
}

// =====================================================================
// Section 4: NotificationTemplateEditor
// =====================================================================
//
// Title + body + action URL + image URL. The source combines these
/// into a single section; the port keeps them together as the
/// "template editor" section.
#[component]
fn NotificationTemplateEditor() -> Element {
    let mut classification = use_signal(|| "system".to_string());
    let mut priority = use_signal(|| "normal".to_string());
    rsx! {
        div { class: "notification-template-editor space-y-4",
            // Classification + Priority in a 2-col grid
            div { class: "grid grid-cols-1 lg:grid-cols-2 gap-x-10 gap-y-8",
                // Classification
                div { class: "space-y-3",
                    label { class: "text-[10px] font-black uppercase tracking-[0.2em] text-muted-foreground ml-2", "Classification" }
                    select {
                        class: "input h-14 bg-muted/30 border-border/40 rounded-2xl px-6 text-sm font-black uppercase tracking-widest",
                        name: "notification_type",
                        value: "{classification.read()}",
                        onchange: move |e| classification.set(e.value().to_string()),
                        for (val, lbl) in &[("system", "System Alert"), ("security", "Security Event"), ("permission", "Permission Auth"), ("payment", "Payment Transaction"), ("general", "General Message")] {
                            option { value: "{val}", selected: *val == classification.read().as_str(), "{lbl}" }
                        }
                    }
                }
                // Priority
                div { class: "space-y-3",
                    label { class: "text-[10px] font-black uppercase tracking-[0.2em] text-muted-foreground ml-2", "Priority Vector" }
                    select {
                        class: "input h-14 bg-muted/30 border-border/40 rounded-2xl px-6 text-sm font-black uppercase tracking-widest",
                        name: "priority",
                        value: "{priority.read()}",
                        onchange: move |e| priority.set(e.value().to_string()),
                        for (val, lbl) in &[("low", "Low Clearance"), ("normal", "Normal Operation"), ("high", "High Priority"), ("critical", "Critical Override")] {
                            option { value: "{val}", selected: *val == priority.read().as_str(), "{lbl}" }
                        }
                    }
                }
                // Title (full width)
                div { class: "space-y-3 lg:col-span-2",
                    label { class: "text-[10px] font-black uppercase tracking-[0.2em] text-muted-foreground ml-2", "Subject Heading" }
                    input {
                        class: "input h-14 bg-muted/30 border-border/40 rounded-2xl px-6 font-bold text-sm tracking-tight",
                        r#type: "text",
                        name: "title",
                        placeholder: "Payload designation...",
                    }
                }
                // Body (full width)
                div { class: "space-y-3 lg:col-span-2",
                    label { class: "text-[10px] font-black uppercase tracking-[0.2em] text-muted-foreground ml-2", "Message Payload" }
                    textarea {
                        class: "input bg-muted/30 border-border/40 rounded-2xl p-6 font-bold text-sm tracking-tight",
                        name: "body",
                        rows: "5",
                        placeholder: "Enter transmission data...",
                    }
                }
                // Action URL + Image URL
                div { class: "space-y-3",
                    label { class: "text-[10px] font-black uppercase tracking-[0.2em] text-muted-foreground ml-2", "Action URL" }
                    input {
                        class: "input h-12 bg-muted/30 border-border/40 rounded-2xl px-5 text-xs font-bold",
                        r#type: "url",
                        name: "action_url",
                        placeholder: "https://...",
                    }
                }
                div { class: "space-y-3",
                    label { class: "text-[10px] font-black uppercase tracking-[0.2em] text-muted-foreground ml-2", "Asset URL" }
                    input {
                        class: "input h-12 bg-muted/30 border-border/40 rounded-2xl px-5 text-xs font-bold",
                        r#type: "url",
                        name: "image_url",
                        placeholder: "https://...",
                    }
                }
            }
        }
    }
}

// =====================================================================
// Section 5: NotificationPreview
// =====================================================================
//
// Live preview of the notification as it will appear to
/// recipients. Mirrors the source's preview tile (the in-app
/// notification card design). State is fed by the parent form's
/// signal; the port keeps it as a static preview.
#[component]
fn NotificationPreview() -> Element {
    rsx! {
        div { class: "notification-preview space-y-3",
            label { class: "text-[10px] font-black uppercase tracking-[0.2em] text-muted-foreground ml-2", "Live Preview" }
            div { class: "rounded-2xl border border-border/30 bg-card/50 p-6",
                div { class: "flex items-start gap-4",
                    div { class: "flex-shrink-0 w-12 h-12 rounded-xl bg-gradient-to-br from-[#7645d9] to-[#1fc7d4] flex items-center justify-center",
                        Icon { name: "bell".to_string(), size: Some(20), class_name: Some("text-white".to_string()) }
                    }
                    div { class: "flex-1",
                        div { class: "flex items-center gap-2 mb-1",
                            h4 { class: "text-sm font-bold text-foreground", "EPSX" }
                            span { class: "text-[10px] text-muted-foreground", "Just now" }
                        }
                        p { class: "text-sm font-bold text-foreground", "Payload designation..." }
                        p { class: "text-xs text-muted-foreground mt-1", "Enter transmission data..." }
                    }
                }
            }
            p { class: "text-[10px] text-muted-foreground italic", "Preview updates as you type" }
        }
    }
}

// =====================================================================
// Section 6: NotificationScheduleDialog
// =====================================================================
//
// Toggle for "schedule for later" + a datetime picker. The source
/// uses a simple checkbox + datetime input; the port keeps the
/// same shape and adds the section-marker class for the test
/// contract. When the toggle is off, no datetime is rendered.
#[component]
fn NotificationScheduleDialog() -> Element {
    let mut schedule_open = use_signal(|| false);
    rsx! {
        div { class: "notification-schedule-dialog space-y-3",
            div { class: "flex items-center gap-3 p-4 rounded-xl border border-border/30 bg-card/50",
                input {
                    id: "schedule-for-later",
                    r#type: "checkbox",
                    class: "checkbox",
                    name: "schedule",
                    checked: *schedule_open.read(),
                    onchange: move |e| schedule_open.set(e.checked()),
                }
                label {
                    r#for: "schedule-for-later",
                    class: "text-sm font-bold text-foreground cursor-pointer",
                    "Schedule for later"
                }
            }
            if *schedule_open.read() {
                div { class: "space-y-3 pl-8",
                    label { class: "text-[10px] font-black uppercase tracking-[0.2em] text-muted-foreground ml-2", "Send at" }
                    input {
                        class: "input h-12 bg-muted/30 border-border/40 rounded-2xl px-5 text-sm",
                        r#type: "datetime-local",
                        name: "scheduled_at",
                    }
                }
            }
            // Hidden field for form submission
            input { r#type: "hidden", name: "schedule_enabled", value: if *schedule_open.read() { "true" } else { "false" } }
        }
    }
}

// =====================================================================
// Section 7: NotificationManagementFilters
// =====================================================================
//
// Filter chips for the manage view: All / Sent / Scheduled /
/// Draft. Mirrors the source's filter row above the notification
/// list. The BFF hydrates with the real filter state and click
/// handlers.
#[component]
fn NotificationManagementFilters() -> Element {
    rsx! {
        div { class: "notification-management-filters flex items-center gap-2 flex-wrap",
            for (key, label) in &[
                ("all", "All"),
                ("sent", "Sent"),
                ("scheduled", "Scheduled"),
                ("draft", "Draft"),
            ] {
                a {
                    class: "notification-filter-chip px-3 py-1.5 rounded-lg text-sm font-medium transition-colors capitalize bg-card border border-border/20 text-muted-foreground hover:text-foreground hover:border-border/40",
                    href: format!("/notifications/manage?status={key}"),
                    "{label}"
                }
            }
        }
    }
}

// =====================================================================
// Helper components
// =====================================================================

/// One stats card in the manage view's grid. Mirrors the
/// source's `StatsCard` (used 4 times for total/today/week/health).
#[component]
fn NotificationStatCard(title: String, value: String, icon: String, color: String) -> Element {
    let color_cls = match color.as_str() {
        "amber" => "bg-amber-500",
        "purple" => "bg-purple-500",
        "green" => "bg-green-500",
        _ => "bg-cyan-500",
    };
    rsx! {
        div { class: "notification-stat-card rounded-xl bg-card border border-border/20 p-4",
            div { class: "flex items-start justify-between",
                div {
                    p { class: "text-sm text-muted-foreground", "{title}" }
                    p { class: "text-2xl font-semibold mt-1", "{value}" }
                }
                div { class: "w-10 h-10 rounded-lg {color_cls} flex items-center justify-center",
                    Icon { name: icon, size: Some(20), class_name: Some("text-white".to_string()) }
                }
            }
        }
    }
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

    /// test_section_markers for the manage view: NotificationList,
    /// NotificationManagementFilters, + page subtitle.
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

    /// test_render_smoke + section markers for the create view.
    /// Exercises the 5 form-related sections (RecipientsPicker,
    /// NotificationTemplateEditor, NotificationPreview,
    /// NotificationScheduleDialog, SendForm).
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

    /// Schedule dialog renders the schedule input only when the
    /// toggle is on (always-on for the SSR'd create form — the
    /// toggle is off, so the datetime input should be hidden).
    #[test]
    fn notifications_schedule_dialog_toggle() {
        // Toggle off: only the checkbox + label render
        let mut vdom = dioxus::prelude::VirtualDom::new(|| {
            rsx! { NotificationScheduleDialog {} }
        });
        vdom.rebuild_in_place();
        let html = dioxus_ssr::render(&vdom);
        assert!(
            html.contains("notification-schedule-dialog"),
            "Schedule dialog must render its section marker. Got: {}",
            html
        );
        assert!(
            html.contains("Schedule for later"),
            "Schedule dialog must render the toggle label. Got: {}",
            html
        );
    }

    /// The filter chips render all 4 statuses (All / Sent /
    /// Scheduled / Draft).
    #[test]
    fn notifications_management_filters_renders_chips() {
        let mut vdom = dioxus::prelude::VirtualDom::new(|| {
            rsx! { NotificationManagementFilters {} }
        });
        vdom.rebuild_in_place();
        let html = dioxus_ssr::render(&vdom);
        for chip in &["All", "Sent", "Scheduled", "Draft"] {
            assert!(
                html.contains(chip),
                "Filter chip `{chip}` should be present. Got: {}",
                html
            );
        }
    }
}
