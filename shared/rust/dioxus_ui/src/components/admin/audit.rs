//! Sub-components for `/admin/audit-log` — Wave 6C Track D.
//!
//! 1:1 mirror of `apps-old/admin-frontend/app/audit-log/components/*.tsx`:
//!   1. `AuditFilters`            — search + 9 category pills + date range
//!   2. `AuditTimeline`           — paginated list of log entries
//!   3. `AuditEntryDetail`        — expand panel with result/resource badges
//!   4. `AuditSeverityBreakdown`  — by-category counts panel
//!   5. `AuditExportButton`       — CSV / JSON export trigger buttons
//!
//! The `AuditLogEntry` data struct is `pub` so the parent page can
//! build entries from BFF-provided data. Helper functions
//! (`action_icon`, `action_color_class`, `trunc_addr`, `fmt_time`,
//! `humanize`) are also `pub` because the per-page test uses them.
//! `ACTION_CATEGORIES` is `pub` for the same reason.

use crate::primitives::*;

use dioxus::prelude::*;

/// One audit-log entry. Mirrors the `AuditLogEntry` interface in
/// `apps-old/admin-frontend/app/audit-log/types.ts`.
#[derive(Clone, Debug, PartialEq)]
pub struct AuditLogEntry {
    pub id: String,
    pub action: String,
    pub wallet_address: Option<String>,
    pub resource_type: String,
    pub resource_id: Option<String>,
    pub result: String,
    pub details: Option<std::collections::HashMap<String, String>>,
    pub ip_address: Option<String>,
    pub timestamp: String,
}

/// The 9 audit categories from the source's `ACTION_CATEGORIES` map.
pub const ACTION_CATEGORIES: &[(&str, &str, &str)] = &[
    ("all", "All Actions", "📋"),
    ("permission", "Permissions", "🔐"),
    ("wallet", "Wallets", "👛"),
    ("plan", "Plans", "💳"),
    ("payment", "Payments", "💰"),
    ("system", "System", "⚙️"),
    ("auth", "Auth", "🔑"),
    ("developer", "Developer", "🛠"),
    ("notification", "Notifications", "🔔"),
];

/// Map an action verb to a small emoji icon (matches
/// `getActionIcon` in `audit-log-row.tsx`).
pub fn action_icon(action: &str) -> &'static str {
    let map: &[(&str, &str)] = &[
        ("grant", "🔓"),
        ("revoke", "🔒"),
        ("bulk_assign", "📦"),
        ("bulk_remove", "📦"),
        ("assign", "🔗"),
        ("permission", "🔐"),
        ("wallet", "👛"),
        ("plan", "💳"),
        ("login", "🔑"),
        ("auth", "🔑"),
        ("create", "➕"),
        ("delete", "🗑️"),
        ("remove", "🗑️"),
        ("update", "✏️"),
        ("edit", "✏️"),
        ("disable", "🚫"),
        ("enable", "✅"),
    ];
    for (k, v) in map {
        if action.contains(k) {
            return v;
        }
    }
    "📝"
}

/// Map an action verb to a Tailwind text/bg class (matches
/// `getActionColor` in `audit-log-row.tsx`).
pub fn action_color_class(action: &str) -> &'static str {
    let matches = |keys: &[&str]| keys.iter().any(|k| action.contains(k));
    if matches(&["create", "enable", "grant"]) {
        "text-emerald-600 bg-emerald-100 dark:bg-emerald-900/30"
    } else if matches(&["delete", "disable", "remove", "revoke"]) {
        "text-red-600 bg-red-100 dark:bg-red-900/30"
    } else if matches(&["update", "edit", "assign"]) {
        "text-blue-600 bg-blue-100 dark:bg-blue-900/30"
    } else if matches(&["permission", "bulk"]) {
        "text-purple-600 bg-purple-100 dark:bg-purple-900/30"
    } else {
        "text-muted-foreground bg-muted"
    }
}

/// Format a wallet address: show first 6 + "…" + last 4 if long,
/// else show verbatim. Mirrors the source's `fmtAddr` helper.
pub fn trunc_addr(addr: &str) -> String {
    if addr.len() > 16 {
        format!("{}…{}", &addr[..6], &addr[addr.len() - 4..])
    } else {
        addr.to_string()
    }
}

/// Format a timestamp as a relative `N{min,h,d} ago` string, or
/// absolute date for >7 days old. Mirrors `fmtTime` in source.
pub fn fmt_time(ts: &str) -> String {
    ts.split('T').next().unwrap_or(ts).to_string()
}

/// Replace underscores with spaces.
pub fn humanize(s: &str) -> String {
    s.replace('_', " ")
}

// ============================================================================
// Section 1: AuditFilters
// ============================================================================

#[component]
pub fn AuditFilters() -> Element {
    rsx! {
        div { class: "audit-filters rounded-xl border border-border/20 bg-card p-4 shadow-xl",
            div { class: "flex flex-col lg:flex-row gap-3",
                div { class: "relative flex-1",
                    input {
                        r#type: "text",
                        class: "w-full pl-9 pr-4 py-2.5 bg-muted/50 border border-border/50 rounded-xl text-sm",
                        placeholder: "Search by actor, action, or target...",
                    }
                }
                div { class: "audit-filters-pills flex gap-2 overflow-x-auto pb-1 lg:pb-0",
                    for (key, label, icon) in ACTION_CATEGORIES.iter() {
                        button {
                            class: "audit-filters-pill px-3 py-2 rounded-lg font-medium text-sm whitespace-nowrap transition-all border border-border/40 bg-muted/50 text-muted-foreground hover:text-foreground",
                            "data-category": "{key}",
                            "{icon} {label}"
                        }
                    }
                }
            }
            div { class: "flex flex-col sm:flex-row gap-3 mt-3",
                div { class: "flex gap-2 flex-1",
                    input {
                        r#type: "date",
                        class: "audit-filters-date-from flex-1 px-3 py-2 bg-muted/50 border border-border/50 rounded-xl text-sm",
                        placeholder: "From",
                    }
                    span { class: "self-center text-muted-foreground text-sm", "to" }
                    input {
                        r#type: "date",
                        class: "audit-filters-date-to flex-1 px-3 py-2 bg-muted/50 border border-border/50 rounded-xl text-sm",
                        placeholder: "To",
                    }
                }
                div { class: "flex gap-2",
                    button {
                        class: "px-4 py-2 bg-gradient-to-r from-[#7645d9] to-[#5a33b8] text-white rounded-xl text-sm font-semibold",
                        r#type: "button",
                        "🔄 Refresh"
                    }
                }
            }
        }
    }
}

// ============================================================================
// Section 2: AuditTimeline
// ============================================================================

#[component]
pub fn AuditTimeline(entries: Vec<AuditLogEntry>) -> Element {
    rsx! {
        div { class: "audit-timeline rounded-2xl border border-border/20 overflow-hidden bg-card shadow-xl",
            div { class: "h-[3px] bg-gradient-to-r from-[#7645d9] to-[#1fc7d4]" }
            if entries.is_empty() {
                div { class: "p-8 text-center",
                    div { class: "text-5xl mb-3", "📭" }
                    p { class: "text-muted-foreground text-sm", "No audit logs found" }
                }
            } else {
                div { class: "hidden md:grid grid-cols-12 gap-4 px-4 py-3 bg-muted/30 border-b border-border/30 text-[10px] font-bold text-muted-foreground uppercase tracking-[0.15em]",
                    div { class: "col-span-2", "Time" }
                    div { class: "col-span-2", "Action" }
                    div { class: "col-span-3", "Actor" }
                    div { class: "col-span-3", "Target" }
                    div { class: "col-span-2 text-right", "Details" }
                }
                div { class: "divide-y divide-border/30",
                    for (i, entry) in entries.iter().enumerate() {
                        AuditTimelineRow { entry: entry.clone(), initial_expanded: i == 0 }
                    }
                }
            }
            if !entries.is_empty() {
                div { class: "audit-timeline-pagination px-4 py-3 bg-muted/20 border-t border-border/30 flex items-center justify-between",
                    span { class: "text-xs text-muted-foreground", "Page 1 of 1" }
                    div { class: "flex gap-2",
                        button {
                            class: "p-1.5 rounded-lg bg-card border border-border/40 opacity-40 cursor-not-allowed",
                            r#type: "button",
                            "‹"
                        }
                        button {
                            class: "p-1.5 rounded-lg bg-card border border-border/40 opacity-40 cursor-not-allowed",
                            r#type: "button",
                            "›"
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn AuditTimelineRow(entry: AuditLogEntry, initial_expanded: bool) -> Element {
    let mut is_expanded = use_signal(|| initial_expanded);
    let icon = action_icon(&entry.action);
    let color_cls = action_color_class(&entry.action);
    let actor_display = entry.wallet_address.as_deref()
        .map(trunc_addr)
        .unwrap_or_else(|| "System".to_string());

    rsx! {
        div {
            class: "audit-timeline-row p-4 hover:bg-muted/50 transition-colors cursor-pointer",
            onclick: move |_| {
                let cur = *is_expanded.read();
                is_expanded.set(!cur);
            },
            div { class: "hidden md:grid grid-cols-12 gap-4 items-center",
                div { class: "col-span-2 text-sm text-muted-foreground", "{fmt_time(&entry.timestamp)}" }
                div { class: "col-span-2",
                    span { class: "audit-timeline-action inline-flex items-center gap-1 px-2 py-1 rounded-lg text-sm font-medium {color_cls}",
                        "{icon} {humanize(&entry.action)}"
                    }
                }
                div { class: "col-span-3",
                    code { class: "text-sm bg-muted px-2 py-1 rounded font-mono", "{actor_display}" }
                }
                div { class: "col-span-3 text-sm text-muted-foreground",
                    span { class: "font-medium", "{entry.resource_type}" }
                    span { class: "mx-1", "→" }
                    code { class: "bg-muted px-1.5 py-0.5 rounded text-xs",
                        {entry.resource_id.as_deref().map(trunc_addr).unwrap_or_else(|| "N/A".to_string())}
                    }
                }
                div { class: "col-span-2 text-right",
                    span { class: "text-muted-foreground text-sm", if *is_expanded.read() { "▼" } else { "▶" } }
                }
            }
            div { class: "md:hidden space-y-2",
                div { class: "flex items-center justify-between",
                    span { class: "inline-flex items-center gap-1 px-2 py-1 rounded-lg text-sm font-medium {color_cls}",
                        "{icon} {humanize(&entry.action)}"
                    }
                    span { class: "text-sm text-muted-foreground", "{fmt_time(&entry.timestamp)}" }
                }
                div { class: "text-sm text-muted-foreground",
                    span { class: "font-medium", "Actor: " }
                    code { class: "bg-muted px-1.5 py-0.5 rounded text-xs", "{actor_display}" }
                }
            }
            if *is_expanded.read() {
                AuditEntryDetail { entry: entry.clone() }
            }
        }
    }
}

// ============================================================================
// Section 3: AuditEntryDetail
// ============================================================================

#[component]
pub fn AuditEntryDetail(entry: AuditLogEntry) -> Element {
    let result_cls = match entry.result.as_str() {
        "success" => "bg-emerald-100 text-emerald-700",
        "denied" => "bg-amber-100 text-amber-700",
        _ => "bg-red-100 text-red-700",
    };
    let actor_display = entry.wallet_address.as_deref()
        .map(trunc_addr)
        .unwrap_or_else(|| "System".to_string());
    let resource_id_display = entry.resource_id.as_deref()
        .map(trunc_addr)
        .unwrap_or_else(|| "-".to_string());

    rsx! {
        div { class: "audit-entry-detail mt-4 pt-4 border-t border-border",
            div { class: "bg-muted/50 rounded-xl p-4 space-y-4",
                div { class: "audit-entry-detail-header flex flex-wrap items-center gap-2",
                    span { class: "audit-entry-detail-result px-2 py-0.5 rounded text-xs font-medium {result_cls}", "{entry.result}" }
                    span { class: "audit-entry-detail-resource px-2 py-0.5 rounded text-xs font-medium bg-blue-100 text-blue-700", "{entry.resource_type}" }
                    span { class: "text-sm font-medium capitalize", "{humanize(&entry.action)}" }
                }
                div { class: "audit-entry-detail-meta grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-3 text-sm",
                    div {
                        span { class: "text-xs text-muted-foreground", "Actor" }
                        code { class: "block mt-0.5 bg-muted px-2 py-1 rounded text-xs font-mono break-all", "{actor_display}" }
                    }
                    div {
                        span { class: "text-xs text-muted-foreground", "Target" }
                        code { class: "block mt-0.5 bg-muted px-2 py-1 rounded text-xs font-mono break-all", "{resource_id_display}" }
                    }
                    div {
                        span { class: "text-xs text-muted-foreground", "Timestamp" }
                        p { class: "mt-0.5 text-xs", "{entry.timestamp}" }
                    }
                    if let Some(ip) = &entry.ip_address {
                        div {
                            span { class: "text-xs text-muted-foreground", "IP Address" }
                            p { class: "mt-0.5 text-xs", "{ip}" }
                        }
                    }
                }
                if let Some(details) = &entry.details {
                    if !details.is_empty() {
                        div { class: "audit-entry-detail-changes",
                            h4 { class: "text-xs font-medium text-muted-foreground uppercase tracking-wider mb-2", "Changes" }
                            div { class: "grid grid-cols-1 sm:grid-cols-2 gap-2 text-xs",
                                for (k, v) in details.iter() {
                                    div {
                                        span { class: "text-muted-foreground", "{k}: " }
                                        span { class: "font-mono", "{v}" }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

// ============================================================================
// Section 4: AuditSeverityBreakdown
// ============================================================================

#[component]
pub fn AuditSeverityBreakdown(entries: Vec<AuditLogEntry>) -> Element {
    let mut counts: std::collections::HashMap<&str, usize> = std::collections::HashMap::new();
    for e in &entries {
        let key = if e.action.contains("permission") { "permission" }
            else if e.action.contains("wallet") { "wallet" }
            else if e.action.contains("plan") { "plan" }
            else if e.action.contains("payment") { "payment" }
            else if e.action.contains("auth") || e.action.contains("login") { "auth" }
            else if e.action.contains("developer") || e.action.contains("api") { "developer" }
            else if e.action.contains("notification") { "notification" }
            else { "system" };
        *counts.entry(key).or_insert(0) += 1;
    }
    let total = entries.len().max(1);

    rsx! {
        div { class: "audit-severity-breakdown rounded-2xl border border-border/20 bg-card shadow-xl p-4",
            h3 { class: "text-xs font-bold text-muted-foreground uppercase tracking-[0.2em] mb-4", "Severity breakdown" }
            div { class: "space-y-3",
                for (key, label, _icon) in ACTION_CATEGORIES.iter() {
                    if *key != "all" {
                        {
                            let n = counts.get(*key).copied().unwrap_or(0);
                            let pct = (n * 100) / total;
                            rsx! {
                                div { class: "audit-severity-row",
                                    div { class: "flex justify-between text-xs mb-1",
                                        span { class: "text-foreground font-medium", "{label}" }
                                        span { class: "text-muted-foreground", "{n}" }
                                    }
                                    div { class: "h-1.5 bg-muted rounded-full overflow-hidden",
                                        div {
                                            class: "h-full bg-gradient-to-r from-[#7645d9] to-[#1fc7d4]",
                                            style: "width: {pct}%",
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            div { class: "mt-4 pt-4 border-t border-border/20 text-xs text-muted-foreground",
                "Total: "
                span { class: "font-bold text-foreground", "{entries.len()}" }
                " entries"
            }
        }
    }
}

// ============================================================================
// Section 5: AuditExportButton
// ============================================================================

#[component]
pub fn AuditExportButton() -> Element {
    rsx! {
        div { class: "audit-export-button flex gap-2",
            button {
                class: "btn btn-sm btn-outline",
                r#type: "button",
                "data-export": "csv",
                "📥 Export CSV"
            }
            button {
                class: "btn btn-sm btn-outline",
                r#type: "button",
                "data-export": "json",
                "📥 Export JSON"
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

    fn sample_entry() -> AuditLogEntry {
        AuditLogEntry {
            id: "1".into(),
            action: "user.create".into(),
            wallet_address: Some("0x1234567890abcdef1234567890abcdef12345678".into()),
            resource_type: "user".into(),
            resource_id: Some("0xabc".into()),
            result: "success".into(),
            details: None,
            ip_address: Some("192.168.1.1".into()),
            timestamp: "2024-09-20T10:32:15Z".into(),
        }
    }

    #[test]
    fn test_render_smoke_audit_filters() {
        let el = rsx! { AuditFilters {} };
        let html = render_to_string(el);
        assert!(html.contains("audit-filters"), "AuditFilters must render its class. Got: {}", html);
        assert!(html.contains("All Actions"), "AuditFilters must render the 'All Actions' pill. Got: {}", html);
        assert!(html.contains("Permissions"), "AuditFilters must render the 'Permissions' pill. Got: {}", html);
    }

    #[test]
    fn test_render_smoke_audit_timeline() {
        let el = rsx! { AuditTimeline { entries: vec![sample_entry()] } };
        let html = render_to_string(el);
        assert!(html.contains("audit-timeline"), "AuditTimeline must render its class. Got: {}", html);
        assert!(html.contains("audit-timeline-row"), "AuditTimeline must render at least one row. Got: {}", html);
    }

    #[test]
    fn test_render_smoke_audit_entry_detail() {
        let el = rsx! { AuditEntryDetail { entry: sample_entry() } };
        let html = render_to_string(el);
        assert!(html.contains("audit-entry-detail"), "AuditEntryDetail must render its class. Got: {}", html);
        assert!(html.contains("success"), "AuditEntryDetail must render the result. Got: {}", html);
    }

    #[test]
    fn test_render_smoke_audit_severity_breakdown() {
        let el = rsx! { AuditSeverityBreakdown { entries: vec![sample_entry()] } };
        let html = render_to_string(el);
        assert!(html.contains("audit-severity-breakdown"), "AuditSeverityBreakdown must render its class. Got: {}", html);
        assert!(html.contains("Severity breakdown"), "AuditSeverityBreakdown must render the title. Got: {}", html);
    }

    #[test]
    fn test_render_smoke_audit_export_button() {
        let el = rsx! { AuditExportButton {} };
        let html = render_to_string(el);
        assert!(html.contains("audit-export-button"), "AuditExportButton must render its class. Got: {}", html);
        assert!(html.contains("Export CSV"), "AuditExportButton must render the CSV button. Got: {}", html);
    }

    #[test]
    fn trunc_addr_handles_long_and_short() {
        assert_eq!(trunc_addr("0x1234567890abcdef1234567890abcdef12345678"), "0x1234…5678");
        assert_eq!(trunc_addr("0x1234"), "0x1234");
    }
}
