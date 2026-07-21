//! Source-parity denial panels for the admin `/access-denied` and
//! `/unauthorized` routes.
//!
//! The pinned Next.js source uses the same `AccessDeniedContent` component for
//! both routes. `/access-denied` additionally accepts `route`, `reason`,
//! `context`, `permission`, and `detail` query values. This SSR port restores
//! that contract while bounding and control-filtering every decoded value.
//! Dioxus owns HTML escaping; query data is never interpolated into markup or
//! script source.

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

#[derive(Clone, Debug, PartialEq, Eq)]
struct DenialModel {
    reason: String,
    route: Option<String>,
    context: Option<String>,
    permission: Option<String>,
    detail: Option<String>,
    safe_return_target: String,
}

/// Render the source denial component. The API-key-create route still uses the
/// historical static denial copy, but is not claimed as A8-aligned by this
/// package because its source is a mutation form rather than a denial page.
pub fn render(ctx: &PageContext) -> (PageMeta, Element) {
    let model = match ctx.path.as_str() {
        "/access-denied" => access_denied_model(ctx),
        "/unauthorized" | "/developer-portal/api-keys/create" => DenialModel {
            reason: ADMIN_REASON.to_string(),
            route: None,
            context: None,
            permission: None,
            detail: None,
            safe_return_target: "/".to_string(),
        },
        _ => DenialModel {
            reason: DEFAULT_REASON.to_string(),
            route: None,
            context: None,
            permission: None,
            detail: None,
            safe_return_target: "/".to_string(),
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

fn access_denied_model(ctx: &PageContext) -> DenialModel {
    // `useSearchParams()` performs the URL-form decode. The source then calls
    // `decodeURIComponent()` once more for reason/detail/route/permission;
    // `context` is intentionally decoded only once. Invalid percent escapes
    // remain literal here instead of crashing the whole client render.
    let reason = source_query_value(ctx, "reason", 240, true)
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| DEFAULT_REASON.to_string());
    let route = source_query_value(ctx, "route", 256, true).filter(|value| !value.is_empty());
    let context = source_query_value(ctx, "context", 64, false).filter(|value| !value.is_empty());
    let permission =
        source_query_value(ctx, "permission", 160, true).filter(|value| !value.is_empty());
    let detail = source_query_value(ctx, "detail", 240, true).filter(|value| !value.is_empty());
    let safe_return_target = safe_return_target(route.as_deref()).to_string();

    DenialModel {
        reason,
        route,
        context,
        permission,
        detail,
        safe_return_target,
    }
}

fn source_query_value(
    ctx: &PageContext,
    key: &str,
    max_chars: usize,
    source_decodes_again: bool,
) -> Option<String> {
    let raw = ctx.query_param(key)?;
    let once = decode_query_text(&raw, max_chars);
    Some(if source_decodes_again {
        decode_query_text(&once, max_chars)
    } else {
        once
    })
}

/// URL-form decode with bounded output. `+` becomes a space, valid `%HH`
/// escapes are decoded, malformed escapes stay literal, and control bytes are
/// removed before the character limit is applied.
fn decode_query_text(value: &str, max_chars: usize) -> String {
    let bytes = value.as_bytes();
    let byte_limit = max_chars.saturating_mul(4).max(max_chars);
    let mut decoded = Vec::with_capacity(bytes.len().min(byte_limit));
    let mut index = 0;
    while index < bytes.len() && decoded.len() < byte_limit {
        match bytes[index] {
            b'+' => {
                decoded.push(b' ');
                index += 1;
            }
            b'%' if index + 2 < bytes.len() => {
                if let (Some(high), Some(low)) =
                    (hex_value(bytes[index + 1]), hex_value(bytes[index + 2]))
                {
                    decoded.push((high << 4) | low);
                    index += 3;
                } else {
                    decoded.push(bytes[index]);
                    index += 1;
                }
            }
            byte => {
                decoded.push(byte);
                index += 1;
            }
        }
    }
    String::from_utf8_lossy(&decoded)
        .chars()
        .filter(|character| !character.is_control())
        .take(max_chars)
        .collect()
}

fn hex_value(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

/// Accept only a local path as a post-auth/history fallback. Auth, denial,
/// browser-internal, static, and API targets are rejected so the page cannot
/// create a loop or turn a display-only query field into an open redirect.
fn safe_return_target(candidate: Option<&str>) -> &str {
    let Some(candidate) = candidate.map(str::trim) else {
        return "/";
    };
    if candidate.is_empty()
        || !candidate.starts_with('/')
        || candidate.starts_with("//")
        || candidate.contains('\\')
        || candidate.chars().any(char::is_control)
    {
        return "/";
    }
    let path = candidate
        .split_once(['?', '#'])
        .map(|(path, _)| path)
        .unwrap_or(candidate);
    if path.split('/').any(|segment| matches!(segment, "." | "..")) {
        return "/";
    }
    if [
        "/.well-known",
        "/_next",
        "/api",
        "/auth",
        "/login",
        "/access-denied",
        "/unauthorized",
        "/favicon",
        "/public",
        "/static",
    ]
    .iter()
    .any(|prefix| path == *prefix || path.starts_with(&format!("{prefix}/")))
    {
        "/"
    } else {
        candidate
    }
}

fn url_encode_query_value(value: &str) -> String {
    let mut output = String::with_capacity(value.len());
    for byte in value.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                output.push(byte as char)
            }
            _ => output.push_str(&format!("%{byte:02X}")),
        }
    }
    output
}

#[component]
fn AccessDeniedPanelInner(model: DenialModel) -> Element {
    let auth_href = format!(
        "/auth?return_url={}",
        url_encode_query_value(&model.safe_return_target)
    );
    let context_is_admin = model.context.as_deref() == Some("admin");

    rsx! {
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
                class: "flex flex-col items-center justify-center min-h-full p-6 sm:p-8 lg:p-12",
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
                                if let Some(route) = model.route.as_ref() {
                                    div { class: "flex justify-between items-start gap-4",
                                        span { class: "text-muted-foreground shrink-0", "Requested Route:" }
                                        code { class: "text-foreground bg-muted/30 border border-border/20 px-2 py-1 rounded text-right break-all min-w-0", "{route}" }
                                    }
                                }
                                if let Some(context) = model.context.as_ref() {
                                    div { class: "flex justify-between items-center gap-4",
                                        span { class: "text-muted-foreground", "Context:" }
                                        span { class: "text-foreground capitalize break-all text-right min-w-0", "{context}" }
                                    }
                                }
                                if let Some(permission) = model.permission.as_ref() {
                                    div { class: "flex justify-between items-start gap-4",
                                        span { class: "text-muted-foreground shrink-0", "Required Permission:" }
                                        code { class: "text-foreground bg-muted/30 border border-border/20 px-2 py-1 rounded text-right break-all min-w-0", "{permission}" }
                                    }
                                }
                                if let Some(detail) = model.detail.as_ref() {
                                    div { class: "flex justify-between items-start gap-4 border-t border-border/20 pt-3 mt-1",
                                        span { class: "text-muted-foreground shrink-0", "Backend Detail:" }
                                        span { class: "text-foreground text-right break-words min-w-0", "{detail}" }
                                    }
                                }
                            }
                        }
                        if context_is_admin {
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
                            href: "{auth_href}",
                            "data-admin-denial-auth": "true",
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
    fn access_denied_decodes_bounds_and_escapes_all_source_query_fields() {
        let query = concat!(
            "reason=Denied+%253Cscript+data-probe%253Ealert%25281%2529%253C%252Fscript%253E",
            "&route=%252Fpayments%253Ftab%253Dhistory%2526probe%253D%253Cimg%253E",
            "&context=admin",
            "&permission=admin%253Apayments%253Aread%253Csvg%253E",
            "&detail=backend+%253Cb%253Esecret%253C%252Fb%253E"
        );
        let (_, html) = render_path("/access-denied", query);

        assert!(html.contains("Denied &#60;script data-probe&#62;alert(1)&#60;/script&#62;"));
        assert!(html.contains("/payments?tab=history&#38;probe=&#60;img&#62;"));
        assert!(html.contains("admin"));
        assert!(html.contains("admin:payments:read&#60;svg&#62;"));
        assert!(html.contains("backend &#60;b&#62;secret&#60;/b&#62;"));
        assert!(html.contains("Admin Access Required:"));
        assert!(!html.contains("<script data-probe>"));
        assert!(!html.contains("<img>"));
        assert!(!html.contains("<svg>"));
        assert!(!html.contains("<b>secret</b>"));
        assert!(html
            .contains("href=\"/auth?return_url=%2Fpayments%3Ftab%3Dhistory%26probe%3D%3Cimg%3E\""));
        assert!(html.contains("href=\"/\" data-admin-denial-back=\"true\""));

        let long = format!("reason={}&detail={}", "x".repeat(600), "y".repeat(600));
        let model = access_denied_model(&PageContext {
            path: "/access-denied".to_string(),
            query: long,
            ..Default::default()
        });
        assert_eq!(model.reason.chars().count(), 240);
        assert_eq!(model.detail.unwrap().chars().count(), 240);
        assert_eq!(
            decode_query_text("bad%2Gline%00%0Abreak", 40),
            "bad%2Glinebreak"
        );
    }

    #[test]
    fn unsafe_or_reserved_return_targets_fail_closed() {
        for unsafe_target in [
            "https://evil.example",
            "//evil.example/path",
            "/\\evil.example",
            "/auth",
            "/auth/continue",
            "/section/../auth",
            "/access-denied?loop=1",
            "/api/v1/auth/logout",
            "",
        ] {
            assert_eq!(
                safe_return_target(Some(unsafe_target)),
                "/",
                "{unsafe_target}"
            );
        }
        assert_eq!(
            safe_return_target(Some("/payments?tab=history#latest")),
            "/payments?tab=history#latest"
        );

        let (_, html) = render_path(
            "/access-denied",
            "route=https%253A%252F%252Fevil.example%252Fsteal",
        );
        assert!(html.contains("href=\"/auth?return_url=%2F\""));
        assert!(html.contains("href=\"/\" data-admin-denial-back=\"true\""));
        assert!(!html.contains("href=\"https://evil.example"));
        assert!(!html.contains("javascript:"));
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
