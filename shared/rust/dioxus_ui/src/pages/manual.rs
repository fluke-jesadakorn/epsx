use crate::primitives::*;

use dioxus::prelude::*;
use super::PageContext;
use super::PageMeta;
use crate::layout::main_layout::MainLayout;
use crate::layout::PageHeader;

pub fn render(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::marketing("Manual");
    (meta, rsx! {
        MainLayout { ctx: ctx.clone(),
            div { class: "container page-content",
                PageHeader { title: "Manual".to_string(), description: Some("Feature reference, with screenshots".to_string()), icon: Some("book".to_string()) }
                div { class: "manual-grid",
                    div { class: "manual-sidebar",
                        div { class: "card card-glass",
                            div { class: "card-header", h3 { class: "card-title", "Categories" } }
                            div { class: "card-body",
                                ul { class: "manual-nav",
                                    li { a { href: "#public", "Public" } }
                                    li { a { href: "#auth", "Auth" } }
                                    li { a { href: "#dashboard", "Dashboard" } }
                                    li { a { href: "#analytics", "Analytics" } }
                                    li { a { href: "#plans", "Plans" } }
                                    li { a { href: "#portfolio", "Portfolio" } }
                                    li { a { href: "#notifications", "Notifications" } }
                                    li { a { href: "#developer", "Developer" } }
                                }
                            }
                        }
                    }
                    div { class: "manual-content",
                        section { id: "public",
                            h2 { class: "section-title mt-8", "Public" }
                            div { class: "manual-feature-grid",
                                div { class: "card card-glass", div { class: "card-body", h4 { "Home" }, p { class: "text-muted-foreground text-sm", "Landing page with hero, top performers, features grid, pricing teaser, news preview, and CTA." } } }
                                div { class: "card card-glass", div { class: "card-body", h4 { "Pricing" }, p { class: "text-muted-foreground text-sm", "Three plan cards with feature lists." } } }
                                div { class: "card card-glass", div { class: "card-body", h4 { "Plans" }, p { class: "text-muted-foreground text-sm", "Plan tabs (Personal/Team/Enterprise/API) with detailed plan cards." } } }
                                div { class: "card card-glass", div { class: "card-body", h4 { "About" }, p { class: "text-muted-foreground text-sm", "Mission, vision, and DataTech platform details." } } }
                                div { class: "card card-glass", div { class: "card-body", h4 { "Contact" }, p { class: "text-muted-foreground text-sm", "Email, documentation, and developer contact cards." } } }
                                div { class: "card card-glass", div { class: "card-body", h4 { "News" }, p { class: "text-muted-foreground text-sm", "Article list with featured cards." } } }
                            }
                        }
                        section { id: "auth",
                            h2 { class: "section-title mt-8", "Auth" }
                            div { class: "manual-feature-grid",
                                div { class: "card card-glass", div { class: "card-body", h4 { "Sign in" }, p { class: "text-muted-foreground text-sm", "Wallet selection + SIWE challenge/verify." } } }
                                div { class: "card card-glass", div { class: "card-body", h4 { "Demo mode" }, p { class: "text-muted-foreground text-sm", "Skip SIWE with a synthetic user (dev only)." } } }
                            }
                        }
                        section { id: "dashboard",
                            h2 { class: "section-title mt-8", "Dashboard" }
                            div { class: "manual-feature-grid",
                                div { class: "card card-glass", div { class: "card-body", h4 { "Stats overview" }, p { class: "text-muted-foreground text-sm", "4 stat cards: earnings, watchlist, plans, API calls." } } }
                                div { class: "card card-glass", div { class: "card-body", h4 { "Quick actions" }, p { class: "text-muted-foreground text-sm", "Buttons linking to analytics, portfolio, plans, developer." } } }
                            }
                        }
                        section { id: "analytics",
                            h2 { class: "section-title mt-8", "Analytics" }
                            div { class: "manual-feature-grid",
                                div { class: "card card-glass", div { class: "card-body", h4 { "Rankings" }, p { class: "text-muted-foreground text-sm", "Grid of stock cards with EPS growth." } } }
                                div { class: "card card-glass", div { class: "card-body", h4 { "Filters" }, p { class: "text-muted-foreground text-sm", "Country, sector, sort, min EPS, min growth." } } }
                                div { class: "card card-glass", div { class: "card-body", h4 { "Export" }, p { class: "text-muted-foreground text-sm", "CSV / JSON export of current view." } } }
                            }
                        }
                        section { id: "plans",
                            h2 { class: "section-title mt-8", "Plans" }
                            div { class: "manual-feature-grid",
                                div { class: "card card-glass", div { class: "card-body", h4 { "Personal" }, p { class: "text-muted-foreground text-sm", "1-day, 1-month, 1-year, lifetime tiers." } } }
                                div { class: "card card-glass", div { class: "card-body", h4 { "Team" }, p { class: "text-muted-foreground text-sm", "Up to 10 seats, custom permissions." } } }
                                div { class: "card card-glass", div { class: "card-body", h4 { "Enterprise" }, p { class: "text-muted-foreground text-sm", "Unlimited seats, SLA, custom contracts." } } }
                            }
                        }
                        section { id: "portfolio",
                            h2 { class: "section-title mt-8", "Portfolio" }
                            div { class: "manual-feature-grid",
                                div { class: "card card-glass", div { class: "card-body", h4 { "Watchlist" }, p { class: "text-muted-foreground text-sm", "Add/remove symbols, see live prices." } } }
                                div { class: "card card-glass", div { class: "card-body", h4 { "Stock cards" }, p { class: "text-muted-foreground text-sm", "Per-symbol card with EPS, price, growth." } } }
                            }
                        }
                        section { id: "notifications",
                            h2 { class: "section-title mt-8", "Notifications" }
                            div { class: "manual-feature-grid",
                                div { class: "card card-glass", div { class: "card-body", h4 { "In-app" }, p { class: "text-muted-foreground text-sm", "Real-time updates via SSE." } } }
                                div { class: "card card-glass", div { class: "card-body", h4 { "Browser push" }, p { class: "text-muted-foreground text-sm", "Web Push API integration." } } }
                                div { class: "card card-glass", div { class: "card-body", h4 { "Email" }, p { class: "text-muted-foreground text-sm", "Handlebars templates via notification service." } } }
                            }
                        }
                        section { id: "developer",
                            h2 { class: "section-title mt-8", "Developer" }
                            div { class: "manual-feature-grid",
                                div { class: "card card-glass", div { class: "card-body", h4 { "API keys" }, p { class: "text-muted-foreground text-sm", "Per-wallet CRUD with masked display." } } }
                                div { class: "card card-glass", div { class: "card-body", h4 { "Usage" }, p { class: "text-muted-foreground text-sm", "Per-day call volume and rate-limit status." } } }
                                div { class: "card card-glass", div { class: "card-body", h4 { "Docs" }, p { class: "text-muted-foreground text-sm", "Endpoint reference with curl examples." } } }
                            }
                        }
                    }
                }
            }
        }
    })
}
