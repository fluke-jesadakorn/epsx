// OIDC Standard Endpoints
// Provides OpenID Connect compliant endpoints for token management

use std::sync::Arc;
use axum::{
    extract::{State, Form},
    response::Json,
    http::{StatusCode, HeaderMap},
};
use serde::{Deserialize, Serialize};
use tracing::{info, warn, error};

use super::service::OIDCService;
use crate::infra::firebase::FirebaseAdmin;

#[derive(Debug, Deserialize)]
pub struct TokenRequest {
    pub grant_type: String,
    pub code: Option<String>,
    pub refresh_token: Option<String>,
    pub client_id: Option<String>,
    pub client_secret: Option<String>,
    pub firebase_id_token: Option<String>, // Firebase ID token for authentication
}

#[derive(Debug, Serialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub id_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_in: u64,
    pub scope: String,
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub error_description: String,
}

#[derive(Debug, Deserialize)]
pub struct IntrospectRequest {
    pub token: String,
    pub token_type_hint: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct IntrospectResponse {
    pub active: bool,
    pub sub: Option<String>,
    pub aud: Option<String>,
    pub iss: Option<String>,
    pub exp: Option<u64>,
    pub iat: Option<u64>,
    pub token_use: Option<String>,
    pub permissions: Option<Vec<String>>,
    pub role: Option<String>,
    pub email: Option<String>,
}

/// POST /token - OIDC Token Endpoint
/// Exchange Firebase ID token for OIDC tokens or refresh tokens
pub async fn token_endpoint(
    State(oidc_service): State<Arc<OIDCService>>,
    State(firebase_admin): State<Arc<FirebaseAdmin>>,
    Form(token_request): Form<TokenRequest>,
) -> Result<Json<TokenResponse>, (StatusCode, Json<ErrorResponse>)> {
    info!("Token request: grant_type={}", token_request.grant_type);

    match token_request.grant_type.as_str() {
        "firebase_id_token" => {
            // Exchange Firebase ID token for OIDC tokens
            let firebase_id_token = token_request.firebase_id_token
                .ok_or_else(|| error_response("invalid_request", "firebase_id_token required for firebase_id_token grant"))?;

            // Verify Firebase ID token and get user
            let firebase_user = firebase_admin.verify_id_token(&firebase_id_token).await
                .map_err(|e| {
                    error!("Firebase ID token verification failed: {}", e);
                    error_response("invalid_grant", "Invalid Firebase ID token")
                })?;

            // Generate OIDC tokens
            let tokens = oidc_service.generate_tokens(&firebase_user, None).await
                .map_err(|e| {
                    error!("OIDC token generation failed: {}", e);
                    error_response("server_error", "Token generation failed")
                })?;

            info!("Generated OIDC tokens for Firebase user: {}", firebase_user.uid);

            Ok(Json(TokenResponse {
                access_token: tokens.access_token,
                id_token: tokens.id_token,
                refresh_token: tokens.refresh_token,
                token_type: tokens.token_type,
                expires_in: tokens.expires_in,
                scope: tokens.scope,
            }))
        }

        "refresh_token" => {
            // Refresh tokens using refresh token
            let refresh_token = token_request.refresh_token
                .ok_or_else(|| error_response("invalid_request", "refresh_token required for refresh_token grant"))?;

            let tokens = oidc_service.refresh_tokens(&refresh_token).await
                .map_err(|e| {
                    error!("Token refresh failed: {}", e);
                    error_response("invalid_grant", "Invalid refresh token")
                })?;

            info!("Refreshed OIDC tokens successfully");

            Ok(Json(TokenResponse {
                access_token: tokens.access_token,
                id_token: tokens.id_token,
                refresh_token: tokens.refresh_token,
                token_type: tokens.token_type,
                expires_in: tokens.expires_in,
                scope: tokens.scope,
            }))
        }

        _ => {
            warn!("Unsupported grant type: {}", token_request.grant_type);
            Err(error_response("unsupported_grant_type", "Only firebase_id_token and refresh_token grants are supported"))
        }
    }
}

/// POST /introspect - Token Introspection Endpoint (RFC 7662)
/// Validate and get information about a token
pub async fn introspect_endpoint(
    State(oidc_service): State<Arc<OIDCService>>,
    Form(introspect_request): Form<IntrospectRequest>,
) -> Result<Json<IntrospectResponse>, (StatusCode, Json<ErrorResponse>)> {
    info!("Token introspection request");

    // Validate the token
    match oidc_service.validate_bearer_token(&introspect_request.token).await {
        Ok(claims) => {
            info!("Token introspection successful for user: {}", claims.sub);
            
            Ok(Json(IntrospectResponse {
                active: true,
                sub: Some(claims.sub),
                aud: Some(claims.aud),
                iss: Some(claims.iss),
                exp: Some(claims.exp),
                iat: Some(claims.iat),
                token_use: Some(claims.token_use),
                permissions: claims.permissions,
                role: claims.role,
                email: claims.email,
            }))
        }
        Err(e) => {
            warn!("Token introspection failed: {}", e);
            
            // Return inactive token response (not an error)
            Ok(Json(IntrospectResponse {
                active: false,
                sub: None,
                aud: None,
                iss: None,
                exp: None,
                iat: None,
                token_use: None,
                permissions: None,
                role: None,
                email: None,
            }))
        }
    }
}

/// GET /.well-known/openid_configuration - OIDC Discovery Endpoint
pub async fn discovery_endpoint(
    State(oidc_service): State<Arc<OIDCService>>,
) -> Json<serde_json::Value> {
    info!("OIDC discovery request");
    Json(oidc_service.get_discovery_document())
}

/// GET /.well-known/jwks.json - JSON Web Key Set Endpoint
pub async fn jwks_endpoint(
    State(oidc_service): State<Arc<OIDCService>>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorResponse>)> {
    info!("JWKS request");
    
    match oidc_service.get_jwks() {
        Ok(jwks) => Ok(Json(jwks)),
        Err(e) => {
            error!("JWKS generation failed: {}", e);
            Err(error_response("server_error", "Failed to generate JWKS"))
        }
    }
}

/// POST /userinfo - OIDC UserInfo Endpoint
/// Returns user information based on access token
pub async fn userinfo_endpoint(
    State(oidc_service): State<Arc<OIDCService>>,
    headers: HeaderMap,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorResponse>)> {
    // Extract Bearer token from Authorization header
    let auth_header = headers.get("authorization")
        .and_then(|h| h.to_str().ok())
        .ok_or_else(|| error_response("invalid_request", "Authorization header required"))?;

    if !auth_header.starts_with("Bearer ") {
        return Err(error_response("invalid_token", "Bearer token required"));
    }

    let token = &auth_header[7..];

    // Validate token and extract user info
    match oidc_service.validate_bearer_token(token).await {
        Ok(claims) => {
            info!("UserInfo request for user: {}", claims.sub);

            let userinfo = serde_json::json!({
                "sub": claims.sub,
                "email": claims.email,
                "email_verified": claims.email_verified.unwrap_or(false),
                "name": claims.name,
                "permissions": claims.permissions.unwrap_or_default(),
                "role": claims.role.unwrap_or_else(|| "user".to_string()),
                "auth_time": claims.auth_time
            });

            Ok(Json(userinfo))
        }
        Err(e) => {
            error!("UserInfo token validation failed: {}", e);
            Err(error_response("invalid_token", "Invalid access token"))
        }
    }
}

/// POST /revoke - Token Revocation Endpoint (RFC 7009)
pub async fn revoke_endpoint(
    Form(_params): Form<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorResponse>)> {
    info!("Token revocation request");
    
    // In a full implementation, this would revoke the token in a token blacklist
    // For now, return success (tokens will naturally expire)
    warn!("Token revocation not fully implemented - tokens will expire naturally");
    
    Ok(Json(serde_json::json!({ "success": true })))
}

/// Health check endpoint for OIDC service
pub async fn health_endpoint(
    State(oidc_service): State<Arc<OIDCService>>,
) -> Json<serde_json::Value> {
    info!("OIDC health check");
    
    // Test JWKS generation to ensure keys are working
    let jwks_health = oidc_service.get_jwks().is_ok();
    
    Json(serde_json::json!({
        "status": if jwks_health { "healthy" } else { "unhealthy" },
        "service": "oidc",
        "version": "1.0.0",
        "jwks_available": jwks_health,
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

/// Helper function to create standardized error responses
fn error_response(error: &str, description: &str) -> (StatusCode, Json<ErrorResponse>) {
    let status = match error {
        "invalid_request" => StatusCode::BAD_REQUEST,
        "invalid_client" => StatusCode::UNAUTHORIZED,
        "invalid_grant" => StatusCode::BAD_REQUEST,
        "unauthorized_client" => StatusCode::UNAUTHORIZED,
        "unsupported_grant_type" => StatusCode::BAD_REQUEST,
        "invalid_scope" => StatusCode::BAD_REQUEST,
        "invalid_token" => StatusCode::UNAUTHORIZED,
        "server_error" => StatusCode::INTERNAL_SERVER_ERROR,
        _ => StatusCode::BAD_REQUEST,
    };

    (status, Json(ErrorResponse {
        error: error.to_string(),
        error_description: description.to_string(),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_error_response_creation() {
        let (status, response) = error_response("invalid_request", "Missing parameter");
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert_eq!(response.0.error, "invalid_request");
        assert_eq!(response.0.error_description, "Missing parameter");
    }

    #[test] 
    fn test_error_status_mapping() {
        let (status, _) = error_response("invalid_token", "Token expired");
        assert_eq!(status, StatusCode::UNAUTHORIZED);

        let (status, _) = error_response("server_error", "Internal error");
        assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[tokio::test]
    async fn test_token_request_deserialization() {
        let form_data = "grant_type=firebase_id_token&firebase_id_token=test_token";
        
        // This would be done by axum's Form extractor in practice
        let parsed: Result<TokenRequest, _> = serde_urlencoded::from_str(form_data);
        assert!(parsed.is_ok());
        
        let token_req = parsed.unwrap();
        assert_eq!(token_req.grant_type, "firebase_id_token");
        assert_eq!(token_req.firebase_id_token, Some("test_token".to_string()));
    }
}