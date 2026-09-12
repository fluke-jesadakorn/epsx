use crate::AppState;
#[cfg(test)]
use axum::response::{Html, Redirect};
use axum::{
    extract::{OriginalUri, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};
use dioxus::prelude::*;
#[cfg(test)]
use epsx_dioxus_ui::payment::orders::{OrdersPage, OrdersPageProps};

#[cfg(test)]
pub async fn page(
    State(state): State<AppState>,
    headers: HeaderMap,
    OriginalUri(uri): OriginalUri,
) -> Response {
    let Some((token, _)) = state.session().verified_access_token(&headers).await else {
        return Redirect::to("/auth?return_url=%2Faccount%2Fpayments").into_response();
    };
    let suffix = uri.path().strip_prefix("/account/payments").unwrap_or("");
    if !suffix.is_empty() && uuid::Uuid::parse_str(suffix.trim_start_matches('/')).is_err() {
        return StatusCode::NOT_FOUND.into_response();
    }
    let base = std::env::var("API_URL").unwrap_or_else(|_| "http://127.0.0.1:8080".into());
    let mut url = format!("{base}/api/payments/pay-orders{suffix}");
    if let Some(q) = uri.query() {
        url.push('?');
        url.push_str(q);
    }
    let Ok(r) = state
        .payment
        .auth_client()
        .get(url)
        .bearer_auth(token)
        .send()
        .await
    else {
        return StatusCode::BAD_GATEWAY.into_response();
    };
    if !r.status().is_success() {
        return r.status().into_response();
    }
    let Ok(data) = r.json().await else {
        return StatusCode::BAD_GATEWAY.into_response();
    };
    let mut dom = VirtualDom::new_with_props(OrdersPage, OrdersPageProps { data, admin: false });
    dom.rebuild_in_place();
    (
        [("cache-control", "no-store")],
        Html(crate::enterprise::document(
            uri.path(),
            "Plan purchases",
            "EPSX",
            None,
            &crate::enterprise::navigation(uri.path(), uri.path(), true, None),
            &dioxus_ssr::render(&dom),
        )),
    )
        .into_response()
}
pub async fn api(
    State(state): State<AppState>,
    headers: HeaderMap,
    OriginalUri(uri): OriginalUri,
) -> Response {
    let Some((token, _)) = state.session().verified_access_token(&headers).await else {
        return StatusCode::UNAUTHORIZED.into_response();
    };
    let base = std::env::var("API_URL").unwrap_or_else(|_| "http://127.0.0.1:8080".into());
    let Ok(r) = state
        .payment
        .auth_client()
        .get(format!(
            "{base}{}",
            uri.path_and_query()
                .map(|v| v.as_str())
                .unwrap_or(uri.path())
        ))
        .bearer_auth(token)
        .send()
        .await
    else {
        return StatusCode::BAD_GATEWAY.into_response();
    };
    let status = r.status();
    let Ok(body) = r.bytes().await else {
        return StatusCode::BAD_GATEWAY.into_response();
    };
    (
        status,
        [
            ("content-type", "application/json"),
            ("cache-control", "no-store"),
        ],
        body,
    )
        .into_response()
}

/// Typed owner-scoped loader shared by SSR and subsequent Dioxus reads.
/// The caller cannot choose an upstream path, wallet, or authorization header.
pub async fn load_fullstack(
    state: AppState,
    query: epsx_dioxus_ui::payment::purchases::PurchaseQuery,
    headers: HeaderMap,
) -> Result<epsx_dioxus_ui::payment::purchases::PurchaseData, epsx_dioxus_ui::fullstack::LoadError>
{
    use epsx_dioxus_ui::{fullstack::LoadError, payment::purchases::PurchaseData};
    query.validate()?;
    let Some((token, _)) = state.session().verified_access_token(&headers).await else {
        return Err(LoadError::Unauthenticated);
    };
    let base = &state.api_url;
    let path = match query.order_id {
        Some(id) => format!("{base}/api/payments/pay-orders/{id}"),
        None => format!("{base}/api/payments/pay-orders?offset={}", query.offset),
    };
    let response = state
        .payment
        .auth_client()
        .get(path)
        .bearer_auth(token)
        .send()
        .await
        .map_err(|_| LoadError::Unavailable)?;
    if !response.status().is_success() {
        return Err(match response.status() {
            StatusCode::UNAUTHORIZED => LoadError::Unauthenticated,
            StatusCode::FORBIDDEN => LoadError::Forbidden,
            StatusCode::BAD_REQUEST => LoadError::InvalidQuery,
            StatusCode::NOT_FOUND => LoadError::NotFound,
            _ => LoadError::Unavailable,
        });
    }
    if query.order_id.is_some() {
        response
            .json()
            .await
            .map(PurchaseData::Detail)
            .map_err(|_| LoadError::Malformed)
    } else {
        response
            .json()
            .await
            .map(PurchaseData::List)
            .map_err(|_| LoadError::Malformed)
    }
}
