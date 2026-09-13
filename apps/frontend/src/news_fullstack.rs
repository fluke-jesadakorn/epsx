//! Strict public content adapters for Dioxus server functions.
use crate::AppState;
use epsx_dioxus_ui::{
    fullstack::LoadError,
    pages::news::{NewsListOutcome, NewsPost},
};

pub async fn load(state: AppState, raw: String) -> Result<NewsListOutcome, LoadError> {
    let query = crate::api::NewsQuery::from_raw_query(&raw).map_err(|_| LoadError::InvalidQuery)?;
    use crate::api::NewsListLoadOutcome as Source;
    Ok(
        match crate::api::load_news_list(state.content.as_ref(), &query).await {
            Source::Ready {
                articles,
                total,
                page,
                limit,
                total_pages,
                query,
                category,
            } => NewsListOutcome::Ready {
                articles: articles
                    .into_iter()
                    .map(|post| NewsPost {
                        id: post.id,
                        slug: post.slug,
                        title: post.title,
                        summary: post.summary,
                        cover_image_url: post.cover_image_url,
                        author: post.author,
                        published_at: post.published_at,
                        read_time: post.read_time,
                        tags: post.tags,
                        featured: post.featured,
                    })
                    .collect(),
                total,
                page,
                limit,
                total_pages,
                query,
                category,
            },
            Source::Empty {
                total,
                page,
                limit,
                total_pages,
                query,
                category,
            } => NewsListOutcome::Empty {
                total,
                page,
                limit,
                total_pages,
                query,
                category,
            },
            Source::Error { code } => NewsListOutcome::Error { code },
        },
    )
}

pub async fn detail(
    state: AppState,
    slug: String,
) -> Result<epsx_dioxus_ui::pages::news_detail::NewsDetailOutcome, LoadError> {
    use crate::api::NewsDetailLoadOutcome as Source;
    use epsx_dioxus_ui::pages::news_detail::{NewsArticle, NewsDetailOutcome};
    if !crate::api::valid_news_slug(&slug) {
        return Ok(NewsDetailOutcome::NotFound);
    }
    Ok(
        match crate::api::load_news_post(state.content.as_ref(), &slug).await {
            Source::Ready { article } => NewsDetailOutcome::Ready {
                article: NewsArticle {
                    id: article.id,
                    slug: article.slug,
                    title: article.title,
                    summary: article.summary,
                    body: article.body,
                    cover_image_url: article.cover_image_url,
                    author: article.author,
                    published_at: article.published_at,
                    tags: article.tags,
                },
            },
            Source::NotFound => NewsDetailOutcome::NotFound,
            Source::Error { code } => NewsDetailOutcome::Error { code },
        },
    )
}

pub async fn plans(
    state: AppState,
) -> Result<epsx_dioxus_ui::pages::plans::PublicPlansLoadOutcome, LoadError> {
    Ok(crate::api::load_public_plans(state.content.as_ref()).await)
}
