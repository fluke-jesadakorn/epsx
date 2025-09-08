use axum::{
    routing::{get, post},
    Router,
};

use crate::web::auth::routes::AppState;
use crate::infrastructure::AppContainer;
use std::sync::Arc;
use super::discovery::*;
use super::token::{oidc_token, oidc_userinfo};
use super::authorization::{authorization_endpoint, handle_authorization_form, register_endpoint, handle_registration_form, password_reset_endpoint, handle_password_reset};
use super::revocation::revoke_token;
use super::introspection::introspect_token;
use super::session::oidc_logout;
use super::token_exchange::exchange_firebase_token;

/// Create OIDC routes with container
pub fn create_oidc_routes(_container: Arc<AppContainer>) -> Router {
    // Create simplified OIDC routes without complex state for now
    Router::new()
        // OIDC Discovery endpoints  
        .route("/.well-known/openid-configuration", get(oidc_discovery))
        .route("/oauth/jwks", get(jwks_endpoint))
        
        // OIDC Core Endpoints (Pure Authorization Code Flow) - using stateless handlers
        .route("/oauth/authorize", get(authorization_endpoint))
        .route("/oauth/register", get(register_endpoint))
        .route("/oauth/reset-password", get(password_reset_endpoint))
}

/// Create OIDC routes
pub fn oidc_routes() -> Router<AppState> {
    Router::new()
        // OIDC Discovery (standard and v2 paths)
        .route("/.well-known/openid-configuration", get(oidc_discovery))
        .route("/oauth/v2/.well-known/openid-configuration", get(oidc_discovery))
        
        // OIDC Core Endpoints (Pure Authorization Code Flow)
        .route("/oauth/authorize", get(authorization_endpoint).post(handle_authorization_form))
        .route("/oauth/token", post(oidc_token))
        .route("/oauth/userinfo", get(oidc_userinfo))
        .route("/oauth/jwks", get(jwks_endpoint))
        
        // Token Exchange Endpoints (Firebase Integration)
        .route("/api/v1/oidc/token/exchange", post(exchange_firebase_token))
        
        // Registration and Password Reset Endpoints
        .route("/oauth/register", get(register_endpoint).post(handle_registration_form))
        .route("/oauth/reset-password", get(password_reset_endpoint).post(handle_password_reset))
        
        // Standard OpenID Connect endpoints (RFC 7009, RFC 7662)
        .route("/oauth/revoke", post(revoke_token))
        .route("/oauth/introspect", post(introspect_token))
        .route("/oauth/logout", get(oidc_logout))
}

