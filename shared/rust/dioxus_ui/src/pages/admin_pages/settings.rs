//! /admin/settings — system settings (config + API keys + email +
//! notifications + sessions).
//!
//! Wave 6C Track C — thin composition of the 5 named sub-components
//! extracted into `crate::components::admin::settings`. The 5 sub-
//! components (SettingsDashboard, ApiKeysList, EmailSettings,
//! NotificationSettings, SessionManagement) live in
//! `components/admin/settings.rs`. The `MaintenanceCard` is a bonus
//! (5th in the page surface) and stays page-local.

use crate::auth::AdminAuthGate;
use crate::components::admin::settings::{
    ApiKeysList, EmailSettings, NotificationSettings, SessionManagement, SettingsDashboard,
};
use crate::layout::admin_shell::AdminShell;
use crate::primitives::*;

use dioxus::prelude::*;
use super::super::{PageContext, PageMeta};

pub fn render(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::admin("Settings");
    (meta, rsx! { RenderSettings { ctx: ctx.clone() } })
}

#[component]
fn RenderSettings(ctx: PageContext) -> Element {
    rsx! {
        AdminAuthGate { user: ctx.user.clone(), feature: Some("platform settings".to_string()), required_permissions: Some(vec!["settings:manage".to_string()]), return_url: Some(ctx.path.clone()),
            AdminShell {
                ctx: ctx.clone(),
                page_title: "Settings".to_string(),
                breadcrumbs: vec![
                    ("Dashboard".to_string(), "/".to_string()),
                    ("Settings".to_string(), "/settings".to_string()),
                ],
                div { class: "container page-content admin-settings",
                    SettingsDashboard {}
                    // 2-col grid of section cards.
                    div { class: "grid grid-cols-1 lg:grid-cols-2 gap-4 mt-4",
                        // Left col: EmailSettings + NotificationSettings.
                        EmailSettings {}
                        NotificationSettings {}
                        // Right col: ApiKeysList + SessionManagement.
                        ApiKeysList {}
                        SessionManagement {}
                        // Maintenance card (bonus, lives in the same surface).
                        MaintenanceCard {}
                    }
                }
            }
        }
    }
}

// ===== MaintenanceCard =====================================================
//
// Bonus card — the TS source's `Maintenance Lock` toggle. Lives in the
// settings surface as the "5th section" alongside the 5 design-doc-named
// sub-components.

#[component]
fn MaintenanceCard() -> Element {
    rsx! {
        div { class: "card card-glass maintenance-card",
            div { class: "card-header",
                div { class: "card-icon", Icon { name: "settings".to_string(), size: Some(20) } }
                div { class: "flex-1",
                    h3 { class: "card-title", "Maintenance" }
                    p { class: "card-description text-xs text-muted-foreground", "Maintenance mode (show banner to users)" }
                }
            }
            div { class: "card-body space-y-4",
                div { class: "flex items-center justify-between p-3 rounded-xl bg-muted/30 border border-border/40",
                    div {
                        span { class: "text-sm font-bold", "Maintenance mode" }
                        p { class: "text-[10px] font-bold text-muted-foreground uppercase opacity-50 mt-0.5", "Isolate network from public operations" }
                    }
                    SettingsToggleSwitch { on: false }
                }
                div { class: "field",
                    label { class: "field-label", "Maintenance message" }
                    textarea { class: "input", name: "message", rows: "3", placeholder: "We'll be back shortly…" }
                }
                FormActions {
                    button { class: "btn btn-primary", r#type: "submit", "Save" }
                }
            }
        }
    }
}

/// Local toggle switch — the NotificationSettings sub-component
/// reuses its own internal `ToggleSwitch`; this one is the bonus
/// maintenance surface so we duplicate the (small) implementation
/// here to avoid leaking the helper.
#[component]
fn SettingsToggleSwitch(on: bool) -> Element {
    let bg = if on { "bg-primary" } else { "bg-muted/30" };
    let pos = if on { "translate-x-[40px]" } else { "translate-x-0" };
    rsx! {
        button {
            class: format!("relative w-20 h-10 rounded-full transition-all duration-300 {bg}"),
            r#type: "button",
            "aria-pressed": "{on}",
            div { class: format!("absolute top-1.5 left-1.5 w-7 h-7 rounded-full bg-white transition-transform duration-300 {pos}") }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::user::{AuthMethod, User};

    /// Authenticated admin context — the page gates on
    /// `settings:manage`, so the fixture user must hold that
    /// permission.
    fn admin_ctx() -> PageContext {
        PageContext {
            user: Some(User {
                id: "u-admin".to_string(),
                address: "0x1234abcd5678ef90".to_string(),
                chain_id: "56".to_string(),
                roles: vec!["admin".to_string()],
                email: Some("admin@epsx.io".to_string()),
                tier: Some("Admin".to_string()),
                permissions: vec!["settings:manage".to_string()],
                last_login_at: None,
                auth_method: AuthMethod::Wallet,
                display_name: Some("Admin".to_string()),
            }),
            path: "/settings".to_string(),
            ..Default::default()
        }
    }

    /// Smoke test.
    #[test]
    fn test_render_smoke() {
        let ctx = admin_ctx();
        let (_meta, el) = render(&ctx);
        let html = dioxus_ssr::render_element(el);
        assert!(!html.trim().is_empty(), "settings must render non-empty HTML. Got: {html}");
        assert!(html.len() > 100, "settings HTML is suspiciously short ({} bytes).", html.len());
    }

    /// Section-marker test.
    #[test]
    fn test_section_markers() {
        let ctx = admin_ctx();
        let (_meta, el) = render(&ctx);
        let html = dioxus_ssr::render_element(el);
        for marker in &[
            "settings-dashboard",
            "api-keys-list",
            "email-settings",
            "notification-settings",
            "session-management",
        ] {
            let needle_a = format!("class=\"{}\"", marker);
            let needle_b = format!("class=\"{mark} ", mark = marker);
            let needle_c = format!(" {}\"", marker);
            let needle_d = format!(" {} ", marker);
            assert!(
                html.contains(&needle_a)
                    || html.contains(&needle_b)
                    || html.contains(&needle_c)
                    || html.contains(&needle_d),
                "settings must contain section marker `{marker}`. Got: {html}"
            );
        }
    }
}
