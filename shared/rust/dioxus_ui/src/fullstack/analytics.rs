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

#[derive(Clone, Copy)]
pub struct AnalyticsLoading {
    pub ready: Signal<bool>,
    pub pending: Signal<bool>,
}

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

async fn reload(
    query: &str,
    was_signed_in: bool,
    recovery: &super::frontend_auth::SessionRecovery,
) -> Result<AnalyticsData, LoadError> {
    let result = load(query).await.and_then(|value| {
        if was_signed_in && !value.signed_in {
            Err(LoadError::Unauthenticated)
        } else {
            Ok(value)
        }
    });
    if result == Err(LoadError::Unauthenticated) {
        recovery.restore().await?;
        // Retry the exact requested page and filters once, after cookies rotate.
        let value = load(query).await?;
        return if value.signed_in {
            Ok(value)
        } else {
            Err(LoadError::Unauthenticated)
        };
    }
    result
}

#[component]
pub fn HydratedAnalytics(query: ReadSignal<String>) -> Element {
    // Only the initial read participates in SSR hydration. Later reads keep the
    // last successful data visible while the requested query is pending.
    let auth_revision = use_context::<super::shell::AuthRevision>();
    let recovery = use_context::<super::frontend_auth::SessionRecovery>();
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
    let mut session_required = use_signal(|| {
        seed.as_ref().is_ok_and(|value| value.signed_in) || seed == Err(LoadError::Unauthenticated)
    });
    let mut data = use_signal(|| seed.clone().ok());
    let mut error = use_signal(|| seed.err());
    let mut pending = use_signal(|| false);
    let mut ready = use_signal(|| false);
    use_context_provider(|| AnalyticsLoading { ready, pending });
    let mut generation = use_signal(|| 0_u64);
    let mut loaded_query = use_signal(|| initial_query);
    let mut loaded_session = use_signal(|| (auth_revision.0)());
    let mut retry = use_signal(|| 0_u64);
    let mut attempted_retry = use_signal(|| 0_u64);
    let mut failed_query = use_signal(|| query.read().clone());
    let navigator = use_navigator();
    let navigate = use_callback(move |url: String| {
        if url.strip_prefix("/analytics?") == Some(query.peek().as_str()) {
            // A backend-capped selection can request the same URL again.
            let next = *retry.peek() + 1;
            retry.set(next);
        } else {
            navigator.push(url);
        }
    });
    use_context_provider(|| AnalyticsNavigation(navigate));
    use_context_provider(|| AnalyticsRequestedQuery(query));
    use_effect(move || {
        let requested = query();
        let session_revision = (auth_revision.0)();
        let retry_count = retry();
        let is_retry = retry_count != *attempted_retry.peek();
        attempted_retry.set(retry_count);
        let ticket = *generation.peek() + 1;
        generation.set(ticket);
        if requested == *loaded_query.peek()
            && session_revision == *loaded_session.peek()
            && !is_retry
            && data.peek().is_some()
        {
            // Back navigation must invalidate any response still in flight.
            pending.set(false);
            return;
        }
        let was_signed_in = *session_required.peek();
        if session_revision != *loaded_session.peek() {
            data.set(None);
        }
        pending.set(true);
        error.set(None);
        let recovery = recovery.clone();
        spawn(async move {
            let result = reload(&requested, was_signed_in, &recovery).await;
            if *generation.peek() != ticket {
                return;
            }
            pending.set(false);
            match result {
                Ok(value) => {
                    loaded_query.set(requested);
                    loaded_session.set(session_revision);
                    if value.signed_in {
                        session_required.set(true);
                    }
                    data.set(Some(value));
                }
                Err(failure) => {
                    failed_query.set(requested.clone());
                    let session_ended =
                        matches!(failure, LoadError::Unauthenticated | LoadError::Forbidden);
                    error.set(Some(failure));
                    if session_ended {
                        data.set(None);
                    } else if data.peek().is_some() {
                        navigator.replace(format!("/analytics?{}", loaded_query.peek()));
                    }
                }
            }
        });
    });
    rsx! {
        document::Title { "Company rankings — EPSX" }
        document::Meta { name: "description", content: "Explore company rankings, EPS performance and upcoming company reports." }
        section { "data-dioxus-analytics": "true", aria_busy: !ready() || pending(),
            onmounted: move |_| ready.set(true),
            if !ready() || pending() {
                div { class: "fe-analytics-loading", role: "status", aria_live: "polite",
                    span { class: "fe-loading-spinner", aria_hidden: "true" }
                    span { if !ready() { "Loading companies…" } else { "Updating companies…" } }
                    div { class: "fe-loading-track", aria_hidden: "true", span {} }
                }
            }
            if let Some(failure) = error() {
                div {
                    crate::fullstack::load_error::LoadErrorNotice { error: failure.clone(), return_path: format!("/analytics?{}", failed_query()),
                    button { r#type: "button", class: "fe-button", onclick: move |_| {
                        navigator.push(format!("/analytics?{}", failed_query.peek()));
                        let value = *retry.peek() + 1;
                        retry.set(value);
                    }, "Try again" } }
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
                    Err(failure) => rsx! { crate::fullstack::load_error::LoadErrorNotice { error: failure.clone(),  } },
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
