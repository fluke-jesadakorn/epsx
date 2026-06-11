//! /admin/audit-log — audit log table with filters, timeline, severity
//! breakdown, and export.
//!
//! Wave 6C Track D — 5 sections per the design doc
//! `docs/wave6c-live-render-and-1to1/design.md` §"Track D":
//!   1. AuditFilters            — search + 9 category pills + date range
//!   2. AuditTimeline           — paginated list of log entries
//!   3. AuditEntryDetail        — expand panel with result/resource badges
//!   4. AuditSeverityBreakdown  — by-category counts panel
//!   5. AuditExportButton       — CSV / JSON export trigger buttons
//!
//! All 5 sub-components + the `AuditLogEntry` data struct + the
//! helper functions (`action_icon`, `action_color_class`,
//! `trunc_addr`, `fmt_time`, `humanize`) + the `ACTION_CATEGORIES`
//! const live in `components::admin::audit`. This page just composes
//! them inside the `AdminAuthGate` wrapper.

use crate::primitives::*;
use crate::components::admin::audit::{
    action_color_class, action_icon, AuditEntryDetail, AuditExportButton, AuditFilters,
    AuditLogEntry, AuditSeverityBreakdown, AuditTimeline, fmt_time, humanize,
    ACTION_CATEGORIES,
};
use crate::data_table::{Column, DataTable, Row, SortDir};

use dioxus::prelude::*;
use super::super::{PageContext, PageMeta};
use crate::auth::AdminAuthGate;

/// Re-export `trunc_addr` from the new components module so existing
/// tests that reference the local helper still compile. (Internal
/// detail; the test section uses the public name from components.)
pub use crate::components::admin::audit::trunc_addr as trunc_addr_compat;

/// Build 3 sample audit entries for the SSR'd initial render.
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

/// Top-level render: the audit-log page. Wires the 5 sub-components
/// from `components::admin::audit` and adds the `DataTable` view
/// that the prior shallow port kept.
pub fn render(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::admin("Audit log");

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
                    // Page header
                    div { class: "flex items-center justify-between mb-6",
                        div {
                            h1 { class: "text-2xl font-bold", "Audit log" }
                            p { class: "text-muted-foreground", "All platform actions by admin users and authenticated wallets" }
                        }
                        // Section 5.
                        AuditExportButton {}
                    }
                    // Section 1.
                    AuditFilters {}
                    // Section 4 + 2 grid.
                    div { class: "grid grid-cols-1 lg:grid-cols-3 gap-6 mt-6",
                        div { class: "lg:col-span-2",
                            // Section 2 + 3.
                            AuditTimeline { entries: entries.clone() }
                        }
                        div {
                            AuditSeverityBreakdown { entries: entries.clone() }
                        }
                    }
                    // DataTable for sort/filter parity.
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

// ============================================================================
// Unit tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::User;

    /// Build an admin-scoped `User` that holds the `audit:read`
    /// permission. Mirrors the Wave 6A pattern.
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
    /// whether they hold the `audit:read` permission.
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

    /// Section 1 (AuditFilters) renders the 9 category pills.
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
    /// counts panel.
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
        assert_eq!(trunc_addr_compat("0x1234567890abcdef1234567890abcdef12345678"), "0x1234…5678");
        assert_eq!(trunc_addr_compat("0x1234"), "0x1234");
    }

    /// `action_icon` and `action_color_class` map known verbs to
    /// the source's emoji + class names. Regression guard for
    /// the icon map.
    #[test]
    fn action_icon_and_color_match_source() {
        assert_eq!(action_icon("permission.update"), "🔐");
        assert_eq!(action_icon("user.create"), "➕");
        assert_eq!(action_icon("plan.update"), "💳");
        assert_eq!(action_icon("wallet.connect"), "👛");
        assert_eq!(action_icon("unknown.action"), "📝");
        assert!(action_color_class("user.create").contains("emerald"));
        assert!(action_color_class("user.delete").contains("red"));
        assert!(action_color_class("wallet.update").contains("blue"));
    }
}
