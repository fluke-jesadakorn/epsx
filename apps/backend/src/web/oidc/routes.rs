use axum::{
    routing::{get, post},
    Router,
};

use crate::web::auth::routes::AppState;
use crate::web::templates::{TemplateFactory, FirebaseAuthTemplate};
use super::handlers::*;
use super::discovery::*;
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
    axum::extract::Form(request): axum::extract::Form<std::collections::HashMap<String, String>>,
) -> Result<axum::http::StatusCode, axum::http::StatusCode> {
    tracing::info!("OIDC token revocation request");
    
    let _token = request.get("token")
        .ok_or(axum::http::StatusCode::BAD_REQUEST)?;
    
    // TODO: Implement token revocation using FirebaseSessionService
    // For now, return success
    tracing::info!("Token revocation not implemented yet");
    
    Ok(axum::http::StatusCode::OK)
}

/// OAuth Token Introspection Endpoint  
/// POST /oauth/introspect
async fn oidc_introspect(
    axum::extract::State(_state): axum::extract::State<AppState>,
    axum::extract::Form(request): axum::extract::Form<std::collections::HashMap<String, String>>,
) -> Result<axum::response::Json<serde_json::Value>, axum::http::StatusCode> {
    tracing::info!("OIDC token introspection request");
    
    let _token = request.get("token")
        .ok_or(axum::http::StatusCode::BAD_REQUEST)?;
    
    // TODO: Implement token introspection using FirebaseSessionService
    // For now, return inactive
    let response = serde_json::json!({
        "active": false
    });
    
    Ok(axum::response::Json(response))
}