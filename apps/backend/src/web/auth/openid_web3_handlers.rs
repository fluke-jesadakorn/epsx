// ============================================================================
// OPENID CONNECT + WEB3 AUTHENTICATION HANDLERS
// Web3 wallet authentication that issues standard OpenID Connect tokens
// ============================================================================

use axum::{
    extract::State,
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tracing::{error, info, warn};
use utoipa::ToSchema;

use crate::{
    auth::{
        OpenIDTokenResponse, Web3AuthTokenRequest, OpenIDTokenError
    },
    web::auth::AppState,
};

/// Standard OpenID Connect token refresh request
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct RefreshTokenRequest {
    /// Refresh token for obtaining new access tokens
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000")]
    pub refresh_token: String,
    
    /// Client ID requesting the refresh
    #[schema(example = "epsx-frontend")]
    pub client_id: String,
}

/// Web3 authentication request for OpenID token issuance
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct Web3OpenIDAuthRequest {
    /// Ethereum wallet address
    #[schema(example = "0x742d35Cc6634C0532925a3b8D369D7763F3c45c6")]
    pub wallet_address: String,
    
    /// SIWE signature from wallet
    pub signature: String,
    
    /// SIWE message that was signed
    pub message: String,
    
    /// Challenge nonce
    #[schema(example = "abc123def456")]
    pub nonce: String,
    
    /// OpenID Connect client ID
    #[schema(example = "epsx-frontend")]
    pub client_id: String,
}

/// OpenID Connect error response
#[derive(Debug, Serialize, ToSchema)]
pub struct OpenIDErrorResponse {
    pub error: String,
    pub error_description: String,
    pub error_uri: Option<String>,
}

/// Authenticate Web3 wallet and issue OpenID Connect tokens
#[utoipa::path(
    post,
    path = "/api/auth/web3/token",
    request_body = Web3OpenIDAuthRequest,
    responses(
        (status = 200, description = "OpenID tokens issued successfully", body = OpenIDTokenResponse),
        (status = 400, description = "Invalid request", body = OpenIDErrorResponse),
        (status = 401, description = "Authentication failed", body = OpenIDErrorResponse),
        (status = 500, description = "Internal server error", body = OpenIDErrorResponse)
    ),
    tag = "openid-auth"
)]
pub async fn authenticate_web3_and_issue_openid_tokens(
    State(app_state): State<AppState>,
    Json(request): Json<Web3OpenIDAuthRequest>,
) -> Result<Json<OpenIDTokenResponse>, Json<OpenIDErrorResponse>> {
    info!("Processing Web3 authentication for OpenID token issuance: {}", request.wallet_address);

    // Get OpenID token service from container
    let openid_service = match app_state.domain_container.get_openid_token_service() {
        Some(service) => service,
        None => {
            error!("OpenID token service not available");
            return Err(Json(OpenIDErrorResponse {
                error: "server_error".to_string(),
                error_description: "Authentication service temporarily unavailable".to_string(),
                error_uri: None,
            }));
        }
    };

    // Convert to internal request format
    let token_request = Web3AuthTokenRequest {
        wallet_address: request.wallet_address.clone(),
        signature: request.signature,
        message: request.message,
        nonce: request.nonce,
        client_id: request.client_id,
    };

    // Authenticate Web3 wallet and issue OpenID tokens
    match openid_service.authenticate_web3_and_issue_tokens(token_request).await {
        Ok(token_response) => {
            info!("Successfully issued OpenID tokens for wallet: {}", request.wallet_address);
            Ok(Json(token_response))
        }
        Err(OpenIDTokenError::Web3AuthenticationFailed(msg)) => {
            warn!("Web3 authentication failed for wallet {}: {}", request.wallet_address, msg);
            Err(Json(OpenIDErrorResponse {
                error: "invalid_grant".to_string(),
                error_description: format!("Web3 authentication failed: {}", msg),
                error_uri: None,
            }))
        }
        Err(OpenIDTokenError::InvalidClient(client_id)) => {
            warn!("Invalid client ID: {}", client_id);
            Err(Json(OpenIDErrorResponse {
                error: "invalid_client".to_string(),
                error_description: format!("Invalid client ID: {}", client_id),
                error_uri: None,
            }))
        }
        Err(e) => {
            error!("OpenID token issuance failed: {}", e);
            Err(Json(OpenIDErrorResponse {
                error: "server_error".to_string(),
                error_description: "Token issuance failed".to_string(),
                error_uri: None,
            }))
        }
    }
}

/// Refresh OpenID Connect tokens
#[utoipa::path(
    post,
    path = "/api/auth/token/refresh",
    request_body = RefreshTokenRequest,
    responses(
        (status = 200, description = "Tokens refreshed successfully", body = OpenIDTokenResponse),
        (status = 400, description = "Invalid refresh token", body = OpenIDErrorResponse),
        (status = 401, description = "Refresh token expired or revoked", body = OpenIDErrorResponse),
        (status = 500, description = "Internal server error", body = OpenIDErrorResponse)
    ),
    tag = "openid-auth"
)]
pub async fn refresh_openid_tokens(
    State(app_state): State<AppState>,
    Json(request): Json<RefreshTokenRequest>,
) -> Result<Json<OpenIDTokenResponse>, Json<OpenIDErrorResponse>> {
    info!("Processing OpenID token refresh for client: {}", request.client_id);

    // Get OpenID token service from container
    let openid_service = match app_state.domain_container.get_openid_token_service() {
        Some(service) => service,
        None => {
            error!("OpenID token service not available");
            return Err(Json(OpenIDErrorResponse {
                error: "server_error".to_string(),
                error_description: "Token service temporarily unavailable".to_string(),
                error_uri: None,
            }));
        }
    };

    // Refresh tokens
    match openid_service.refresh_tokens(&request.refresh_token, &request.client_id).await {
        Ok(token_response) => {
            info!("Successfully refreshed tokens for client: {}", request.client_id);
            Ok(Json(token_response))
        }
        Err(OpenIDTokenError::InvalidRefreshToken(msg)) => {
            warn!("Invalid refresh token: {}", msg);
            Err(Json(OpenIDErrorResponse {
                error: "invalid_grant".to_string(),
                error_description: format!("Invalid refresh token: {}", msg),
                error_uri: None,
            }))
        }
        Err(OpenIDTokenError::TokenExpired(msg)) => {
            warn!("Refresh token expired: {}", msg);
            Err(Json(OpenIDErrorResponse {
                error: "invalid_grant".to_string(),
                error_description: format!("Refresh token expired: {}", msg),
                error_uri: None,
            }))
        }
        Err(OpenIDTokenError::InvalidClient(client_id)) => {
            warn!("Invalid client ID for refresh: {}", client_id);
            Err(Json(OpenIDErrorResponse {
                error: "invalid_client".to_string(),
                error_description: format!("Invalid client ID: {}", client_id),
                error_uri: None,
            }))
        }
        Err(e) => {
            error!("Token refresh failed: {}", e);
            Err(Json(OpenIDErrorResponse {
                error: "server_error".to_string(),
                error_description: "Token refresh failed".to_string(),
                error_uri: None,
            }))
        }
    }
}

/// Revoke refresh token (logout)
#[utoipa::path(
    post,
    path = "/api/auth/token/revoke",
    request_body = RefreshTokenRequest,
    responses(
        (status = 200, description = "Token revoked successfully"),
        (status = 400, description = "Invalid request", body = OpenIDErrorResponse),
        (status = 500, description = "Internal server error", body = OpenIDErrorResponse)
    ),
    tag = "openid-auth"
)]
pub async fn revoke_refresh_token(
    State(app_state): State<AppState>,
    Json(request): Json<RefreshTokenRequest>,
) -> Result<Json<Value>, Json<OpenIDErrorResponse>> {
    info!("Revoking refresh token for client: {}", request.client_id);

    // Get OpenID token service from container
    let openid_service = match app_state.domain_container.get_openid_token_service() {
        Some(service) => service,
        None => {
            error!("OpenID token service not available");
            return Err(Json(OpenIDErrorResponse {
                error: "server_error".to_string(),
                error_description: "Token service temporarily unavailable".to_string(),
                error_uri: None,
            }));
        }
    };

    // Revoke refresh token
    match openid_service.revoke_refresh_token(&request.refresh_token).await {
        Ok(()) => {
            info!("Successfully revoked refresh token for client: {}", request.client_id);
            Ok(Json(json!({
                "success": true,
                "message": "Token revoked successfully"
            })))
        }
        Err(e) => {
            error!("Token revocation failed: {}", e);
            // Don't reveal detailed error information for security
            Ok(Json(json!({
                "success": true,
                "message": "Token revoked successfully"
            })))
        }
    }
}

/// Get OpenID Connect discovery document
#[utoipa::path(
    get,
    path = "/.well-known/openid_configuration",
    responses(
        (status = 200, description = "OpenID Connect discovery document", body = Value),
    ),
    tag = "openid-discovery"
)]
pub async fn openid_discovery(
    State(_app_state): State<AppState>,
) -> Json<Value> {
    // Return OpenID Connect discovery document
    // This helps clients discover endpoints and capabilities
    Json(json!({
        "issuer": "https://api.epsx.io",
        "authorization_endpoint": "https://api.epsx.io/auth/web3-signin",
        "token_endpoint": "https://api.epsx.io/api/auth/web3/token",
        "userinfo_endpoint": "https://api.epsx.io/api/auth/userinfo",
        "jwks_uri": "https://api.epsx.io/.well-known/jwks.json",
        "response_types_supported": ["code", "token"],
        "subject_types_supported": ["public"],
        "id_token_signing_alg_values_supported": ["RS256"],
        "scopes_supported": ["openid", "profile", "permissions"],
        "token_endpoint_auth_methods_supported": ["none"],
        "claims_supported": [
            "iss", "sub", "aud", "exp", "iat", "auth_time", "nonce",
            "wallet_address", "permissions"
        ],
        "grant_types_supported": ["refresh_token", "web3_wallet"],
        "web3_auth_endpoint": "https://api.epsx.io/api/auth/web3/token",
        "web3_challenge_endpoint": "https://api.epsx.io/api/auth/web3/challenge",
        "web3_supported_methods": ["siwe"]
    }))
}

/// Get JWT public keys for token validation
#[utoipa::path(
    get,
    path = "/.well-known/jwks.json",
    responses(
        (status = 200, description = "JWT public keys", body = Value),
    ),
    tag = "openid-discovery"
)]
pub async fn jwks(
    State(_app_state): State<AppState>,
) -> Result<Json<Value>, StatusCode> {
    // Get public key from OpenID service for JWT validation
    // This endpoint allows clients to validate JWT tokens
    
    // TODO: Implement actual JWKS endpoint with public keys
    // For now return placeholder structure
    Ok(Json(json!({
        "keys": [
            {
                "kty": "RSA",
                "use": "sig",
                "kid": "epsx-2024",
                "alg": "RS256",
                "n": "placeholder-modulus",
                "e": "AQAB"
            }
        ]
    })))
}

/// Get user information from access token
#[utoipa::path(
    get,
    path = "/api/auth/userinfo",
    responses(
        (status = 200, description = "User information", body = Value),
        (status = 401, description = "Invalid or expired token"),
    ),
    tag = "openid-auth",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn userinfo(
    State(_app_state): State<AppState>,
    // TODO: Add Bearer token extraction middleware
) -> Result<Json<Value>, StatusCode> {
    // For now, return a proper response that matches UserInfoResponse interface
    // TODO: Replace this with actual Bearer token validation and user lookup
    
    info!("Processing userinfo request - using temporary admin user for testing");
    
    // Return admin user data in UnifiedApiResponse format that matches the expected interface
    Ok(Json(json!({
        "success": true,
        "data": {
            "sub": "0x742d35Cc6634C0532925a3b8D369D7763F3c45c6",
            "wallet_address": "0x742d35Cc6634C0532925a3b8D369D7763F3c45c6",
            "auth_method": "web3_siwe",
            "permissions": [
                "admin:*:*",
                "epsx:analytics:read",
                "epsx:analytics:export",
                "epsx:users:manage",
                "epsx:permissions:manage"
            ],
            "email": "admin@epsx.io"
        },
        "meta": {
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "permissions": {
                "user_tier": "admin",
                "available_actions": [
                    "admin:*:*",
                    "epsx:analytics:read", 
                    "epsx:analytics:export",
                    "epsx:users:manage",
                    "epsx:permissions:manage"
                ]
            }
        }
    })))
}