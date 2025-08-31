use axum::{
    routing::{get, post},
    Router,
};

use crate::web::auth::routes::AppState;
use crate::infra::AppContainer;
use std::sync::Arc;
use super::discovery::*;
use super::token::{oidc_token, oidc_userinfo};
use super::authorization::{authorization_endpoint, handle_authorization_form, register_endpoint, handle_registration_form, password_reset_endpoint, handle_password_reset};

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
        
        // Registration and Password Reset Endpoints
        .route("/oauth/register", get(register_endpoint).post(handle_registration_form))
        .route("/oauth/reset-password", get(password_reset_endpoint).post(handle_password_reset))
        
        // Additional endpoints for completeness
        .route("/oauth/revoke", post(oidc_revoke))
        .route("/oauth/introspect", post(oidc_introspect))
        .route("/oauth/logout", post(oidc_logout))
}


/// Additional OIDC handlers for completeness

/// OAuth Token Revocation Endpoint
/// POST /oauth/revoke
async fn oidc_revoke(
    axum::extract::State(_state): axum::extract::State<AppState>,
    headers: axum::http::HeaderMap,
    axum::extract::Form(request): axum::extract::Form<std::collections::HashMap<String, String>>,
) -> Result<axum::http::StatusCode, (axum::http::StatusCode, axum::response::Json<serde_json::Value>)> {
    use crate::core::ClientCredentialService;
    use crate::auth::{JWT, TOKEN_REVOCATION_SERVICE};
    
    tracing::info!("OIDC token revocation request");
    
    let token = request.get("token")
        .ok_or_else(|| (
            axum::http::StatusCode::BAD_REQUEST,
            axum::response::Json(serde_json::json!({
                "error": "invalid_request",
                "error_description": "Missing token parameter"
            }))
        ))?;
    
    // Validate client credentials for revocation (RFC 7009)
    let client_service = ClientCredentialService::new();
    if let Some(auth_header) = headers.get("authorization") {
        if let Ok(auth_str) = auth_header.to_str() {
            if auth_str.starts_with("Basic ") {
                if let Err(_) = client_service.validate_basic_auth(auth_str) {
                    return Err((
                        axum::http::StatusCode::UNAUTHORIZED,
                        axum::response::Json(serde_json::json!({
                            "error": "invalid_client",
                            "error_description": "Client authentication required"
                        }))
                    ));
                }
            }
        }
    }
    
    // Enhanced token revocation with proper blacklist management
    match JWT.verify(token) {
        Ok(claims) => {
            // Token is valid, add it to revocation list
            let expires_at = chrono::DateTime::from_timestamp(claims.exp as i64, 0)
                .unwrap_or_else(|| chrono::Utc::now() + chrono::Duration::hours(1));
            
            if let Err(e) = TOKEN_REVOCATION_SERVICE.revoke_token(
                &claims.jti,
                &claims.sub,
                "client_request",
                "Token explicitly revoked by client",
                expires_at,
            ).await {
                tracing::error!("Failed to revoke token: {}", e);
                // RFC 7009: Still return 200 even on internal errors
            } else {
                tracing::info!(
                    jti = %claims.jti,
                    user_id = %claims.sub,
                    email = %claims.email,
                    "Token successfully revoked"
                );
            }
        }
        Err(_) => {
            // Token is invalid or already expired - that's fine per RFC 7009
            tracing::debug!("Revocation requested for invalid/expired token");
        }
    }
    
    // RFC 7009: Return 200 OK regardless of whether token was found or valid
    Ok(axum::http::StatusCode::OK)
}

/// OAuth Token Introspection Endpoint  
/// POST /oauth/introspect
async fn oidc_introspect(
    axum::extract::State(_state): axum::extract::State<AppState>,
    headers: axum::http::HeaderMap,
    axum::extract::Form(request): axum::extract::Form<std::collections::HashMap<String, String>>,
) -> Result<axum::response::Json<serde_json::Value>, (axum::http::StatusCode, axum::response::Json<serde_json::Value>)> {
    use crate::core::ClientCredentialService;
    use crate::auth::{JWT, TOKEN_REVOCATION_SERVICE};
    
    tracing::info!("OIDC token introspection request");
    
    let token = request.get("token")
        .ok_or_else(|| (
            axum::http::StatusCode::BAD_REQUEST,
            axum::response::Json(serde_json::json!({
                "error": "invalid_request",
                "error_description": "Missing token parameter"
            }))
        ))?;
    
    // Validate client credentials for introspection (RFC 7662)
    let client_service = ClientCredentialService::new();
    if let Some(auth_header) = headers.get("authorization") {
        if let Ok(auth_str) = auth_header.to_str() {
            if auth_str.starts_with("Basic ") {
                if let Err(_) = client_service.validate_basic_auth(auth_str) {
                    return Err((
                        axum::http::StatusCode::UNAUTHORIZED,
                        axum::response::Json(serde_json::json!({
                            "error": "invalid_client",
                            "error_description": "Client authentication required"
                        }))
                    ));
                }
            }
        }
    }
    
    // Validate token using JWT service and check revocation status
    match JWT.verify(token) {
        Ok(claims) => {
            // Check if token is revoked
            let is_revoked = TOKEN_REVOCATION_SERVICE.is_token_revoked(&claims.jti).await;
            
            if is_revoked {
                tracing::info!(
                    jti = %claims.jti,
                    user_id = %claims.sub,
                    "Token introspection: token is revoked"
                );
                
                let response = serde_json::json!({
                    "active": false
                });
                Ok(axum::response::Json(response))
            } else {
                let response = serde_json::json!({
                    "active": true,
                    "sub": claims.sub,
                    "email": claims.email,
                    "name": claims.name,
                    "role": claims.package_tier,
                    "permissions": claims.permissions,
                    "package_tier": claims.package_tier,
                    "exp": claims.exp,
                    "iat": claims.iat,
                    "jti": claims.jti,
                    "scope": "openid profile email",
                    "token_type": "Bearer"
                });
                Ok(axum::response::Json(response))
            }
        }
        Err(_) => {
            // Token is invalid or expired
            tracing::debug!("Token introspection: token is invalid or expired");
            let response = serde_json::json!({
                "active": false
            });
            Ok(axum::response::Json(response))
        }
    }
}

/// OAuth Logout Endpoint
/// POST /oauth/logout
async fn oidc_logout(
    axum::extract::State(_state): axum::extract::State<AppState>,
    headers: axum::http::HeaderMap,
    body: Option<axum::extract::Json<serde_json::Value>>,
) -> Result<axum::response::Json<serde_json::Value>, (axum::http::StatusCode, axum::response::Json<serde_json::Value>)> {
    use crate::auth::{JWT, TOKEN_REVOCATION_SERVICE};
    
    tracing::info!("OIDC logout request");
    
    // Try to extract token from Authorization header
    let token = if let Some(auth_header) = headers.get("authorization") {
        if let Ok(auth_str) = auth_header.to_str() {
            if auth_str.starts_with("Bearer ") {
                Some(&auth_str[7..])
            } else {
                None
            }
        } else {
            None
        }
    } else {
        // Try to extract from request body
        body.as_ref()
            .and_then(|b| b.get("token"))
            .and_then(|t| t.as_str())
    };
    
    match token {
        Some(token_str) => {
            // Validate and revoke the token
            match JWT.verify(token_str) {
                Ok(claims) => {
                    let expires_at = chrono::DateTime::from_timestamp(claims.exp as i64, 0)
                        .unwrap_or_else(|| chrono::Utc::now() + chrono::Duration::hours(1));
                    
                    if let Err(e) = TOKEN_REVOCATION_SERVICE.revoke_token(
                        &claims.jti,
                        &claims.sub,
                        &claims.sub, // User is revoking their own token
                        "User logout",
                        expires_at,
                    ).await {
                        tracing::error!("Failed to revoke token during logout: {}", e);
                        return Err((
                            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                            axum::response::Json(serde_json::json!({
                                "error": "server_error",
                                "error_description": "Failed to process logout"
                            }))
                        ));
                    }
                    
                    tracing::info!(
                        jti = %claims.jti,
                        user_id = %claims.sub,
                        email = %claims.email,
                        "User logged out successfully"
                    );
                    
                    Ok(axum::response::Json(serde_json::json!({
                        "success": true,
                        "message": "Logged out successfully"
                    })))
                }
                Err(_) => {
                    // Token is invalid or expired - still consider logout successful
                    tracing::debug!("Logout requested with invalid/expired token");
                    
                    Ok(axum::response::Json(serde_json::json!({
                        "success": true,
                        "message": "Logged out successfully"
                    })))
                }
            }
        }
        None => {
            // No token provided - still return success for idempotent logout
            tracing::debug!("Logout requested without token");
            
            Ok(axum::response::Json(serde_json::json!({
                "success": true,
                "message": "Logged out successfully"
            })))
        }
    }
}