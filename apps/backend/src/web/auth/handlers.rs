// Web3 Authentication Handlers
// Complete Web3 authentication handlers integrating SIWE with plan permissions

use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
// use std::sync::Arc; // Removed - unused import
use tracing::{info, error, warn};

use utoipa::{ToSchema, IntoParams};

use crate::{
    auth::auth_service::{
        Web3VerificationRequest, Web3AuthError,
    },
    web::auth::AppState,
};

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct ChallengeRequest {
    /// Ethereum wallet address
    #[schema(example = "0x1234567890123456789012345678901234567890")]
    pub wallet_address: String,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct SignatureVerificationRequest {
    /// SIWE message that was signed
    pub message: String,
    /// Cryptographic signature from wallet
    pub signature: String,
    /// Ethereum wallet address
    #[schema(example = "0x1234567890123456789012345678901234567890")]
    pub wallet_address: String,
    /// Challenge nonce
    #[schema(example = "abc123def456")]
    pub nonce: String,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct LogoutRequest {
    /// Ethereum wallet address to logout
    #[schema(example = "0x1234567890123456789012345678901234567890")]
    pub wallet_address: String,
}

#[derive(Debug, Deserialize, Serialize, ToSchema, IntoParams)]
pub struct PermissionCheckQuery {
    /// Permission to check (format: platform:resource:action)
    #[schema(example = "epsx:analytics:read")]
    pub permission: String,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct GrantPermissionRequest {
    /// Target wallet address to grant permission to
    #[schema(example = "0x1234567890123456789012345678901234567890")]
    pub wallet_address: String,
    /// Permission to grant (format: platform:resource:action[:timestamp])
    #[schema(example = "epsx:analytics:read")]
    pub permission: String,
    /// Optional expiration timestamp for time-limited permissions
    #[schema(example = "2024-12-31T23:59:59Z")]
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct RevokePermissionRequest {
    /// Target wallet address to revoke permission from
    #[schema(example = "0x1234567890123456789012345678901234567890")]
    pub wallet_address: String,
    /// Permission to revoke (format: platform:resource:action)
    #[schema(example = "epsx:analytics:read")]
    pub permission: String,
}

/// Generate SIWE challenge for Web3 authentication
#[utoipa::path(
    post,
    path = "/api/auth/web3/challenge",
    request_body = ChallengeRequest,
    responses(
        (status = 200, description = "Challenge generated successfully", body = Value),
        (status = 400, description = "Invalid wallet address", body = Value),
        (status = 500, description = "Internal server error")
    ),
    tag = "auth"
)]
pub async fn generate_challenge_handler(
    State(app_state): State<AppState>,
    Json(request): Json<ChallengeRequest>,
) -> Result<Json<Value>, StatusCode> {
    info!("Generating Web3 challenge for wallet: {}", request.wallet_address);

    // Get Web3 auth service from domain container
    let web3_auth_service = match app_state.domain_container.get_auth_service() {
        Some(service) => service,
        None => {
            error!("Web3 auth service not available");
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    match web3_auth_service.generate_challenge(&request.wallet_address).await {
        Ok(challenge) => {
            info!("Generated challenge for wallet: {}", request.wallet_address);
            Ok(Json(json!({
                "success": true,
                "nonce": challenge.nonce,
                "message": challenge.message,
                "expires_at": challenge.expires_at.timestamp(),
                "wallet_address": challenge.wallet_address
            })))
        }
        Err(Web3AuthError::InvalidWalletAddress(address)) => {
            warn!("Invalid wallet address format: {}", address);
            Ok(Json(json!({
                "success": false,
                "error": "invalid_wallet_address",
                "message": format!("Invalid wallet address format: {}", address)
            })))
        }
        Err(e) => {
            error!("Failed to generate challenge: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Verify SIWE signature and authenticate wallet
#[utoipa::path(
    post,
    path = "/api/auth/web3/verify",
    request_body = SignatureVerificationRequest,
    responses(
        (status = 200, description = "Signature verified successfully", body = Value),
        (status = 400, description = "Invalid signature or expired challenge", body = Value),
        (status = 401, description = "Authentication failed", body = Value),
        (status = 500, description = "Internal server error")
    ),
    tag = "auth"
)]
pub async fn verify_signature_handler(
    State(app_state): State<AppState>,
    Json(request): Json<SignatureVerificationRequest>,
) -> Result<Json<Value>, StatusCode> {
    info!("Verifying Web3 signature for wallet: {}", request.wallet_address);

    // Get services from domain container
    let web3_auth_service = match app_state.domain_container.get_auth_service() {
        Some(service) => service,
        None => {
            error!("Web3 auth service not available");
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    let web3_permission_service = match app_state.domain_container.get_web3_permission_adapter() {
        Some(service) => service,
        None => {
            error!("Web3 permission service not available");
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    // Web3 plan bridge functionality integrated into permission service

    // Verify signature using Web3AuthService
    let verification_request = Web3VerificationRequest {
        message: request.message,
        signature: request.signature,
        wallet_address: request.wallet_address.clone(),
        nonce: request.nonce,
    };

    match web3_auth_service.verify_and_authenticate(verification_request).await {
        Ok(auth_result) => {
            // Signature verification successful - auth_result contains validated data
            info!("Signature verification successful for wallet: {}", auth_result.wallet_address);

            // Authentication successful - permissions handled by Web3PermissionService
            info!("Authentication successful for wallet: {}", auth_result.wallet_address);

            // Also process legacy automatic permissions for backward compatibility
            let permissions_granted = match web3_permission_service
                .process_automatic_permissions(&auth_result.wallet_address)
                .await
            {
                Ok(permissions) => permissions,
                Err(e) => {
                    error!("Failed to process automatic permissions: {}", e);
                    Vec::new() // Continue without auto-permissions
                }
            };

            // Get user's current permissions (both legacy and plan-based)
            let user_permissions = match web3_permission_service
                .get_user_permissions(&auth_result.wallet_address)
                .await
            {
                Ok(permissions) => permissions,
                Err(e) => {
                    warn!("Failed to get user permissions: {}", e);
                    Vec::new()
                }
            };

            info!(
                "Successful Web3 authentication for wallet: {}, granted {} permissions",
                auth_result.wallet_address,
                permissions_granted.len()
            );

            Ok(Json(json!({
                "success": true,
                "authenticated": true,
                "is_new_user": auth_result.is_new_user,
                "wallet_address": auth_result.wallet_address,
                "permissions": user_permissions,
                "permissions_granted": permissions_granted,
                "access_token": auth_result.bearer_token.clone().unwrap_or(auth_result.access_token),
                "refresh_token": auth_result.refresh_token
            })))
        }
        Err(Web3AuthError::ExpiredNonce(msg)) => {
            warn!("Web3 challenge error for wallet: {}: {}", request.wallet_address, msg);
            Ok(Json(json!({
                "success": false,
                "authenticated": false,
                "error": "challenge_error",
                "message": msg
            })))
        }
        Err(Web3AuthError::InvalidSignature(_)) => {
            warn!("Invalid signature for wallet: {}", request.wallet_address);
            Ok(Json(json!({
                "success": false,
                "authenticated": false,
                "error": "invalid_signature",
                "message": "Invalid signature provided"
            })))
        }
        Err(Web3AuthError::ChallengeAlreadyUsed(_)) => {
            warn!("Challenge already used for wallet: {}", request.wallet_address);
            Ok(Json(json!({
                "success": false,
                "authenticated": false,
                "error": "challenge_used",
                "message": "Challenge has already been used. Please request a new challenge."
            })))
        }
        Err(e) => {
            error!("Authentication error: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Logout and invalidate Web3 session
#[utoipa::path(
    delete,
    path = "/api/auth/web3/logout",
    request_body = LogoutRequest,
    responses(
        (status = 200, description = "Logout successful", body = Value),
        (status = 500, description = "Internal server error")
    ),
    tag = "auth"
)]
pub async fn logout_handler(
    State(_app_state): State<AppState>,
    Json(request): Json<LogoutRequest>,
) -> Result<Json<Value>, StatusCode> {
    info!("Web3 logout for wallet: {}", request.wallet_address);

    // For Web3 auth, logout is primarily client-side
    // We don't need to invalidate server-side sessions like traditional auth
    // Just confirm the logout action

    Ok(Json(json!({
        "success": true,
        "message": "Logged out successfully",
        "wallet_address": request.wallet_address
    })))
}

/// Token refresh request body
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct TokenRefreshRequest {
    /// Refresh token
    pub refresh_token: String,
}

/// Refresh access token using refresh token
#[utoipa::path(
    post,
    path = "/api/auth/session/refresh",
    request_body = TokenRefreshRequest,
    responses(
        (status = 200, description = "Token refreshed successfully", body = Value),
        (status = 401, description = "Invalid refresh token", body = Value),
        (status = 500, description = "Internal server error")
    ),
    tag = "auth"
)]
pub async fn refresh_token_handler(
    State(app_state): State<AppState>,
    Json(request): Json<TokenRefreshRequest>,
) -> Result<Json<Value>, StatusCode> {
    info!("Processing token refresh request");

    let web3_auth_service = match app_state.domain_container.get_auth_service() {
        Some(service) => service,
        None => {
            error!("Auth service not available");
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    match web3_auth_service.refresh_tokens(&request.refresh_token).await {
        Ok(tokens) => {
            Ok(Json(json!({
                "success": true,
                "authenticated": true,
                "access_token": tokens.access_token,
                "refresh_token": tokens.refresh_token,
                "expires_in": tokens.expires_in
            })))
        },
        Err(e) => {
            tracing::warn!("Token refresh failed: {}", e);
            Err(StatusCode::UNAUTHORIZED)
        }
    }
}

/// Get current Web3 session status
#[utoipa::path(
    get,
    path = "/api/auth/web3/session",
    responses(
        (status = 200, description = "Session status retrieved", body = Value),
        (status = 401, description = "Not authenticated", body = Value),
        (status = 500, description = "Internal server error")
    ),
    tag = "auth"
)]
pub async fn get_session_handler(
    State(app_state): State<AppState>,
    request: axum::extract::Request,
) -> Result<Json<Value>, StatusCode> {
    // Try to get wallet address from middleware context first
    use crate::web::middleware::auth_middleware::get_web3_context;
    
    if let Some(auth_context) = get_web3_context(&request) {
        // Middleware already validated - use context directly
        let wallet_address = &auth_context.wallet_address;
        info!("Session check via middleware context for wallet: {}", wallet_address);
        
        let web3_permission_service = match app_state.domain_container.get_web3_permission_adapter() {
            Some(service) => service,
            None => {
                error!("Web3 permission service not available");
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        };

        let user_permissions = match web3_permission_service
            .get_user_permissions(wallet_address)
            .await
        {
            Ok(permissions) => permissions,
            Err(e) => {
                warn!("Failed to get user permissions: {}", e);
                Vec::new()
            }
        };

        return Ok(Json(json!({
            "authenticated": true,
            "wallet_address": wallet_address,
            "permissions": user_permissions,
            "session_type": "web3"
        })));
    }

    // Fallback: Validate token directly (like SSE handlers do)
    let auth_header = request.headers()
        .get("authorization")
        .and_then(|h| h.to_str().ok());
    
    let token = match auth_header {
        Some(header) if header.starts_with("Bearer ") => &header[7..],
        _ => {
            warn!("Session check: No valid Authorization header");
            return Err(StatusCode::UNAUTHORIZED);
        }
    };

    // Get token service and validate
    let token_service = match app_state.domain_container.get_token_service() {
        Some(service) => service,
        None => {
            error!("Token service not available for session check");
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    let claims = match token_service.validate_access_token(token).await {
        Ok(c) => c,
        Err(e) => {
            warn!("Session check: Token validation failed: {}", e);
            return Err(StatusCode::UNAUTHORIZED);
        }
    };

    let wallet_address = claims.wallet_address.to_lowercase();
    info!("Session check via direct token validation for wallet: {}", wallet_address);

    // Get permissions from scope claim (already in the token)
    let permissions: Vec<String> = claims.scope
        .split_whitespace()
        .filter(|s| *s != "openid" && *s != "profile")
        .map(|s| s.to_string())
        .collect();

    Ok(Json(json!({
        "authenticated": true,
        "wallet_address": wallet_address,
        "permissions": permissions,
        "session_type": "web3"
    })))
}

/// Check if wallet has specific permission
#[utoipa::path(
    post,
    path = "/api/auth/web3/permissions/check",
    params(
        PermissionCheckQuery
    ),
    responses(
        (status = 200, description = "Permission check result", body = Value),
        (status = 401, description = "Not authenticated", body = Value),
        (status = 500, description = "Internal server error")
    ),
    tag = "auth"
)]
pub async fn check_permission_handler(
    State(app_state): State<AppState>,
    Query(query): Query<PermissionCheckQuery>,
    request: axum::extract::Request,
) -> Result<Json<Value>, StatusCode> {
    // Extract wallet address from authenticated Web3 context
    use crate::web::middleware::auth_middleware::get_web3_context;

    let auth_context = get_web3_context(&request).ok_or(StatusCode::UNAUTHORIZED)?;
    let wallet_address = &auth_context.wallet_address;

    info!(
        "Checking permission '{}' for wallet: {}",
        query.permission, wallet_address
    );

    let web3_permission_service = match app_state.domain_container.get_web3_permission_adapter() {
        Some(service) => service,
        None => {
            error!("Web3 permission service not available");
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    match web3_permission_service
        .has_permission(wallet_address, &query.permission)
        .await
    {
        Ok(has_permission) => Ok(Json(json!({
            "has_permission": has_permission,
            "wallet_address": wallet_address,
            "permission": query.permission,
            "checked_at": chrono::Utc::now()
        }))),
        Err(e) => {
            error!("Failed to check permission: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Grant manual permission to wallet (admin only)
#[utoipa::path(
    post,
    path = "/api/auth/web3/permissions/grant",
    request_body = GrantPermissionRequest,
    responses(
        (status = 200, description = "Permission granted successfully", body = Value),
        (status = 400, description = "Invalid request data", body = Value),
        (status = 403, description = "Insufficient permissions", body = Value),
        (status = 500, description = "Internal server error")
    ),
    tag = "auth",
    security(
        ("bearerAuth" = [])
    )
)]
pub async fn grant_permission_handler(
    State(app_state): State<AppState>,
    Json(request): Json<GrantPermissionRequest>,
) -> Result<Json<Value>, StatusCode> {
    info!(
        "Granting permission '{}' to wallet: {} (expires: {:?})",
        request.permission, request.wallet_address, request.expires_at
    );

    let web3_permission_service = match app_state.domain_container.get_web3_permission_adapter() {
        Some(service) => service,
        None => {
            error!("Web3 permission service not available");
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    match web3_permission_service
        .grant_manual_permission(&request.wallet_address, &request.permission, None, request.expires_at)
        .await
    {
        Ok(()) => {
            info!(
                "Granted permission '{}' to wallet: {}",
                request.permission, request.wallet_address
            );
            Ok(Json(json!({
                "success": true,
                "operation": "grant_permission",
                "wallet_address": request.wallet_address,
                "permission": request.permission,
                "expires_at": request.expires_at,
                "granted_at": chrono::Utc::now()
            })))
        }
        Err(e) => {
            error!("Failed to grant permission: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Revoke permission from wallet (admin only)
#[utoipa::path(
    delete,
    path = "/api/auth/web3/permissions/revoke",
    request_body = RevokePermissionRequest,
    responses(
        (status = 200, description = "Permission revoked successfully", body = Value),
        (status = 400, description = "Invalid request data", body = Value),
        (status = 403, description = "Insufficient permissions", body = Value),
        (status = 500, description = "Internal server error")
    ),
    tag = "auth",
    security(
        ("bearerAuth" = [])
    )
)]
pub async fn revoke_permission_handler(
    State(app_state): State<AppState>,
    Json(request): Json<RevokePermissionRequest>,
) -> Result<Json<Value>, StatusCode> {
    info!(
        "Revoking permission '{}' from wallet: {}",
        request.permission, request.wallet_address
    );

    let web3_permission_service = match app_state.domain_container.get_web3_permission_adapter() {
        Some(service) => service,
        None => {
            error!("Web3 permission service not available");
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    match web3_permission_service
        .revoke_permission(&request.wallet_address, &request.permission)
        .await
    {
        Ok(()) => {
            info!(
                "Successfully revoked permission '{}' from wallet: {}",
                request.permission, request.wallet_address
            );
            Ok(Json(json!({
                "success": true,
                "wallet_address": request.wallet_address,
                "permission": request.permission,
                "revoked_at": chrono::Utc::now()
            })))
        }
        Err(e) => {
            error!("Failed to revoke permission: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Get user permissions (both legacy and plan-based)
#[utoipa::path(
    get,
    path = "/api/auth/web3/plans/permissions/{wallet_address}",
    params(
        ("wallet_address" = String, Path, description = "Wallet address to get permissions for")
    ),
    responses(
        (status = 200, description = "User permissions retrieved", body = Vec<String>),
        (status = 404, description = "User not found", body = Value),
        (status = 500, description = "Internal server error")
    ),
    tag = "auth"
)]
pub async fn get_user_permissions_handler(
    State(app_state): State<AppState>,
    axum::extract::Path(wallet_address): axum::extract::Path<String>,
) -> Result<Json<Vec<String>>, StatusCode> {
    // Validate wallet address format briefly (basic check)
    if !wallet_address.starts_with("0x") || wallet_address.len() != 42 {
        warn!("Invalid wallet address format for permission check: {}", wallet_address);
        return Err(StatusCode::BAD_REQUEST);
    }

    info!("Getting permissions for wallet: {}", wallet_address);

    let web3_permission_service = match app_state.domain_container.get_web3_permission_adapter() {
        Some(service) => service,
        None => {
            error!("Web3 permission service not available");
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    match web3_permission_service.get_user_permissions(&wallet_address).await {
        Ok(permissions) => Ok(Json(permissions)),
        Err(e) => {
            error!("Failed to get user permissions: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}