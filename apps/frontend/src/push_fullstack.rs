//! Browser push uses the same origin/auth/validation gates as the existing API.
use crate::AppState;
use axum::{
    body::Body,
    extract::State,
    http::{HeaderMap, Request, StatusCode},
};
use epsx_dioxus_ui::{
    fullstack::LoadError,
    pages::account::push::{PushCommand, PushStatus},
};

pub async fn run(
    state: AppState,
    command: PushCommand,
    headers: HeaderMap,
) -> Result<PushStatus, LoadError> {
    let response = match command {
        PushCommand::Status => crate::api::notification_push_status(State(state), headers).await,
        PushCommand::Subscribe(subscription) => {
            let mut request = Request::builder()
                .method("PUT")
                .uri("/api/v1/notifications/push")
                .body(Body::from(
                    serde_json::to_vec(&subscription).map_err(|_| LoadError::InvalidQuery)?,
                ))
                .map_err(|_| LoadError::InvalidQuery)?;
            *request.headers_mut() = headers;
            crate::api::notification_push_subscribe(State(state), request).await
        }
        PushCommand::Unsubscribe { endpoint } => {
            let mut request = Request::builder()
                .method("DELETE")
                .uri("/api/v1/notifications/push")
                .body(Body::from(
                    serde_json::to_vec(&serde_json::json!({"endpoint":endpoint}))
                        .map_err(|_| LoadError::InvalidQuery)?,
                ))
                .map_err(|_| LoadError::InvalidQuery)?;
            *request.headers_mut() = headers;
            crate::api::notification_push_unsubscribe(State(state), request).await
        }
    };
    if !response.status().is_success() {
        return Err(match response.status() {
            StatusCode::UNAUTHORIZED => LoadError::Unauthenticated,
            StatusCode::FORBIDDEN => LoadError::Forbidden,
            StatusCode::BAD_REQUEST | StatusCode::PAYLOAD_TOO_LARGE => LoadError::InvalidQuery,
            _ => LoadError::Unavailable,
        });
    }
    let body = axum::body::to_bytes(response.into_body(), 8 * 1024)
        .await
        .map_err(|_| LoadError::Malformed)?;
    serde_json::from_slice(&body).map_err(|_| LoadError::Malformed)
}
