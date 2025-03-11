use std::sync::Arc;
use std::future::Future;
use axum::{
    extract::{FromRequestParts, State},
    http::{Request, request::Parts},
    middleware::Next,
    response::Response,
    body::Body,
};

use crate::{
    app_state::AppState,
    error::AppError,
    models::auth::UserClaims,
};

const BEARER_PREFIX: &str = "Bearer ";

#[derive(Debug, Clone)]
pub struct AuthUser {
    pub id: String,
    pub claims: Option<UserClaims>,
}

impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
{
    type Rejection = AppError;

    fn from_request_parts(parts: &mut Parts, _state: &S) -> impl Future<Output = Result<Self, Self::Rejection>> + Send {
        async move {
            // Extract user ID from extensions (set by our middleware)
            let user_id = parts
                .extensions
                .get::<String>()
                .ok_or_else(|| AppError::Auth("User not authenticated".into()))?
                .clone();

            // Extract claims if available
            let claims = parts
                .extensions
                .get::<UserClaims>()
                .cloned();

            Ok(AuthUser {
                id: user_id,
                claims,
            })
        }
    }
}

pub async fn firebase_auth_middleware(
    State(state): State<Arc<AppState>>,
    mut request: Request<Body>,
    next: Next,
) -> Result<Response, AppError> {
    // Get the auth header
    let auth_header = request
        .headers()
        .get("Authorization")
        .and_then(|header| header.to_str().ok())
        .ok_or_else(|| AppError::Auth("Missing authorization header".into()))?;

    // Check if it's a Bearer token and extract the token
    if !auth_header.starts_with(BEARER_PREFIX) {
        return Err(AppError::Auth("Invalid authorization header format".into()));
    }
    let token = &auth_header[BEARER_PREFIX.len()..];

    // Verify the Firebase token
    let verified_token = state
        .firebase
        .verify_id_token(token)
        .await
        .map_err(|e| AppError::Auth(format!("Invalid token: {}", e)))?;

    // Extract claims if available
    if let Some(claims_value) = verified_token.claims.get("claims") {
        if let Ok(claims) = serde_json::from_value::<UserClaims>(claims_value.clone()) {
            request.extensions_mut().insert(claims);
        }
    }

    // Add the user ID to request extensions
    request.extensions_mut().insert(verified_token.sub);

    // Continue with the request
    Ok(next.run(request).await)
}

pub async fn session_auth_middleware(
    State(state): State<Arc<AppState>>,
    mut request: Request<Body>,
    next: Next,
) -> Result<Response, AppError> {
    // Get the session cookie
    let session_cookie = request
        .headers()
        .get("Cookie")
        .and_then(|header| header.to_str().ok())
        .and_then(|cookies| {
            cookies
                .split(';')
                .find(|cookie| cookie.trim().starts_with("__session="))
        })
        .and_then(|session| session.trim().strip_prefix("__session="))
        .ok_or_else(|| AppError::Auth("Missing session cookie".into()))?;

    // Verify the session cookie
    let verified_token = state
        .firebase
        .verify_session_cookie(session_cookie)
        .await
        .map_err(|e| AppError::Auth(format!("Invalid session: {}", e)))?;

    // Extract claims if available
    if let Some(claims_value) = verified_token.claims.get("claims") {
        if let Ok(claims) = serde_json::from_value::<UserClaims>(claims_value.clone()) {
            request.extensions_mut().insert(claims);
        }
    }

    // Add the user ID to request extensions
    request.extensions_mut().insert(verified_token.sub);

    // Continue with the request
    Ok(next.run(request).await)
}
