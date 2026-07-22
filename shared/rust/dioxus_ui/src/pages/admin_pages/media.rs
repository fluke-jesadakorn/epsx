//! `/media` — authenticated admin media workspace.
//!
//! The Rust admin does not yet have a backend-authoritative media inventory or
//! mutation contract. This route therefore keeps the page-owned admin shell
//! while rendering an explicit unavailable state. It does not infer storage
//! totals, render sample objects, trust legacy parameters, or expose upload,
//! view, copy, filter, and delete controls.

use dioxus::prelude::*;

use crate::auth::AuthGate;
use crate::layout::admin_shell::AdminShell;
use crate::primitives::Icon;

use super::super::{PageContext, PageMeta};

pub fn render(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::admin("Media unavailable");
    (meta, rsx! { RenderMedia { ctx: ctx.clone() } })
}

/// Keep the media workspace private without treating frontend roles or
/// capabilities as policy authority. Query and route parameters are
/// intentionally ignored: they cannot supply storage objects, bucket names,
/// filters, totals, or authorization decisions.
#[component]
fn RenderMedia(ctx: PageContext) -> Element {
    rsx! {
        AuthGate {
            user: ctx.user.clone(),
            feature: Some("the private media workspace".to_string()),
            return_url: Some("/media".to_string()),
            AdminShell {
                ctx: ctx.clone(),
                page_title: "Media".to_string(),
                breadcrumbs: vec![
                    ("Dashboard".to_string(), "/".to_string()),
                    ("Media".to_string(), "/media".to_string()),
                ],
                div {
                    class: "container page-content admin-media py-8",
                    "data-admin-media-state": "unavailable",
                    div { class: "grid gap-6 xl:grid-cols-[minmax(0,1.7fr)_minmax(18rem,0.8fr)]",
                        section {
                            class: "relative overflow-hidden rounded-3xl border border-border/40 bg-card shadow-2xl",
                            role: "status",
                            aria_labelledby: "admin-media-unavailable-title",
                            "data-section": "admin-media-unavailable",
                            div { class: "absolute inset-x-0 top-0 h-1 bg-gradient-to-r from-[#1fc7d4] via-[#7645d9] to-[#ffb237]" }
                            div { class: "p-8 md:p-12",
                                div { class: "flex flex-col gap-6 sm:flex-row sm:items-start",
                                    div {
                                        class: "flex h-16 w-16 shrink-0 items-center justify-center rounded-2xl border border-cyan-500/20 bg-cyan-500/10 text-[#1fc7d4]",
                                        aria_hidden: "true",
                                        Icon { name: "image".to_string(), size: Some(30) }
                                    }
                                    div { class: "min-w-0",
                                        p { class: "text-xs font-black uppercase tracking-[0.22em] text-[#1fc7d4]",
                                            "Storage workspace"
                                        }
                                        h1 {
                                            id: "admin-media-unavailable-title",
                                            class: "mt-3 text-3xl font-black tracking-tight text-foreground",
                                            "Media storage is unavailable"
                                        }
                                        p { class: "mt-4 max-w-3xl text-sm leading-6 text-muted-foreground",
                                            "No files, object names, sizes, upload times, buckets, storage totals, or previews are shown because a verified media inventory is not connected. An unavailable inventory is not presented as an empty one."
                                        }
                                        nav { class: "mt-8 flex flex-wrap gap-3", aria_label: "Media recovery",
                                            a { class: "btn btn-primary", href: "/media",
                                                Icon { name: "refresh-cw".to_string(), size: Some(16) }
                                                " Check again"
                                            }
                                            a { class: "btn btn-outline", href: "/", "Admin home" }
                                        }
                                    }
                                }
                            }
                        }

                        aside {
                            class: "rounded-3xl border border-border/40 bg-card/70 p-6",
                            aria_labelledby: "admin-media-contract-title",
                            "data-section": "admin-media-backend-contract",
                            h2 {
                                id: "admin-media-contract-title",
                                class: "text-sm font-bold text-foreground",
                                "Backend media contract required"
                            }
                            p { class: "mt-3 text-sm leading-6 text-muted-foreground",
                                "The backend must own authenticated inventory reads, dedicated media authorization, storage-provider access, bounded pagination, and validated upload and deletion workflows before operations can be enabled."
                            }
                            p { class: "mt-4 text-xs leading-5 text-muted-foreground",
                                "Frontend session roles and capabilities are not used to grant media access or derive storage policy."
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
    use crate::auth::user::{AuthMethod, User};

    fn signed_in_ctx() -> PageContext {
        PageContext {
            user: Some(User {
                id: "admin-1".to_string(),
                address: "0x1234abcd5678ef90".to_string(),
                chain_id: "56".to_string(),
                roles: vec![],
                email: None,
                tier: None,
                permissions: vec![],
                last_login_at: None,
                auth_method: AuthMethod::Wallet,
                display_name: None,
            }),
            path: "/media".to_string(),
            ..Default::default()
        }
    }

    fn html(ctx: &PageContext) -> String {
        let (_, element) = render(ctx);
        dioxus_ssr::render_element(element)
    }

    #[test]
    fn signed_out_route_keeps_media_state_private() {
        let rendered = html(&PageContext {
            path: "/media".to_string(),
            ..Default::default()
        });

        assert!(rendered.contains("Sign in required"));
        assert!(rendered.contains("href=\"/auth?return_url=%2Fmedia\""));
        assert!(!rendered.contains("data-admin-media-state"));
        assert!(!rendered.contains("Media storage is unavailable"));
    }

    #[test]
    fn role_empty_authenticated_session_reaches_explicit_unavailable_state() {
        let rendered = html(&signed_in_ctx());

        assert!(rendered.contains("data-admin-media-state=\"unavailable\""));
        assert!(rendered.contains("Media storage is unavailable"));
        assert!(rendered.contains("Backend media contract required"));
        assert!(!rendered.contains("Permission required"));
    }

    #[test]
    fn hostile_and_legacy_params_cannot_create_media_claims() {
        let mut ctx = signed_in_ctx();
        ctx.query = "bucket=news&total=1247&size=847MB&view=grid".to_string();
        ctx.params = HashMap::from([
            ("filename".to_string(), "private-report.pdf".to_string()),
            ("oldest".to_string(), "2024-01-15".to_string()),
        ]);
        let rendered = html(&ctx);

        for forbidden in [
            "private-report.pdf",
            "2024-01-15",
            "Bucket: news",
            "1,247",
            "847 MB",
        ] {
            assert!(
                !rendered.contains(forbidden),
                "hostile or legacy media value leaked: {forbidden}"
            );
        }
    }

    #[test]
    fn unavailable_workspace_suppresses_samples_and_media_controls() {
        let rendered = html(&signed_in_ctx());

        for forbidden in [
            "news_2024-09-20_banner.png",
            "chat_avatar_001.png",
            "public_whitepaper.pdf",
            "Total files",
            "Total size",
            "Newest upload",
            "Oldest upload",
            "Drop files here",
            "Upload",
            "Copy URL",
            "Delete",
            "Grid view",
            "List view",
            "All types",
            "Filter files",
            "<form",
            "<input",
            "<select",
        ] {
            assert!(
                !rendered.contains(forbidden),
                "sample or inert media control leaked: {forbidden}"
            );
        }
    }

    #[test]
    fn page_owns_one_admin_shell_and_safe_native_recovery() {
        let rendered = html(&signed_in_ctx());

        assert_eq!(
            rendered
                .matches("class=\"admin-shell admin-shell-page\"")
                .count(),
            1
        );
        assert!(rendered.contains("class=\"admin-shell-main\""));
        assert!(rendered.contains("href=\"/media\""));
        assert!(rendered.contains("href=\"/\""));
        assert!(!rendered.contains("javascript:"));
        assert!(!rendered.contains("onclick="));
    }
}
