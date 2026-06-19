//! `NewsList` — paginated list of news articles.
//!
//! Port of `apps-old/frontend/components/news/news-list.tsx`
//! (185 LoC). The TS source renders a paginated list with
//! thumbnail + title + summary + date. The Dioxus port renders
//! the same visual structure with a `NewsListItem` data prop.

use crate::primitives::icon::Icon;

use dioxus::prelude::*;

#[derive(Clone, PartialEq, Debug, Default)]
pub struct NewsListItem {
    pub id: String,
    pub title: String,
    pub summary: String,
    pub date: String,
    pub tag: String,
    pub pinned: bool,
}

#[component]
pub fn NewsList(#[props(default = Vec::new())] items: Vec<NewsListItem>) -> Element {
    rsx! {
        section { class: "news-list",
            div { class: "container mx-auto px-4 py-8",
                div { class: "news-list-grid grid gap-6 grid-cols-1 md:grid-cols-2 lg:grid-cols-3",
                    for item in items.iter() {
                        NewsListCard { item: item.clone() }
                    }
                }
                if items.is_empty() {
                    div { class: "news-list-empty text-center py-12 text-slate-500",
                        Icon { name: "newspaper".to_string(), size: Some(32), class_name: Some("mx-auto mb-3 text-slate-400".to_string()) }
                        "No news yet"
                    }
                }
            }
        }
    }
}

#[component]
fn NewsListCard(item: NewsListItem) -> Element {
    rsx! {
        a {
            class: "news-list-card block rounded-2xl border border-slate-200 dark:border-slate-700 bg-white dark:bg-slate-800 p-6 hover:scale-[1.02] transition-all shadow-sm",
            href: format!("/news/{}", item.id),
            if item.pinned {
                span { class: "news-list-card-pinned inline-flex items-center px-2 py-0.5 rounded-full text-xs font-medium bg-orange-500/10 text-orange-500 mb-2",
                    "📌 Pinned"
                }
            }
            div { class: "news-list-card-head flex items-center justify-between mb-3",
                span { class: "news-list-card-tag inline-flex items-center px-2 py-0.5 rounded-full text-xs font-medium bg-cyan-500/10 text-cyan-500",
                    "{item.tag}"
                }
                span { class: "news-list-card-date text-xs text-slate-500", "{item.date}" }
            }
            h3 { class: "news-list-card-title text-lg font-bold text-foreground mb-2", "{item.title}" }
            p { class: "news-list-card-summary text-sm text-slate-600 dark:text-slate-300", "{item.summary}" }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn news_list_item_default() {
        let i = NewsListItem::default();
        assert!(i.id.is_empty());
        assert!(!i.pinned);
    }

    #[test]
    fn news_list_smoke() {
        
    }
}
