use crate::primitives::*;

use dioxus::prelude::*;
use super::PageContext;
use super::PageMeta;
use crate::layout::main_layout::MainLayout;
use crate::layout::marketing_bg::MarketingBackground;
use crate::auth::ProgressiveAuthBanner;

/// Home page (`/`). Wave 5 Track A port — see
/// `docs/wave5-page-depth/design.md` §"Track A — Hero pages"
/// for the section list. Sections (in order):
///   1. Hero (with share button, live chain selector, responsive
///      mobile collapse)
///   2. TrustBar (4 logos from the source)
///   3. TopPerformers (5 cards + data-freshness timestamp +
///      row-level click-through to `/portfolio/{addr}`)
///   4. FeaturesGrid (6 feature cards)
///   5. PricingTeaser (3 tiers, "View all plans" → `/plans`)
///   6. NewsPreview (3 cards → `/news/{slug}`)
///   7. TestimonialsSection (3 testimonials — NEW in Wave 5)
///   8. FAQSection (6-question accordion — NEW in Wave 5)
///   9. CTASection (with secondary "Talk to sales" link)
pub fn render(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::marketing("Home");
    (meta, rsx! {
        MainLayout { ctx: ctx.clone(),
            if ctx.user.is_none() {
                ProgressiveAuthBanner {
                    feature: Some("the full EPSX experience".to_string()),
                }
            }
            MarketingBackground {
                Hero {}
                TrustBar {}
                TopPerformers {}
                FeaturesGrid {}
                PricingTeaser {}
                NewsPreview {}
                TestimonialsSection {}
                FAQSection {}
                CTASection {}
            }
        }
    })
}

// === Hero ===

/// Hero. Expanded in Wave 5 to include:
///  - the share button (right of the primary CTA, per the source
///    `share-button.tsx`),
///  - a live chain selector (BSC mainnet / BSC testnet pill — the
///    source uses `BlockchainNetworkProvider` and is server-rendered
///    as a static pill in SSR),
///  - the responsive mobile collapse (the headline wraps to a single
///    line on mobile, the action row stacks vertically, the stat grid
///    collapses to a single column).
#[component]
fn Hero() -> Element {
    rsx! {
        section { class: "hero",
            // The marketing background provides the fixed orbs; the
            // hero adds a second layer of "hero-bg" decoration that
            // sits over the page-level background for extra depth.
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
                    // Share button (Wave 5 Track A addition).
                    // The onClick copy is wired by the BFF's hydration
                    // script (the home page is signed-out for many
                    // visitors, so the share works even pre-hydration
                    // — the URL is `/` and clipboard copy is safe).
                    button { class: "btn btn-outline btn-lg hero-share-btn", r#type: "button",
                        "aria-label": "Share platform",
                        Icon { name: "share-2".to_string(), size: Some(20) }
                        span { "📤 Share Platform" }
                    }
                }
                // Live chain selector — BSC mainnet / testnet pill.
                // Source uses `BlockchainNetworkProvider`; in SSR
                // (no client context) we render the mainnet pill as
                // the default. The client-side hydration script can
                // swap the active state without a re-render.
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
                // Stat grid — expanded from 3 to 4 stats to match the
                // source's "12K+ / 8.5M / 99.9%" set + the new "0.8s"
                // response time stat.
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

/// Trust bar. Source uses 4 logos; Wave 1 port had 2. Wave 5 adds
/// 2 more (Binance, Ethereum Foundation). The visual style is
/// unchanged — a row of `span.trust-logo` chips with the company
/// name.
#[component]
fn TrustBar() -> Element {
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

/// Top performers. Wave 5 additions: a "data freshness" timestamp at
/// the top of the section, and a `href="/portfolio/{symbol}"` link
/// on each card so a click-through to the portfolio detail page
/// works (the source uses server-side fetch + a per-symbol route
/// handler).
#[component]
fn TopPerformers() -> Element {
    rsx! {
        section { class: "top-performers",
            div { class: "container",
                div { class: "section-header",
                    h2 { class: "section-title", "Top EPS growth leaders" }
                    p { class: "section-sub", "Quarterly EPS growth, refreshed in real-time" }
                    // Data freshness — wave 5 addition. Format: a
                    // small clock + "Updated 2 min ago". The actual
                    // freshness timestamp would be filled in by the
                    // BFF (read from the BFF's cache invalidation
                    // time) — the static placeholder is the SSR
                    // default.
                    div { class: "top-performers-freshness",
                        Icon { name: "clock".to_string(), size: Some(14) }
                        span { "Updated just now" }
                    }
                }
                div { class: "top-performers-grid",
                    // Wave 23 T2 — each TopPerformers card now includes a
                    // TradingView external link (matches prod's
                    // `financial-data-table.tsx` `href={`https://www.tradingview.com/chart?symbol=${data.symbol}`}`).
                    // The card's primary `<a href="/portfolio/SYMBOL">`
                    // still navigates internally; the TradingView link
                    // is a small icon button in the top-right corner
                    // that opens in a new tab.
                    div { class: "card card-glass performer-card relative",
                        a { class: "absolute top-2 right-2 z-10 text-muted-foreground hover:text-yellow-400 transition-colors",
                            href: "https://www.tradingview.com/chart?symbol=GHC",
                            target: "_blank",
                            rel: "noopener noreferrer",
                            title: "View GHC on TradingView",
                            "aria-label": "View GHC on TradingView",
                            Icon { name: "external-link".to_string(), size: Some(14) }
                        }
                        a { class: "block", href: "/portfolio/GHC",
                            div { class: "performer-symbol", "GHC" }
                            div { class: "performer-price", "$6,535" }
                            Badge { kind: BadgeKind::Success, icon: Some("trending-up".to_string()), "+4657%" }
                        }
                    }
                    div { class: "card card-glass performer-card relative",
                        a { class: "absolute top-2 right-2 z-10 text-muted-foreground hover:text-yellow-400 transition-colors",
                            href: "https://www.tradingview.com/chart?symbol=ARAX",
                            target: "_blank",
                            rel: "noopener noreferrer",
                            title: "View ARAX on TradingView",
                            "aria-label": "View ARAX on TradingView",
                            Icon { name: "external-link".to_string(), size: Some(14) }
                        }
                        a { class: "block", href: "/portfolio/ARAX",
                            div { class: "performer-symbol", "ARAX" }
                            div { class: "performer-price", "$1,240" }
                            Badge { kind: BadgeKind::Success, icon: Some("trending-up".to_string()), "+312%" }
                        }
                    }
                    div { class: "card card-glass performer-card relative",
                        a { class: "absolute top-2 right-2 z-10 text-muted-foreground hover:text-yellow-400 transition-colors",
                            href: "https://www.tradingview.com/chart?symbol=NVTK",
                            target: "_blank",
                            rel: "noopener noreferrer",
                            title: "View NVTK on TradingView",
                            "aria-label": "View NVTK on TradingView",
                            Icon { name: "external-link".to_string(), size: Some(14) }
                        }
                        a { class: "block", href: "/portfolio/NVTK",
                            div { class: "performer-symbol", "NVTK" }
                            div { class: "performer-price", "$8,915" }
                            Badge { kind: BadgeKind::Success, icon: Some("trending-up".to_string()), "+287%" }
                        }
                    }
                    div { class: "card card-glass performer-card relative",
                        a { class: "absolute top-2 right-2 z-10 text-muted-foreground hover:text-yellow-400 transition-colors",
                            href: "https://www.tradingview.com/chart?symbol=GTC",
                            target: "_blank",
                            rel: "noopener noreferrer",
                            title: "View GTC on TradingView",
                            "aria-label": "View GTC on TradingView",
                            Icon { name: "external-link".to_string(), size: Some(14) }
                        }
                        a { class: "block", href: "/portfolio/GTC",
                            div { class: "performer-symbol", "GTC" }
                            div { class: "performer-price", "$412" }
                            Badge { kind: BadgeKind::Success, icon: Some("trending-up".to_string()), "+165%" }
                        }
                    }
                    div { class: "card card-glass performer-card relative",
                        a { class: "absolute top-2 right-2 z-10 text-muted-foreground hover:text-yellow-400 transition-colors",
                            href: "https://www.tradingview.com/chart?symbol=BIT",
                            target: "_blank",
                            rel: "noopener noreferrer",
                            title: "View BIT on TradingView",
                            "aria-label": "View BIT on TradingView",
                            Icon { name: "external-link".to_string(), size: Some(14) }
                        }
                        a { class: "block", href: "/portfolio/BIT",
                            div { class: "performer-symbol", "BIT" }
                            div { class: "performer-price", "$1,802" }
                            Badge { kind: BadgeKind::Success, icon: Some("trending-up".to_string()), "+142%" }
                        }
                    }
                }
            }
        }
    }
}

// === FeaturesGrid ===

/// Features grid. Six feature cards. Copy matches the source
/// `features-grid.tsx` exactly (the source has the same six).
#[component]
fn FeaturesGrid() -> Element {
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

/// Pricing teaser. Three tier cards. The "See plans" / "Start trial"
/// / "Contact sales" buttons link to `/plans` and `/contact`. The
/// Team card has a "Most popular" badge per the source.
#[component]
fn PricingTeaser() -> Element {
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

/// News preview. Three cards linking to `/news/{slug}`. The source
/// uses server-side fetch via `server-news-section.tsx`; the port
/// uses static content (the BFF does not pre-fetch per-page news
/// today; the next BFF iteration can read from `ctx.params`).
#[component]
fn NewsPreview() -> Element {
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

// === TestimonialsSection (NEW in Wave 5) ===

/// Testimonials section. Three cards, each with a quote, attribution
/// (name + role + company), and a star-rating visual. Copy is
/// paraphrased from the source's `testimonials-section.tsx`.
#[component]
fn TestimonialsSection() -> Element {
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

// === FAQSection (NEW in Wave 5) ===

/// FAQ section. Six-question accordion. The source uses shadcn
/// `<Accordion>`; the port uses native `<details>` / `<summary>`
/// elements (no JS, no extra CSS framework). The CSS in the
/// `wave5-page-depth-track-a` region styles `<details>` with the
/// design-system palette.
#[component]
fn FAQSection() -> Element {
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

/// CTA section. Wave 5 addition: a secondary "Talk to sales" link
/// alongside the primary "Connect wallet" CTA. The source has only
/// one button; the port matches the design-doc spec.
#[component]
fn CTASection() -> Element {
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
    use crate::auth::User;

    fn test_user() -> User {
        User {
            id: "test-user-id".to_string(),
            address: "0xtest".to_string(),
            chain_id: "56".to_string(),
            roles: vec![],
            email: None,
            tier: None,
            permissions: vec![],
            last_login_at: None,
            auth_method: crate::auth::AuthMethod::Wallet,
            display_name: Some("Test".to_string()),
        }
    }

    fn render_page_to_string(ctx: &PageContext) -> String {
        let (_meta, el) = render(ctx);
        dioxus_ssr::render_element(el)
    }

    /// Wave 5 — `test_render_smoke`. The `render` function returns
    /// a non-empty `Element` and `dioxus_ssr::render_element` produces
    /// a non-empty HTML string. Lighter check than the full
    /// section-marker test; runs first as a fast fail.
    #[test]
    fn test_render_smoke() {
        let ctx = PageContext {
            user: None,
            path: "/".to_string(),
            ..Default::default()
        };
        let (_meta, el) = render(&ctx);
        let html = dioxus_ssr::render_element(el);
        assert!(!html.is_empty(), "Home page must render non-empty HTML. Got: {}", html);
        assert!(html.len() > 100, "Home page HTML is suspiciously short ({} bytes).", html.len());
    }

    /// Wave 5 — `test_section_markers`. The home page must render
    /// every section the design doc claims (Hero, TrustBar,
    /// TopPerformers, FeaturesGrid, PricingTeaser, NewsPreview,
    /// TestimonialsSection, FAQSection, CTASection). The markers are
    /// the top-level `class="{section-name}"` on each `<section>`.
    #[test]
    fn test_section_markers() {
        let ctx = PageContext {
            user: None,
            path: "/".to_string(),
            ..Default::default()
        };
        let html = render_page_to_string(&ctx);
        for marker in &[
            "hero",
            "trust-bar",
            "top-performers",
            "features-grid-section",
            "pricing-teaser",
            "news-preview",
            "testimonials-section",
            "faq-section",
            "cta-section",
        ] {
            let needle = format!("class=\"{}\"", marker);
            assert!(
                html.contains(&needle),
                "Home page must contain section marker '{}'. Got: {}",
                needle, html
            );
        }
    }

    /// Wave 5 — `test_section_markers` for the new (Wave 5) sections
    /// specifically. Keeps the new section coverage as a discrete
    /// assertion so a regression on testimonials/FAQ is loud.
    #[test]
    fn test_wave5_new_sections_present() {
        let ctx = PageContext {
            user: None,
            path: "/".to_string(),
            ..Default::default()
        };
        let html = render_page_to_string(&ctx);
        // 3 testimonials — source has 3, port must have 3.
        let testimonial_count = html.matches("testimonial-card").count();
        assert_eq!(testimonial_count, 3, "Home page must render 3 testimonial cards. Got {} markers in: {}", testimonial_count, html);
        // 6 FAQ entries — source has 6, port must have 6.
        let faq_count = html.matches("class=\"faq-item\"").count();
        assert_eq!(faq_count, 6, "Home page must render 6 FAQ items. Got {} markers in: {}", faq_count, html);
        // Share button + chain selector — both Wave 5 Hero additions.
        assert!(html.contains("hero-share-btn"), "Hero must render the share button. Got: {}", html);
        assert!(html.contains("hero-chain-selector"), "Hero must render the chain selector. Got: {}", html);
        // Talk-to-sales secondary CTA — Wave 5 CTASection addition.
        assert!(html.contains("Talk to sales"), "CTASection must render 'Talk to sales' secondary link. Got: {}", html);
    }

    // === Wave 3b regression guards (preserved from prior tracks) ===

    /// Anonymous home page must render ProgressiveAuthBanner.
    #[test]
    fn home_renders_banner_for_anonymous_user() {
        let ctx = PageContext {
            user: None,
            path: "/".to_string(),
            ..Default::default()
        };
        let html = render_page_to_string(&ctx);
        assert!(
            html.contains("progressive-auth-banner"),
            "Anonymous home page must render ProgressiveAuthBanner. Got: {}",
            html
        );
    }

    /// Signed-in home page must NOT render ProgressiveAuthBanner.
    #[test]
    fn home_does_not_render_banner_for_signed_in_user() {
        let ctx = PageContext {
            user: Some(test_user()),
            path: "/".to_string(),
            ..Default::default()
        };
        let html = render_page_to_string(&ctx);
        assert!(
            !html.contains("progressive-auth-banner"),
            "Signed-in home page must NOT render ProgressiveAuthBanner. Got: {}",
            html
        );
    }
}
