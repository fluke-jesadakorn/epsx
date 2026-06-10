use crate::primitives::icon::Icon;

use dioxus::prelude::*;

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
