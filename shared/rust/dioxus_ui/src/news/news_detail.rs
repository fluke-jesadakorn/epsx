//! `NewsDetail` — full-page news article view.
//!
//! Port of `apps-old/frontend/components/news/news-detail.tsx`
//! (148 LoC). The TS source renders a full-bleed article view
//! with title + meta + body. The Dioxus port renders the same
//! structure with a `NewsDetailItem` data prop.

use crate::primitives::icon::Icon;

use dioxus::prelude::*;

#[derive(Clone, PartialEq, Debug, Default)]
pub struct NewsDetailItem {
    pub id: String,
    pub title: String,
    pub body: String,
    pub date: String,
    pub author: String,
    pub tag: String,
}

#[component]
pub fn NewsDetail(item: NewsDetailItem) -> Element {
    rsx! {
        article { class: "news-detail",
            div { class: "container mx-auto px-4 py-12 max-w-3xl",
                div { class: "news-detail-meta flex items-center gap-3 mb-4 text-sm text-slate-500",
                    span { class: "news-detail-tag inline-flex items-center px-2 py-0.5 rounded-full text-xs font-medium bg-cyan-500/10 text-cyan-500",
                        "{item.tag}"
                    }
                    span { class: "news-detail-date", "{item.date}" }
                    span { " · " }
                    span { class: "news-detail-author", "by {item.author}" }
                }
                h1 { class: "news-detail-title text-3xl sm:text-4xl font-bold text-foreground mb-6", "{item.title}" }
                div { class: "news-detail-body prose dark:prose-invert max-w-none",
                    for line in item.body.split('\n') {
                        if !line.is_empty() {
                            p { class: "news-detail-paragraph text-base text-slate-700 dark:text-slate-300 mb-4", "{line}" }
                        }
                    }
                }
                a { class: "news-detail-back mt-8 inline-flex items-center gap-2 text-orange-500 hover:text-orange-600",
                    href: "/news",
                    Icon { name: "arrow-left".to_string(), size: Some(16) }
                    "Back to news"
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn news_detail_item_default() {
        let i = NewsDetailItem::default();
        assert!(i.id.is_empty());
        assert!(i.body.is_empty());
    }

    #[test]
    fn news_detail_smoke() {
        
    }
}
