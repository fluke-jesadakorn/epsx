//! /admin/media — media browser + uploader.
//!
//! Wave 6C Track C — thin composition of the 4 named sub-components
//! extracted into `crate::components::admin::media`. The 4 sub-
//! components (MediaBrowser, MediaUploader, MediaFilters, MediaStats)
//! live in `components/admin/media.rs`.

use crate::auth::AdminAuthGate;
use crate::components::admin::media::{MediaBrowser, MediaFilters, MediaStats, MediaUploader};
use crate::layout::admin_shell::AdminShell;

use dioxus::prelude::*;
use super::super::{PageContext, PageMeta};

pub fn render(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::admin("Media");
    (meta, rsx! { RenderMedia { ctx: ctx.clone() } })
}

#[component]
fn RenderMedia(ctx: PageContext) -> Element {
    let mut bucket = use_signal(|| "news".to_string());
    let mut view = use_signal(|| "grid".to_string());
    rsx! {
        AdminAuthGate { user: ctx.user.clone(), feature: Some("media management".to_string()), required_permissions: Some(vec!["media:manage".to_string()]), return_url: Some(ctx.path.clone()),
            AdminShell {
                ctx: ctx.clone(),
                page_title: "Media Browser".to_string(),
                breadcrumbs: vec![
                    ("Dashboard".to_string(), "/".to_string()),
                    ("Media".to_string(), "/media".to_string()),
                ],
                div { class: "container page-content admin-media",
                    MediaStats {}
                    MediaUploader {}
                    MediaFilters { bucket: bucket.read().clone(), view: view.read().clone() }
                    MediaBrowser { bucket: bucket.read().clone(), view: view.read().clone() }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::user::{AuthMethod, User};

    /// Authenticated admin context — the page gates on
    /// `media:manage`, so the fixture user must hold that permission.
    fn admin_ctx() -> PageContext {
        PageContext {
            user: Some(User {
                id: "u-admin".to_string(),
                address: "0x1234abcd5678ef90".to_string(),
                chain_id: "56".to_string(),
                roles: vec!["admin".to_string()],
                email: Some("admin@epsx.io".to_string()),
                tier: Some("Admin".to_string()),
                permissions: vec!["media:manage".to_string()],
                last_login_at: None,
                auth_method: AuthMethod::Wallet,
                display_name: Some("Admin".to_string()),
            }),
            path: "/media".to_string(),
            ..Default::default()
        }
    }

    /// Smoke test.
    #[test]
    fn test_render_smoke() {
        let ctx = admin_ctx();
        let (_meta, el) = render(&ctx);
        let html = dioxus_ssr::render_element(el);
        assert!(!html.trim().is_empty(), "media must render non-empty HTML. Got: {html}");
        assert!(html.len() > 100, "media HTML is suspiciously short ({} bytes).", html.len());
    }

    /// Section-marker test.
    #[test]
    fn test_section_markers() {
        let ctx = admin_ctx();
        let (_meta, el) = render(&ctx);
        let html = dioxus_ssr::render_element(el);
        for marker in &[
            "media-browser",
            "media-uploader",
            "media-filters",
            "media-stats",
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
                "media must contain section marker `{marker}`. Got: {html}"
            );
        }
    }
}
