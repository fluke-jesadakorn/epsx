use super::LoadError;
use crate::pages::analytics::{
    AnalyticsFilters, AnalyticsQueryState, AnalyticsResponse, WatchlistData,
};
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AnalyticsData {
    pub query: AnalyticsQueryState,
    pub rankings: Result<AnalyticsResponse, LoadError>,
    pub filters: Result<AnalyticsFilters, LoadError>,
    pub watchlist: Result<Option<WatchlistData>, LoadError>,
    pub signed_in: bool,
}

/// No token, cookie, or upstream URL is serialized into the hydrated payload.
#[cfg(feature = "server")]
#[derive(Clone)]
pub struct AnalyticsProvider(pub AnalyticsProviderCallback);

#[server(prefix = "/_server/frontend", endpoint = "analytics")]
pub async fn read_analytics(
    query: AnalyticsQueryState,
) -> Result<Result<AnalyticsData, LoadError>, ServerFnError> {
    use dioxus_server::axum::Extension;
    let Extension(provider) =
        dioxus_fullstack::FullstackContext::extract::<Extension<AnalyticsProvider>, _>()
            .await
            .map_err(|_| ServerFnError::new("Analytics provider unavailable"))?;
    let headers = dioxus_fullstack::FullstackContext::extract::<http::HeaderMap, _>()
        .await
        .map_err(|_| ServerFnError::new("Request context unavailable"))?;
    Ok((provider.0)(query, headers).await)
}

/// Passed through context so shared SSR components opt into Dioxus events only
/// when mounted beneath the hydrated application.
#[derive(Clone, Copy)]
pub struct AnalyticsNavigation(pub EventHandler<String>);

#[derive(Clone, Copy)]
pub struct AnalyticsRequestedQuery(pub ReadSignal<String>);

pub fn follow_link(event: MouseEvent, navigation: Option<AnalyticsNavigation>, url: &str) {
    if let Some(navigation) = navigation {
        if event.modifiers().is_empty() {
            event.prevent_default();
            navigation.0.call(url.to_string());
        }
    }
}

fn parse_query(query: &str) -> Result<AnalyticsQueryState, LoadError> {
    AnalyticsQueryState::from_normalized_query(query).map_err(|_| LoadError::InvalidQuery)
}

async fn load(query: &str) -> Result<AnalyticsData, LoadError> {
    let value = read_analytics(parse_query(query)?)
        .await
        .map_err(|_| LoadError::Unavailable)??;
    value.rankings.as_ref().map_err(Clone::clone)?;
    Ok(value)
}

#[component]
pub fn HydratedAnalytics(query: ReadSignal<String>) -> Element {
    // Only the initial read participates in SSR hydration. Later reads keep the
    // last successful data visible while the requested query is pending.
    let initial_query = use_hook(|| query.read().clone());
    let seed_query = initial_query.clone();
    let initial = use_server_future(move || {
        let query = seed_query.clone();
        async move { load(&query).await }
    })?;
    let seed = initial
        .read()
        .clone()
        .unwrap_or(Err(LoadError::Unavailable));
    #[cfg(feature = "server")]
    if let Some(mut context) = dioxus_fullstack::FullstackContext::current() {
        let status = match &seed {
            Ok(_) => http::StatusCode::OK,
            Err(LoadError::InvalidQuery) => http::StatusCode::BAD_REQUEST,
            Err(LoadError::Unauthenticated) => http::StatusCode::UNAUTHORIZED,
            Err(LoadError::Forbidden) => http::StatusCode::FORBIDDEN,
            Err(LoadError::NotFound) => http::StatusCode::NOT_FOUND,
            Err(_) => http::StatusCode::BAD_GATEWAY,
        };
        context.set_current_http_status(dioxus_fullstack::HttpError {
            status,
            message: None,
        });
    }
    let mut data = use_signal(|| seed.clone().ok());
    let mut error = use_signal(|| seed.err());
    let mut pending = use_signal(|| false);
    let mut generation = use_signal(|| 0_u64);
    let mut loaded_query = use_signal(|| initial_query);
    let mut retry = use_signal(|| 0_u64);
    let mut attempted_retry = use_signal(|| 0_u64);
    let mut failed_query = use_signal(|| query.read().clone());
    let navigator = use_navigator();
    let navigate = use_callback(move |url: String| {
        navigator.push(url);
    });
    use_context_provider(|| AnalyticsNavigation(navigate));
    use_context_provider(|| AnalyticsRequestedQuery(query));
    use_effect(move || {
        let requested = query();
        let retry_count = retry();
        let is_retry = retry_count != *attempted_retry.peek();
        attempted_retry.set(retry_count);
        if requested == *loaded_query.peek() && !is_retry {
            return;
        }
        let ticket = *generation.peek() + 1;
        generation.set(ticket);
        pending.set(true);
        error.set(None);
        spawn(async move {
            let result = load(&requested).await.and_then(|value| {
                value.rankings.as_ref().map_err(Clone::clone)?;
                Ok(value)
            });
            if *generation.peek() != ticket {
                return;
            }
            pending.set(false);
            match result {
                Ok(value) => {
                    loaded_query.set(requested);
                    data.set(Some(value));
                }
                Err(failure) => {
                    failed_query.set(requested);
                    error.set(Some(failure));
                    if data.peek().is_some() {
                        navigator.replace(format!("/analytics?{}", loaded_query.peek()));
                    }
                }
            }
        });
    });
    rsx! {
        document::Title { "Company rankings — EPSX" }
        document::Meta { name: "description", content: "Explore company rankings, EPS performance and upcoming company reports." }
        section { "data-dioxus-analytics": "true", aria_busy: pending(),
            if pending() { p { role: "status", class: "fe-purchase-note", "Updating results…" } }
            if let Some(failure) = error() {
                div { role: "status", class: "fe-purchase-note",
                    p { "{failure.message()}" }
                    button { r#type: "button", class: "fe-button", onclick: move |_| {
                        navigator.push(format!("/analytics?{}", failed_query.peek()));
                        let value = *retry.peek() + 1;
                        retry.set(value);
                    }, "Try again" }
                }
            }
            if let Some(snapshot) = data() {
                match snapshot.rankings {
                    Ok(response) => rsx! { crate::pages::analytics::AnalyticsPage {
                        enterprise: true, signed_in: snapshot.signed_in,
                        response, filters: snapshot.filters.clone().ok(),
                        filters_state: (if snapshot.filters.is_ok() { "ready" } else { "unavailable" }).to_string(),
                        query: snapshot.query,
                        watchlist: snapshot.watchlist.clone().ok().flatten(),
                        watchlist_state: (if !snapshot.signed_in { "signed_out" } else if snapshot.watchlist.is_ok() { "ready" } else { "unavailable" }).to_string(),
                    } },
                    Err(failure) => rsx! { p { role: "status", "{failure.message()}" } },
                }
            }
        }
    }
}

pub fn submit_filters(event: FormEvent, navigation: Option<AnalyticsNavigation>) {
    if let Some(navigation) = navigation {
        event.prevent_default();
        let mut query = url::form_urlencoded::Serializer::new(String::new());
        for (key, value) in event.values() {
            if let dioxus::html::FormValue::Text(value) = value {
                if !value.is_empty() {
                    query.append_pair(&key, &value);
                }
            }
        }
        navigation.0.call(format!("/analytics?{}", query.finish()));
    }
}

#[cfg(feature = "server")]
pub type AnalyticsProviderCallback = std::sync::Arc<
    dyn Fn(
            AnalyticsQueryState,
            http::HeaderMap,
        ) -> std::pin::Pin<
            Box<dyn std::future::Future<Output = Result<AnalyticsData, LoadError>> + Send>,
        > + Send
        + Sync,
>;
