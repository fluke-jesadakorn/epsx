use axum::{
    extract::{Query, State},
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{error, info, warn};

// Web3 services are accessed via container
use crate::infrastructure::container::AppContainer;
use crate::core::errors::AppError;
type ApiResult<T> = Result<T, AppError>;
type ApiError = AppError;

/// Request to generate authentication challenge
#[derive(Debug, Deserialize)]
pub struct ChallengeRequest {
    pub wallet_address: String,
}

/// Response containing authentication challenge
#[derive(Debug, Serialize)]
pub struct ChallengeResponse {
    pub nonce: String,
    pub message: String,
    pub expires_at: String,
}

/// Request to verify wallet signature
#[derive(Debug, Deserialize)]
pub struct VerifyRequest {
    pub message: String,
    pub signature: String,
    pub wallet_address: String,
}

/// Response for successful verification
#[derive(Debug, Serialize)]
pub struct VerifyResponse {
    pub access_token: String,
    pub id_token: String,
    pub refresh_token: String,
    pub user_id: String,
    pub wallet_address: String,
    pub permissions: Vec<String>,
    pub expires_in: u64,
}

/// Request to link wallet to existing user
#[derive(Debug, Deserialize)]
pub struct LinkWalletRequest {
    pub wallet_address: String,
    pub signature: String,
    pub message: String,
    pub user_id: String,
}

/// Response for wallet permissions
#[derive(Debug, Serialize)]
pub struct PermissionsResponse {
    pub wallet_address: String,
    pub permissions: Vec<PermissionInfo>,
    pub automatic_grants: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct PermissionInfo {
    pub permission: String,
    pub permission_type: String,
    pub granted_at: String,
    pub expires_at: Option<String>,
    pub is_active: bool,
}

/// Web3 authentication routes
pub fn create_routes() -> Router<AppContainer> {
    Router::new()
        .route("/challenge", post(generate_challenge))
        .route("/verify", post(verify_signature))
        .route("/link-wallet", post(link_wallet))
        .route("/permissions", get(get_permissions))
        .route("/permissions/process", post(process_automatic_permissions))
        .route("/status", get(get_wallet_status))
}

/// Generate Web3 authentication challenge
/// POST /api/auth/web3/challenge
pub async fn generate_challenge(
    State(container): State<AppContainer>,
    Json(request): Json<ChallengeRequest>,
) -> ApiResult<Json<ChallengeResponse>> {
    info!("Generating Web3 challenge for wallet: {}", request.wallet_address);

    let web3_auth = container.web3_auth_service();

    let challenge = web3_auth
        .generate_challenge(&request.wallet_address)
        .await
        .map_err(|e| {
            error!("Failed to generate challenge: {}", e);
            match e.to_string().as_str() {
                s if s.contains("Invalid wallet address") => ApiError::bad_request("Invalid wallet address format"),
                _ => ApiError::internal_server_error("Failed to generate challenge"),
            }
        })?;

    let response = ChallengeResponse {
        nonce: challenge.nonce,
        message: challenge.message,
        expires_at: challenge.expires_at.to_rfc3339(),
    };

    Ok(Json(response))
}

/// Verify wallet signature and authenticate
/// POST /api/auth/web3/verify
pub async fn verify_signature(
    State(container): State<AppContainer>,
    Json(request): Json<VerifyRequest>,
) -> ApiResult<Json<VerifyResponse>> {
    info!("Verifying Web3 signature for wallet: {}", request.wallet_address);

    let web3_auth = container.web3_auth_service();

    let web3_permissions = container.web3_permission_service();

    // Verify the signature
    let auth_result = web3_auth
        .verify_signature(crate::auth::web3_auth_service::VerifyRequest {
            message: request.message,
            signature: request.signature,
            wallet_address: request.wallet_address.clone(),
        })
        .await
        .map_err(|e| {
            error!("Failed to verify signature: {}", e);
            ApiError::internal_server_error("Signature verification failed")
        })?;

    if !auth_result.is_valid {
        warn!("Invalid signature for wallet: {}", request.wallet_address);
        return Err(ApiError::unauthorized("Invalid signature"));
    }

    let user_id = auth_result.user_id
        .ok_or_else(|| ApiError::internal_server_error("Failed to get user ID"))?;

    // Get wallet permissions
    let permissions = web3_permissions
        .get_wallet_permissions(&request.wallet_address)
        .await
        .map_err(|e| {
            error!("Failed to get wallet permissions: {}", e);
            ApiError::internal_server_error("Failed to get permissions")
        })?;

    let permission_strings: Vec<String> = permissions
        .iter()
        .filter(|p| p.is_active)
        .map(|p| p.permission.clone())
        .collect();

    // Process automatic permissions
    let automatic_grants = web3_permissions
        .process_automatic_permissions(&request.wallet_address)
        .await
        .unwrap_or_default();

    // Generate OIDC tokens using existing service
    let jwt_service = container.jwt_service()
        .map_err(|e| {
            error!("Failed to get JWT service: {}", e);
            ApiError::internal_server_error("Token generation failed")
        })?;

    // Create JWT user data for Web3 authentication
    let user_data = crate::auth::jwt::UserData {
        id: user_id.to_string(),
        email: format!("{}@wallet.epsx.io", request.wallet_address), // Temporary email for wallet users
        name: Some(format!("Wallet User {}", &request.wallet_address[..8])),
        permissions: Some(permission_strings.clone()),
        audience: Some("epsx-ecosystem".to_string()),
        ttl_seconds: Some(3600), // 1 hour
        permission_version: Some(1),
        permission_last_updated: Some(chrono::Utc::now().timestamp() as u64),
        verified: Some(false), // Wallet users don't need email verification
    };

    let access_token = jwt_service
        .create(user_data)
        .map_err(|e| {
            error!("Failed to generate access token: {}", e);
            ApiError::internal_server_error("Token generation failed")
        })?;

    // For now, use the same token for id_token and refresh_token
    // TODO: Implement proper OIDC id_token and refresh_token generation
    let id_token = access_token.clone();
    let refresh_token = access_token.clone();

    info!("Successful Web3 authentication for user: {}", user_id);

    let response = VerifyResponse {
        access_token,
        id_token,
        refresh_token,
        user_id: user_id.to_string(),
        wallet_address: request.wallet_address,
        permissions: permission_strings,
        expires_in: 3600,
    };

    Ok(Json(response))
}

/// Link wallet to existing user account
/// POST /api/auth/web3/link-wallet
pub async fn link_wallet(
    State(container): State<AppContainer>,
    Json(request): Json<LinkWalletRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    info!("Linking wallet {} to user {}", request.wallet_address, request.user_id);

    let web3_auth = container.web3_auth_service();

    // Parse user ID
    let user_id = uuid::Uuid::parse_str(&request.user_id)
        .map_err(|_| ApiError::bad_request("Invalid user ID format"))?;

    // Link the wallet
    web3_auth
        .link_wallet_to_user(
            user_id,
            &request.wallet_address,
            &request.signature,
            &request.message,
        )
        .await
        .map_err(|e| {
            error!("Failed to link wallet: {}", e);
            match e.to_string().as_str() {
                s if s.contains("Invalid wallet address") => ApiError::bad_request("Invalid wallet address format"),
                s if s.contains("already linked") => ApiError::conflict("Wallet already linked"),
                _ => ApiError::internal_server_error("Failed to link wallet"),
            }
        })?;

    info!("Successfully linked wallet {} to user {}", request.wallet_address, request.user_id);

    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Wallet linked successfully",
        "user_id": request.user_id,
        "wallet_address": request.wallet_address
    })))
}

/// Get wallet permissions
/// GET /api/auth/web3/permissions?wallet_address=0x...
pub async fn get_permissions(
    State(container): State<AppContainer>,
    Query(params): Query<HashMap<String, String>>,
) -> ApiResult<Json<PermissionsResponse>> {
    let wallet_address = params
        .get("wallet_address")
        .ok_or_else(|| ApiError::bad_request("wallet_address parameter required"))?;

    info!("Getting permissions for wallet: {}", wallet_address);

    let web3_permissions = container.web3_permission_service();

    let permissions = web3_permissions
        .get_wallet_permissions(wallet_address)
        .await
        .map_err(|e| {
            error!("Failed to get wallet permissions: {}", e);
            ApiError::internal_server_error("Failed to get permissions")
        })?;

    let permission_infos: Vec<PermissionInfo> = permissions
        .into_iter()
        .map(|p| PermissionInfo {
            permission: p.permission,
            permission_type: p.permission_type,
            granted_at: p.granted_at.to_rfc3339(),
            expires_at: p.expires_at.map(|dt| dt.to_rfc3339()),
            is_active: p.is_active,
        })
        .collect();

    let response = PermissionsResponse {
        wallet_address: wallet_address.clone(),
        permissions: permission_infos,
        automatic_grants: vec![], // This will be populated by process_automatic_permissions
    };

    Ok(Json(response))
}

/// Process automatic permission granting
/// POST /api/auth/web3/permissions/process
pub async fn process_automatic_permissions(
    State(container): State<AppContainer>,
    Json(request): Json<serde_json::Value>,
) -> ApiResult<Json<PermissionsResponse>> {
    let wallet_address = request["wallet_address"]
        .as_str()
        .ok_or_else(|| ApiError::bad_request("wallet_address field required"))?;

    info!("Processing automatic permissions for wallet: {}", wallet_address);

    let web3_permissions = container.web3_permission_service();

    let automatic_grants = web3_permissions
        .process_automatic_permissions(wallet_address)
        .await
        .map_err(|e| {
            error!("Failed to process automatic permissions: {}", e);
            ApiError::internal_server_error("Failed to process permissions")
        })?;

    // Get updated permissions
    let permissions = web3_permissions
        .get_wallet_permissions(wallet_address)
        .await
        .map_err(|e| {
            error!("Failed to get updated permissions: {}", e);
            ApiError::internal_server_error("Failed to get permissions")
        })?;

    let permission_infos: Vec<PermissionInfo> = permissions
        .into_iter()
        .map(|p| PermissionInfo {
            permission: p.permission,
            permission_type: p.permission_type,
            granted_at: p.granted_at.to_rfc3339(),
            expires_at: p.expires_at.map(|dt| dt.to_rfc3339()),
            is_active: p.is_active,
        })
        .collect();

    let response = PermissionsResponse {
        wallet_address: wallet_address.to_string(),
        permissions: permission_infos,
        automatic_grants,
    };

    Ok(Json(response))
}

/// Get wallet authentication status
/// GET /api/auth/web3/status?wallet_address=0x...
pub async fn get_wallet_status(
    State(container): State<AppContainer>,
    Query(params): Query<HashMap<String, String>>,
) -> ApiResult<Json<serde_json::Value>> {
    let wallet_address = params
        .get("wallet_address")
        .ok_or_else(|| ApiError::bad_request("wallet_address parameter required"))?;

    info!("Getting status for wallet: {}", wallet_address);

    let web3_auth = container.web3_auth_service();

    let user_id = web3_auth
        .get_user_by_wallet(wallet_address)
        .await
        .map_err(|e| {
            error!("Failed to get user by wallet: {}", e);
            ApiError::internal_server_error("Failed to get user status")
        })?;

    let is_available = web3_auth
        .is_wallet_available(wallet_address)
        .await
        .map_err(|e| {
            error!("Failed to check wallet availability: {}", e);
            ApiError::internal_server_error("Failed to check availability")
        })?;

    let response = serde_json::json!({
        "wallet_address": wallet_address,
        "is_registered": user_id.is_some(),
        "is_available": is_available,
        "user_id": user_id.map(|id| id.to_string()),
        "status": if user_id.is_some() { "registered" } else { "available" }
    });

    Ok(Json(response))
}