use axum::{
    extract::Request,
    http::{header, StatusCode},
    middleware::Next,
    response::Response,
};
use epsx_crypto::JwtService;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub mod permissions;
pub use permissions::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthUser {
    pub user_id: String,
    pub address: String,
    pub chain_id: String,
    pub roles: Vec<String>,
}

impl AuthUser {
    pub fn is_admin(&self) -> bool {
        self.roles.iter().any(|r| r == "admin")
    }

    pub fn is_editor(&self) -> bool {
        self.roles.iter().any(|r| r == "admin" || r == "editor" || r == "content_manager")
    }

    pub fn is_merchant(&self) -> bool {
        self.roles.iter().any(|r| r == "admin" || r == "merchant")
    }

    pub fn has_role(&self, role: &str) -> bool {
        self.roles.iter().any(|r| r == role)
    }
}

#[derive(Debug, Clone)]
pub struct JwtAuth {
    service: Arc<JwtService>,
}

impl JwtAuth {
    pub fn new(service: Arc<JwtService>) -> Self {
        Self { service }
    }

    pub fn from_secret(secret: &str) -> Self {
        let svc = Arc::new(JwtService::new(secret, 3600, 604800));
        Self { service: svc }
    }

    pub fn verify(&self, token: &str) -> Result<AuthUser, AuthError> {
        let claims = self
            .service
            .verify_token(token)
            .map_err(|e| AuthError::InvalidToken(e.to_string()))?;
        Ok(AuthUser {
            user_id: claims.sub,
            address: claims.address,
            chain_id: claims.chain_id,
            roles: claims.roles,
        })
    }

    pub fn extract_bearer(headers: &axum::http::HeaderMap) -> Option<&str> {
        headers
            .get(header::AUTHORIZATION)
            .and_then(|h| h.to_str().ok())
            .and_then(|s| s.strip_prefix("Bearer "))
    }
}

#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("Missing or invalid Authorization header")]
    Missing,
    #[error("Invalid token: {0}")]
    InvalidToken(String),
    #[error("Insufficient permissions")]
    Forbidden,
    #[error("Service error: {0}")]
    Service(String),
}

impl axum::response::IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (status, msg) = match self {
            AuthError::Missing => (StatusCode::UNAUTHORIZED, "Missing Authorization".to_string()),
            AuthError::InvalidToken(_) => (StatusCode::UNAUTHORIZED, "Invalid token".to_string()),
            AuthError::Forbidden => (StatusCode::FORBIDDEN, "Forbidden".to_string()),
            AuthError::Service(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Auth service error".to_string()),
        };
        (status, axum::Json(serde_json::json!({ "error": msg }))).into_response()
    }
}

impl<S> axum::extract::FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
{
    type Rejection = AuthError;

    async fn from_request_parts(parts: &mut axum::http::request::Parts, _state: &S) -> Result<Self, Self::Rejection> {
        if let Some(_token) = JwtAuth::extract_bearer(&parts.headers) {
            // Token is verified by the middleware before this extractor runs
        }
        parts
            .extensions
            .get::<AuthUser>()
            .cloned()
            .ok_or_else(|| AuthError::Service("Auth middleware not installed".into()))
    }
}

pub async fn jwt_middleware(
    axum::extract::State(auth): axum::extract::State<Arc<JwtAuth>>,
    mut req: Request,
    next: Next,
) -> Result<Response, AuthError> {
    if let Some(token) = JwtAuth::extract_bearer(req.headers()) {
        if let Ok(user) = auth.verify(token) {
            req.extensions_mut().insert(user);
        }
    }
    Ok(next.run(req).await)
}

pub async fn require_auth(
    axum::extract::State(auth): axum::extract::State<Arc<JwtAuth>>,
    mut req: Request,
    next: Next,
) -> Result<Response, AuthError> {
    let token = JwtAuth::extract_bearer(req.headers()).ok_or(AuthError::Missing)?;
    let user = auth.verify(token)?;
    req.extensions_mut().insert(user);
    Ok(next.run(req).await)
}

pub fn require_admin(user: &AuthUser) -> Result<(), AuthError> {
    if user.is_admin() {
        Ok(())
    } else {
        Err(AuthError::Forbidden)
    }
}

pub fn require_editor(user: &AuthUser) -> Result<(), AuthError> {
    if user.is_editor() {
        Ok(())
    } else {
        Err(AuthError::Forbidden)
    }
}

pub fn require_merchant(user: &AuthUser) -> Result<(), AuthError> {
    if user.is_merchant() {
        Ok(())
    } else {
        Err(AuthError::Forbidden)
    }
}
