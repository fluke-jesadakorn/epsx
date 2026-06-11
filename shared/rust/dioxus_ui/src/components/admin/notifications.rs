//! Sub-components for `/admin/notifications/manage` + `/create` — Wave 6C Track D.
//!
//! 1:1 mirror of `apps-old/admin-frontend/components/notifications/*.tsx`:
//!   1. `NotificationList`             — list of sent/scheduled/draft
//!   2. `SendForm`                     — compose form footer
//!   3. `RecipientsPicker`             — targeted client vs. global broadcast
//!   4. `NotificationTemplateEditor`   — title + body + action URL
//!   5. `NotificationPreview`          — live preview pane
//!   6. `NotificationScheduleDialog`   — schedule-for-later
//!   7. `NotificationManagementFilters`— filter chips
//!
//! The `Notification` + `NotificationStats` data structs are `pub` so
//! the parent page can build lists from BFF data.

use crate::primitives::*;
use crate::feedback::admin_action_confirm::{AdminActionConfirm, ConfirmVariant};

use dioxus::prelude::*;

// ============================================================================
// Data
// ============================================================================

/// Notification data. Mirrors the `Notification` interface in the
/// source's `notification-management.tsx`.
#[derive(Clone, Debug, PartialEq)]
pub struct Notification {
    pub id: String,
    pub title: String,
    pub message: String,
    pub priority: String,
    pub notification_type: String,
    pub timestamp: Option<String>,
}

/// Notification stats.
#[derive(Clone, Debug, PartialEq)]
pub struct NotificationStats {
    pub total: i32,
    pub sent_today: i32,
    pub sent_this_week: i32,
}

// ============================================================================
// Section 1: NotificationList
// ============================================================================

#[component]
pub fn NotificationList(notifications: Vec<Notification>) -> Element {
    rsx! {
        div { class: "notification-list mt-6 rounded-2xl border border-border/20 overflow-hidden bg-card shadow-xl",
            div { class: "h-[3px] bg-gradient-to-r from-[#ffb237] to-[#7645d9]" }
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
            div { class: "flex-shrink-0 w-14 h-14 rounded-xl bg-muted/30 border border-border/40 flex items-center justify-center text-muted-foreground",
                Icon { name: type_icon.to_string(), size: Some(24) }
            }
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
            div { class: "notification-list-actions opacity-0 group-hover:opacity-100 transition-opacity",
                button {
                    class: "p-3 text-red-400 hover:text-red-300 hover:bg-red-500/10 rounded-2xl border border-transparent hover:border-red-500/20",
                    r#type: "button",
                    title: "Purge broadcast",
                    onclick: move |_| confirm_purge_open.set(true),
                    Icon { name: "trash-2".to_string(), size: Some(24) }
                }
            }
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

// ============================================================================
// Section 2: SendForm
// ============================================================================

#[component]
pub fn SendForm() -> Element {
    rsx! {
        Form { method: "POST".to_string(), action: "/api/v1/notification/send".to_string(),
            div { class: "send-form space-y-4",
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

// ============================================================================
// Section 3: RecipientsPicker
// ============================================================================

#[component]
pub fn RecipientsPicker() -> Element {
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
            input { r#type: "hidden", name: "recipient_type", value: "{recipient_type.read()}" }
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

// ============================================================================
// Section 4: NotificationTemplateEditor
// ============================================================================

#[component]
pub fn NotificationTemplateEditor() -> Element {
    let mut classification = use_signal(|| "system".to_string());
    let mut priority = use_signal(|| "normal".to_string());
    rsx! {
        div { class: "notification-template-editor space-y-4",
            div { class: "grid grid-cols-1 lg:grid-cols-2 gap-x-10 gap-y-8",
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
                div { class: "space-y-3 lg:col-span-2",
                    label { class: "text-[10px] font-black uppercase tracking-[0.2em] text-muted-foreground ml-2", "Subject Heading" }
                    input {
                        class: "input h-14 bg-muted/30 border-border/40 rounded-2xl px-6 font-bold text-sm tracking-tight",
                        r#type: "text",
                        name: "title",
                        placeholder: "Payload designation...",
                    }
                }
                div { class: "space-y-3 lg:col-span-2",
                    label { class: "text-[10px] font-black uppercase tracking-[0.2em] text-muted-foreground ml-2", "Message Payload" }
                    textarea {
                        class: "input bg-muted/30 border-border/40 rounded-2xl p-6 font-bold text-sm tracking-tight",
                        name: "body",
                        rows: "5",
                        placeholder: "Enter transmission data...",
                    }
                }
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

// ============================================================================
// Section 5: NotificationPreview
// ============================================================================

#[component]
pub fn NotificationPreview() -> Element {
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

// ============================================================================
// Section 6: NotificationScheduleDialog
// ============================================================================

#[component]
pub fn NotificationScheduleDialog() -> Element {
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
            input { r#type: "hidden", name: "schedule_enabled", value: if *schedule_open.read() { "true" } else { "false" } }
        }
    }
}

// ============================================================================
// Section 7: NotificationManagementFilters
// ============================================================================

#[component]
pub fn NotificationManagementFilters() -> Element {
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

// ============================================================================
// Helper components
// ============================================================================

#[component]
pub fn NotificationStatCard(title: String, value: String, icon: String, color: String) -> Element {
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

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn render_to_string(el: Element) -> String {
        dioxus_ssr::render_element(el)
    }

    fn sample_notification() -> Notification {
        Notification {
            id: "n1".into(),
            title: "Welcome".into(),
            message: "Body text".into(),
            priority: "normal".into(),
            notification_type: "general".into(),
            timestamp: Some("2024-09-20T10:32:15Z".into()),
        }
    }

    #[test]
    fn test_render_smoke_notification_list() {
        let el = rsx! { NotificationList { notifications: vec![sample_notification()] } };
        let html = render_to_string(el);
        assert!(html.contains("notification-list"), "NotificationList must render its class. Got: {}", html);
        assert!(html.contains("Welcome"), "NotificationList must render the notification title. Got: {}", html);
    }

    #[test]
    fn test_render_smoke_send_form() {
        let el = rsx! { SendForm {} };
        let html = render_to_string(el);
        assert!(html.contains("send-form"), "SendForm must render its class. Got: {}", html);
        assert!(html.contains("Notification title"), "SendForm must render the title input placeholder. Got: {}", html);
    }

    #[test]
    fn test_render_smoke_recipients_picker() {
        let el = rsx! { RecipientsPicker {} };
        let html = render_to_string(el);
        assert!(html.contains("recipients-picker"), "RecipientsPicker must render its class. Got: {}", html);
        assert!(html.contains("Targeted Client"), "RecipientsPicker must render the targeted client option. Got: {}", html);
    }

    #[test]
    fn test_render_smoke_notification_template_editor() {
        let el = rsx! { NotificationTemplateEditor {} };
        let html = render_to_string(el);
        assert!(html.contains("notification-template-editor"), "NotificationTemplateEditor must render its class. Got: {}", html);
        assert!(html.contains("Classification"), "NotificationTemplateEditor must render the classification label. Got: {}", html);
    }

    #[test]
    fn test_render_smoke_notification_preview() {
        let el = rsx! { NotificationPreview {} };
        let html = render_to_string(el);
        assert!(html.contains("notification-preview"), "NotificationPreview must render its class. Got: {}", html);
        assert!(html.contains("Live Preview"), "NotificationPreview must render the preview label. Got: {}", html);
    }

    #[test]
    fn test_render_smoke_notification_schedule_dialog() {
        let el = rsx! { NotificationScheduleDialog {} };
        let html = render_to_string(el);
        assert!(html.contains("notification-schedule-dialog"), "NotificationScheduleDialog must render its class. Got: {}", html);
        assert!(html.contains("Schedule for later"), "NotificationScheduleDialog must render the toggle label. Got: {}", html);
    }

    #[test]
    fn test_render_smoke_notification_management_filters() {
        let el = rsx! { NotificationManagementFilters {} };
        let html = render_to_string(el);
        assert!(html.contains("notification-management-filters"), "NotificationManagementFilters must render its class. Got: {}", html);
        for chip in &["All", "Sent", "Scheduled", "Draft"] {
            assert!(html.contains(chip), "Filter chip `{chip}` should be present. Got: {}", html);
        }
    }
}
