//! /news/[slug] — news article detail.
//!
//! Wave 6A Track D — port of `apps-old/frontend/app/news/[slug]/page.tsx` +
//! `components/news/news-detail.tsx`.
//!
//! Section coverage (matches design doc §"Track D — news_detail"):
//! - `NewsDetailBody` — hero (with optional cover image + scrim) +
//!   tags + title + meta (date + read time) + accent bar + body
//! - `RelatedNewsList` — 3 related articles (cross-link)
//!
//! The source uses `MarkdownAsync` for body rendering; we render a
//! simple structured body via hardcoded headings/paragraphs that
/// match the static default post. The full markdown pipeline is a
/// Wave 7 enhancement (would need a markdown parser dep).

use crate::primitives::*;
use crate::feedback::*;

use dioxus::prelude::*;
use super::PageContext;
use super::PageMeta;
use crate::layout::main_layout::MainLayout;
use crate::auth::ProgressiveAuthBanner;

pub fn render(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::marketing("News article");
    let slug = ctx.params.get("slug").cloned().unwrap_or_default();
    let (title, date, read_time, author, content_blocks) = article_for(&slug);
    (meta, rsx! {
        MainLayout { ctx: ctx.clone(),
            if ctx.user.is_none() {
                ProgressiveAuthBanner {
                    feature: Some("news articles".to_string()),
                }
            }
            // === wave22-t3-news-blog news-detail body (no AuthGate; public page) ===
            article { class: "news-detail-body",
                // === wave6-auth-pages-depth-track-d news-detail hero ===
                section { class: "relative w-full overflow-hidden news-detail-hero",
                    div { class: "absolute inset-0 bg-gradient-to-br from-cyan-500/8 via-background to-purple-500/8" }
                    div { class: "relative z-10 max-w-4xl mx-auto px-4 sm:px-6 pt-8 pb-12 flex flex-col min-h-[240px] sm:min-h-[300px]",
                        a { class: "inline-flex items-center gap-2 text-sm mb-auto transition-colors news-detail-back",
                            href: "/news",
                            Icon { name: "arrow-left".to_string(), size: Some(16) }
                            " Back to News"
                        }
                        div {
                            div { class: "flex flex-wrap gap-2 mb-5",
                                span { class: "px-3 py-1 rounded-full text-[11px] font-bold tracking-[0.15em] uppercase bg-cyan-500/15 text-cyan-500 border border-cyan-500/25", "EPSX" }
                                span { class: "px-3 py-1 rounded-full text-[11px] font-bold tracking-[0.15em] uppercase bg-cyan-500/15 text-cyan-500 border border-cyan-500/25", "Update" }
                            }
                            h1 { class: "text-3xl sm:text-4xl lg:text-[2.75rem] font-extrabold leading-[1.1] tracking-tight mb-5 text-foreground",
                                "{title}"
                            }
                            div { class: "flex items-center gap-5 text-sm text-muted-foreground",
                                span { class: "flex items-center gap-1.5", Icon { name: "calendar".to_string(), size: Some(14) } " {date}" }
                                span { class: "flex items-center gap-1.5", Icon { name: "clock".to_string(), size: Some(14) } " {read_time} read" }
                                span { class: "flex items-center gap-1.5", Icon { name: "user".to_string(), size: Some(14) } " {author}" }
                            }
                        }
                    }
                }
                // === wave6-auth-pages-depth-track-d news-detail accent bar ===
                div { class: "h-[3px] news-detail-accent bg-gradient-to-r from-cyan-500 via-purple-500 to-cyan-500" }
                // === wave6-auth-pages-depth-track-d news-detail article body ===
                div { class: "max-w-3xl mx-auto px-4 sm:px-6 pt-12 pb-20 news-detail-content",
                    div { class: "prose prose-lg prose-neutral max-w-none",
                        for (heading, paragraphs) in content_blocks.iter() {
                            if !heading.is_empty() {
                                h2 { class: "text-2xl font-bold mt-12 mb-4 pb-3 border-b border-cyan-500/20 news-detail-h2", "{heading}" }
                            }
                            for p in paragraphs.iter() {
                                p { class: "leading-[1.8] text-muted-foreground news-detail-p", "{p}" }
                            }
                        }
                    }
                    div { class: "mt-16 pt-8 border-t border-border/20 news-detail-footer",
                        a { class: "inline-flex items-center gap-3 px-5 py-3 rounded-xl text-sm font-medium text-muted-foreground hover:text-foreground bg-card/50 hover:bg-card border border-border/20 hover:border-border/40 transition-all group news-detail-back-link",
                            href: "/news",
                            Icon { name: "arrow-left".to_string(), size: Some(16) }
                            " Back to all articles"
                        }
                    }
                }
                // === wave6-auth-pages-depth-track-d news-detail related list ===
                RelatedNewsList {}
                div { class: "mt-6 text-sm text-muted-foreground max-w-3xl mx-auto px-4 sm:px-6",
                    "Slug: " code { class: "font-mono", "{slug}" }
                }
            }
        }
    })
}

/// Look up the article metadata for a slug. Returns a default
/// "Welcome to EPSX" article if the slug is unknown (matches the
/// static Next.js fallback). Each `content_blocks` entry is a
/// `(heading, paragraphs)` tuple; an empty heading means "no
/// heading, just paragraphs" (used for the intro).
fn article_for(slug: &str) -> (&'static str, &'static str, &'static str, &'static str, Vec<(&'static str, Vec<&'static str>)>) {
    match slug {
        "bsc-integration" => (
            "BSC mainnet integration live",
            "September 10, 2024",
            "5 min",
            "EPSX Engineering",
            vec![
                ("", vec![
                    "Full BSC mainnet support is now live with low fees and fast finality for all EPSX features.",
                ]),
                ("What's available", vec![
                    "Every EPSX feature — payments, subscriptions, permissions, analytics — now settles on BNB Smart Chain. End-to-end transaction costs drop by an order of magnitude vs. legacy mainnets.",
                    "Connect with MetaMask, WalletConnect, or Trust Wallet. No special RPC config required; we ship public endpoints in the SDK.",
                ]),
                ("What changes for you", vec![
                    "Existing users don't need to do anything. New wallets created from this point forward default to BSC; older wallets are still accessible via the in-app chain switcher.",
                ]),
            ],
        ),
        "subscription-v2" => (
            "Subscription v2: programmable plans",
            "September 1, 2024",
            "4 min",
            "EPSX Product",
            vec![
                ("", vec![
                    "Create, edit, and manage on-chain subscription plans with full merchant controls and refunds.",
                ]),
                ("New plan primitives", vec![
                    "Plans now support tiered pricing, group access, and time-bounded upgrades. Merchants can define per-tier features, cap concurrent subscribers, and offer prorated refunds.",
                ]),
                ("Migration", vec![
                    "v1 plans continue to work. v2 plan authoring is opt-in from the merchant dashboard.",
                ]),
            ],
        ),
        _ => (
            "Welcome to EPSX",
            "September 15, 2024",
            "3 min",
            "EPSX Team",
            vec![
                ("", vec![
                    "We're excited to launch the new EPSX platform — a Web3 commerce and analytics platform built for modern teams.",
                ]),
                ("What's new", vec![
                    "EPSX brings together everything you need to run a modern Web3 business: a visual page builder, on-chain payments, programmable subscriptions, real-time analytics, and developer APIs — all in one platform.",
                ]),
                ("Built on BSC", vec![
                    "We chose BSC mainnet for its low fees, fast finality, and broad wallet support. All EPSX features work seamlessly with MetaMask, WalletConnect, and Trust Wallet.",
                ]),
                ("Get started", vec![
                    "Connect your wallet to access dashboards, analytics, payments, and developer tools. No email, no password — just your wallet.",
                ]),
            ],
        ),
    }
}

/// `RelatedNewsList` — 3 cross-linked related articles. Mirrors the
/// bottom of `news-detail.tsx` (the "related" section in the source
/// is implicit; we make it explicit so the section marker exists).
#[component]
fn RelatedNewsList() -> Element {
    rsx! {
        section { class: "max-w-3xl mx-auto px-4 sm:px-6 pb-12 news-related-list",
            h3 { class: "text-lg font-bold mb-4", "Related articles" }
            div { class: "grid grid-cols-1 sm:grid-cols-3 gap-4",
                RelatedCard { slug: "welcome-to-epsx".to_string(), title: "Welcome to EPSX".to_string(), read_time: "3 min".to_string() }
                RelatedCard { slug: "bsc-integration".to_string(), title: "BSC mainnet integration live".to_string(), read_time: "5 min".to_string() }
                RelatedCard { slug: "subscription-v2".to_string(), title: "Subscription v2: programmable plans".to_string(), read_time: "4 min".to_string() }
            }
        }
    }
}

#[component]
fn RelatedCard(slug: String, title: String, read_time: String) -> Element {
    rsx! {
        a { class: "card card-glass p-4 hover:border-cyan-500/40 transition-all news-related-card", href: "/news/{slug}",
            div { class: "text-xs text-muted-foreground mb-2", "{read_time} read" }
            div { class: "font-bold line-clamp-2", "{title}" }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::User;
    use crate::pages::PageContext;
    use crate::auth::user::AuthMethod;

    fn ctx(path: &str, slug: &str) -> PageContext {
        let user = User {
            id: "u1".to_string(),
            address: "0x1234…abcd".to_string(),
            chain_id: "56".to_string(),
            roles: vec!["user".to_string()],
            email: Some("test@epsx.io".to_string()),
            tier: Some("Pro".to_string()),
            permissions: vec!["news:read".to_string()],
            last_login_at: None,
            auth_method: AuthMethod::Wallet,
            display_name: Some("Test".to_string()),
        };
        let mut c = PageContext { user: Some(user), path: path.to_string(), ..Default::default() };
        c.params.insert("slug".into(), slug.to_string());
        c
    }

    #[test]
    fn test_render_smoke() {
        let (_meta, element) = render(&ctx("/news/welcome-to-epsx", "welcome-to-epsx"));
        let html = dioxus_ssr::render_element(element);
        assert!(html.contains("Welcome to EPSX"), "news detail must render title. Got: {}", html);
    }

    #[test]
    fn test_section_markers() {
        let (_meta, element) = render(&ctx("/news/welcome-to-epsx", "welcome-to-epsx"));
        let html = dioxus_ssr::render_element(element);
        for marker in ["news-detail-hero", "news-detail-content", "news-related-list", "news-detail-accent"] {
            assert!(html.contains(marker), "missing section marker: {}", marker);
        }
    }
}
