//! /admin/audit-log — audit log table with filters.

use crate::primitives::*;
use crate::data_table::{Column, DataTable, Row, SortDir};

use dioxus::prelude::*;
use super::super::{PageContext, PageMeta};
use crate::auth::AdminAuthGate;

pub fn render(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::admin("Audit log");
    let columns = vec![
        Column { key: "time".into(), label: "Time".into(), sortable: true, align: crate::primitives::data_table::Align::Left, width: Some("20%".into()), class_name: None },
        Column { key: "actor".into(), label: "Actor".into(), sortable: true, align: crate::primitives::data_table::Align::Left, width: Some("25%".into()), class_name: None },
        Column { key: "action".into(), label: "Action".into(), sortable: true, align: crate::primitives::data_table::Align::Left, width: Some("20%".into()), class_name: None },
        Column { key: "resource".into(), label: "Resource".into(), sortable: false, align: crate::primitives::data_table::Align::Left, width: Some("20%".into()), class_name: None },
        Column { key: "ip".into(), label: "IP".into(), sortable: false, align: crate::primitives::data_table::Align::Right, width: Some("15%".into()), class_name: None },
    ];
    let rows = vec![
        Row { id: "1".into(), cells: vec!["2024-09-20 10:32:15".into(), "admin@epsx.io".into(), "user.create".into(), "user/0xabc".into(), "192.168.1.1".into()] },
        Row { id: "2".into(), cells: vec!["2024-09-20 10:30:01".into(), "admin@epsx.io".into(), "plan.update".into(), "plan/pro".into(), "192.168.1.1".into()] },
        Row { id: "3".into(), cells: vec!["2024-09-20 10:25:42".into(), "0x1234…5678".into(), "wallet.connect".into(), "wallet/0x1234".into(), "10.0.0.1".into()] },
        Row { id: "4".into(), cells: vec!["2024-09-20 10:20:00".into(), "admin@epsx.io".into(), "news.publish".into(), "news/welcome".into(), "192.168.1.1".into()] },
    ];
    (meta, rsx! {
        AdminAuthGate { user: ctx.user.clone(), feature: Some("the audit log".to_string()), required_permissions: Some(vec!["audit:read".to_string()]), return_url: Some(ctx.path.clone()),
            div { class: "container page-content",
                div { class: "flex items-center justify-between mb-6",
                    div {
                        h1 { class: "text-2xl font-bold", "Audit log" }
                        p { class: "text-muted-foreground", "All platform actions by admin users and authenticated wallets" }
                    }
                    div { class: "flex gap-2",
                        button { class: "btn btn-sm btn-outline", r#type: "button", "Export CSV" }
                        button { class: "btn btn-sm btn-outline", r#type: "button", "Export JSON" }
                    }
                }
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
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pages::PageContext;
    use crate::auth::User;

    /// Build a `User` with the admin role + the `audit:read` permission.
    fn test_user_admin() -> User {
        User {
            id: "test-admin".to_string(),
            address: "0xADMIN0000000000000000000000000000000001".to_string(),
            chain_id: "56".to_string(),
            roles: vec!["admin".to_string()],
            email: Some("admin@epsx.io".to_string()),
            tier: Some("admin".to_string()),
            permissions: vec!["audit:read".to_string()],
            ..Default::default()
        }
    }

    /// Build an authenticated user that is NOT an admin and has no
    /// `audit:read` permission. This is the user the `AdminAuthGate`
    /// should bounce.
    fn test_user_not_admin() -> User {
        User {
            id: "test-user".to_string(),
            address: "0xUSER000000000000000000000000000000000001".to_string(),
            chain_id: "56".to_string(),
            roles: vec!["user".to_string()],
            email: Some("user@epsx.io".to_string()),
            tier: Some("free".to_string()),
            permissions: vec![],
            ..Default::default()
        }
    }

    /// Render the admin page's `Element` to an HTML string. Mirrors the
    /// helper in `layout/main_layout.rs` (Wave 3a Track A pattern).
    fn render_to_string(el: Element) -> String {
        dioxus_ssr::render_element(el)
    }

    /// A signed-in non-admin user hitting `/admin/audit-log` must see
    /// the admin gate panel (`auth-gate-admin` class), not the audit
    /// log body. The admin variant of the gate fires for `!is_admin()`
    /// users, regardless of `required_permissions`.
    #[test]
    fn admin_audit_page_gates_non_admin_user() {
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
        // The page body's <h1> is "Audit log" — the gate replaces the
        // body with the panel, so the heading should NOT be present
        // in the gate-firing case.
        assert!(
            !html.contains("All platform actions by admin users"),
            "Audit log body must NOT be rendered when the admin gate fires. Got: {}",
            html
        );
    }

    /// A signed-in non-admin user with NO `user` set at all (i.e.
    /// anonymous / not signed in) must also see the admin gate.
    #[test]
    fn admin_audit_page_gates_anonymous_user() {
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

    /// A signed-in admin user with the required `audit:read`
    /// permission must see the page body, NOT the gate panel.
    #[test]
    fn admin_audit_page_renders_body_for_admin_user() {
        let ctx = PageContext {
            user: Some(test_user_admin()),
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
        // The body must be present — assert on the unique page subtitle
        // text that lives inside the gated body.
        assert!(
            html.contains("All platform actions by admin users"),
            "Audit log body must be rendered for an admin user. Got: {}",
            html
        );
    }

    /// A signed-in admin user who is missing the `audit:read`
    /// permission must STILL be bounced by the gate (the admin
    /// variant also fires on missing permissions).
    #[test]
    fn admin_audit_page_gates_admin_user_missing_permission() {
        let u = User {
            id: "test-admin-no-perm".to_string(),
            address: "0xADMIN0000000000000000000000000000000002".to_string(),
            chain_id: "56".to_string(),
            roles: vec!["admin".to_string()],
            email: Some("admin@epsx.io".to_string()),
            tier: Some("admin".to_string()),
            permissions: vec![], // admin role, no audit:read
            ..Default::default()
        };
        let ctx = PageContext {
            user: Some(u),
            path: "/admin/audit-log".to_string(),
            ..Default::default()
        };
        let (_, el) = render(&ctx);
        let html = render_to_string(el);
        assert!(
            html.contains("auth-gate-admin"),
            "AdminAuthGate must fire for an admin missing the required permission. Got: {}",
            html
        );
    }
}
