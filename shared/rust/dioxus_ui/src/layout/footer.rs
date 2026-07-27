//! Site + admin footers.
//!
//! - [`Footer`] (also re-exported as `SiteFooter`) — the full 4-column
//!   site footer for the public site. Used by the frontend BFF and the
//!   existing pages.
//! - [`AdminFooter`] — thin 2-line admin footer ("EPSX Admin Dashboard"
//!   / "Version 2.0") used inside `MainLayout`'s shell.

use dioxus::prelude::*;

/// Full 4-column site footer. Matches the original `Footer` in
/// `apps-old/frontend/components/footer.tsx`.
///
/// Wave 2 keeps this API stable — see Wave 1's Public API Stability
/// rule. Both `Footer` and `SiteFooter` (alias added below) are
/// importable so the frontend nav cluster (Track B) and the admin
/// shell (Track A) can refer to it by either name.
#[component]
pub fn Footer() -> Element {
    rsx! {
        footer { class: "site-footer",
            div { class: "footer-grid",
                div { class: "footer-col",
                    div { class: "footer-brand",
                        span { dangerous_inner_html: "{epsx_templates::epsx_icon_svg()}" }
                        span { class: "gradient-text text-lg font-semibold", "EPSX" }
                    }
                    p { class: "footer-text", "Web3 commerce platform: visual page builder, on-chain payments, programmable subscriptions, paymaster-sponsored gas." }
                }
                div { class: "footer-col",
                    h4 { class: "footer-heading", "Platform" }
                    ul { class: "footer-list",
                        li { a { href: "/", "Home" } }
                        li { a { href: "/pricing", "Pricing" } }
                        li { a { href: "/plans", "Plans" } }
                        li { a { href: "/analytics", "Analytics" } }
                        li { a { href: "/portfolio", "Portfolio" } }
                    }
                }
                div { class: "footer-col",
                    h4 { class: "footer-heading", "Developers" }
                    ul { class: "footer-list",
                        li { a { href: "/developer", "Developer Portal" } }
                        li { a { href: "/developer/docs", "API Docs" } }
                        li { a { href: "/manual", "Manual" } }
                    }
                }
                div { class: "footer-col",
                    h4 { class: "footer-heading", "Company" }
                    ul { class: "footer-list",
                        li { a { href: "/about", "About" } }
                        li { a { href: "/contact", "Contact" } }
                        li { a { href: "/news", "News" } }
                        li { a { href: "/terms", "Terms" } }
                        li { a { href: "/privacy", "Privacy" } }
                    }
                }
            }
            div { class: "footer-bottom", p { "© 2025 EPSX. All rights reserved." } }
        }
    }
}

/// Re-export alias for [`Footer`] — Track B's design doc notes the
/// frontend footer rename. Both names are importable.
#[component]
pub fn SiteFooter() -> Element {
    rsx! { Footer {} }
}

/// Thin admin shell footer. Two-line strip ("EPSX Admin Dashboard" /
/// "Version 2.0") with a glass background and top border, matching the
/// footer rendered at the bottom of `MainLayout`.
///
/// This is a deliberately tiny component — the admin chrome's bottom
/// strip in the TS source is also only 3 lines of markup. Wave 3 can
/// add version-from-Cargo or links if needed.
#[component]
pub fn AdminFooter() -> Element {
    rsx! {
        footer { class: "border-t border-border/40 bg-card px-4 py-3 admin-footer",
            div { class: "flex items-center justify-between",
                span { class: "text-sm font-medium text-foreground", "EPSX Admin Dashboard" }
                span { class: "text-sm text-muted-foreground", "Version 2.0" }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The `Footer` primitive renders a 4-column site footer
    /// (`<footer class="site-footer">`) with the four standard
    /// column headings (Platform / Developers / Company) and a
    /// copyright bottom strip. It's a public primitive — pages
    /// can opt in by calling `<Footer />` directly, even though
    /// `MainLayout` no longer renders it by default (the public
    /// site has no footer in prod).
    #[test]
    fn footer_renders_four_columns_and_copyright() {
        let html = dioxus_ssr::render_element(rsx! { Footer {} });
        assert!(
            html.contains("site-footer"),
            "Footer must render <footer class=\"site-footer\">. Got: {html}"
        );
        // 4 columns by class name.
        for col in &[
            "footer-col",
            "footer-grid",
            "footer-brand",
            "footer-heading",
            "footer-list",
            "footer-bottom",
        ] {
            assert!(
                html.contains(col),
                "Footer must contain the `{col}` class. Got: {html}"
            );
        }
        // 3 column headings.
        for h in &["Platform", "Developers", "Company"] {
            assert!(
                html.contains(h),
                "Footer must include heading `{h}`. Got: {html}"
            );
        }
        // Bottom strip.
        assert!(
            html.contains("2025 EPSX"),
            "Footer must include the copyright year + brand. Got: {html}"
        );
    }

    /// `SiteFooter` is the alias for `Footer` — it must render the
    /// same `<footer class="site-footer">` element so call sites
    /// that imported either name keep working.
    #[test]
    fn site_footer_is_an_alias_for_footer() {
        let html_a = dioxus_ssr::render_element(rsx! { Footer {} });
        let html_b = dioxus_ssr::render_element(rsx! { SiteFooter {} });
        assert_eq!(
            html_a, html_b,
            "SiteFooter must produce the same HTML as Footer (it's a re-export alias)"
        );
    }

    /// `AdminFooter` renders the prod-EXACT 2-line admin strip
    /// ("EPSX Admin Dashboard" / "Version 2.0") with the
    /// `admin-footer` class. This is what the `shell::MainLayout`
    /// and the `admin_shell::AdminShell` components render at the
    /// bottom of every admin page — it's the only footer the
    /// dev BFF currently emits (no public-site footer at all).
    #[test]
    fn admin_footer_renders_admin_dashboard_strip() {
        let html = dioxus_ssr::render_element(rsx! { AdminFooter {} });
        assert!(
            html.contains("admin-footer"),
            "AdminFooter must render <footer class=\"admin-footer\">. Got: {html}"
        );
        assert!(
            html.contains("EPSX Admin Dashboard"),
            "AdminFooter must show 'EPSX Admin Dashboard'. Got: {html}"
        );
        assert!(
            html.contains("Version 2.0"),
            "AdminFooter must show 'Version 2.0'. Got: {html}"
        );
        // Must use the same border/bg/padding class chain as the
        // prod `<footer>` (matches the prod `MainLayout`).
        assert!(
            html.contains("border-t border-border/40 bg-card px-4 py-3"),
            "AdminFooter must use the prod class chain. Got: {html}"
        );
    }
}
