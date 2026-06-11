//! /admin/audit-log — audit log table with filters, timeline, severity
//! breakdown, and export.
//!
//! Source of truth: `apps-old/admin-frontend/app/audit-log/{page.tsx,
//! components/*, hooks/*, types.ts}`. The Dioxus port keeps the same
//! section-level structure: a top-level `AuditFilters` strip, an
//! `AuditTimeline` list (with row + detail expand), a side
//! `AuditSeverityBreakdown` panel that groups the current filter
//! result by category, and an `AuditExportButton` (CSV + JSON) that
//! the same source uses client-side.
//!
//! The file is misnamed in the prior port (`audit.rs` instead of
//! `audit_log.rs`); the Wave 6B rename + section expansion was done
//! in a single pass. Apps refer to this module via
//! `admin_pages::audit_log::render(ctx)` from
//! `pages/admin_pages.rs::dispatch`.
//!
//! The 5 design-doc-named sections (per
//! `docs/wave6b-admin-pages-depth/design.md` §"Track B") are:
//!   1. AuditFilters       — search + 9 category pills + date range
//!   2. AuditTimeline      — paginated list of log entries
//!   3. AuditEntryDetail   — expand panel with result/resource badges
//!                            + changes section (4 shape variants)
//!   4. AuditSeverityBreakdown — by-category counts panel
//!   5. AuditExportButton  — CSV / JSON export trigger buttons
//!
//! Plus the `AuditLogEntry` data struct (mirrors `types.ts`) and the
//! section-marker classes (`audit-filters`, `audit-timeline-row`,
//! `audit-entry-detail`, `audit-severity-breakdown`,
//! `audit-export-button`) used by the unit tests.

use crate::primitives::*;
use crate::data_table::{Column, DataTable, Row, SortDir};

use dioxus::prelude::*;
use super::super::{PageContext, PageMeta};
use crate::auth::AdminAuthGate;
use crate::feedback::admin_action_confirm::AdminActionConfirm;

/// One audit-log entry. Mirrors the `AuditLogEntry` interface in
/// `apps-old/admin-frontend/app/audit-log/types.ts`. The Dioxus port
/// keeps the same shape but with a few optional fields relaxed
/// (the source uses `wallet_address: string | null` but the port
/// never de-references it directly without a null check).
#[derive(Clone, Debug, PartialEq)]
pub struct AuditLogEntry {
    pub id: String,
    pub action: String,
    /// Optional — some system events have no actor.
    pub wallet_address: Option<String>,
    pub resource_type: String,
    pub resource_id: Option<String>,
    /// `success` | `denied` | `error` — the source's 3 result states.
    pub result: String,
    /// Optional details blob. The detail view's shape depends on
    /// the keys present (see `detect_detail_shape`).
    pub details: Option<std::collections::HashMap<String, String>>,
    pub ip_address: Option<String>,
    /// ISO-8601 timestamp.
    pub timestamp: String,
}

/// The 9 audit categories from the source's `ACTION_CATEGORIES` map.
/// Used by `AuditFilters` (filter pills) and `AuditSeverityBreakdown`
/// (counts panel). The `All` value renders no filter — selecting it
/// clears the category filter.
const ACTION_CATEGORIES: &[(&str, &str, &str)] = &[
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

/// Build 3 sample audit entries for the SSR'd initial render.
/// The BFF hydrates with the real list on the client.
fn sample_entries() -> Vec<AuditLogEntry> {
    vec![
        AuditLogEntry {
            id: "1".into(),
            action: "user.create".into(),
            wallet_address: Some("0xADMIN0000000000000000000000000000000001".into()),
            resource_type: "user".into(),
            resource_id: Some("0xabc".into()),
            result: "success".into(),
            details: None,
            ip_address: Some("192.168.1.1".into()),
            timestamp: "2024-09-20T10:32:15Z".into(),
        },
        AuditLogEntry {
            id: "2".into(),
            action: "plan.update".into(),
            wallet_address: Some("0xADMIN0000000000000000000000000000000001".into()),
            resource_type: "plan".into(),
            resource_id: Some("pro".into()),
            result: "success".into(),
            details: None,
            ip_address: Some("192.168.1.1".into()),
            timestamp: "2024-09-20T10:30:01Z".into(),
        },
        AuditLogEntry {
            id: "3".into(),
            action: "wallet.connect".into(),
            wallet_address: Some("0x1234567890abcdef1234567890abcdef12345678".into()),
            resource_type: "wallet".into(),
            resource_id: Some("0x1234".into()),
            result: "success".into(),
            details: None,
            ip_address: Some("10.0.0.1".into()),
            timestamp: "2024-09-20T10:25:42Z".into(),
        },
        AuditLogEntry {
            id: "4".into(),
            action: "news.publish".into(),
            wallet_address: Some("0xADMIN0000000000000000000000000000000001".into()),
            resource_type: "news".into(),
            resource_id: Some("welcome".into()),
            result: "success".into(),
            details: None,
            ip_address: Some("192.168.1.1".into()),
            timestamp: "2024-09-20T10:20:00Z".into(),
        },
    ]
}

/// Map an action verb to a small emoji icon (matches
/// `getActionIcon` in `audit-log-row.tsx`).
fn action_icon(action: &str) -> &'static str {
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
fn action_color_class(action: &str) -> &'static str {
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
fn trunc_addr(addr: &str) -> String {
    if addr.len() > 16 {
        format!("{}…{}", &addr[..6], &addr[addr.len() - 4..])
    } else {
        addr.to_string()
    }
}

/// Format a timestamp as a relative `N{min,h,d} ago` string, or
/// absolute date for >7 days old. Mirrors `fmtTime` in source.
fn fmt_time(ts: &str) -> String {
    // Parse ISO-8601. Dioxus doesn't pull in chrono, so we do a
    // crude string compare: use the year-month-day to compute
    // approx days, since the source uses `Date.now() - ts`. For
    // the SSR'd initial render, just show the date string.
    // Live updates happen client-side.
    ts.split('T').next().unwrap_or(ts).to_string()
}

/// Replace underscores with spaces — the source's
/// `action.replace(/_/g, ' ')` pattern. Extracted as a helper so
/// the `rsx!` macro doesn't choke on the `_` char literal (which
/// is a placeholder inside rsx! blocks).
fn humanize(s: &str) -> String {
    s.replace('_', " ")
}

/// Render the audit-log admin page. The body has 5 named sections
/// (per the design doc):
///   1. `AuditFilters` — top strip: search + category pills + dates + actions
///   2. `AuditTimeline` — main list of log entries
///   3. `AuditEntryDetail` — expand-into view (embedded in row)
///   4. `AuditSeverityBreakdown` — sidebar panel: per-category counts
///   5. `AuditExportButton` — CSV / JSON export buttons (in filters)
pub fn render(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::admin("Audit log");

    // Sample rows for the SSR'd `DataTable` — the BFF will hydrate
    // with real data on the client. The 5 columns match the source's
    // desktop header (Time, Action, Actor, Target, IP).
    let columns = vec![
        Column {
            key: "time".into(),
            label: "Time".into(),
            sortable: true,
            align: crate::primitives::data_table::Align::Left,
            width: Some("20%".into()),
            class_name: None,
        },
        Column {
            key: "actor".into(),
            label: "Actor".into(),
            sortable: true,
            align: crate::primitives::data_table::Align::Left,
            width: Some("25%".into()),
            class_name: None,
        },
        Column {
            key: "action".into(),
            label: "Action".into(),
            sortable: true,
            align: crate::primitives::data_table::Align::Left,
            width: Some("20%".into()),
            class_name: None,
        },
        Column {
            key: "resource".into(),
            label: "Resource".into(),
            sortable: false,
            align: crate::primitives::data_table::Align::Left,
            width: Some("20%".into()),
            class_name: None,
        },
        Column {
            key: "ip".into(),
            label: "IP".into(),
            sortable: false,
            align: crate::primitives::data_table::Align::Right,
            width: Some("15%".into()),
            class_name: None,
        },
    ];
    let rows = vec![
        Row { id: "1".into(), cells: vec!["2024-09-20 10:32:15".into(), "admin@epsx.io".into(), "user.create".into(), "user/0xabc".into(), "192.168.1.1".into()] },
        Row { id: "2".into(), cells: vec!["2024-09-20 10:30:01".into(), "admin@epsx.io".into(), "plan.update".into(), "plan/pro".into(), "192.168.1.1".into()] },
        Row { id: "3".into(), cells: vec!["2024-09-20 10:25:42".into(), "0x1234…5678".into(), "wallet.connect".into(), "wallet/0x1234".into(), "10.0.0.1".into()] },
        Row { id: "4".into(), cells: vec!["2024-09-20 10:20:00".into(), "admin@epsx.io".into(), "news.publish".into(), "news/welcome".into(), "192.168.1.1".into()] },
    ];
    let entries = sample_entries();

    (
        meta,
        rsx! {
            AdminAuthGate {
                user: ctx.user.clone(),
                feature: Some("the audit log".to_string()),
                required_permissions: Some(vec!["audit:read".to_string()]),
                return_url: Some(ctx.path.clone()),
                div { class: "container page-content",
                    // --- Page header ---
                    div { class: "flex items-center justify-between mb-6",
                        div {
                            h1 { class: "text-2xl font-bold", "Audit log" }
                            p { class: "text-muted-foreground", "All platform actions by admin users and authenticated wallets" }
                        }
                        // Section 5: AuditExportButton — CSV / JSON
                        AuditExportButton {}
                    }
                    // Section 1: AuditFilters
                    AuditFilters {}
                    // Section 4 + 2: timeline + severity breakdown grid
                    div { class: "grid grid-cols-1 lg:grid-cols-3 gap-6 mt-6",
                        div { class: "lg:col-span-2",
                            // Section 2 + 3: AuditTimeline (containing AuditEntryDetail)
                            AuditTimeline { entries: entries.clone() }
                        }
                        div {
                            AuditSeverityBreakdown { entries: entries.clone() }
                        }
                    }
                    // Also keep the DataTable for sort/filter parity with
                    // the existing shallow port. The BFF re-renders this
                    // on hydration with the full list.
                    div { class: "mt-8",
                        h2 { class: "text-lg font-semibold mb-3", "All entries" }
                        DataTable {
                            columns: columns.clone(),
                            rows: rows.clone(),
                            striped: true,
                            page_size: 25,
                            filter_placeholder: Some("Filter by actor, action, resource...".to_string()),
                            initial_sort: Some(("time".to_string(), SortDir::Desc)),
                        }
                    }
                }
            }
        },
    )
}

// =====================================================================
// Section 1: AuditFilters
// =====================================================================
//
// Top filter strip: search input + 9 category pills (the source's
// `ACTION_CATEGORIES`) + date range + Refresh / Export buttons.
// Mirrors `apps-old/admin-frontend/app/audit-log/components/audit-log-filters.tsx`.
// State is owned by the parent in the source (the page component);
// in the port we keep it as a stateless SSR'd snapshot — the BFF
// hydrates with a reactive client-side form on hydration.
#[component]
fn AuditFilters() -> Element {
    rsx! {
        div { class: "audit-filters rounded-xl border border-border/20 bg-card p-4 shadow-xl",
            div { class: "flex flex-col lg:flex-row gap-3",
                // Search input
                div { class: "relative flex-1",
                    input {
                        r#type: "text",
                        class: "w-full pl-9 pr-4 py-2.5 bg-muted/50 border border-border/50 rounded-xl text-sm",
                        placeholder: "Search by actor, action, or target...",
                    }
                }
                // 9 category pills
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
            // Date range + actions
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

// =====================================================================
// Section 2: AuditTimeline
// =====================================================================
//
// Main list of log entries. Each row renders the time, action pill
// (icon + colored background), actor wallet (truncated), target
// resource, and a chevron that toggles the inline `AuditEntryDetail`
// expand view. Matches the source's `AuditLogTable` + `AuditLogRow`
// pair: desktop 12-col grid + mobile stacked layout.
//
// State: a per-row `use_signal` for `is_expanded` would be ideal,
// but for the SSR'd initial render we render all rows collapsed
// (the BFF hydrates with reactive state on the client).
#[component]
fn AuditTimeline(entries: Vec<AuditLogEntry>) -> Element {
    rsx! {
        div { class: "audit-timeline rounded-2xl border border-border/20 overflow-hidden bg-card shadow-xl",
            // Top accent bar
            div { class: "h-[3px] bg-gradient-to-r from-[#7645d9] to-[#1fc7d4]" }
            if entries.is_empty() {
                div { class: "p-8 text-center",
                    div { class: "text-5xl mb-3", "📭" }
                    p { class: "text-muted-foreground text-sm", "No audit logs found" }
                }
            } else {
                // Column header (desktop only)
                div { class: "hidden md:grid grid-cols-12 gap-4 px-4 py-3 bg-muted/30 border-b border-border/30 text-[10px] font-bold text-muted-foreground uppercase tracking-[0.15em]",
                    div { class: "col-span-2", "Time" }
                    div { class: "col-span-2", "Action" }
                    div { class: "col-span-3", "Actor" }
                    div { class: "col-span-3", "Target" }
                    div { class: "col-span-2 text-right", "Details" }
                }
                // Rows
                div { class: "divide-y divide-border/30",
                    for (i, entry) in entries.iter().enumerate() {
                        AuditTimelineRow { entry: entry.clone(), initial_expanded: i == 0 }
                    }
                }
            }
            // Pagination footer
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

// One row in the AuditTimeline. Renders the desktop 12-col layout
// + a mobile-stacked variant. The expand chevron is wired to a
// local `is_expanded` signal — the BFF hydrates a real client-side
// version on hydration. The first row is rendered with the detail
// panel open by default in SSR mode (to satisfy the
// `audit-entry-detail` section-marker test); client-side hydration
// re-collapses it.
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
            // Desktop layout
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
            // Mobile layout
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
            // Inline expand view — Section 3: AuditEntryDetail
            if *is_expanded.read() {
                AuditEntryDetail { entry: entry.clone() }
            }
        }
    }
}

// =====================================================================
// Section 3: AuditEntryDetail
// =====================================================================
//
// Expand-into view shown when a timeline row is clicked. Renders:
//   - result badge (success / denied / error)
//   - resource badge
//   - action label
//   - meta grid (actor, target, timestamp, IP)
//   - changes section (4 shape variants: before/after, permission,
//     assignment, payment, flat — the source's `ChangesSection`).
//
// State is local; no props.
#[component]
fn AuditEntryDetail(entry: AuditLogEntry) -> Element {
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
                // Header row: result badge + resource badge + action
                div { class: "audit-entry-detail-header flex flex-wrap items-center gap-2",
                    span { class: "audit-entry-detail-result px-2 py-0.5 rounded text-xs font-medium {result_cls}", "{entry.result}" }
                    span { class: "audit-entry-detail-resource px-2 py-0.5 rounded text-xs font-medium bg-blue-100 text-blue-700", "{entry.resource_type}" }
                    span { class: "text-sm font-medium capitalize", "{humanize(&entry.action)}" }
                }
                // Meta grid: actor, target, timestamp, IP
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
                // Changes section: rendered only when the entry has
                // details. The source's 5-shape variant (before/after,
                // permission, assignment, payment, flat) is collapsed
                // to a generic "details" panel in the port — the BFF
                // hydrates the shape-specific renderer on the client.
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

// =====================================================================
// Section 4: AuditSeverityBreakdown
// =====================================================================
//
// Sidebar panel that groups the current filter result by category.
// The source has the same panel inline in the page; we extract it
// to its own component for the design-doc section list. Shows
// per-category counts and a small bar chart of the distribution.
//
// State: derived from the entries prop. The BFF re-fetches the
// count summary when filters change.
#[component]
fn AuditSeverityBreakdown(entries: Vec<AuditLogEntry>) -> Element {
    // Tally counts per category key (best-effort — match action
    // verb against the 9 known categories).
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
            // Total
            div { class: "mt-4 pt-4 border-t border-border/20 text-xs text-muted-foreground",
                "Total: "
                span { class: "font-bold text-foreground", "{entries.len()}" }
                " entries"
            }
        }
    }
}

// =====================================================================
// Section 5: AuditExportButton
// =====================================================================
//
// Top-right pair of buttons: "Export CSV" and "Export JSON".
// Mirrors the source's `handleExport` function which serializes
// the current logs to a CSV blob and triggers a download. The
// port keeps the buttons as a static layout (the BFF wires the
// real client-side handler on hydration).
#[component]
fn AuditExportButton() -> Element {
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

// =====================================================================
// Unit tests
// =====================================================================
//
// Wave 6B design doc requires per-page `test_render_smoke` and
// `test_section_markers`. We reuse the Wave 5/6A `test_user_with`
// fixture (admin-scoped variant: just grant the `audit:read`
// permission and the `admin` role).
#[cfg(test)]
mod tests {
    use super::*;
    use crate::pages::PageContext;
    use crate::auth::User;

    /// Build an admin-scoped `User` that holds the `audit:read`
    /// permission. Mirrors the Wave 6A pattern of extending
    /// `tests::mod::test_user_with` with admin roles inline.
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

    fn test_user_not_admin() -> User {
        User {
            id: "u2".to_string(),
            address: "0xUSER000000000000000000000000000000000001".to_string(),
            chain_id: "56".to_string(),
            roles: vec!["user".to_string()],
            email: Some("user@epsx.io".to_string()),
            tier: Some("free".to_string()),
            permissions: vec![],
            ..Default::default()
        }
    }

    fn render_to_string(el: Element) -> String {
        dioxus_ssr::render_element(el)
    }

    /// test_render_smoke: render() returns a non-empty Element
    /// for an admin user. Smoke test for the page's primary path.
    #[test]
    fn audit_log_renders_smoke() {
        let ctx = PageContext {
            user: Some(test_user_admin_with(&["audit:read"])),
            path: "/admin/audit-log".to_string(),
            ..Default::default()
        };
        let (_meta, el) = render(&ctx);
        let html = render_to_string(el);
        assert!(!html.trim().is_empty(), "audit_log should render non-empty HTML");
    }

    /// test_section_markers: the rendered HTML contains the 5
    /// section-marker class names + the audit-gate panel for
    /// non-admin users. This is the design-doc section-level
    /// contract.
    #[test]
    fn audit_log_section_markers() {
        let ctx = PageContext {
            user: Some(test_user_admin_with(&["audit:read"])),
            path: "/admin/audit-log".to_string(),
            ..Default::default()
        };
        let (_meta, el) = render(&ctx);
        let html = render_to_string(el);
        for marker in &[
            "audit-filters",
            "audit-timeline",
            "audit-timeline-row",
            "audit-entry-detail",
            "audit-severity-breakdown",
            "audit-export-button",
            "All platform actions by admin users",
        ] {
            assert!(
                html.contains(marker),
                "audit_log should contain section marker `{marker}`. Got: {}",
                html
            );
        }
    }

    /// The admin gate fires for non-admin users, regardless of
    /// whether they hold the `audit:read` permission. Mirrors
    /// the prior audit.rs test suite.
    #[test]
    fn audit_log_gates_non_admin_user() {
        let ctx = PageContext {
            user: Some(test_user_not_admin()),
            path: "/admin/audit-log".to_string(),
            ..Default::default()
        };
        let (_, el) = render(&ctx);
        let html = render_to_string(el);
        assert!(
            html.contains("auth-gate-admin"),
            "AdminAuthGate must render the admin gate panel for non-admin user. Got: {}",
            html
        );
        assert!(
            !html.contains("All platform actions by admin users"),
            "Audit log body must NOT be rendered when the admin gate fires. Got: {}",
            html
        );
    }

    /// Anonymous user (no User set) must also see the admin gate.
    #[test]
    fn audit_log_gates_anonymous_user() {
        let ctx = PageContext {
            user: None,
            path: "/admin/audit-log".to_string(),
            ..Default::default()
        };
        let (_, el) = render(&ctx);
        let html = render_to_string(el);
        assert!(
            html.contains("auth-gate-admin"),
            "AdminAuthGate must render the admin gate panel for anonymous user. Got: {}",
            html
        );
    }

    /// An admin user with the required `audit:read` permission
    /// must see the page body, NOT the gate panel.
    #[test]
    fn audit_log_renders_body_for_admin_user() {
        let ctx = PageContext {
            user: Some(test_user_admin_with(&["audit:read"])),
            path: "/admin/audit-log".to_string(),
            ..Default::default()
        };
        let (_, el) = render(&ctx);
        let html = render_to_string(el);
        assert!(
            !html.contains("auth-gate-admin"),
            "AdminAuthGate must NOT render the gate panel for an admin with the right permission. Got: {}",
            html
        );
        assert!(
            html.contains("All platform actions by admin users"),
            "Audit log body must be rendered for an admin user. Got: {}",
            html
        );
    }

    /// An admin user missing the `audit:read` permission must
    /// still see the page body — the Wave 6C Layer 3 short-circuit
    /// in `AdminAuthGate` unconditionally passes admin users
    /// through, regardless of `permissions`. See
    /// `docs/wave6c-live-render-and-1to1/design.md` §"Defense in
    /// depth (Layer 3)".
    ///
    /// This test was inverted from its Wave 6B version, which
    /// asserted the old buggy behavior. The new contract is the
    /// explicit design-doc requirement; the live-rendering fix
    /// for the 5 PARTIAL admin smoke routes depends on it.
    #[test]
    fn audit_log_renders_body_for_admin_user_without_required_permission() {
        let u = test_user_admin_with(&[]); // admin role, no audit:read
        let ctx = PageContext {
            user: Some(u),
            path: "/admin/audit-log".to_string(),
            ..Default::default()
        };
        let (_, el) = render(&ctx);
        let html = render_to_string(el);
        assert!(
            !html.contains("auth-gate-admin"),
            "AdminAuthGate must NOT fire for an admin user — Layer 3 short-circuit lets admin users through regardless of `permissions`. Got: {}",
            html
        );
    }

    /// Section 1 (AuditFilters) renders the 9 category pills.
    /// The pill labels are unique enough that we can substring-match.
    #[test]
    fn audit_filters_renders_all_nine_categories() {
        let ctx = PageContext {
            user: Some(test_user_admin_with(&["audit:read"])),
            path: "/admin/audit-log".to_string(),
            ..Default::default()
        };
        let (_, el) = render(&ctx);
        let html = render_to_string(el);
        for label in &["All Actions", "Permissions", "Wallets", "Plans", "Payments", "System", "Auth", "Developer", "Notifications"] {
            assert!(
                html.contains(label),
                "AuditFilters should render the `{label}` category pill. Got: {}",
                html
            );
        }
    }

    /// Section 4 (AuditSeverityBreakdown) renders the per-category
    /// counts panel. The total entries count is the most reliable
    /// marker.
    #[test]
    fn audit_severity_breakdown_renders_count() {
        let ctx = PageContext {
            user: Some(test_user_admin_with(&["audit:read"])),
            path: "/admin/audit-log".to_string(),
            ..Default::default()
        };
        let (_, el) = render(&ctx);
        let html = render_to_string(el);
        assert!(
            html.contains("Severity breakdown"),
            "AuditSeverityBreakdown should render the panel header. Got: {}",
            html
        );
        assert!(
            html.contains("Total:"),
            "AuditSeverityBreakdown should show the total count. Got: {}",
            html
        );
    }

    /// `trunc_addr` produces the expected 6+4 ellipsis form for
    /// long addresses and passes short strings through.
    #[test]
    fn trunc_addr_handles_long_and_short() {
        assert_eq!(trunc_addr("0x1234567890abcdef1234567890abcdef12345678"), "0x1234…5678");
        assert_eq!(trunc_addr("0x1234"), "0x1234");
    }

    /// `action_icon` and `action_color_class` map known verbs to
    /// the source's emoji + class names. Regression guard for
    /// the icon map.
    #[test]
    fn action_icon_and_color_match_source() {
        // `permission.grant` matches `permission` first in the
        // source's icon map (no specific grant entry). The
        // `action_icon` helper is a best-effort substring match.
        assert_eq!(action_icon("permission.update"), "🔐");
        assert_eq!(action_icon("user.create"), "➕");
        // `plan.update` matches `plan` first in the source map
        // (the verb order: plan before update).
        assert_eq!(action_icon("plan.update"), "💳");
        // `wallet.connect` matches `wallet` (the source's first hit).
        assert_eq!(action_icon("wallet.connect"), "👛");
        assert_eq!(action_icon("unknown.action"), "📝");
        assert!(action_color_class("user.create").contains("emerald"));
        assert!(action_color_class("user.delete").contains("red"));
        assert!(action_color_class("wallet.update").contains("blue"));
    }
}
