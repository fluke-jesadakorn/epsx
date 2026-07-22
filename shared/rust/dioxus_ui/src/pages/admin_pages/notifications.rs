//! Truthful admin notification surfaces.
//!
//! Notification inventory, delivery metrics, audience policy, templates,
//! scheduling, and send/cancel/purge mutations remain backend-owned. Until A11
//! provides typed owner-scoped read and mutation contracts, these routes fail
//! closed instead of rendering legacy samples or locally simulated actions.

use dioxus::prelude::*;

use super::super::{PageContext, PageMeta};
use crate::auth::AuthGate;
use crate::primitives::Icon;

/// `/notifications/manage` — authenticated shell with no inferred capability
/// policy and no unowned notification data.
pub fn render_manage(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::admin("Notifications");
    (
        meta,
        rsx! {
            AuthGate {
                user: ctx.user.clone(),
                feature: Some("notification management".to_string()),
                return_url: Some(ctx.path.clone()),
                NotificationUnavailable {
                    mode: NotificationUnavailableMode::Manage,
                }
            }
        },
    )
}

/// `/notifications/create` — authenticated fail-closed compose shell. No form
/// is emitted until the service owns a typed, authorized, idempotent mutation.
pub fn render_create(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::admin("New notification");
    (
        meta,
        rsx! {
            AuthGate {
                user: ctx.user.clone(),
                feature: Some("creating notifications".to_string()),
                return_url: Some(ctx.path.clone()),
                NotificationUnavailable {
                    mode: NotificationUnavailableMode::Create,
                }
            }
        },
    )
}

#[derive(Clone, Copy, PartialEq)]
enum NotificationUnavailableMode {
    Manage,
    Create,
}

#[component]
fn NotificationUnavailable(mode: NotificationUnavailableMode) -> Element {
    let (state_marker, eyebrow, title, detail, retry_href, retry_label) = match mode {
        NotificationUnavailableMode::Manage => (
            "unavailable",
            "Delivery workspace",
            "Notification management is unavailable",
            "No broadcasts, recipients, delivery counts, schedules, drafts, or statuses are shown because a backend-authoritative notification inventory is not connected.",
            "/notifications/manage",
            "Retry notification management",
        ),
        NotificationUnavailableMode::Create => (
            "unavailable",
            "Compose workspace",
            "Notification creation is unavailable",
            "No recipient, template, preview, scheduling, or send controls are shown because an authorized notification mutation contract is not connected.",
            "/notifications/create",
            "Retry notification creation",
        ),
    };

    rsx! {
        main {
            class: "container page-content max-w-5xl py-10",
            "data-admin-notifications-state": state_marker,
            "data-admin-notifications-surface": match mode {
                NotificationUnavailableMode::Manage => "manage",
                NotificationUnavailableMode::Create => "create",
            },
            section {
                class: "relative overflow-hidden rounded-3xl border border-border/40 bg-card shadow-2xl",
                aria_labelledby: "notification-unavailable-title",
                div { class: "absolute inset-x-0 top-0 h-1 bg-gradient-to-r from-[#1fc7d4] via-[#7645d9] to-[#ffb237]" }
                div { class: "grid gap-8 p-8 md:grid-cols-[auto_1fr] md:p-12",
                    div { class: "flex h-16 w-16 items-center justify-center rounded-2xl border border-cyan-500/20 bg-cyan-500/10 text-[#1fc7d4]",
                        Icon { name: "bell".to_string(), size: Some(30) }
                    }
                    div {
                        p { class: "text-xs font-black uppercase tracking-[0.22em] text-[#1fc7d4]", "{eyebrow}" }
                        h1 { id: "notification-unavailable-title", class: "mt-3 text-3xl font-black tracking-tight text-foreground", "{title}" }
                        div {
                            class: "mt-5 rounded-2xl border border-amber-500/20 bg-amber-500/10 p-5",
                            role: "status",
                            aria_live: "polite",
                            p { class: "text-sm font-semibold leading-6 text-foreground", "{detail}" }
                        }
                        p { class: "mt-5 max-w-3xl text-sm leading-6 text-muted-foreground",
                            "The Rust backend must supply authenticated reads, explicit authorization, validation, audit records, and idempotency before this workspace can safely expose notification operations."
                        }
                        nav { class: "mt-8 flex flex-wrap gap-3", aria_label: "Notification recovery",
                            a { class: "btn btn-primary", href: retry_href,
                                Icon { name: "refresh-cw".to_string(), size: Some(16) }
                                " {retry_label}"
                            }
                            if mode == NotificationUnavailableMode::Create {
                                a { class: "btn btn-outline", href: "/notifications/manage",
                                    Icon { name: "arrow-left".to_string(), size: Some(16) }
                                    " Back to notification management"
                                }
                            }
                            a { class: "btn btn-ghost", href: "/",
                                "Admin home"
                            }
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;
    use crate::auth::User;

    fn admin() -> User {
        User {
            id: "admin-1".to_string(),
            address: "0xADMIN0000000000000000000000000000000001".to_string(),
            chain_id: "56".to_string(),
            roles: vec![],
            permissions: vec![],
            ..Default::default()
        }
    }

    fn context(path: &str, signed_in: bool) -> PageContext {
        PageContext {
            user: signed_in.then(admin),
            path: path.to_string(),
            ..Default::default()
        }
    }

    fn html(element: Element) -> String {
        dioxus_ssr::render_element(element)
    }

    #[test]
    fn signed_out_routes_keep_notification_workspaces_private() {
        for (path, render) in [
            (
                "/notifications/manage",
                render_manage as fn(&PageContext) -> (PageMeta, Element),
            ),
            (
                "/notifications/create",
                render_create as fn(&PageContext) -> (PageMeta, Element),
            ),
        ] {
            let rendered = html(render(&context(path, false)).1);
            assert!(rendered.contains("Sign in required"));
            assert!(!rendered.contains("data-admin-notifications-state"));
            assert!(!rendered.contains("Notification management is unavailable"));
            assert!(!rendered.contains("Notification creation is unavailable"));
        }
    }

    #[test]
    fn authenticated_manage_fails_closed_with_safe_recovery() {
        let rendered = html(render_manage(&context("/notifications/manage", true)).1);
        assert!(rendered.contains("data-admin-notifications-state=\"unavailable\""));
        assert!(rendered.contains("data-admin-notifications-surface=\"manage\""));
        assert!(rendered.contains("Notification management is unavailable"));
        assert!(rendered.contains("href=\"/notifications/manage\""));
        assert!(rendered.contains("href=\"/\""));
    }

    #[test]
    fn authenticated_create_fails_closed_with_safe_recovery() {
        let rendered = html(render_create(&context("/notifications/create", true)).1);
        assert!(rendered.contains("data-admin-notifications-state=\"unavailable\""));
        assert!(rendered.contains("data-admin-notifications-surface=\"create\""));
        assert!(rendered.contains("Notification creation is unavailable"));
        assert!(rendered.contains("href=\"/notifications/create\""));
        assert!(rendered.contains("href=\"/notifications/manage\""));
        assert!(rendered.contains("href=\"/\""));
    }

    #[test]
    fn unavailable_surfaces_emit_no_samples_metrics_or_mutations() {
        let manage = html(render_manage(&context("/notifications/manage", true)).1);
        let create = html(render_create(&context("/notifications/create", true)).1);
        let combined = format!("{manage}{create}");

        for forbidden in [
            "Welcome to the platform",
            "New feature: charts",
            "Maintenance window",
            "1,234",
            "Total Sent",
            "Today's Pulse",
            "Weekly Volume",
            "System Health",
            "Synchronize",
            "Analytics",
            "Purge",
            "Resend",
            "Cancel",
            "Send notification",
            concat!("/api/v1/notification/", "send"),
            "<form",
            "<button",
        ] {
            assert!(
                !combined.contains(forbidden),
                "leaked forbidden UI: {forbidden}"
            );
        }
    }

    #[test]
    fn hostile_and_legacy_params_cannot_create_notification_claims() {
        let mut ctx = context("/notifications/manage", true);
        ctx.params = HashMap::from([
            (
                "status".to_string(),
                "sent\" onclick=\"alert(1)".to_string(),
            ),
            (
                "data_notifications".to_string(),
                "Welcome to the platform".to_string(),
            ),
            ("delivery_total".to_string(), "999999".to_string()),
        ]);

        let rendered = html(render_manage(&ctx).1);
        assert!(rendered.contains("data-admin-notifications-state=\"unavailable\""));
        assert!(!rendered.contains("onclick"));
        assert!(!rendered.contains("Welcome to the platform"));
        assert!(!rendered.contains("999999"));
        assert!(!rendered.contains("status=sent"));
    }
}
