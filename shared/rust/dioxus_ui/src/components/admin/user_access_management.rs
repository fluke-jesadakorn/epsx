//! Admin `UserAccessManagement` family — Wave 38b T2 admin domain port.
//!
//! Mirrors
//! `apps-old/admin-frontend/components/payments/user-access-management.tsx`,
//! which exports a desktop table + mobile card view for users
//! with active plan access.
//!
//! | Component | Use case |
//! | --- | --- |
//! | `UserAccessDesktopTable` | Desktop table (wallet / plan / status / days / expires / actions) |
//! | `UserAccessMobileCards` | Mobile card list |
//! | `UserAccessPaginationBar` | Prev / Next page controls |
//!
//! ## Status color helper
//!
//! `user_access_status_class(status)` — maps a user-access status
//! string to the design-system pill class (success / warning /
//! destructive / muted).
//!
//! ## Tests
//!
//! Each component gets a colocated `#[cfg(test)] mod tests` block
//! with smoke render + key prop handling.

use dioxus::prelude::*;

// ============================================================================
// Status helper
// ============================================================================
//
// Returns the Tailwind pill class for a user-access status.
// Exposed publicly so admin pages can reuse the same treatment.

/// Maps a user-access status string to the Tailwind pill class.
pub fn user_access_status_class(status: &str) -> &'static str {
    match status {
        "active" => "bg-success/10 text-success border border-success/20",
        "expiring_soon" => "bg-warning/10 text-warning border border-warning/20",
        "expired" => "bg-destructive/10 text-destructive border border-destructive/20",
        _ => "bg-muted text-muted-foreground border border-border/50",
    }
}

// ============================================================================
// UserAccess data shape
// ============================================================================

#[derive(Clone, Debug, PartialEq)]
pub struct UserAccessData {
    pub wallet_address: String,
    pub plan_name: Option<String>,
    pub status: String,
    pub days_remaining: i32,
    pub plan_expires_at: Option<String>,
}

// ============================================================================
// UserAccessDesktopTable
// ============================================================================
//
// Desktop table view. Mirrors the source's `UserAccessDesktopTable`.

#[component]
pub fn UserAccessDesktopTable(user_access: Vec<UserAccessData>) -> Element {
    rsx! {
        div { class: "user-access-management-table hidden sm:block overflow-x-auto",
            table { class: "min-w-full",
                thead {
                    tr { class: "border-b border-border/50",
                        th { class: "px-4 py-3 text-left text-xs font-semibold text-muted-foreground uppercase tracking-wider", "Wallet" }
                        th { class: "px-4 py-3 text-left text-xs font-semibold text-muted-foreground uppercase tracking-wider", "Plan" }
                        th { class: "px-4 py-3 text-left text-xs font-semibold text-muted-foreground uppercase tracking-wider", "Status" }
                        th { class: "px-4 py-3 text-left text-xs font-semibold text-muted-foreground uppercase tracking-wider", "Days Left" }
                        th { class: "px-4 py-3 text-left text-xs font-semibold text-muted-foreground uppercase tracking-wider", "Expires" }
                        th { class: "px-4 py-3 text-left text-xs font-semibold text-muted-foreground uppercase tracking-wider", "Actions" }
                    }
                }
                tbody { class: "divide-y divide-border/50",
                    for user in user_access.iter() {
                        {
                            let status_cls = user_access_status_class(&user.status);
                            let display_status = if user.status == "no_plan" { "No Plan".to_string() } else { user.status.clone() };
                            let wallet_short = format!("{}\u{2026}{}",
                                &user.wallet_address[..user.wallet_address.len().min(10)],
                                &user.wallet_address[user.wallet_address.len().saturating_sub(4)..]
                            );
                            let plan_display = user.plan_name.clone().unwrap_or_else(|| "No Plan".to_string());
                            let days_display = if user.days_remaining > 0 { format!("{} days", user.days_remaining) } else { "-".to_string() };
                            let expires_display = user.plan_expires_at.clone().unwrap_or_else(|| "Never".to_string());
                            rsx! {
                                tr { key: "{user.wallet_address}", class: "user-access-management-row hover:bg-muted/30 transition-colors",
                                    td { class: "px-4 py-4 whitespace-nowrap",
                                        div { class: "text-xs font-mono text-muted-foreground", title: "{user.wallet_address}",
                                            "{wallet_short}"
                                        }
                                    }
                                    td { class: "px-4 py-4 whitespace-nowrap text-sm font-semibold text-foreground", "{plan_display}" }
                                    td { class: "px-4 py-4 whitespace-nowrap",
                                        span { class: "inline-flex items-center px-2.5 py-1 rounded-full text-xs font-semibold {status_cls}",
                                            "{display_status}"
                                        }
                                    }
                                    td { class: "px-4 py-4 whitespace-nowrap text-sm text-secondary", "{days_display}" }
                                    td { class: "px-4 py-4 whitespace-nowrap text-sm text-muted-foreground", "{expires_display}" }
                                    td { class: "px-4 py-4 whitespace-nowrap",
                                        button {
                                            class: "px-3 py-1.5 text-sm font-semibold rounded-xl text-primary hover:bg-primary/10 border border-primary/20 transition-all",
                                            r#type: "button",
                                            "View"
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
}

// ============================================================================
// UserAccessMobileCards
// ============================================================================
//
// Mobile card list. Mirrors the source's `UserAccessMobileCards`.

#[component]
pub fn UserAccessMobileCards(user_access: Vec<UserAccessData>) -> Element {
    rsx! {
        div { class: "user-access-management-mobile block sm:hidden space-y-4",
            for user in user_access.iter() {
                {
                    let status_cls = user_access_status_class(&user.status);
                    let display_status = if user.status == "no_plan" { "No Plan".to_string() } else { user.status.clone() };
                    let wallet_short = format!("{}\u{2026}{}",
                        &user.wallet_address[..user.wallet_address.len().min(10)],
                        &user.wallet_address[user.wallet_address.len().saturating_sub(4)..]
                    );
                    let plan_display = user.plan_name.clone().unwrap_or_else(|| "No Plan".to_string());
                    let days_display = if user.days_remaining > 0 { format!("{} days", user.days_remaining) } else { "-".to_string() };
                    let expires_display = match &user.plan_expires_at { Some(s) => s.clone(), None => "Never".to_string() };
                    rsx! {
                        div { key: "{user.wallet_address}", class: "p-4 bg-muted/30 border border-border/50 rounded-2xl",
                            div { class: "flex items-center justify-between mb-3",
                                div { class: "font-mono text-xs text-muted-foreground", title: "{user.wallet_address}",
                                    "{wallet_short}"
                                }
                                span { class: "inline-flex items-center px-2.5 py-1 rounded-full text-xs font-semibold {status_cls}",
                                    "{display_status}"
                                }
                            }
                            div { class: "grid grid-cols-2 gap-3",
                                div { class: "bg-card rounded-xl p-3 border border-border/50",
                                    div { class: "text-sm font-medium text-muted-foreground", "Plan" }
                                    div { class: "text-lg font-bold text-primary", "{plan_display}" }
                                }
                                div { class: "bg-card rounded-xl p-3 border border-border/50",
                                    div { class: "text-sm font-medium text-muted-foreground", "Days Left" }
                                    div { class: "text-lg font-bold text-secondary", "{days_display}" }
                                }
                            }
                            div { class: "mt-3 text-xs text-muted-foreground",
                                "Expires: {expires_display}"
                            }
                        }
                    }
                }
            }
        }
    }
}

// ============================================================================
// UserAccessPaginationBar
// ============================================================================
//
// Prev / Next page controls. Mirrors the source's pagination row.

#[component]
pub fn UserAccessPaginationBar(
    current: u32,
    /// Disable the Next button when true (e.g., last page reached).
    disable_next: Option<bool>,
    on_prev: EventHandler<()>,
    on_next: EventHandler<()>,
) -> Element {
    let disable_next = disable_next.unwrap_or(false);
    rsx! {
        div { class: "user-access-management-pagination mt-6 flex items-center justify-between",
            button {
                class: "px-4 py-2 text-sm font-semibold rounded-xl border border-border/40 text-muted-foreground hover:bg-muted/50 disabled:opacity-40 transition-all",
                r#type: "button",
                disabled: current == 1,
                onclick: move |_| on_prev.call(()),
                "Previous"
            }
            span { class: "text-sm text-muted-foreground", "Page {current}" }
            button {
                class: "px-4 py-2 text-sm font-semibold rounded-xl border border-border/40 text-muted-foreground hover:bg-muted/50 disabled:opacity-40 transition-all",
                r#type: "button",
                disabled: disable_next,
                onclick: move |_| on_next.call(()),
                "Next"
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

    fn sample_user() -> UserAccessData {
        UserAccessData {
            wallet_address: "0x1234567890abcdef1234567890abcdef12345678".to_string(),
            plan_name: Some("Pro".to_string()),
            status: "active".to_string(),
            days_remaining: 23,
            plan_expires_at: Some("2024-10-15 00:00".to_string()),
        }
    }

    /// `UserAccessDesktopTable` renders the 6 column headers + each
    /// user's row.
    #[test]
    fn user_access_desktop_table_renders_headers_and_rows() {
        fn harness() -> Element {
            rsx! { UserAccessDesktopTable { user_access: vec![sample_user()] } }
        }
        let mut vdom = dioxus::prelude::VirtualDom::new(harness);
        vdom.rebuild_in_place();
        let html = dioxus_ssr::render(&vdom);
        for col in &["Wallet", "Plan", "Status", "Days Left", "Expires", "Actions"] {
            assert!(html.contains(col), "UserAccessDesktopTable must render `{col}` header. Got: {html}");
        }
        assert!(html.contains("0x12345678\u{2026}5678"), "UserAccessDesktopTable must render truncated wallet. Got: {html}");
        assert!(html.contains("Pro"), "UserAccessDesktopTable must render plan_name. Got: {html}");
        assert!(html.contains("active"), "UserAccessDesktopTable must render status. Got: {html}");
        assert!(html.contains("23 days"), "UserAccessDesktopTable must render days_remaining. Got: {html}");
        assert!(html.contains("bg-success/10"), "UserAccessDesktopTable active status must use success class. Got: {html}");
    }

    /// `UserAccessDesktopTable` with no users renders empty body.
    #[test]
    fn user_access_desktop_table_renders_empty() {
        fn harness() -> Element {
            rsx! { UserAccessDesktopTable { user_access: vec![] } }
        }
        let mut vdom = dioxus::prelude::VirtualDom::new(harness);
        vdom.rebuild_in_place();
        let html = dioxus_ssr::render(&vdom);
        assert!(html.contains("user-access-management-table"), "UserAccessDesktopTable must render container. Got: {html}");
        assert!(!html.contains("user-access-management-row"), "UserAccessDesktopTable empty body must omit rows. Got: {html}");
    }

    /// `UserAccessDesktopTable` with `expiring_soon` uses warning class.
    #[test]
    fn user_access_desktop_table_uses_warning_for_expiring_soon() {
        fn harness() -> Element {
            let mut user = sample_user();
            user.status = "expiring_soon".to_string();
            rsx! { UserAccessDesktopTable { user_access: vec![user] } }
        }
        let mut vdom = dioxus::prelude::VirtualDom::new(harness);
        vdom.rebuild_in_place();
        let html = dioxus_ssr::render(&vdom);
        assert!(html.contains("bg-warning/10"), "UserAccessDesktopTable expiring_soon must use warning class. Got: {html}");
    }

    /// `UserAccessMobileCards` renders the mobile card layout.
    #[test]
    fn user_access_mobile_cards_renders_layout() {
        fn harness() -> Element {
            rsx! { UserAccessMobileCards { user_access: vec![sample_user()] } }
        }
        let mut vdom = dioxus::prelude::VirtualDom::new(harness);
        vdom.rebuild_in_place();
        let html = dioxus_ssr::render(&vdom);
        assert!(html.contains("user-access-management-mobile"), "UserAccessMobileCards must render container. Got: {html}");
        assert!(html.contains("Days Left"), "UserAccessMobileCards must render Days Left label. Got: {html}");
        assert!(html.contains("Expires:"), "UserAccessMobileCards must render Expires label. Got: {html}");
        assert!(html.contains("23 days"), "UserAccessMobileCards must render days_remaining. Got: {html}");
    }

    /// `UserAccessMobileCards` shows "No Plan" when status is "no_plan".
    #[test]
    fn user_access_mobile_cards_renders_no_plan_label() {
        fn harness() -> Element {
            let mut user = sample_user();
            user.status = "no_plan".to_string();
            user.plan_name = None;
            rsx! { UserAccessMobileCards { user_access: vec![user] } }
        }
        let mut vdom = dioxus::prelude::VirtualDom::new(harness);
        vdom.rebuild_in_place();
        let html = dioxus_ssr::render(&vdom);
        assert!(html.contains("No Plan"), "UserAccessMobileCards must render No Plan label. Got: {html}");
    }

    /// `UserAccessMobileCards` shows "Never" when plan_expires_at is None.
    #[test]
    fn user_access_mobile_cards_renders_never_when_no_expiry() {
        fn harness() -> Element {
            let mut user = sample_user();
            user.plan_expires_at = None;
            rsx! { UserAccessMobileCards { user_access: vec![user] } }
        }
        let mut vdom = dioxus::prelude::VirtualDom::new(harness);
        vdom.rebuild_in_place();
        let html = dioxus_ssr::render(&vdom);
        assert!(html.contains("Never"), "UserAccessMobileCards must render Never when no expiry. Got: {html}");
    }

    /// `UserAccessPaginationBar` renders Prev / Next + page indicator.
    #[test]
    fn user_access_pagination_bar_renders() {
        fn harness() -> Element {
            rsx! { UserAccessPaginationBar { current: 2u32, disable_next: Some(false), on_prev: move |_: ()| {}, on_next: move |_: ()| {} } }
        }
        let mut vdom = dioxus::prelude::VirtualDom::new(harness);
        vdom.rebuild_in_place();
        let html = dioxus_ssr::render(&vdom);
        assert!(html.contains("Previous"), "UserAccessPaginationBar must render Previous. Got: {html}");
        assert!(html.contains("Next"), "UserAccessPaginationBar must render Next. Got: {html}");
        assert!(html.contains("Page 2"), "UserAccessPaginationBar must render Page indicator. Got: {html}");
    }

    /// `user_access_status_class` returns the expected class per status.
    #[test]
    fn user_access_status_class_matches_source() {
        assert_eq!(user_access_status_class("active"), "bg-success/10 text-success border border-success/20");
        assert_eq!(user_access_status_class("expiring_soon"), "bg-warning/10 text-warning border border-warning/20");
        assert_eq!(user_access_status_class("expired"), "bg-destructive/10 text-destructive border border-destructive/20");
        assert_eq!(user_access_status_class("unknown"), "bg-muted text-muted-foreground border border-border/50");
    }
}
