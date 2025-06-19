use axum::{
    extract::State,
    http::{Request, StatusCode},
    response::Response,
    body::Body,
    middleware::Next,
};
use std::sync::Arc;

use super::AuthService;

pub async fn auth_middleware(
    State(auth_service): State<Arc<AuthService>>,
    mut request: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode>
{
    let auth_header = request
        .headers()
        .get("Authorization")
        .and_then(|value| value.to_str().ok());

    match auth_header {
        Some(auth) if auth.starts_with("Bearer ") => {
            let token = &auth[7..];
            match auth_service.verify_token(token).await {
                Ok(claims) => {
                    // Add user claims to request extensions
                    request.extensions_mut().insert(claims);
                    Ok(next.run(request).await)
                }
                Err(_) => Err(StatusCode::UNAUTHORIZED),
            }
        }
        _ => Err(StatusCode::UNAUTHORIZED),
    }
}
