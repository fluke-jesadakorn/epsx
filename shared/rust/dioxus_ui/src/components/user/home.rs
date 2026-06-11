//! Sub-components extracted from `pages/home.rs` during
//! Wave 6C Track E (1:1 user-side component parity).
//!
//! Nine named sub-components: `Hero`, `TrustBar`, `TopPerformers`,
//! `FeaturesGrid`, `PricingTeaser`, `NewsPreview`,
//! `TestimonialsSection`, `FAQSection`, `CTASection`.

use crate::primitives::*;

use dioxus::prelude::*;

// === Hero ===

/// Hero. Includes share button, live chain selector, responsive
/// mobile collapse, and the 4-card stat grid.
#[component]
pub fn Hero() -> Element {
    rsx! {
        section { class: "hero",
            div { class: "hero-bg", "aria-hidden": "true",
                div { class: "hero-orb hero-orb-1" }
                div { class: "hero-orb hero-orb-2" }
                div { class: "hero-orb hero-orb-3" }
                div { class: "hero-orb hero-orb-4" }
            }
            div { class: "hero-inner container",
                div { class: "hero-badge",
                    span { class: "hero-badge-dot" }
                    span { "Performance Analytics Platform" }
                }
                h1 { class: "hero-title",
                    span { class: "hero-title-line", "📈 Track Your" }
                    span { class: "hero-title-line hero-title-gradient",
                        "Performance Growth"
                    }
                    span { class: "hero-title-line", "Metrics ✨" }
                }
                p { class: "hero-subtitle",
                    "🚀 Discover comprehensive data insights with our advanced analytics platform! "
                    span { class: "hero-subtitle-accent",
                        "Make informed decisions with real-time insights"
                    }
                    span { " 📈" }
                }
                div { class: "hero-actions",
                    a { class: "btn btn-gradient btn-lg hero-cta-primary", href: "/analytics",
                        span { "🚀 Start Exploration" }
                        span { class: "ml-2", Icon { name: "arrow-right".to_string(), size: Some(20) } }
                    }
                    button { class: "btn btn-outline btn-lg hero-share-btn", r#type: "button",
                        "aria-label": "Share platform",
                        Icon { name: "share-2".to_string(), size: Some(20) }
                        span { "📤 Share Platform" }
                    }
                }
                div { class: "hero-chain-selector",
                    span { class: "hero-chain-label", "Network:" }
                    div { class: "hero-chain-pill hero-chain-pill-active",
                        span { class: "hero-chain-dot" }
                        "BSC Mainnet"
                    }
                    div { class: "hero-chain-pill",
                        span { class: "hero-chain-dot hero-chain-dot-testnet" }
                        "BSC Testnet"
                    }
                }
                div { class: "hero-stats",
                    div { class: "hero-stat",
                        div { class: "hero-stat-icon", Icon { name: "users".to_string(), size: Some(24), class_name: Some("text-primary".to_string()) } }
                        div { class: "hero-stat-value", "12K+" }
                        div { class: "hero-stat-label", "Active users" }
                    }
                    div { class: "hero-stat",
                        div { class: "hero-stat-icon", Icon { name: "chart-line".to_string(), size: Some(24), class_name: Some("text-primary".to_string()) } }
                        div { class: "hero-stat-value", "8.5M" }
                        div { class: "hero-stat-label", "EPS rankings processed" }
                    }
                    div { class: "hero-stat",
                        div { class: "hero-stat-icon", Icon { name: "zap".to_string(), size: Some(24), class_name: Some("text-primary".to_string()) } }
                        div { class: "hero-stat-value", "< 1s" }
                        div { class: "hero-stat-label", "Response time" }
                    }
                    div { class: "hero-stat",
                        div { class: "hero-stat-icon", Icon { name: "shield".to_string(), size: Some(24), class_name: Some("text-primary".to_string()) } }
                        div { class: "hero-stat-value", "99.9%" }
                        div { class: "hero-stat-label", "Uptime" }
                    }
                }
            }
        }
    }
}

// === TrustBar ===

/// Trust bar. Row of trust-logo chips.
#[component]
pub fn TrustBar() -> Element {
    rsx! {
        section { class: "trust-bar",
            div { class: "trust-bar-inner container",
                p { class: "trust-bar-label", "Built for" }
                div { class: "trust-bar-logos",
                    span { class: "trust-logo", "Traders" }
                    span { class: "trust-logo", "Analysts" }
                    span { class: "trust-logo", "Merchants" }
                    span { class: "trust-logo", "Developers" }
                    span { class: "trust-logo", "DAOs" }
                    span { class: "trust-logo", "Binance" }
                    span { class: "trust-logo", "Ethereum Foundation" }
                }
            }
        }
    }
}

// === TopPerformers ===

/// Top performers. 5 cards with click-through links.
#[component]
pub fn TopPerformers() -> Element {
    rsx! {
        section { class: "top-performers",
            div { class: "container",
                div { class: "section-header",
                    h2 { class: "section-title", "Top EPS growth leaders" }
                    p { class: "section-sub", "Quarterly EPS growth, refreshed in real-time" }
                    div { class: "top-performers-freshness",
                        Icon { name: "clock".to_string(), size: Some(14) }
                        span { "Updated just now" }
                    }
                }
                div { class: "top-performers-grid",
                    a { class: "card card-glass performer-card", href: "/portfolio/GHC",
                        div { class: "performer-symbol", "GHC" }
                        div { class: "performer-price", "$6,535" }
                        Badge { kind: BadgeKind::Success, icon: Some("trending-up".to_string()), "+4657%" }
                    }
                    a { class: "card card-glass performer-card", href: "/portfolio/ARAX",
                        div { class: "performer-symbol", "ARAX" }
                        div { class: "performer-price", "$1,240" }
                        Badge { kind: BadgeKind::Success, icon: Some("trending-up".to_string()), "+312%" }
                    }
                    a { class: "card card-glass performer-card", href: "/portfolio/NVTK",
                        div { class: "performer-symbol", "NVTK" }
                        div { class: "performer-price", "$8,915" }
                        Badge { kind: BadgeKind::Success, icon: Some("trending-up".to_string()), "+287%" }
                    }
                    a { class: "card card-glass performer-card", href: "/portfolio/GTC",
                        div { class: "performer-symbol", "GTC" }
                        div { class: "performer-price", "$412" }
                        Badge { kind: BadgeKind::Success, icon: Some("trending-up".to_string()), "+165%" }
                    }
                    a { class: "card card-glass performer-card", href: "/portfolio/BIT",
                        div { class: "performer-symbol", "BIT" }
                        div { class: "performer-price", "$1,802" }
                        Badge { kind: BadgeKind::Success, icon: Some("trending-up".to_string()), "+142%" }
                    }
                }
            }
        }
    }
}

// === FeaturesGrid ===

/// Features grid. Six feature cards.
#[component]
pub fn FeaturesGrid() -> Element {
    rsx! {
        section { class: "features-grid-section",
            div { class: "container",
                div { class: "section-header",
                    h2 { class: "section-title", "Everything you need to ship" }
                    p { class: "section-sub", "Six core modules. One composable platform." }
                }
                div { class: "features-grid",
                    div { class: "card card-glass feature-card",
                        div { class: "feature-icon", Icon { name: "layout-dashboard".to_string(), size: Some(28), class_name: Some("text-primary".to_string()) } }
                        h3 { class: "feature-title", "Visual Builder" }
                        p { class: "feature-description text-muted-foreground", "Drag-and-drop blocks. Live preview. Theme system." }
                    }
                    div { class: "card card-glass feature-card",
                        div { class: "feature-icon", Icon { name: "credit-card".to_string(), size: Some(28), class_name: Some("text-primary".to_string()) } }
                        h3 { class: "feature-title", "On-Chain Payments" }
                        p { class: "feature-description text-muted-foreground", "USDT/USDC/BNB on BSC. Escrow built-in. Paymaster gas." }
                    }
                    div { class: "card card-glass feature-card",
                        div { class: "feature-icon", Icon { name: "zap".to_string(), size: Some(28), class_name: Some("text-primary".to_string()) } }
                        h3 { class: "feature-title", "Programmable Subscriptions" }
                        p { class: "feature-description text-muted-foreground", "Stream-based vaults. Per-merchant isolation. Grace period." }
                    }
                    div { class: "card card-glass feature-card",
                        div { class: "feature-icon", Icon { name: "chart-line".to_string(), size: Some(28), class_name: Some("text-primary".to_string()) } }
                        h3 { class: "feature-title", "Real-Time Analytics" }
                        p { class: "feature-description text-muted-foreground", "EPS rankings, country/sector filters, CSV export." }
                    }
                    div { class: "card card-glass feature-card",
                        div { class: "feature-icon", Icon { name: "briefcase".to_string(), size: Some(28), class_name: Some("text-primary".to_string()) } }
                        h3 { class: "feature-title", "Watchlist + Portfolio" }
                        p { class: "feature-description text-muted-foreground", "Track symbols, get alerts, manage positions." }
                    }
                    div { class: "card card-glass feature-card",
                        div { class: "feature-icon", Icon { name: "code".to_string(), size: Some(28), class_name: Some("text-primary".to_string()) } }
                        h3 { class: "feature-title", "API-First" }
                        p { class: "feature-description text-muted-foreground", "Authenticated REST. SIWE. Webhooks. Sandbox + production." }
                    }
                }
            }
        }
    }
}

// === PricingTeaser ===

/// Pricing teaser. Three tier cards.
#[component]
pub fn PricingTeaser() -> Element {
    rsx! {
        section { class: "pricing-teaser",
            div { class: "container",
                div { class: "section-header",
                    h2 { class: "section-title", "Plans that scale with you" }
                    p { class: "section-sub", "Personal, team, and enterprise tiers. Pay in stablecoins." }
                }
                div { class: "pricing-teaser-grid",
                    div { class: "card card-glass pricing-teaser-card",
                        div { class: "pricing-teaser-tier", "Personal" }
                        div { class: "pricing-teaser-price", "From $9" }
                        ul { class: "pricing-teaser-features",
                            li { "Full analytics" }
                            li { "Watchlist + alerts" }
                            li { "API access" }
                        }
                        a { class: "btn btn-outline btn-block", href: "/plans", "See plans" }
                    }
                    div { class: "card card-primary-solid pricing-teaser-card highlighted",
                        div { class: "pricing-teaser-tier", "Team" }
                        div { class: "pricing-teaser-price", "From $49" }
                        Badge { kind: BadgeKind::Brand, "Most popular" }
                        ul { class: "pricing-teaser-features",
                            li { "Up to 10 seats" }
                            li { "Custom permissions" }
                            li { "Paymaster gas" }
                            li { "Priority support" }
                        }
                        a { class: "btn btn-gradient btn-block", href: "/plans", "Start trial" }
                    }
                    div { class: "card card-glass pricing-teaser-card",
                        div { class: "pricing-teaser-tier", "Enterprise" }
                        div { class: "pricing-teaser-price", "Custom" }
                        ul { class: "pricing-teaser-features",
                            li { "Unlimited seats" }
                            li { "SLA + dedicated support" }
                            li { "Custom contracts" }
                        }
                        a { class: "btn btn-outline btn-block", href: "/contact", "Contact sales" }
                    }
                }
            }
        }
    }
}

// === NewsPreview ===

/// News preview. Three cards linking to `/news/{slug}`.
#[component]
pub fn NewsPreview() -> Element {
    rsx! {
        section { class: "news-preview",
            div { class: "container",
                div { class: "section-header",
                    h2 { class: "section-title", "From the EPSX blog" }
                    p { class: "section-sub", "Product news, engineering deep-dives, market commentary." }
                }
                div { class: "news-preview-grid",
                    a { class: "card card-glass news-preview-card", href: "/news/scalable-foundation",
                        div { class: "news-preview-tag", "Engineering" }
                        h3 { class: "news-preview-title", "Building a scalable foundation" }
                        p { class: "news-preview-excerpt text-muted-foreground", "How we architected a 9-service Rust backend on Kubernetes." }
                    }
                    a { class: "card card-glass news-preview-card", href: "/news/optimizing-high-throughput-analytics-rust",
                        div { class: "news-preview-tag", "Engineering" }
                        h3 { class: "news-preview-title", "Optimizing high-throughput analytics" }
                        p { class: "news-preview-excerpt text-muted-foreground", "Sub-millisecond EPS ranking over 8.5M data points." }
                    }
                    a { class: "card card-glass news-preview-card", href: "/news/real-time-intelligence",
                        div { class: "news-preview-tag", "Product" }
                        h3 { class: "news-preview-title", "Real-time intelligence, made simple" }
                        p { class: "news-preview-excerpt text-muted-foreground", "How we made complex analytics feel instant." }
                    }
                }
                div { class: "text-center mt-8",
                    a { class: "btn btn-outline", href: "/news", "View all posts" }
                }
            }
        }
    }
}

// === TestimonialsSection ===

/// Testimonials section. Three cards with quote + attribution +
/// star rating.
#[component]
pub fn TestimonialsSection() -> Element {
    rsx! {
        section { class: "testimonials-section",
            div { class: "container",
                div { class: "section-header",
                    h2 { class: "section-title", "Loved by traders, analysts, and teams" }
                    p { class: "section-sub", "Hear from teams shipping on EPSX every day." }
                }
                div { class: "testimonials-grid",
                    div { class: "card card-glass testimonial-card",
                        div { class: "testimonial-rating",
                            span { class: "testimonial-star" }
                            span { class: "testimonial-star" }
                            span { class: "testimonial-star" }
                            span { class: "testimonial-star" }
                            span { class: "testimonial-star" }
                        }
                        p { class: "testimonial-quote",
                            "\"EPSX gave us sub-second EPS rankings over 8.5M data points. The on-chain payment flow cut our merchant onboarding from days to minutes.\""
                        }
                        div { class: "testimonial-meta",
                            div { class: "testimonial-avatar testimonial-avatar-1" }
                            div { class: "testimonial-meta-text",
                                div { class: "testimonial-name", "Sarah Chen" }
                                div { class: "testimonial-role", "Head of Analytics, Meridian Capital" }
                            }
                        }
                    }
                    div { class: "card card-glass testimonial-card",
                        div { class: "testimonial-rating",
                            span { class: "testimonial-star" }
                            span { class: "testimonial-star" }
                            span { class: "testimonial-star" }
                            span { class: "testimonial-star" }
                            span { class: "testimonial-star" }
                        }
                        p { class: "testimonial-quote",
                            "\"The visual page builder is the real deal. Our marketing team ships campaign pages without a single PR review — and the on-chain payments just work.\""
                        }
                        div { class: "testimonial-meta",
                            div { class: "testimonial-avatar testimonial-avatar-2" }
                            div { class: "testimonial-meta-text",
                                div { class: "testimonial-name", "Marcus Patel" }
                                div { class: "testimonial-role", "CTO, Northwind Markets" }
                            }
                        }
                    }
                    div { class: "card card-glass testimonial-card",
                        div { class: "testimonial-rating",
                            span { class: "testimonial-star" }
                            span { class: "testimonial-star" }
                            span { class: "testimonial-star" }
                            span { class: "testimonial-star" }
                            span { class: "testimonial-star" }
                        }
                        p { class: "testimonial-quote",
                            "\"Paymaster-sponsored gas removed the biggest onboarding friction. Our Pro users complete wallet auth in under 30 seconds — zero BNB required.\""
                        }
                        div { class: "testimonial-meta",
                            div { class: "testimonial-avatar testimonial-avatar-3" }
                            div { class: "testimonial-meta-text",
                                div { class: "testimonial-name", "Aisha Okonkwo" }
                                div { class: "testimonial-role", "VP Product, ChainLabs DAO" }
                            }
                        }
                    }
                }
            }
        }
    }
}

// === FAQSection ===

/// FAQ section. Six-question accordion (native `<details>`/`<summary>`).
#[component]
pub fn FAQSection() -> Element {
    rsx! {
        section { class: "faq-section",
            div { class: "container",
                div { class: "section-header",
                    h2 { class: "section-title", "Frequently asked questions" }
                    p { class: "section-sub", "Everything you need to know to get started." }
                }
                div { class: "faq-list",
                    details { class: "faq-item",
                        summary { class: "faq-question",
                            span { "What is EPSX?" }
                            span { class: "faq-chevron", Icon { name: "chevron-down".to_string(), size: Some(18) } }
                        }
                        div { class: "faq-answer",
                            p {
                                "EPSX is a production-grade Web3 commerce platform: visual page builder, on-chain payments, programmable subscriptions, and paymaster-sponsored gas — all running as Rust microservices on BSC."
                            }
                        }
                    }
                    details { class: "faq-item",
                        summary { class: "faq-question",
                            span { "How does the Sign-In With Ethereum (SIWE) flow work?" }
                            span { class: "faq-chevron", Icon { name: "chevron-down".to_string(), size: Some(18) } }
                        }
                        div { class: "faq-answer",
                            p {
                                "Connect a BSC-compatible wallet (MetaMask, WalletConnect, etc.), receive a one-time challenge nonce, sign it, and the BFF issues a session cookie + JWT. No email, no password. Pro users get paymaster-sponsored gas."
                            }
                        }
                    }
                    details { class: "faq-item",
                        summary { class: "faq-question",
                            span { "Which payment methods do you support?" }
                            span { class: "faq-chevron", Icon { name: "chevron-down".to_string(), size: Some(18) } }
                        }
                        div { class: "faq-answer",
                            p {
                                "USDT, USDC, and BNB on BSC mainnet. Subscriptions use stream-based vaults with per-merchant isolation. Enterprise customers can deploy custom contracts."
                            }
                        }
                    }
                    details { class: "faq-item",
                        summary { class: "faq-question",
                            span { "Is there a free tier?" }
                            span { class: "faq-chevron", Icon { name: "chevron-down".to_string(), size: Some(18) } }
                        }
                        div { class: "faq-answer",
                            p {
                                "Yes — the Personal plan starts at $9/mo with a 14-day free trial. The demo account (no signup) is also available at /auth for evaluation."
                            }
                        }
                    }
                    details { class: "faq-item",
                        summary { class: "faq-question",
                            span { "Can I build my own UI on top of EPSX?" }
                            span { class: "faq-chevron", Icon { name: "chevron-down".to_string(), size: Some(18) } }
                        }
                        div { class: "faq-answer",
                            p {
                                "Absolutely. The API is fully documented at /developer. SIWE-authenticated REST endpoints, webhooks for real-time events, and a sandbox environment so you can ship without touching production."
                            }
                        }
                    }
                    details { class: "faq-item",
                        summary { class: "faq-question",
                            span { "How is EPSX different from a traditional analytics platform?" }
                            span { class: "faq-chevron", Icon { name: "chevron-down".to_string(), size: Some(18) } }
                        }
                        div { class: "faq-answer",
                            p {
                                "Two things: on-chain identity and paymaster-sponsored transactions. Your wallet IS your account — no emails, no password resets. And the gas economics work for mass-market users, not just crypto natives."
                            }
                        }
                    }
                }
            }
        }
    }
}

// === CTASection ===

/// CTA section. Connect wallet + View pricing + Talk to sales.
#[component]
pub fn CTASection() -> Element {
    rsx! {
        section { class: "cta-section",
            div { class: "container",
                div { class: "card card-primary-solid cta-card",
                    h2 { class: "cta-title", "Ready to build with EPSX?" }
                    p { class: "cta-subtitle", "Sign in with your wallet and start exploring in under 30 seconds." }
                    div { class: "cta-actions",
                        a { class: "btn btn-glass btn-lg", href: "/auth", "Connect wallet" }
                        a { class: "btn btn-outline btn-lg", href: "/plans", "View pricing" }
                        a { class: "btn btn-ghost btn-lg cta-secondary-link", href: "/contact",
                            span { "Talk to sales" }
                            Icon { name: "arrow-right".to_string(), size: Some(16) }
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

    /// Wave 6C Track E — `test_render_smoke` for the extracted
    /// home sub-components.
    #[test]
    fn home_subcomponents_render_smoke() {
        // Hero
        let el = rsx! { Hero {} };
        let html = dioxus_ssr::render_element(el);
        assert!(html.contains("hero"), "Hero missing section-marker");
        assert!(html.contains("Performance Growth"));
        assert!(html.contains("hero-share-btn"));
        assert!(html.contains("hero-chain-selector"));

        // TrustBar
        let el = rsx! { TrustBar {} };
        let html = dioxus_ssr::render_element(el);
        assert!(html.contains("trust-bar"));
        assert!(html.contains("Binance"));

        // TopPerformers
        let el = rsx! { TopPerformers {} };
        let html = dioxus_ssr::render_element(el);
        assert!(html.contains("top-performers"));
        assert!(html.contains("GHC"));

        // FeaturesGrid
        let el = rsx! { FeaturesGrid {} };
        let html = dioxus_ssr::render_element(el);
        assert!(html.contains("features-grid-section"));
        assert!(html.contains("Visual Builder"));

        // PricingTeaser
        let el = rsx! { PricingTeaser {} };
        let html = dioxus_ssr::render_element(el);
        assert!(html.contains("pricing-teaser"));
        assert!(html.contains("Most popular"));

        // NewsPreview
        let el = rsx! { NewsPreview {} };
        let html = dioxus_ssr::render_element(el);
        assert!(html.contains("news-preview"));
        assert!(html.contains("From the EPSX blog"));

        // TestimonialsSection
        let el = rsx! { TestimonialsSection {} };
        let html = dioxus_ssr::render_element(el);
        assert!(html.contains("testimonials-section"));
        let testimonial_count = html.matches("testimonial-card").count();
        assert_eq!(testimonial_count, 3, "TestimonialsSection must render 3 testimonial cards. Got: {}", testimonial_count);

        // FAQSection
        let el = rsx! { FAQSection {} };
        let html = dioxus_ssr::render_element(el);
        assert!(html.contains("faq-section"));
        let faq_count = html.matches("class=\"faq-item\"").count();
        assert_eq!(faq_count, 6, "FAQSection must render 6 FAQ items. Got: {}", faq_count);

        // CTASection
        let el = rsx! { CTASection {} };
        let html = dioxus_ssr::render_element(el);
        assert!(html.contains("cta-section"));
        assert!(html.contains("Talk to sales"));
    }
}
