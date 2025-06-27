use axum::{ http::{ Request, StatusCode }, middleware::Next, response::Response };
use tracing::{ debug, warn };
use std::sync::Arc;

pub async fn auth_middleware(
    mut req: Request<axum::body::Body>,
    next: Next
) -> Result<Response, StatusCode> {
    // Extract the Authorization header
    let auth_header = req.headers().get("Authorization");

    if let Some(auth_value) = auth_header {
        if let Ok(auth_str) = auth_value.to_str() {
            if auth_str.starts_with("Bearer ") {
                let token = &auth_str[7..]; // Remove "Bearer " prefix
                debug!("Verifying token from Authorization header");

                // Extract auth_service from request state
                let auth_service = req.extensions().get::<Arc<crate::auth::AuthService>>()
                    .ok_or_else(|| {
                        warn!("AuthService not found in request extensions");
                        StatusCode::INTERNAL_SERVER_ERROR
                    })?.clone();

                match auth_service.firebase_admin.verify_token(token).await {
                    Ok(user) => {
                        // Attach user to request extensions for downstream handlers
                        req.extensions_mut().insert(user);
                        return Ok(next.run(req).await);
                    }
                    Err(e) => {
                        warn!("Token verification failed: {:?}", e);
                        return Err(StatusCode::UNAUTHORIZED);
                    }
                }
            }
        }
    }

    // No valid auth header found
    warn!("No valid Authorization header found");
    Err(StatusCode::UNAUTHORIZED)
}

#[allow(dead_code)]
pub async fn admin_middleware(
    req: Request<axum::body::Body>,
    next: Next
) -> Result<Response, StatusCode> {
    // Check if user is in request extensions (set by auth_middleware)
    if let Some(user) = req.extensions().get::<crate::auth::FirebaseUser>() {
        if user.roles.contains(&crate::auth::UserRole::Admin) {
            return Ok(next.run(req).await);
        }
    }

    warn!("Admin access required but user is not admin");
    Err(StatusCode::FORBIDDEN)
}
