//! Public home dependencies retain independent outcomes and backend validation.
use crate::AppState;
use epsx_dioxus_ui::{
    fullstack::LoadError,
    pages::{home::HomeData, news::NewsListOutcome, plans::PublicPlansLoadOutcome},
};

pub async fn load(state: AppState) -> HomeData {
    let (rankings, news, plans) = tokio::join!(
        crate::ssr::load_home_analytics(state.analytics.as_ref(), "/"),
        crate::news_fullstack::load(state.clone(), String::new()),
        crate::news_fullstack::plans(state.clone()),
    );
    HomeData {
        rankings: rankings
            .unwrap_or(Err(crate::ssr::AnalyticsLoadError::Unavailable))
            .map_err(|error| match error {
                crate::ssr::AnalyticsLoadError::Malformed => LoadError::Malformed,
                crate::ssr::AnalyticsLoadError::Restricted => LoadError::Forbidden,
                crate::ssr::AnalyticsLoadError::Unavailable => LoadError::Unavailable,
            }),
        news: news.unwrap_or(NewsListOutcome::Error {
            code: "news_unavailable".into(),
        }),
        plans: plans.unwrap_or(PublicPlansLoadOutcome::Error {
            code: "plans_unavailable".into(),
        }),
    }
}
