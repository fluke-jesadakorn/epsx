use crate::primitives::*;

use dioxus::prelude::*;
use super::PageContext;
use super::PageMeta;
use crate::layout::{Navbar, Footer};

pub fn render(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::marketing("Home");
    let path = ctx.path.clone();
    (meta, rsx! {
        Navbar { user: ctx.user.clone(), current_path: Some(path) }
        Hero {}
        TrustBar {}
        TopPerformers {}
        FeaturesGrid {}
        PricingTeaser {}
        NewsPreview {}
        CTASection {}
        Footer {}
    })
}

#[component]
fn Hero() -> Element {
    rsx! {
        section { class: "hero",
            div { class: "hero-bg",
                div { class: "hero-orb hero-orb-1" }
                div { class: "hero-orb hero-orb-2" }
                div { class: "hero-orb hero-orb-3" }
            }
            div { class: "hero-inner",
                div { class: "hero-badge",
                    span { class: "hero-badge-dot" }
                    span { "Now live on BSC mainnet" }
                }
                h1 { class: "hero-title",
                    span { "The " }
                    span { class: "gradient-text", "Web3 commerce" }
                    span { " platform" }
                }
                p { class: "hero-subtitle",
                    "Visual page builder. On-chain payments. Programmable subscriptions. Paymaster-sponsored gas. All in one platform built for builders, merchants, and analysts."
                }
                div { class: "hero-actions",
                    a { class: "btn btn-gradient btn-lg", href: "/auth",
                        span { "Get started" }
                        span { class: "ml-2", Icon { name: "arrow-right".to_string(), size: Some(16) } }
                    }
                    a { class: "btn btn-outline btn-lg", href: "/plans", "View pricing" }
                }
                div { class: "hero-stats",
                    div { class: "hero-stat",
                        div { class: "hero-stat-value", "12K+" }
                        div { class: "hero-stat-label", "Active users" }
                    }
                    div { class: "hero-stat",
                        div { class: "hero-stat-value", "8.5M" }
                        div { class: "hero-stat-label", "EPS rankings processed" }
                    }
                    div { class: "hero-stat",
                        div { class: "hero-stat-value", "99.9%" }
                        div { class: "hero-stat-label", "Uptime" }
                    }
                }
            }
        }
    }
}

#[component]
fn TrustBar() -> Element {
    rsx! {
        section { class: "trust-bar",
            div { class: "trust-bar-inner",
                p { class: "trust-bar-label", "Built for" }
                div { class: "trust-bar-logos",
                    span { class: "trust-logo", "Traders" }
                    span { class: "trust-logo", "Analysts" }
                    span { class: "trust-logo", "Merchants" }
                    span { class: "trust-logo", "Developers" }
                    span { class: "trust-logo", "DAOs" }
                }
            }
        }
    }
}

#[component]
fn TopPerformers() -> Element {
    rsx! {
        section { class: "top-performers",
            div { class: "container",
                div { class: "section-header",
                    h2 { class: "section-title", "Top EPS growth leaders" }
                    p { class: "section-sub", "Quarterly EPS growth, refreshed in real-time" }
                }
                div { class: "top-performers-grid",
                    div { class: "card card-glass performer-card",
                        div { class: "performer-symbol", "GHC" }
                        div { class: "performer-price", "$6,535" }
                        Badge { kind: BadgeKind::Success, icon: Some("trending-up".to_string()), "+4657%" }
                    }
                    div { class: "card card-glass performer-card",
                        div { class: "performer-symbol", "ARAX" }
                        div { class: "performer-price", "$1,240" }
                        Badge { kind: BadgeKind::Success, icon: Some("trending-up".to_string()), "+312%" }
                    }
                    div { class: "card card-glass performer-card",
                        div { class: "performer-symbol", "NVTK" }
                        div { class: "performer-price", "$8,915" }
                        Badge { kind: BadgeKind::Success, icon: Some("trending-up".to_string()), "+287%" }
                    }
                    div { class: "card card-glass performer-card",
                        div { class: "performer-symbol", "GTC" }
                        div { class: "performer-price", "$412" }
                        Badge { kind: BadgeKind::Success, icon: Some("trending-up".to_string()), "+165%" }
                    }
                    div { class: "card card-glass performer-card",
                        div { class: "performer-symbol", "BIT" }
                        div { class: "performer-price", "$1,802" }
                        Badge { kind: BadgeKind::Success, icon: Some("trending-up".to_string()), "+142%" }
                    }
                }
            }
        }
    }
}

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
                    }
                }
            }
        }
    }
}
