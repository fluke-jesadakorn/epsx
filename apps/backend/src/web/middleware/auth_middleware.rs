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
impl FromRequestParts<crate::web::auth::AppState> for AuthCtx
{
    type Rejection = StatusCode;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &crate::web::auth::AppState,
    ) -> Result<Self, Self::Rejection> {
        // Try to get session from cookie
        let cookies = Cookies::from_request_parts(parts, state)
            .await
            .map_err(|_| StatusCode::UNAUTHORIZED)?;

        if let Some(session_cookie) = cookies.get("sess_id") {
            let session_id = session_cookie.value();
            tracing::debug!("Found session cookie: {}", session_id);
            
            // Parse session ID
            let sess_id = SessId::from_str(session_id)
                .map_err(|_| {
                    tracing::warn!("Invalid session ID format: {}", session_id);
                    StatusCode::UNAUTHORIZED
                })?;
            
            // Validate session using PostgreSQL repository
            match state.session_repo.get(&sess_id).await {
                Ok(Some(session)) => {
                    tracing::debug!("Valid session found for user: {}", session.user_id());
                    
                    // Get user details to determine role
                    match state.user_repo.find_by_id(session.user_id()).await {
                        Ok(user) => {
                            Ok(AuthCtx {
                                user_id: user.id().clone(),
                                role: user.role().clone(),
                                sess: sess_id,
                            })
                        },
                        Err(_) => {
                            tracing::warn!("User not found for session: {}", session.user_id());
                            Err(StatusCode::UNAUTHORIZED)
                        }
                    }
                },
                Ok(None) => {
                    tracing::warn!("Session not found or expired: {}", session_id);
                    Err(StatusCode::UNAUTHORIZED)
                },
                Err(e) => {
                    tracing::error!("Database error during session validation: {:?}", e);
                    Err(StatusCode::INTERNAL_SERVER_ERROR)
                }
            }
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