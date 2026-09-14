//! Public news reads and navigation, owned by Dioxus from SSR through hydration.
use super::{NewsListOutcome, NewsPageBody};
use crate::fullstack::LoadError;
use dioxus::prelude::*;

#[cfg(feature = "server")]
#[derive(Clone)]
pub struct NewsProvider(pub NewsProviderCallback);

#[server(prefix = "/_server/frontend", endpoint = "news")]
pub async fn read_news(query: String) -> Result<Result<NewsListOutcome, LoadError>, ServerFnError> {
    use dioxus_server::axum::Extension;
    let Extension(provider) =
        dioxus_fullstack::FullstackContext::extract::<Extension<NewsProvider>, _>()
            .await
            .map_err(|_| ServerFnError::new("News provider unavailable"))?;
    Ok((provider.0)(query).await)
}
#[derive(Clone, Copy)]
pub struct NewsNavigation(pub EventHandler<String>);
pub fn follow(event: MouseEvent, navigation: Option<NewsNavigation>, url: String) {
    if event.modifiers().is_empty() {
        if let Some(navigation) = navigation {
            event.prevent_default();
            navigation.0.call(url);
        }
    }
}
#[component]
pub fn HydratedNews(query: ReadSignal<String>) -> Element {
    let initial_query = use_hook(|| query.read().clone());
    let seed_query = initial_query.clone();
    let initial = use_server_future(move || {
        let query = seed_query.clone();
        async move { read_news(query).await.map_err(|_| LoadError::Unavailable)? }
    })?;
    let seed = initial
        .read()
        .clone()
        .unwrap_or(Err(LoadError::Unavailable));
    let mut data = use_signal(|| seed.clone().ok());
    let mut error = use_signal(|| seed.err());
    let mut pending = use_signal(|| false);
    let mut generation = use_signal(|| 0_u64);
    let mut loaded_query = use_signal(|| initial_query);
    let mut retry = use_signal(|| 0_u64);
    let navigator = use_navigator();
    let navigate = use_callback(move |url: String| {
        navigator.push(url);
    });
    use_context_provider(|| NewsNavigation(navigate));
    use_effect(move || {
        let requested = query();
        let revision = retry();
        if requested == *loaded_query.peek() && revision == 0 {
            return;
        }
        let ticket = *generation.peek() + 1;
        generation.set(ticket);
        pending.set(true);
        error.set(None);
        spawn(async move {
            let result = read_news(requested.clone())
                .await
                .unwrap_or(Err(LoadError::Unavailable));
            if *generation.peek() != ticket {
                return;
            }
            pending.set(false);
            match result {
                Ok(NewsListOutcome::Error { .. }) => error.set(Some(LoadError::Unavailable)),
                Ok(value) => {
                    loaded_query.set(requested);
                    data.set(Some(value));
                }
                Err(failure) => error.set(Some(failure)),
            }
        });
    });
    rsx! {
        document::Title { "News — EPSX" }
        document::Meta { name: "description", content: "Latest EPSX platform news and updates." }
        section { "data-dioxus-news": "true", aria_busy: pending(),
            if pending() { p { role: "status", "Updating articles…" } }
            if let Some(failure) = error() {
                div { crate::fullstack::load_error::LoadErrorNotice { error: failure.clone(),
                    button { r#type: "button", class: "fe-button", disabled: pending(), onclick: move |_| { let next = *retry.peek() + 1; retry.set(next); }, "Try again" } }
                }
            }
            if let Some(outcome) = data() { NewsPageBody { outcome, retry_href: format!("/news?{}", query()) } }
        }
    }
}

pub(crate) fn response_status(status: u16) {
    #[cfg(feature = "server")]
    if let Some(mut context) = dioxus_fullstack::FullstackContext::current() {
        if let Ok(status) = http::StatusCode::from_u16(status) {
            context.set_current_http_status(dioxus_fullstack::HttpError {
                status,
                message: None,
            });
        }
    }
    #[cfg(not(feature = "server"))]
    let _ = status;
}

#[cfg(feature = "server")]
pub type NewsProviderCallback = std::sync::Arc<
    dyn Fn(
            String,
        ) -> std::pin::Pin<
            Box<dyn std::future::Future<Output = Result<NewsListOutcome, LoadError>> + Send>,
        > + Send
        + Sync,
>;
