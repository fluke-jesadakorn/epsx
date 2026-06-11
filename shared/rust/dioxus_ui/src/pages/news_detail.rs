//! /news/[slug] — news article detail.

use crate::primitives::*;
use crate::feedback::*;

use dioxus::prelude::*;
use super::PageContext;
use super::PageMeta;
use crate::layout::main_layout::MainLayout;
use crate::auth::AuthGate;
use crate::auth::ProgressiveAuthBanner;

pub fn render(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::marketing("News article");
    let slug = ctx.params.get("slug").cloned().unwrap_or_default();
    (meta, rsx! {
        MainLayout { ctx: ctx.clone(),
            if ctx.user.is_none() {
                ProgressiveAuthBanner {
                    feature: Some("news articles".to_string()),
                }
            }
            AuthGate { user: ctx.user.clone(), feature: Some("news articles".to_string()),
                article { class: "container page-content max-w-3xl",
                    a { class: "btn btn-sm btn-ghost mb-4", href: "/news", Icon { name: "arrow-left".to_string(), size: Some(16) } " Back to news" }
                    header { class: "mb-6",
                        h1 { class: "text-4xl font-black", "Welcome to EPSX" }
                        p { class: "text-muted-foreground mt-2", "Published 2024-09-15 · 3 min read · by EPSX Team" }
                    }
                    div { class: "prose prose-invert max-w-none",
                        p { class: "text-lg", "We're excited to launch the new EPSX platform — a Web3 commerce and analytics platform built for modern teams." }
                        h2 { class: "text-2xl font-bold mt-6", "What's new" }
                        p { "EPSX brings together everything you need to run a modern Web3 business: a visual page builder, on-chain payments, programmable subscriptions, real-time analytics, and developer APIs — all in one platform." }
                        h2 { class: "text-2xl font-bold mt-6", "Built on BSC" }
                        p { "We chose BSC mainnet for its low fees, fast finality, and broad wallet support. All EPSX features work seamlessly with MetaMask, WalletConnect, and Trust Wallet." }
                        h2 { class: "text-2xl font-bold mt-6", "Get started" }
                        p { "Connect your wallet to access dashboards, analytics, payments, and developer tools. No email, no password — just your wallet." }
                    }
                    div { class: "mt-8 flex gap-2",
                        a { class: "btn btn-primary", href: "/auth", "Get started" }
                        a { class: "btn btn-outline", href: "/plans", "View plans" }
                    }
                    div { class: "mt-6 text-sm text-muted-foreground",
                        "Slug: " code { "{slug}" }
                    }
                }
            }
        }
    })
}
