use std::sync::Arc;
use axum::{
    extract::{Path, State},
    routing::{post, get, delete},
    Router,
    Json,
    http::{StatusCode, header, HeaderMap},
    middleware::from_fn_with_state,
};
use serde::{Deserialize, Serialize};

use crate::{
    app_state::AppState,
    error::AppError,
    middleware::{firebase_auth_middleware, auth::AuthUser},
    models::auth::{UserClaims, UserRole},
};

#[derive(Deserialize)]
struct CreateSessionRequest {
    id_token: String,
}

#[derive(Serialize)]
struct CreateSessionResponse {
    session_cookie: String,
}

#[derive(Serialize)]
struct VerifyResponse {
    user_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    claims: Option<UserClaims>,
}

// Example of how to protect routes with Firebase authentication:
// ```rust
// Router::new()
//     .route("/protected", get(protected_handler))
//     .layer(from_fn_with_state(state, firebase_auth_middleware))
// ```
//
// Or for session-based authentication:
// ```rust
// Router::new()
//     .route("/protected", get(protected_handler))
//     .layer(from_fn_with_state(state, session_auth_middleware))
// ```
pub fn auth_routes(state: Arc<AppState>) -> Router {
    let app_state = state.clone();
    Router::new()
        .with_state(app_state)
        .route("/session", post(create_session))
        .route("/verify", get(verify_session))
        .route(
            "/revoke/:user_id",
            delete(revoke_tokens)
                .layer(from_fn_with_state(state.clone(), firebase_auth_middleware))
        )
}

async fn create_session(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateSessionRequest>,
) -> Result<(StatusCode, Json<CreateSessionResponse>), AppError> {
    // Verify the ID token and create a session cookie (5 days expiration)
    let session_cookie = state
        .firebase
        .create_session_cookie(&req.id_token, Some(std::time::Duration::from_secs(432000)))
        .await?;

    Ok((
        StatusCode::CREATED,
        Json(CreateSessionResponse { session_cookie }),
    ))
}

async fn verify_session(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
) -> Result<Json<VerifyResponse>, AppError> {
    let cookie_header = headers.get(header::COOKIE)
        .and_then(|value| value.to_str().ok())
        .ok_or_else(|| AppError::Auth("No cookie header provided".into()))?;

    let session_cookie = cookie_header
        .split(';')
        .find(|cookie| cookie.trim().starts_with("__session="))
        .and_then(|cookie| cookie.trim().strip_prefix("__session="))
        .ok_or_else(|| AppError::Auth("No session cookie provided".into()))?;

    let verified_token = state
        .firebase
        .verify_session_cookie(&session_cookie)
        .await?;

    // Get user claims if available
    let claims = verified_token.claims.get("claims")
        .and_then(|v| serde_json::from_value::<UserClaims>(v.clone()).ok());

    Ok(Json(VerifyResponse {
        user_id: verified_token.sub,
        claims,
    }))
}

async fn revoke_tokens(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<String>,
    auth_user: AuthUser,
) -> Result<StatusCode, AppError> {
    // Only admins can revoke tokens for other users
    if auth_user.id != user_id {
        let is_admin = auth_user.claims
            .map(|claims| matches!(claims.role, UserRole::Admin))
            .unwrap_or(false);
        
        if !is_admin {
            return Err(AppError::Auth("Unauthorized: Only admins can revoke tokens for other users".into()));
        }
    }
    
    state.firebase.revoke_refresh_tokens(&user_id).await?;
    Ok(StatusCode::NO_CONTENT)
}
