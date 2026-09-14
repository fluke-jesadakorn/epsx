//! Typed developer UI adapters reuse the canonical origin, session and input gates.
use crate::AppState;
use axum::{
    body::Body,
    extract::{Path, State},
    http::{header, HeaderMap, Request, StatusCode},
};
use epsx_dioxus_ui::{
    fullstack::LoadError,
    pages::developer::{hydrated::*, DeveloperOverview},
};

fn failure(error: crate::api::DeveloperLoadError) -> LoadError {
    match error {
        crate::api::DeveloperLoadError::Forbidden => LoadError::Forbidden,
        crate::api::DeveloperLoadError::Unavailable => LoadError::Unavailable,
        crate::api::DeveloperLoadError::Malformed => LoadError::Malformed,
    }
}
pub async fn load(
    state: AppState,
    days: i32,
    headers: HeaderMap,
) -> Result<DeveloperOverview, LoadError> {
    if !matches!(days, 7 | 30 | 90) {
        return Err(LoadError::InvalidQuery);
    }
    let Some((token, _)) = state.session().verified_access_token(&headers).await else {
        return Err(LoadError::Unauthenticated);
    };
    crate::api::load_developer_overview_for_ssr(state.wallet.as_ref(), &token, days)
        .await
        .map_err(failure)
}
pub async fn docs(state: AppState) -> Result<serde_json::Value, LoadError> {
    crate::api::load_developer_openapi_for_ssr(state.wallet.as_ref())
        .await
        .map_err(failure)
}
fn request(
    headers: HeaderMap,
    value: impl serde::Serialize,
    identity: Option<String>,
) -> Result<Request<Body>, LoadError> {
    let mut request = Request::builder()
        .method("POST")
        .uri("/api/developer/fullstack")
        .body(Body::from(
            serde_json::to_vec(&value).map_err(|_| LoadError::InvalidQuery)?,
        ))
        .map_err(|_| LoadError::InvalidQuery)?;
    *request.headers_mut() = headers;
    request.headers_mut().insert(
        header::CONTENT_TYPE,
        header::HeaderValue::from_static("application/json"),
    );
    if let Some(identity) = identity {
        request.headers_mut().insert(
            "idempotency-key",
            identity.parse().map_err(|_| LoadError::InvalidQuery)?,
        );
    }
    Ok(request)
}
pub async fn mutate(
    state: AppState,
    command: DeveloperCommand,
    headers: HeaderMap,
) -> Result<DeveloperMutationResult, LoadError> {
    let (kind, response) = match command {
        DeveloperCommand::Create {
            input,
            idempotency_key,
        } => (
            0,
            crate::api::developer_key_create(
                State(state),
                request(headers, input, Some(idempotency_key))?,
            )
            .await,
        ),
        DeveloperCommand::Revoke {
            id,
            reason,
            idempotency_key,
        } => (
            1,
            crate::api::developer_key_revoke(
                State(state),
                Path(id),
                request(
                    headers,
                    serde_json::json!({"reason":reason}),
                    Some(idempotency_key),
                )?,
            )
            .await,
        ),
        DeveloperCommand::Try(input) => (
            2,
            crate::api::developer_try(State(state), request(headers, input, None)?).await,
        ),
    };
    if !response.status().is_success() {
        return Err(match response.status() {
            StatusCode::UNAUTHORIZED => LoadError::Unauthenticated,
            StatusCode::FORBIDDEN => LoadError::Forbidden,
            StatusCode::BAD_REQUEST | StatusCode::CONFLICT | StatusCode::PAYLOAD_TOO_LARGE => {
                LoadError::InvalidQuery
            }
            _ => LoadError::Unavailable,
        });
    }
    let bytes = axum::body::to_bytes(response.into_body(), 2 * 1024 * 1024)
        .await
        .map_err(|_| LoadError::Malformed)?;
    let envelope: serde_json::Value =
        serde_json::from_slice(&bytes).map_err(|_| LoadError::Malformed)?;
    if envelope.get("success").and_then(|value| value.as_bool()) != Some(true) {
        return Err(LoadError::Malformed);
    }
    let data = envelope.get("data").cloned().ok_or(LoadError::Malformed)?;
    match kind {
        0 => serde_json::from_value(data)
            .map(DeveloperMutationResult::Created)
            .map_err(|_| LoadError::Malformed),
        1 => {
            #[derive(serde::Deserialize)]
            struct Revoked {
                id: uuid::Uuid,
                status: String,
                replayed: bool,
            }
            let value: Revoked = serde_json::from_value(data).map_err(|_| LoadError::Malformed)?;
            Ok(DeveloperMutationResult::Revoked {
                id: value.id,
                status: value.status,
                replayed: value.replayed,
            })
        }
        _ => serde_json::from_value(data)
            .map(DeveloperMutationResult::Tried)
            .map_err(|_| LoadError::Malformed),
    }
}
