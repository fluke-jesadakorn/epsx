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

use crate::{
    // auth::web3_permission_service::Web3PermissionService, // Used via container methods
    domain::authentication::services::web3_auth_service::{
        // Web3AuthService, // Used via container methods
        Web3VerificationRequest, Web3AuthError,
    },
    web::auth::routes::AppState,
};

#[derive(Debug, Deserialize, Serialize)]
pub struct ChallengeRequest {
    pub wallet_address: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SignatureVerificationRequest {
    pub message: String,
    pub signature: String,
    pub wallet_address: String,
    pub nonce: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct LogoutRequest {
    pub wallet_address: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PermissionCheckQuery {
    pub permission: String,
}

/// POST /api/v1/auth/web3/challenge - Generate SIWE challenge
pub async fn generate_challenge_handler(
    State(app_state): State<AppState>,
    Json(request): Json<ChallengeRequest>,
) -> Result<Json<Value>, StatusCode> {
    info!("Generating Web3 challenge for wallet: {}", request.wallet_address);

    // Get Web3 auth service from domain container
    let web3_auth_service = match app_state.domain_container.get_web3_auth_service() {
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
                "expires_at": challenge.expires_at,
                "wallet_address": challenge.wallet_address
            })))
        }
        Err(Web3AuthError::InvalidWalletAddress { address }) => {
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

/// POST /api/v1/auth/web3/verify - Verify SIWE signature and authenticate
pub async fn verify_signature_handler(
    State(app_state): State<AppState>,
    Json(request): Json<SignatureVerificationRequest>,
) -> Result<Json<Value>, StatusCode> {
    info!("Verifying Web3 signature for wallet: {}", request.wallet_address);

    // Get services from domain container
    let web3_auth_service = match app_state.domain_container.get_web3_auth_service() {
        Some(service) => service,
        None => {
            error!("Web3 auth service not available");
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    let web3_permission_service = match app_state.domain_container.get_web3_permission_service() {
        Some(service) => service,
        None => {
            error!("Web3 permission service not available");
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    let web3_group_bridge = app_state.domain_container.get_web3_group_bridge();

    // Verify signature using Web3AuthService
    let verification_request = Web3VerificationRequest {
        message: request.message,
        signature: request.signature,
        wallet_address: request.wallet_address.clone(),
        nonce: request.nonce,
    };

    match web3_auth_service.verify_signature(verification_request).await {
        Ok(auth_result) => {
            if !auth_result.is_valid {
                warn!("Invalid signature for wallet: {}", request.wallet_address);
                return Ok(Json(json!({
                    "success": false,
                    "authenticated": false,
                    "error": "invalid_signature",
                    "message": "Signature verification failed"
                })));
            }

            // Authentication successful - process automatic group assignments
            let group_assignment_result = if let Some(bridge) = web3_group_bridge {
                match bridge.process_wallet_group_assignments(&auth_result.wallet_address).await {
                    Ok(result) => {
                        info!(
                            "Processed group assignments for {}: {} groups assigned",
                            auth_result.wallet_address,
                            result.groups_assigned.len()
                        );
                        Some(result)
                    }
                    Err(e) => {
                        error!("Failed to process group assignments: {}", e);
                        None
                    }
                }
            } else {
                None
            };

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

            let groups_assigned_count = group_assignment_result
                .as_ref()
                .map(|r| r.groups_assigned.len())
                .unwrap_or(0);

            info!(
                "Successful Web3 authentication for wallet: {}, granted {} permissions, assigned to {} groups",
                request.wallet_address,
                permissions_granted.len(),
                groups_assigned_count
            );

            Ok(Json(json!({
                "success": true,
                "authenticated": true,
                "user_id": auth_result.user_id,
                "wallet_address": auth_result.wallet_address,
                "permissions": user_permissions,
                "permissions_granted": permissions_granted,
                "groups_assigned": group_assignment_result.as_ref().map(|r| &r.groups_assigned).unwrap_or(&Vec::new()),
                "group_assignment_errors": group_assignment_result.as_ref().map(|r| &r.verification_errors).unwrap_or(&Vec::new()),
                "security_context": auth_result.security_context
            })))
        }
        Err(Web3AuthError::ChallengeNotFound) => {
            warn!("Challenge not found or expired for wallet: {}", request.wallet_address);
            Ok(Json(json!({
                "success": false,
                "authenticated": false,
                "error": "challenge_not_found",
                "message": "Challenge not found or expired. Please request a new challenge."
            })))
        }
        Err(Web3AuthError::InvalidSignature) => {
            warn!("Invalid signature for wallet: {}", request.wallet_address);
            Ok(Json(json!({
                "success": false,
                "authenticated": false,
                "error": "invalid_signature",
                "message": "Invalid signature provided"
            })))
        }
        Err(Web3AuthError::ChallengeAlreadyUsed) => {
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

/// DELETE /api/v1/auth/web3/logout - Logout (invalidate session)
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

/// GET /api/v1/auth/web3/session - Get current session status
pub async fn get_session_handler(
    State(app_state): State<AppState>,
    // TODO: Extract wallet address from auth middleware context
) -> Result<Json<Value>, StatusCode> {
    // This is a placeholder - in a real implementation, the wallet address
    // would be extracted from the authentication middleware context
    let wallet_address = "0x742d35Cc6634C0532925a3b8D369D7763F3c45c6"; // Demo wallet

    info!("Getting session for wallet: {}", wallet_address);

    let web3_permission_service = match app_state.domain_container.get_web3_permission_service() {
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

/// POST /api/v1/auth/web3/permissions/check - Check specific permission
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

    let web3_permission_service = match app_state.domain_container.get_web3_permission_service() {
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

    let web3_permission_service = match app_state.domain_container.get_web3_permission_service() {
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
        Ok(permission_id) => {
            info!(
                "Granted permission '{}' to wallet: {} with ID: {}",
                permission, wallet_address, permission_id
            );
            Ok(Json(json!({
                "success": true,
                "permission_id": permission_id,
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

    let web3_permission_service = match app_state.domain_container.get_web3_permission_service() {
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
        Ok(was_revoked) => {
            if was_revoked {
                info!(
                    "Successfully revoked permission '{}' from wallet: {}",
                    permission, wallet_address
                );
            } else {
                warn!(
                    "Permission '{}' was not found for wallet: {}",
                    permission, wallet_address
                );
            }

            Ok(Json(json!({
                "success": true,
                "revoked": was_revoked,
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