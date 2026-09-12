//! Fixed watchlist operations preserve canonical owner and same-origin checks.
use crate::AppState;
use axum::{
    body::Body,
    extract::{Path, State},
    http::{HeaderMap, Request, StatusCode},
};
use epsx_dioxus_ui::{
    fullstack::LoadError,
    pages::{
        analytics::WatchlistData,
        portfolio::{hydrated::*, WatchlistLayoutData},
    },
};
async fn decode<T: serde::de::DeserializeOwned>(
    response: axum::response::Response,
) -> Result<T, LoadError> {
    if !response.status().is_success() {
        return Err(match response.status() {
            StatusCode::UNAUTHORIZED => LoadError::Unauthenticated,
            StatusCode::FORBIDDEN => LoadError::Forbidden,
            StatusCode::BAD_REQUEST | StatusCode::CONFLICT => LoadError::InvalidQuery,
            _ => LoadError::Unavailable,
        });
    }
    let bytes = axum::body::to_bytes(response.into_body(), 1024 * 1024)
        .await
        .map_err(|_| LoadError::Malformed)?;
    let value: serde_json::Value =
        serde_json::from_slice(&bytes).map_err(|_| LoadError::Malformed)?;
    if value.get("success").and_then(|value| value.as_bool()) != Some(true) {
        return Err(LoadError::Malformed);
    }
    serde_json::from_value(value.get("data").cloned().ok_or(LoadError::Malformed)?)
        .map_err(|_| LoadError::Malformed)
}
pub async fn load(state: AppState, headers: HeaderMap) -> Result<WatchlistLayoutData, LoadError> {
    decode(crate::api::watchlist_layout_get(State(state), headers).await).await
}
fn request(headers: HeaderMap, value: impl serde::Serialize) -> Result<Request<Body>, LoadError> {
    let mut request = Request::builder()
        .method("POST")
        .uri("/api/users/watchlist")
        .body(Body::from(
            serde_json::to_vec(&value).map_err(|_| LoadError::InvalidQuery)?,
        ))
        .map_err(|_| LoadError::InvalidQuery)?;
    *request.headers_mut() = headers;
    Ok(request)
}
pub async fn mutate(
    state: AppState,
    command: WatchlistCommand,
    headers: HeaderMap,
) -> Result<WatchlistChange, LoadError> {
    let response = match command {
        WatchlistCommand::Save { symbol, group_ids } => {
            return decode::<WatchlistData>(
                crate::api::watchlist_post(
                    State(state),
                    request(
                        headers,
                        serde_json::json!({"symbol":symbol,"group_ids":group_ids}),
                    )?,
                )
                .await,
            )
            .await
            .map(WatchlistChange::Symbols)
        }
        WatchlistCommand::Remove { symbol } => {
            return decode::<WatchlistData>(
                crate::api::watchlist_delete(State(state), Path(symbol), headers).await,
            )
            .await
            .map(WatchlistChange::Symbols)
        }
        WatchlistCommand::CreateGroup { name } => {
            crate::api::watchlist_group_post(
                State(state),
                request(headers, serde_json::json!({"name":name}))?,
            )
            .await
        }
        WatchlistCommand::RenameGroup { id, name } => {
            crate::api::watchlist_group_put(
                State(state),
                Path(id),
                request(headers, serde_json::json!({"name":name}))?,
            )
            .await
        }
        WatchlistCommand::DeleteGroup { id } => {
            crate::api::watchlist_group_delete(State(state), Path(id), headers).await
        }
        WatchlistCommand::Layout(value) => {
            crate::api::watchlist_layout_put(State(state), request(headers, value)?).await
        }
    };
    decode(response).await.map(WatchlistChange::Layout)
}
