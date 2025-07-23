// Authentication middleware for web layer

use axum::{
    async_trait,
    extract::{Request, FromRequestParts},
    http::{request::Parts, header::AUTHORIZATION, StatusCode},
    middleware::Next,
    response::Response,
};
use serde::{Serialize, Deserialize};
use tower_cookies::Cookies;
use crate::dom::values::{UserId, Role, SessId};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthCtx {
    pub user_id: UserId,
    pub role: Role,
    pub sess: SessId,
}

#[async_trait]
impl<S> FromRequestParts<S> for AuthCtx
where
    S: Send + Sync,
{
    type Rejection = StatusCode;

    async fn from_request_parts(
        parts: &mut Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        // Try to get session from cookie
        let cookies = Cookies::from_request_parts(parts, _state)
            .await
            .map_err(|_| StatusCode::UNAUTHORIZED)?;

        if let Some(session_cookie) = cookies.get("sess_id") {
            let session_id = session_cookie.value();
            tracing::debug!("Found session cookie: {}", session_id);
            
            // For now, create a mock AuthCtx
            // In production, you would validate the session and get user info
            let sess_id = SessId::from_str(session_id).unwrap_or_else(|_| SessId::generate());
            let user_id = UserId::generate(); // Mock user ID
            let role = Role::User; // Mock role
            
            Ok(AuthCtx {
                user_id,
                role,
                sess: sess_id,
            })
        } else {
            // Try Authorization header as fallback
            let auth_header = parts
                .headers
                .get(AUTHORIZATION)
                .and_then(|header| header.to_str().ok());

            if let Some(auth_header) = auth_header {
                if let Some(token) = auth_header.strip_prefix("Bearer ") {
                    tracing::debug!("Found Bearer token: {}", &token[..std::cmp::min(10, token.len())]);
                    
                    // Mock implementation - validate token and create AuthCtx  
                    let sess_id = SessId::generate(); // Generate new session for token auth
                    let user_id = UserId::generate();
                    let role = Role::User;
                    
                    Ok(AuthCtx {
                        user_id,
                        role,
                        sess: sess_id,
                    })
                } else {
                    Err(StatusCode::UNAUTHORIZED)
                }
            } else {
                tracing::warn!("No authentication found in request");
                Err(StatusCode::UNAUTHORIZED)
            }
        }
    }
}

#[derive(Debug)]
pub struct AuthenticatedRequest {
    pub auth: AuthCtx,
}

pub async fn auth_middleware(
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // The actual authentication is now handled by the AuthCtx extractor
    // This middleware can be used for additional logic if needed
    Ok(next.run(req).await)
}

pub fn require_permission(
    _permission: &str,
) -> impl Fn(Request, Next) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Response, StatusCode>> + Send>> + Clone {
    move |req, next| {
        Box::pin(async move {
            // Permission checking logic would go here
            // For now, just proceed to next middleware
            Ok(next.run(req).await)
        })
    }
}