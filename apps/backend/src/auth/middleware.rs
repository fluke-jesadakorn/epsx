use crate::auth::AuthService;
use axum::{
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
};
use http::header::AUTHORIZATION;
use tracing::{debug, error, info, warn};

#[derive(Clone, Debug)]
pub struct AuthUser {
    pub user_id: String,
}

use axum::body::Body;

pub async fn auth_middleware(
    State(auth_service): State<AuthService>,
    mut request: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    debug!("Processing authentication for request to {}", request.uri().path());

    // Extract bearer token from Authorization header
    let token = request
        .headers()
        .get(AUTHORIZATION)
        .and_then(|auth_header| auth_header.to_str().ok())
        .and_then(|auth_str| {
            if auth_str.starts_with("Bearer ") {
                Some(auth_str[7..].to_string())
            } else {
                None
            }
        })
        .ok_or_else(|| {
            warn!("Missing or invalid Authorization header");
            StatusCode::UNAUTHORIZED
        })?;

    debug!("Successfully extracted bearer token");

    // Verify token with Firebase Admin
    let user_id = auth_service
        .verify_id_token(&token)
        .await
        .map_err(|e| {
            error!("Token verification failed: {}", e);
            StatusCode::UNAUTHORIZED
        })?;

    info!("User authenticated successfully: {}", user_id);

    // Add authenticated user to request extensions
    request.extensions_mut().insert(AuthUser { user_id });

    // Continue with the request
    Ok(next.run(request).await)
}

// Helper function to extract AuthUser from request extensions
pub fn get_auth_user(request: &Request<Body>) -> Result<&AuthUser, StatusCode> {
    request
        .extensions()
        .get::<AuthUser>()
        .ok_or_else(|| {
            warn!("AuthUser not found in request extensions");
            StatusCode::UNAUTHORIZED
        })
}
