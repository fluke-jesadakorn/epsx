//! /news + /news/[slug] — news listing and detail.

use crate::primitives::*;
use crate::feedback::*;

use dioxus::prelude::*;
use super::PageContext;
use super::PageMeta;
use crate::layout::main_layout::MainLayout;
use crate::layout::PageHeader;
use crate::auth::AuthGate;
use crate::auth::ProgressiveAuthBanner;

pub fn render(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::marketing("News");
    let data: Option<serde_json::Value> = ctx.params.get("data_news")
        .and_then(|s| serde_json::from_str(s).ok());
    let posts: Vec<NewsPost> = data.as_ref()
        .and_then(|d| serde_json::from_value(d.get("posts").cloned().unwrap_or(serde_json::json!([]))).ok())
        .unwrap_or_else(default_posts);

    (meta, rsx! {
        MainLayout { ctx: ctx.clone(),
            if ctx.user.is_none() {
                ProgressiveAuthBanner {
                    feature: Some("personalized news".to_string()),
                }
            }
            AuthGate { user: ctx.user.clone(), feature: Some("news articles".to_string()),
                div { class: "container page-content",
                    PageHeader {
                        title: "News".to_string(),
                        description: Some("Latest updates from EPSX".to_string()),
                        icon: Some("newspaper".to_string()),
                    }
                    div { class: "grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6",
                        for p in posts.iter() {
                            NewsCard { post: p.clone() }
                        }
                    }
                }
            }
        }
    })
}

fn default_posts() -> Vec<NewsPost> {
    vec![
        NewsPost { slug: "welcome-to-epsx".into(), title: "Welcome to EPSX".into(), excerpt: "We're excited to launch the new EPSX platform — a Web3 commerce and analytics platform built for modern teams.".into(), author: "EPSX Team".into(), published_at: "2024-09-15".into(), read_time: "3 min".into() },
        NewsPost { slug: "bsc-integration".into(), title: "BSC mainnet integration live".into(), excerpt: "Full BSC mainnet support is now live with low fees and fast finality for all EPSX features.".into(), author: "EPSX Engineering".into(), published_at: "2024-09-10".into(), read_time: "5 min".into() },
        NewsPost { slug: "subscription-v2".into(), title: "Subscription v2: programmable plans".into(), excerpt: "Create, edit, and manage on-chain subscription plans with full merchant controls and refunds.".into(), author: "EPSX Product".into(), published_at: "2024-09-01".into(), read_time: "4 min".into() },
    ]
}

#[derive(Clone, Debug, serde::Deserialize, PartialEq)]
pub struct NewsPost {
    #[serde(default)] pub slug: String,
    #[serde(default)] pub title: String,
    #[serde(default)] pub excerpt: String,
    #[serde(default)] pub author: String,
    #[serde(default)] pub published_at: String,
    #[serde(default)] pub read_time: String,
}

#[component]
fn NewsCard(post: NewsPost) -> Element {
    rsx! {
        a { class: "card card-glass news-card block", href: "/news/{post.slug}",
            div { class: "card-body",
                span { class: "text-xs text-muted-foreground", "{post.published_at} · {post.read_time}" }
                h3 { class: "text-lg font-bold mt-2", "{post.title}" }
                p { class: "text-sm text-muted-foreground mt-1", "{post.excerpt}" }
                div { class: "flex items-center gap-2 mt-3",
                    span { class: "text-xs text-muted-foreground", "By {post.author}" }
                }
            }
        }
    }
}
