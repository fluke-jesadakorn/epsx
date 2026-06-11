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
