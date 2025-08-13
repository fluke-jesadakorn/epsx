use axum::{
    routing::{get, post},
    Router,
};

use crate::web::auth::routes::AppState;
use crate::web::templates::{TemplateFactory, FirebaseAuthTemplate};
use super::discovery::*;
use super::token::{oidc_token, oidc_userinfo};
use super::authorization::{authorization_endpoint, handle_authorization_form};

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
        
        // Firebase authentication endpoint
        .route("/firebase-auth", get(firebase_auth_handler))
        
        // Additional endpoints for completeness
        .route("/oauth/revoke", post(oidc_revoke))
        .route("/oauth/introspect", post(oidc_introspect))
}

/// Firebase Authentication Handler
/// GET /firebase-auth
async fn firebase_auth_handler(
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Result<FirebaseAuthTemplate, axum::http::StatusCode> {
    tracing::info!("Firebase auth page requested");
    
    let client_id = params.get("client_id").cloned().unwrap_or_default();
    let redirect_uri = params.get("redirect_uri").cloned().unwrap_or_default();
    let state = params.get("state").cloned().unwrap_or_default();
    let scope = params.get("scope").cloned().unwrap_or_default();
    let tenant_hint = params.get("tenant_hint").cloned();
    
    tracing::info!(
        client_id = %client_id,
        redirect_uri = %redirect_uri,
        state = %state,
        scope = %scope,
        tenant_hint = ?tenant_hint,
        "Firebase auth template parameters"
    );
    
    let template = TemplateFactory::create_firebase_auth_template(
        client_id,
        redirect_uri,
        state,
        scope,
        tenant_hint,
    );
    
    Ok(template)
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
    
    // Create in-memory token manager for revocation
    use super::token_management::TokenManagementTrait;
    let token_manager = super::token_management::InMemoryTokenManager::new();
    let revocation_request = super::token_management::TokenRevocationRequest {
        token: token.clone(),
        token_type_hint: request.get("token_type_hint").cloned(),
        revocation_reason: Some("Client requested revocation".to_string()),
        revoked_by: Some("oauth_client".to_string()),
        revoke_all: None,
    };
    
    match token_manager.revoke_token(revocation_request).await {
        Ok(_) => {
            tracing::info!("Token revocation successful");
            Ok(axum::http::StatusCode::OK)
        }
        Err(e) => {
            tracing::error!("Token revocation failed: {}", e);
            // RFC 7009: Return 200 OK regardless of whether token was found
            Ok(axum::http::StatusCode::OK)
        }
    }
}

/// OAuth Token Introspection Endpoint  
/// POST /oauth/introspect
async fn oidc_introspect(
    axum::extract::State(_state): axum::extract::State<AppState>,
    headers: axum::http::HeaderMap,
    axum::extract::Form(request): axum::extract::Form<std::collections::HashMap<String, String>>,
) -> Result<axum::response::Json<serde_json::Value>, (axum::http::StatusCode, axum::response::Json<serde_json::Value>)> {
    use crate::core::ClientCredentialService;
    use crate::auth::JWT_SERVICE;
    
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
    
    // Validate token using JWT service
    match JWT_SERVICE.validate_token(token) {
        Ok(claims) => {
            let response = serde_json::json!({
                "active": true,
                "sub": claims.sub,
                "email": claims.email,
                "name": claims.name,
                "role": claims.role,
                "permissions": claims.permissions,
                "package_tier": claims.package_tier,
                "admin_modules": claims.admin_modules,
                "exp": claims.exp,
                "iat": claims.iat,
                "scope": "openid profile email",
                "token_type": "Bearer"
            });
            Ok(axum::response::Json(response))
        }
        Err(_) => {
            // Token is invalid or expired
            let response = serde_json::json!({
                "active": false
            });
            Ok(axum::response::Json(response))
        }
    }
}