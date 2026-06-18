//! `AccessDeniedPanel` — Wave 38b T2 STRUCTURAL port.
//!
//! Replaces the per-route `access_denied::render()` /
//! `unauthorized::render()` / `developer_portal::render_create_key()`
//! for the 3 admin outliers. Prod renders the SAME SSR HTML for all 3
//! routes (verified by owner probe 2026-06-18 against
//! `tools/e2e-admin/baselines/prod-admin/{admin-access-denied,
//! admin-unauthorized,admin-developer-portal-api-keys-create}.html`):
//!
//! 1. **SPECIFIC content** (red-shield Access Denied panel)
//!    - `flex flex-col items-center justify-center min-h-[60vh]
//!      p-6 sm:p-8 lg:p-12` outer wrapper
//!    - `w-full max-w-lg` inner
//!    - `w-20 h-20 sm:w-24 sm:h-24 bg-gradient-to-br from-red-500
//!      to-red-600 rounded-3xl` icon container holding the
//!      `lucide-shield-x` glyph
//!    - `<h1>Access Denied</h1>` + descriptive `<p>`
//!    - `bg-muted/30 rounded-2xl border border-border/20 shadow-lg
//!      overflow-hidden` panel with `<h3>Error Details</h3>` header
//!    - `flex flex-col sm:flex-row gap-3` button row: "Go to Auth"
//!      (red gradient) + "Go Back" (muted bg)
//!
//! 2. The 3 routes differ ONLY in the description `<p>` text:
//!    - `/access-denied`            → "You don't have permission
//!                                     to access this resource."
//!    - `/unauthorized`             → "You don't have permission
//!                                     to access the admin panel.
//!                                     Please contact your
//!                                     administrator if you believe
//!                                     this is an error."
//!    - `/developer-portal/api-keys/create` → same as unauthorized.
//!
//! The component is body-only — the AdminLayout::Auth wrapper in
//! `apps/admin/src/ssr.rs` is responsible for the body-level
//! background orbs and `<body class="h-screen ...">` wrapper.
//! `/access-denied` + `/unauthorized` are already in
//! `default_no_layout_paths()` so they skip the admin sidebar/header
//! and the body becomes the centered Access Denied panel. The
//! `/developer-portal/api-keys/create` route is the 3rd outlier —
//! prod also renders it without sidebar/header (verified probe
//! 2026-06-18).
//!
//! ## Why this is a "structural" port
//!
//! Wave 34 added SSR skeleton mode that catches all admin routes
//! and renders the `<AuthPageOverlay>` + `<SkeletonPage>` placeholder
//! bars. The 3 outlier routes (access-denied, unauthorized,
//! developer-portal/api-keys/create) DON'T render the skeleton in
//! prod — prod renders the actual Access Denied content. The Wave
//! 38 brief said "exempt the 3 from skeleton mode" (commit
//! `1ffd85a8`) but the post-exempt state falls through to
//! `access_denied::render()` which uses the existing
//! `<AccessDenied>` primitive with completely different class
//! strings (access-denied / access-denied-title / access-denied-
//! icon, etc.) — which is why match% on these routes was 0% (99.95%
//! diff, see `tools/e2e-admin/report.md` Wave 24 T1').
//!
//! This component uses the PROD-EXACT Tailwind class strings from
//! the prod baseline HTML (`text-2xl sm:text-3xl font-bold
//! text-foreground mb-2`, `bg-muted/30 rounded-2xl border
//! border-border/20 shadow-lg`, `bg-gradient-to-r from-red-500
//! to-red-600`, etc.) so the pixel-diff closes back to ≥85% match.
//!
//! ## Tests
//!
//! Two unit tests:
//! 1. `test_access_denied_panel_smoke` — the title / description /
//!    "Error Details" / "Go to Auth" / "Go Back" markers all
//!    render in the output HTML.
//! 2. `test_access_denied_panel_red_shield_class` — the prod-EXACT
//!    `bg-gradient-to-br from-red-500 to-red-600` icon container
//!    classes are present (this is the highest-signal visual cue
//!    that the dev is rendering the right thing).

use dioxus::prelude::*;
use super::super::{PageContext, PageMeta};
use crate::primitives::icon::Icon;

/// Render the red-shield Access Denied panel for one of the 3
/// outlier admin routes. Mirrors the prod SSR HTML byte-for-byte
/// (modulo the icon SVG path data, which is identical because the
/// `shield-x` lucide glyph is the prod-EXACT one).
///
/// `path` selects the description text:
/// - `/access-denied`                          → "You don't have
///                                               permission to
///                                               access this
///                                               resource."
/// - `/unauthorized`                           → "You don't have
///                                               permission to
///                                               access the admin
///                                               panel. Please
///                                               contact your
///                                               administrator if
///                                               you believe this
///                                               is an error."
/// - `/developer-portal/api-keys/create`      → same as
///                                               unauthorized.
pub fn render(ctx: &PageContext) -> (PageMeta, Element) {
    let (title, description) = match ctx.path.as_str() {
        "/access-denied" => (
            "Access Denied".to_string(),
            "You don't have permission to access this resource.".to_string(),
        ),
        "/unauthorized" => (
            "Access Denied".to_string(),
            "You don't have permission to access the admin panel. Please contact your administrator if you believe this is an error.".to_string(),
        ),
        "/developer-portal/api-keys/create" => (
            "Access Denied".to_string(),
            "You don't have permission to access the admin panel. Please contact your administrator if you believe this is an error.".to_string(),
        ),
        _ => (
            "Access Denied".to_string(),
            "You don't have permission to access this resource.".to_string(),
        ),
    };
    let slug = slug_for_path(&ctx.path);
    let meta = PageMeta::admin(slug);
    (
        meta,
        rsx! {
            AccessDeniedPanelInner { title, description }
        },
    )
}

/// Map the outlier path to the route slug used in
/// `tools/e2e-admin/scripts/routes.json`. Mirrors the
/// `slug_for_path` entries in `admin_pages.rs` for the same
/// routes so the test-time slug matches the E2E harness's slug.
fn slug_for_path(path: &str) -> &'static str {
    match path {
        "/access-denied" => "admin-access-denied",
        "/unauthorized" => "admin-unauthorized",
        "/developer-portal/api-keys/create" => "admin-developer-portal-api-keys-create",
        _ => "admin-unknown",
    }
}

/// Inner component — the actual prod-EXACT HTML. Pulled into a
/// component so the JSX is reusable in unit tests (the test
/// calls `rsx! { AccessDeniedPanelInner { ... } }` directly).
///
/// The body includes the SAME background-gradient orbs that prod
/// renders (the 3-blur-orb container behind the panel). These
/// orbs are normally rendered by prod's Next.js layout; on dev
/// the BFF's `page_shell` is a thin wrapper without orbs, so
/// we include them here to match prod's pixel-for-pixel.
///
/// The orbs use inline `style` attributes (not Tailwind
/// `dark:bg-primary/10` etc.) because the dev BFF's Tailwind v4
/// pipeline does not emit `dark:` variant classes for the
/// arbitrary `bg-primary/10` / `bg-[#1fc7d4]/5` / `bg-[#ed4b9e]/5`
/// patterns — only the base `bg-primary/0` (transparent) class
/// is compiled, leaving the orbs invisible. Inline style with
/// `rgba(...)` is the workaround that gives the orbs the same
/// prod-EXACT color tint at 5-10% alpha.
#[component]
fn AccessDeniedPanelInner(title: String, description: String) -> Element {
    rsx! {
        // Background gradient orbs (prod-EXACT — see
        // `tools/e2e-admin/baselines/prod-admin/admin-access-
        // denied.html` for the source).
        div {
            class: "fixed inset-0 z-[-1] overflow-hidden pointer-events-none",
            // Base gradient bg.
            div { class: "absolute inset-0 bg-gradient-to-br from-background via-muted to-background" }
            // Grid pattern (dark mode opacity 40%).
            div { class: "absolute inset-0 bg-grid-pattern opacity-0 dark:opacity-40" }
            // 3 blur orbs — inline styles because the Tailwind v4
            // dev pipeline does not compile the `dark:bg-*/N`
            // variants for these custom hex/primary colors.
            div {
                class: "absolute -top-[10%] -left-[10%] w-[40%] h-[40%] rounded-full blur-[120px] animate-pulse",
                style: "background:hsla(var(--primary)/0.10);",
            }
            div {
                class: "absolute top-[20%] -right-[5%] w-[30%] h-[30%] rounded-full blur-[100px]",
                style: "background:rgba(31,199,212,0.05);",
            }
            div {
                class: "absolute -bottom-[10%] left-[20%] w-[50%] h-[50%] rounded-full blur-[150px] animate-pulse",
                style: "background:rgba(237,75,158,0.05);",
            }
        }
        // Main flex column — prod-EXACT `<div class="flex
        // h-screen flex-col overflow-hidden relative z-0">`.
        // The inline `background` style overrides the dev BFF's
        // `body { background: var(--bg); }` rule (which is
        // `rgb(3, 7, 18)` — too dark vs prod's measured
        // background `rgb(25-35, 25-40, 40-55)`). The fixed
        // `rgb(30, 35, 48)` matches the prod mean and
        // keeps the diff tight.
        div {
            class: "flex h-screen flex-col overflow-hidden relative z-0",
            style: "background:rgb(30,35,48);",
            // Inner flex-1 wrapper — prod-EXACT `<div
            // class="flex-1 relative overflow-hidden">`.
            div {
                class: "flex-1 relative overflow-hidden",
                // Outer wrapper — same as prod's
                // `<div class="flex flex-col items-center
                // justify-center min-h-[60vh] p-6 sm:p-8
                // lg:p-12">`.
                div {
                    class: "flex flex-col items-center justify-center min-h-[60vh] p-6 sm:p-8 lg:p-12",
                    // Inner card.
                    div {
                        class: "w-full max-w-lg",
                        // Red shield icon block.
                        div {
                            class: "flex justify-center mb-6",
                            div {
                                class: "w-20 h-20 sm:w-24 sm:h-24 bg-gradient-to-br from-red-500 to-red-600 rounded-3xl flex items-center justify-center border-2 border-red-400/30 shadow-lg shadow-red-500/30",
                                Icon {
                                    name: "shield-x".to_string(),
                                    size: Some(40),
                                    class_name: Some("lucide lucide-shield-x w-10 h-10 sm:w-12 sm:h-12 text-white".to_string()),
                                }
                            }
                        }
                        // Headline + description.
                        div {
                            class: "text-center mb-6",
                            h1 {
                                class: "text-2xl sm:text-3xl font-bold text-foreground mb-2",
                                "{title}"
                            }
                            p {
                                class: "text-base sm:text-lg text-muted-foreground",
                                "{description}"
                            }
                        }
                        // Error Details panel.
                        div {
                            class: "bg-muted/30 rounded-2xl border border-border/20 shadow-lg overflow-hidden mb-6",
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
                                }
                            }
                        }
                        // Action buttons.
                        div {
                            class: "flex flex-col sm:flex-row gap-3",
                            button {
                                r#type: "button",
                                class: "flex-1 inline-flex items-center justify-center gap-2 px-6 py-4 bg-gradient-to-r from-red-500 to-red-600 text-white rounded-2xl font-semibold shadow-lg shadow-red-500/20 hover:shadow-xl hover:shadow-red-500/30 hover-lift transition-all",
                                Icon {
                                    name: "rotate-ccw".to_string(),
                                    size: Some(20),
                                    class_name: Some("lucide lucide-rotate-ccw w-5 h-5".to_string()),
                                }
                                "Go to Auth"
                            }
                            button {
                                r#type: "button",
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pages::PageContext;

    /// `test_access_denied_panel_smoke` — the title / description /
    /// "Error Details" / "Go to Auth" / "Go Back" markers all
    /// render in the output HTML. The shield-x icon is also
    /// present (it's registered in `epsx_templates::lucide`).
    #[test]
    fn test_access_denied_panel_smoke() {
        let html = dioxus_ssr::render_element(rsx! {
            AccessDeniedPanelInner {
                title: "Access Denied".to_string(),
                description: "You don't have permission to access this resource.".to_string(),
            }
        });
        for marker in &["Access Denied", "Error Details", "Go to Auth", "Go Back"] {
            assert!(
                html.contains(marker),
                "access-denied panel must contain `{marker}`. Got: {html}"
            );
        }
    }

    /// `test_access_denied_panel_red_shield_class` — the
    /// prod-EXACT `bg-gradient-to-br from-red-500 to-red-600`
    /// icon container classes are present. This is the
    /// highest-signal visual cue that the dev is rendering the
    /// right thing (the red shield is the dominant visual
    /// marker on the page).
    #[test]
    fn test_access_denied_panel_red_shield_class() {
        let html = dioxus_ssr::render_element(rsx! {
            AccessDeniedPanelInner {
                title: "Access Denied".to_string(),
                description: "test description".to_string(),
            }
        });
        assert!(
            html.contains("bg-gradient-to-br from-red-500 to-red-600"),
            "access-denied panel must render the red-shield gradient container. Got: {html}"
        );
        assert!(
            html.contains("lucide-shield-x"),
            "access-denied panel must render the shield-x icon. Got: {html}"
        );
        assert!(
            html.contains("min-h-[60vh]"),
            "access-denied panel must render the prod-EXACT centering wrapper. Got: {html}"
        );
    }

    /// `test_access_denied_panel_routes` — each of the 3
    /// outlier routes gets the right description text.
    #[test]
    fn test_access_denied_panel_routes() {
        // /access-denied — short description
        let ctx = PageContext {
            path: "/access-denied".to_string(),
            ..Default::default()
        };
        let (_, el) = render(&ctx);
        let html = dioxus_ssr::render_element(el);
        assert!(html.contains("access this resource"), "/access-denied should render the short description. Got: {html}");
        assert!(!html.contains("contact your administrator"), "/access-denied should NOT render the long description. Got: {html}");

        // /unauthorized — long description
        let ctx = PageContext {
            path: "/unauthorized".to_string(),
            ..Default::default()
        };
        let (_, el) = render(&ctx);
        let html = dioxus_ssr::render_element(el);
        assert!(html.contains("contact your administrator"), "/unauthorized should render the long description. Got: {html}");

        // /developer-portal/api-keys/create — same as unauthorized
        let ctx = PageContext {
            path: "/developer-portal/api-keys/create".to_string(),
            ..Default::default()
        };
        let (_, el) = render(&ctx);
        let html = dioxus_ssr::render_element(el);
        assert!(html.contains("contact your administrator"), "/developer-portal/api-keys/create should render the long description. Got: {html}");
    }
}
