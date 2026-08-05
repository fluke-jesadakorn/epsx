//! Denial panels for the admin `/access-denied` and `/unauthorized` routes.
//!
//! The pinned Next.js source reflects `route`, `reason`, `context`,
//! `permission`, and `detail` query values as if they were authoritative
//! denial data. Query parameters are browser-controlled, so this Rust surface
//! deliberately ignores all of them. Only static copy may be rendered until a
//! denial reason is supplied by an authenticated backend response.

use super::super::{PageContext, PageMeta, PageStatus};
use crate::primitives::icon::Icon;
use dioxus::prelude::*;

const DEFAULT_REASON: &str = "You don't have permission to access this resource.";
const ADMIN_REASON: &str = "You don't have permission to access the admin panel. Please contact your administrator if you believe this is an error.";
const ADMIN_DESCRIPTION: &str =
    "Administrative interface for EPSX data analytics platform - User management and system monitoring";
const ADMIN_KEYWORDS: &str = "EPSX, admin, analytics, user management, dashboard";
const DENIAL_BODY_CLASS: &str =
    "__variable_a460b5 h-screen bg-background text-foreground overflow-hidden font-sans";
const DENIAL_INLINE_CSS: &str = r#"
/* The Tailwind CDN does not emit slash-opacity utilities used by the source
 * panel. Keep those visual tokens local to the denial surface instead of
 * changing the shared border defaults used by the rest of admin. */
.admin-denial-runtime-root .border-border\/20 {
  border-color: rgba(148, 163, 184, 0.20) !important;
}
.admin-denial-runtime-root .bg-muted\/30 {
  background-color: rgba(148, 163, 184, 0.12) !important;
}
"#;

#[derive(Clone, Debug, PartialEq, Eq)]
struct DenialModel {
    reason: String,
    show_admin_guidance: bool,
}

/// Render the source denial component. The API-key-create route still uses the
/// historical static denial copy, but is not claimed as A8-aligned by this
/// package because its source is a mutation form rather than a denial page.
pub fn render(ctx: &PageContext) -> (PageMeta, Element) {
    let model = match ctx.path.as_str() {
        "/access-denied" => DenialModel {
            reason: DEFAULT_REASON.to_string(),
            show_admin_guidance: false,
        },
        "/unauthorized" | "/developer-portal/api-keys/create" => DenialModel {
            reason: ADMIN_REASON.to_string(),
            show_admin_guidance: true,
        },
        _ => DenialModel {
            reason: DEFAULT_REASON.to_string(),
            show_admin_guidance: false,
        },
    };

    let meta = denial_meta();
    (
        meta,
        rsx! {
            AccessDeniedPanelInner { model }
        },
    )
}

fn denial_meta() -> PageMeta {
    PageMeta {
        // Both source pages inherit the root admin metadata; neither declares
        // route-owned metadata.
        title: "EPSX Admin".to_string(),
        description: ADMIN_DESCRIPTION.to_string(),
        keywords: Some(ADMIN_KEYWORDS.to_string()),
        status: PageStatus::Ok,
        body_class: Some(DENIAL_BODY_CLASS.to_string()),
        include_footer: false,
        use_epsx_header: false,
    }
}

#[component]
fn AccessDeniedPanelInner(model: DenialModel) -> Element {
    rsx! {
        style { "{DENIAL_INLINE_CSS}" }
        div {
            class: "fixed inset-0 z-[-1] overflow-hidden pointer-events-none",
            "aria-hidden": "true",
            div { class: "absolute inset-0 bg-gradient-to-br from-background via-muted to-background" }
            div { class: "absolute inset-0 bg-grid-pattern opacity-0 dark:opacity-40" }
            div {
                class: "absolute -top-[10%] -left-[10%] w-[40%] h-[40%] rounded-full blur-[120px] animate-pulse",
                "data-admin-denial-orb": "primary",
                style: "background:rgba(59,130,246,0.10);",
            }
            div {
                class: "absolute top-[20%] -right-[5%] w-[30%] h-[30%] rounded-full blur-[100px]",
                "data-admin-denial-orb": "cyan",
                style: "background:rgba(31,199,212,0.05);",
            }
            div {
                class: "absolute -bottom-[10%] left-[20%] w-[50%] h-[50%] rounded-full blur-[150px] animate-pulse",
                "data-admin-denial-orb": "pink",
                style: "background:rgba(237,75,158,0.05);",
            }
        }
        div {
            class: "admin-denial-runtime-root flex h-screen flex-col overflow-y-auto overflow-x-hidden relative z-0 bg-background",
            "data-admin-denial-runtime": "true",
            section {
                class: "flex flex-col items-center justify-start min-h-full p-6 sm:p-8 lg:p-12",
                role: "alert",
                "aria-labelledby": "admin-denial-title",
                div {
                    class: "w-full max-w-lg",
                    div {
                        class: "flex justify-center mb-6",
                        div {
                            class: "w-20 h-20 sm:w-24 sm:h-24 bg-gradient-to-br from-red-500 to-red-600 rounded-3xl flex items-center justify-center border-2 border-red-400/30 shadow-lg shadow-red-500/30",
                            "aria-hidden": "true",
                            Icon {
                                name: "shield-x".to_string(),
                                size: Some(40),
                                class_name: Some("lucide lucide-shield-x w-10 h-10 sm:w-12 sm:h-12 text-white".to_string()),
                            }
                        }
                    }
                    div {
                        class: "text-center mb-6",
                        h1 {
                            id: "admin-denial-title",
                            class: "text-2xl sm:text-3xl font-bold text-foreground mb-2",
                            "Access Denied"
                        }
                        p { class: "text-base sm:text-lg text-muted-foreground break-words", "{model.reason}" }
                    }
                    div {
                        class: "bg-muted/30 rounded-2xl border border-border/20 shadow-lg overflow-hidden mb-6",
                        "aria-label": "Error details",
                        div {
                            class: "p-6",
                            h3 {
                                class: "text-sm font-semibold text-foreground mb-4 flex items-center gap-2",
                                Icon {
                                    name: "triangle-alert".to_string(),
                                    size: Some(16),
                                    class_name: Some("lucide lucide-triangle-alert w-4 h-4 text-destructive".to_string()),
                                }
                                "Error Details"
                            }
                            div {
                                class: "space-y-3 text-sm",
                                p { class: "text-muted-foreground",
                                    "Access is determined from your authenticated session and backend permissions. URL parameters cannot grant access or change this message."
                                }
                            }
                        }
                        if model.show_admin_guidance {
                            div { class: "border-t border-border/20 bg-gradient-to-r from-purple-500/10 to-orange-500/10 p-4",
                                p { class: "text-sm text-foreground",
                                    span { class: "font-medium", "Admin Access Required:" }
                                    " Only authorized administrators can access this panel. Contact your system administrator if you believe this is an error."
                                }
                            }
                        }
                    }
                    nav {
                        class: "flex flex-col sm:flex-row gap-3",
                        "aria-label": "Access denied actions",
                        a {
                            href: "/auth?return_url=%2F",
                            "data-admin-denial-auth": "true",
                            "data-epsx-logout": "true",
                            "data-epsx-logout-target": "/auth?return_url=%2F",
                            class: "flex-1 inline-flex items-center justify-center gap-2 px-6 py-4 bg-gradient-to-r from-red-500 to-red-600 text-white rounded-2xl font-semibold shadow-lg shadow-red-500/20 hover:shadow-xl hover:shadow-red-500/30 hover-lift transition-all",
                            Icon {
                                name: "rotate-ccw".to_string(),
                                size: Some(20),
                                class_name: Some("lucide lucide-rotate-ccw w-5 h-5".to_string()),
                            }
                            "Go to Auth"
                        }
                        a {
                            href: "/",
                            "data-admin-denial-back": "true",
                            "data-epsx-action": "back",
                            class: "flex-1 inline-flex items-center justify-center gap-2 px-6 py-4 bg-muted/30 border border-border/20 text-foreground rounded-2xl font-semibold hover:bg-muted/50 transition-colors",
                            Icon {
                                name: "arrow-left".to_string(),
                                size: Some(20),
                                class_name: Some("lucide lucide-arrow-left w-5 h-5".to_string()),
                            }
                            "Go Back"
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn render_path(path: &str, query: &str) -> (PageMeta, String) {
        let ctx = PageContext {
            path: path.to_string(),
            query: query.to_string(),
            ..Default::default()
        };
        let (meta, element) = render(&ctx);
        (meta, dioxus_ssr::render_element(element))
    }

    #[test]
    fn denial_routes_preserve_source_copy_metadata_and_semantics() {
        let (access_meta, access) = render_path("/access-denied", "");
        assert_eq!(access_meta.title, "EPSX Admin");
        assert_eq!(access_meta.description, ADMIN_DESCRIPTION);
        assert_eq!(access_meta.keywords.as_deref(), Some(ADMIN_KEYWORDS));
        assert_eq!(access_meta.status, PageStatus::Ok);
        assert_eq!(access.matches("<h1").count(), 1);
        assert!(
            access.contains("You don&#39;t have permission to access this resource."),
            "{access}"
        );
        assert!(access.contains("role=\"alert\""));
        assert!(access.contains("aria-label=\"Access denied actions\""));

        let (_, unauthorized) = render_path("/unauthorized", "reason=ignored&route=%2Fevil");
        assert!(unauthorized.contains("contact your administrator"));
        assert!(!unauthorized.contains("Requested Route:"));
        assert!(!unauthorized.contains("ignored"));
    }

    #[test]
    fn access_denied_ignores_all_query_controlled_authority_claims() {
        let query = concat!(
            "reason=Denied+%253Cscript+data-probe%253Ealert%25281%2529%253C%252Fscript%253E",
            "&route=%252Fpayments%253Ftab%253Dhistory%2526probe%253D%253Cimg%253E",
            "&context=admin",
            "&permission=admin%253Apayments%253Aread%253Csvg%253E",
            "&detail=backend+%253Cb%253Esecret%253C%252Fb%253E"
        );
        let (_, html) = render_path("/access-denied", query);

        assert!(html.contains("You don&#39;t have permission to access this resource."));
        assert!(html.contains("backend permissions"));
        assert!(html.contains("URL parameters cannot grant access"));
        for untrusted_claim in [
            "data-probe",
            "/payments?tab=history",
            "admin:payments:read",
            "backend &#60;b&#62;secret",
            "Admin Access Required:",
            "Requested Route:",
            "Required Permission:",
            "Backend Detail:",
        ] {
            assert!(
                !html.contains(untrusted_claim),
                "query-controlled denial claim leaked: {untrusted_claim}"
            );
        }
        assert!(html.contains("href=\"/auth?return_url=%2F\""));
        assert!(html.contains("href=\"/\" data-admin-denial-back=\"true\""));
    }

    #[test]
    fn query_controlled_return_targets_always_fail_closed() {
        for untrusted_target in [
            "https://evil.example",
            "//evil.example/path",
            "/\\evil.example",
            "/auth",
            "/auth/continue",
            "/section/../auth",
            "/access-denied?loop=1",
            "/api/v1/auth/logout",
            "/payments?tab=history#latest",
            "",
        ] {
            let (_, html) = render_path(
                "/access-denied",
                &format!("route={untrusted_target}&reason={untrusted_target}"),
            );
            assert!(html.contains("href=\"/auth?return_url=%2F\""));
            assert!(html.contains("href=\"/\" data-admin-denial-back=\"true\""));
            assert!(!html.contains("evil.example"));
            assert!(!html.contains("tab=history"));
            assert!(!html.contains("javascript:"));
        }
    }

    #[test]
    fn denial_layout_is_scrollable_and_theme_aware() {
        let (meta, html) = render_path("/access-denied", "");
        let body_class = meta.body_class.unwrap();
        assert!(body_class.contains("h-screen"));
        assert!(body_class.contains("overflow-hidden"));
        assert!(html.contains("overflow-y-auto overflow-x-hidden"));
        assert!(html.contains("bg-background"));
        assert!(!html.contains("background:rgb(30,35,48)"));
        assert!(html.contains("data-admin-denial-orb=\"primary\""));
        assert!(html.contains("background:rgba(59,130,246,0.10)"));
    }
}
