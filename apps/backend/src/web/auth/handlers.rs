// Web3 Authentication Handlers
// Complete Web3 authentication handlers integrating SIWE with group permissions

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

    // Web3 group bridge functionality integrated into permission service

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

            // Get user's current permissions (both legacy and group-based)
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
                "access_token": auth_result.access_token
            })))
        }
        Err(Web3AuthError::ExpiredNonce(_)) => {
            warn!("Challenge not found or expired for wallet: {}", request.wallet_address);
            Ok(Json(json!({
                "success": false,
                "authenticated": false,
                "error": "challenge_not_found",
                "message": "Challenge not found or expired. Please request a new challenge."
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
    // TODO: Extract wallet address from auth middleware context
) -> Result<Json<Value>, StatusCode> {
    // This is a placeholder - in a real implementation, the wallet address
    // would be extracted from the authentication middleware context
    let wallet_address = "0x742d35Cc6634C0532925a3b8D369D7763F3c45c6"; // Demo wallet

    info!("Getting session for wallet: {}", wallet_address);

    let web3_permission_service = match app_state.domain_container.get_web3_permission_adapter() {
        Some(service) => service,
        None => {
            error!("Web3 permission service not available");
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    // Get user's current permissions
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

    Ok(Json(json!({
        "authenticated": true,
        "wallet_address": wallet_address,
        "permissions": user_permissions,
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
    // TODO: Extract wallet address from auth middleware context
) -> Result<Json<Value>, StatusCode> {
    // This is a placeholder - in a real implementation, the wallet address
    // would be extracted from the authentication middleware context
    let wallet_address = "0x742d35Cc6634C0532925a3b8D369D7763F3c45c6"; // Demo wallet

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

/// POST /api/v1/auth/web3/permissions/grant - Grant manual permission (admin only)
pub async fn grant_permission_handler(
    State(app_state): State<AppState>,
    Json(request): Json<Value>,
) -> Result<Json<Value>, StatusCode> {
    // Extract request data
    let wallet_address = request
        .get("wallet_address")
        .and_then(|v| v.as_str())
        .ok_or(StatusCode::BAD_REQUEST)?;

    let permission = request
        .get("permission")
        .and_then(|v| v.as_str())
        .ok_or(StatusCode::BAD_REQUEST)?;

    let expires_at = request
        .get("expires_at")
        .and_then(|v| v.as_str())
        .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
        .map(|dt| dt.with_timezone(&chrono::Utc));

    info!(
        "Granting permission '{}' to wallet: {} (expires: {:?})",
        permission, wallet_address, expires_at
    );

    let web3_permission_service = match app_state.domain_container.get_web3_permission_adapter() {
        Some(service) => service,
        None => {
            error!("Web3 permission service not available");
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    match web3_permission_service
        .grant_manual_permission(wallet_address, permission, None, expires_at)
        .await
    {
        Ok(()) => {
            info!(
                "Granted permission '{}' to wallet: {}",
                permission, wallet_address
            );
            Ok(Json(json!({
                "success": true,
                "operation": "grant_permission",
                "wallet_address": wallet_address,
                "permission": permission,
                "expires_at": expires_at,
                "granted_at": chrono::Utc::now()
            })))
        }
        Err(e) => {
            error!("Failed to grant permission: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// DELETE /api/v1/auth/web3/permissions/revoke - Revoke permission (admin only)
pub async fn revoke_permission_handler(
    State(app_state): State<AppState>,
    Json(request): Json<Value>,
) -> Result<Json<Value>, StatusCode> {
    let wallet_address = request
        .get("wallet_address")
        .and_then(|v| v.as_str())
        .ok_or(StatusCode::BAD_REQUEST)?;

    let permission = request
        .get("permission")
        .and_then(|v| v.as_str())
        .ok_or(StatusCode::BAD_REQUEST)?;

    info!(
        "Revoking permission '{}' from wallet: {}",
        permission, wallet_address
    );

    let web3_permission_service = match app_state.domain_container.get_web3_permission_adapter() {
        Some(service) => service,
        None => {
            error!("Web3 permission service not available");
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    match web3_permission_service
        .revoke_permission(wallet_address, permission)
        .await
    {
        Ok(()) => {
            info!(
                "Successfully revoked permission '{}' from wallet: {}",
                permission, wallet_address
            );
            Ok(Json(json!({
                "success": true,
                "wallet_address": wallet_address,
                "permission": permission,
                "revoked_at": chrono::Utc::now()
            })))
        }
        Err(e) => {
            error!("Failed to revoke permission: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}