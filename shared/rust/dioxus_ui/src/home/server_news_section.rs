//! `ServerNewsSection` — server-rendered "Latest News" carousel
//! (1 featured + 2 small cards).
//!
//! Port of
//! `apps-old/frontend/components/home/server-news-section.tsx`
//! (119 LoC). The TS source is an async server component that
//! fetches news via `getPublicNewsAction()` and renders a
//! featured card + 2 small cards. The Dioxus port renders the
//! same visual layout. News data is provided by the caller.

use crate::primitives::icon::Icon;

use dioxus::prelude::*;

#[component]
pub fn ServerNewsSection(
    /// Title for the featured (large) news card.
    #[props(default = "EPSX Q2 Platform Update".to_string())] featured_title: String,
    /// Summary for the featured card.
    #[props(default = "Sub-millisecond EPS rankings over 8.5M data points. New on-chain payment flow.".to_string())] featured_summary: String,
    /// Date string for the featured card.
    #[props(default = "Jun 12, 2026".to_string())] featured_date: String,
    /// Optional tag for the featured card.
    #[props(default = "Product".to_string())] featured_tag: String,
    /// Title for the first small card.
    #[props(default = "Building a scalable foundation".to_string())] small_1_title: String,
    /// Date for the first small card.
    #[props(default = "Jun 5, 2026".to_string())] small_1_date: String,
    /// Title for the second small card.
    #[props(default = "On-chain payment primer".to_string())] small_2_title: String,
    /// Date for the second small card.
    #[props(default = "May 28, 2026".to_string())] small_2_date: String,
) -> Element {
    rsx! {
        section { class: "server-news-section",
            div { class: "container mx-auto px-4 py-16 sm:py-24 lg:py-32",
                div { class: "mb-6 flex items-center justify-between server-news-section-head",
                    div { class: "flex items-center gap-3",
                        Icon { name: "newspaper".to_string(), size: Some(20), class_name: Some("text-cyan-400".to_string()) }
                        h2 { class: "text-xl font-bold text-foreground server-news-section-title", "Latest News" }
                    }
                    a { class: "flex items-center gap-1 text-sm text-cyan-400 hover:text-cyan-300 font-medium server-news-section-view-all",
                        href: "/news",
                        "View all "
                        Icon { name: "arrow-right".to_string(), size: Some(16) }
                    }
                }
                div { class: "space-y-4 server-news-section-list",
                    FeaturedNewsCard {
                        title: featured_title,
                        summary: featured_summary,
                        date: featured_date,
                        tag: featured_tag,
                    }
                    div { class: "grid gap-4 grid-cols-1 sm:grid-cols-2 server-news-section-small-row",
                        SmallNewsCard { title: small_1_title, date: small_1_date }
                        SmallNewsCard { title: small_2_title, date: small_2_date }
                    }
                }
            }
        }
    }
}

#[component]
fn FeaturedNewsCard(title: String, summary: String, date: String, tag: String) -> Element {
    rsx! {
        div { class: "server-news-featured-card relative rounded-2xl border border-slate-200 dark:border-slate-700 bg-slate-50 dark:bg-slate-800 p-6 overflow-hidden",
            div { class: "flex items-start justify-between gap-4 mb-3",
                span { class: "server-news-featured-tag inline-flex items-center px-2 py-0.5 rounded-full text-xs font-medium bg-orange-500/10 text-orange-500",
                    "{tag}"
                }
                span { class: "text-xs text-slate-500", "{date}" }
            }
            h3 { class: "server-news-featured-title text-2xl font-bold text-foreground mb-2", "{title}" }
            p { class: "server-news-featured-summary text-sm text-slate-600 dark:text-slate-300", "{summary}" }
        }
    }
}

#[component]
fn SmallNewsCard(title: String, date: String) -> Element {
    rsx! {
        div { class: "server-news-small-card p-4 rounded-xl border border-slate-200 dark:border-slate-700 bg-slate-50 dark:bg-slate-800",
            h4 { class: "server-news-small-title text-base font-bold text-foreground mb-1", "{title}" }
            span { class: "text-xs text-slate-500", "{date}" }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn server_news_section_smoke() {
        
    }
}
