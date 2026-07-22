// Web3 Authentication Handlers
// Complete Web3 authentication handlers integrating SIWE with plan permissions

use axum::{
    extract::{rejection::JsonRejection, Query, State},
    http::{header::CACHE_CONTROL, HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
// use std::sync::Arc; // Removed - unused import
use tracing::{error, info, warn};

use utoipa::{IntoParams, ToSchema};

use crate::{
    auth::auth_service::{Web3AuthError, Web3VerificationRequest},
    auth::key_manager::JWKS,
    infrastructure::services::audit_service::{AuditCtx, AuditEntry},
    web::auth::AppState,
};

const JWKS_CACHE_CONTROL: &str = "public, max-age=300, must-revalidate";
const AUTH_SESSION_CACHE_CONTROL: &str = "no-store";
const REFRESH_OUTCOME_HEADER: &str = "x-epsx-refresh-outcome";
const REFRESH_OUTCOME_ROTATED: &str = "rotated";
const REFRESH_OUTCOME_NOT_ROTATED: &str = "not_rotated";
const REFRESH_OUTCOME_REJECTED: &str = "rejected";
const REFRESH_OUTCOME_UNKNOWN: &str = "outcome_unknown";

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
    /// BFF audience receiving the session ("epsx-frontend" or "epsx-admin")
    #[serde(default)]
    pub client_id: Option<String>,
}

#[derive(Deserialize, Serialize, ToSchema)]
pub struct LogoutRequest {
    /// Ethereum wallet address to logout. Retained for backward compatibility and audit context.
    #[schema(example = "0x1234567890123456789012345678901234567890")]
    #[serde(default)]
    pub wallet_address: Option<String>,
    /// Opaque refresh token to revoke. Server-side BFFs may alternatively send a canonical cookie.
    #[serde(default)]
    pub refresh_token: Option<String>,
}

fn canonical_refresh_token_from_cookies(headers: &HeaderMap) -> Option<String> {
    let cookies = headers.get("cookie")?.to_str().ok()?;

    cookies.split(';').find_map(|cookie| {
        let (name, value) = cookie.trim().split_once('=')?;
        if !matches!(
            name.trim(),
            "epsx.refresh_token" | "__Host-epsx.refresh_token"
        ) {
            return None;
        }

        let value = value.trim();
        (!value.is_empty()).then(|| value.to_string())
    })
}

fn logout_refresh_token(request: Option<&LogoutRequest>, headers: &HeaderMap) -> Option<String> {
    request
        .and_then(|request| request.refresh_token.as_deref())
        .map(str::trim)
        .filter(|token| !token.is_empty())
        .map(str::to_string)
        .or_else(|| canonical_refresh_token_from_cookies(headers))
}

fn remaining_access_token_seconds(
    token_expires_at: Option<DateTime<Utc>>,
    now: DateTime<Utc>,
) -> Option<i64> {
    token_expires_at.map(|expires_at| (expires_at - now).num_seconds().max(0))
}

fn logout_success_response(wallet_address: Option<&str>) -> Value {
    json!({
        "success": true,
        "message": "Logged out successfully",
        "wallet_address": wallet_address
    })
}

fn jwks_response_headers() -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(CACHE_CONTROL, HeaderValue::from_static(JWKS_CACHE_CONTROL));
    headers
}

/// Publish the configured OpenID signing public keys. Private key material never enters the DTO.
pub async fn jwks_handler(
    State(app_state): State<AppState>,
) -> Result<(HeaderMap, Json<JWKS>), StatusCode> {
    let token_service = app_state
        .domain_container
        .get_token_service()
        .ok_or_else(|| {
            error!("Token service not available for JWKS publication");
            StatusCode::SERVICE_UNAVAILABLE
        })?;

    let jwks = token_service
        .get_key_manager()
        .generate_jwks()
        .map_err(|error| {
            error!("Failed to generate JWKS: {}", error);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok((jwks_response_headers(), Json(jwks)))
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
    info!(
        "Generating Web3 challenge for wallet: {}",
        request.wallet_address
    );

    // Get Web3 auth service from domain container
    let web3_auth_service = match app_state.domain_container.get_auth_service() {
        Some(service) => service,
        None => {
            error!("Web3 auth service not available");
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    match web3_auth_service
        .generate_challenge(&request.wallet_address)
        .await
    {
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
    headers: HeaderMap,
    Json(request): Json<SignatureVerificationRequest>,
) -> Result<Json<Value>, StatusCode> {
    info!(
        "Verifying Web3 signature for wallet: {}",
        request.wallet_address
    );

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

    // Verify signature using Web3AuthService and bind the access token to this BFF.
    let client_id = request
        .client_id
        .as_deref()
        .unwrap_or("epsx-frontend")
        .to_string();
    let verification_request = Web3VerificationRequest {
        message: request.message,
        signature: request.signature,
        wallet_address: request.wallet_address.clone(),
        nonce: request.nonce,
    };

    match web3_auth_service
        .verify_and_authenticate_for_client(verification_request, &client_id)
        .await
    {
        Ok(auth_result) => {
            // Signature verification successful - auth_result contains validated data
            info!(
                "Signature verification successful for wallet: {}",
                auth_result.wallet_address
            );

            // Authentication successful - permissions handled by Web3PermissionService
            info!(
                "Authentication successful for wallet: {}",
                auth_result.wallet_address
            );

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

            // Log successful login to audit trail
            let ctx = AuditCtx::from_wallet(&auth_result.wallet_address, &headers);
            app_state.audit.log(
                ctx,
                AuditEntry::new("session", "login", "auth")
                    .id(&auth_result.wallet_address)
                    .after(serde_json::json!({ "wallet": auth_result.wallet_address })),
            );

            let expires_in =
                remaining_access_token_seconds(auth_result.token_expires_at, Utc::now());

            Ok(Json(json!({
                "success": true,
                "authenticated": true,
                "is_new_user": auth_result.is_new_user,
                "wallet_address": auth_result.wallet_address,
                "permissions": user_permissions,
                "permissions_granted": permissions_granted,
                "access_token": auth_result.bearer_token.clone().unwrap_or(auth_result.access_token),
                "refresh_token": auth_result.refresh_token,
                "expires_in": expires_in,
                "refresh_expires_in": auth_result.refresh_expires_in
            })))
        }
        Err(Web3AuthError::ExpiredNonce(msg)) => {
            warn!(
                "Web3 challenge error for wallet: {}: {}",
                request.wallet_address, msg
            );
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
            warn!(
                "Challenge already used for wallet: {}",
                request.wallet_address
            );
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
        (status = 503, description = "Token service unavailable"),
        (status = 500, description = "Internal server error")
    ),
    tag = "auth"
)]
pub async fn logout_handler(
    State(app_state): State<AppState>,
    headers: HeaderMap,
    request: Option<Json<LogoutRequest>>,
) -> Result<Json<Value>, StatusCode> {
    let request = request.as_ref().map(|Json(request)| request);
    let wallet_address = request
        .and_then(|request| request.wallet_address.as_deref())
        .map(str::trim)
        .filter(|wallet| !wallet.is_empty());
    let refresh_token = logout_refresh_token(request, &headers);

    info!(
        "Web3 logout requested for wallet: {}",
        wallet_address.unwrap_or("unknown")
    );

    if let Some(refresh_token) = refresh_token.as_deref() {
        let token_service = app_state
            .domain_container
            .get_token_service()
            .ok_or_else(|| {
                error!("Token service unavailable while processing logout");
                StatusCode::SERVICE_UNAVAILABLE
            })?;

        // UPDATE affects zero rows for an unknown or already-revoked token. That is deliberately
        // indistinguishable from a newly revoked token so this endpoint cannot become a token oracle.
        token_service
            .revoke_refresh_token(refresh_token)
            .await
            .map_err(|error| {
                error!("Failed to process refresh-token revocation: {}", error);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;
    }

    let audit_context = wallet_address
        .map(|wallet| AuditCtx::from_wallet(wallet, &headers))
        .unwrap_or_else(|| AuditCtx::from_headers(&headers));
    app_state.audit.log(
        audit_context,
        AuditEntry::new("session", "logout", "auth").after(json!({
            "refresh_token_supplied": refresh_token.is_some()
        })),
    );

    Ok(Json(logout_success_response(wallet_address)))
}

/// Token refresh request body
#[derive(Deserialize, Serialize, ToSchema)]
pub struct TokenRefreshRequest {
    /// Refresh token
    pub refresh_token: Option<String>,
    /// Client identifier ("epsx-frontend" or "epsx-admin")
    pub client_id: String,
}

/// Refresh access token using refresh token
#[utoipa::path(
    post,
    path = "/api/auth/session/refresh",
    request_body = TokenRefreshRequest,
    responses(
        (status = 200, description = "Token refreshed successfully", body = Value),
        (status = 400, description = "Malformed request or unsupported client"),
        (status = 401, description = "Invalid refresh token", body = Value),
        (status = 500, description = "Internal server error"),
        (status = 503, description = "Refresh dependency unavailable")
    ),
    tag = "auth"
)]
pub async fn refresh_token_handler(
    State(app_state): State<AppState>,
    headers: HeaderMap,
    request: Result<Json<TokenRefreshRequest>, JsonRejection>,
) -> Response {
    info!("Processing token refresh request");

    let Json(request) = match request {
        Ok(request) => request,
        Err(_) => {
            return refresh_status_response(StatusCode::BAD_REQUEST, REFRESH_OUTCOME_NOT_ROTATED)
        }
    };

    // 1. Try to get token from request body
    let mut refresh_token = request.refresh_token;

    // 2. If not in body, try to get from cookies
    if refresh_token.is_none() {
        if let Some(cookie_header) = headers.get("cookie").and_then(|h| h.to_str().ok()) {
            // Parse cookies manually to avoid extra dependencies for now
            // Looking for epsx.refresh_token or __Host-epsx.refresh_token
            for cookie in cookie_header.split(';') {
                let parts: Vec<&str> = cookie.trim().split('=').collect();
                if parts.len() >= 2 {
                    let key = parts[0].trim();
                    let value = parts[1].trim();
                    if key == "epsx.refresh_token" || key == "__Host-epsx.refresh_token" {
                        refresh_token = Some(value.to_string());
                        break;
                    }
                }
            }
        }
    }

    let token = match refresh_token {
        Some(t) => t,
        None => {
            warn!("No refresh token provided in body or cookies");
            return refresh_status_response(StatusCode::UNAUTHORIZED, REFRESH_OUTCOME_REJECTED);
        }
    };

    let web3_auth_service = match app_state.domain_container.get_auth_service() {
        Some(service) => service,
        None => {
            error!("Auth service not available");
            return refresh_status_response(
                StatusCode::SERVICE_UNAVAILABLE,
                REFRESH_OUTCOME_NOT_ROTATED,
            );
        }
    };

    if !matches!(request.client_id.as_str(), "epsx-frontend" | "epsx-admin") {
        warn!("Unsupported client supplied to token refresh");
        return refresh_status_response(StatusCode::BAD_REQUEST, REFRESH_OUTCOME_NOT_ROTATED);
    }

    match web3_auth_service
        .refresh_tokens(&token, &request.client_id)
        .await
    {
        Ok((tokens, wallet_address, permissions)) => refresh_json_response(json!({
            "success": true,
            "authenticated": true,
            "access_token": tokens.access_token,
            "refresh_token": tokens.refresh_token,
            "expires_in": tokens.expires_in,
            "refresh_expires_in": tokens.refresh_expires_in,
            "user": {
                "wallet": wallet_address,
                "permissions": permissions
            }
        })),
        Err(Web3AuthError::InvalidClient(_)) => {
            refresh_status_response(StatusCode::BAD_REQUEST, REFRESH_OUTCOME_NOT_ROTATED)
        }
        Err(Web3AuthError::InvalidRefreshToken) => {
            tracing::warn!("Token refresh credential was rejected");
            refresh_status_response(StatusCode::UNAUTHORIZED, REFRESH_OUTCOME_REJECTED)
        }
        Err(Web3AuthError::DatabaseError(error) | Web3AuthError::BlockchainError(error)) => {
            tracing::error!("Token refresh dependency failed: {}", error);
            refresh_status_response(StatusCode::SERVICE_UNAVAILABLE, REFRESH_OUTCOME_UNKNOWN)
        }
        Err(Web3AuthError::TokenGenerationFailed(error)) => {
            tracing::error!("Token refresh signing failed before rotation: {}", error);
            refresh_status_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                REFRESH_OUTCOME_NOT_ROTATED,
            )
        }
        Err(error) => {
            tracing::error!("Token refresh failed internally: {}", error);
            refresh_status_response(StatusCode::INTERNAL_SERVER_ERROR, REFRESH_OUTCOME_UNKNOWN)
        }
    }
}

fn refresh_json_response(body: Value) -> Response {
    let mut response = Json(body).into_response();
    mark_refresh_response(&mut response, REFRESH_OUTCOME_ROTATED);
    response
}

fn refresh_status_response(status: StatusCode, outcome: &'static str) -> Response {
    let mut response = status.into_response();
    mark_refresh_response(&mut response, outcome);
    response
}

fn mark_refresh_response(response: &mut Response, outcome: &'static str) {
    response.headers_mut().insert(
        CACHE_CONTROL,
        HeaderValue::from_static(AUTH_SESSION_CACHE_CONTROL),
    );
    response
        .headers_mut()
        .insert(REFRESH_OUTCOME_HEADER, HeaderValue::from_static(outcome));
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
        info!(
            "Session check via middleware context for wallet: {}",
            wallet_address
        );

        let web3_permission_service = match app_state.domain_container.get_web3_permission_adapter()
        {
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
    let auth_header = request
        .headers()
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
    info!(
        "Session check via direct token validation for wallet: {}",
        wallet_address
    );

    // Get permissions from scope claim (already in the token)
    let permissions: Vec<String> = claims
        .scope
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
    axum::Extension(user_ctx): axum::Extension<
        crate::web::middleware::bearer_middleware::OpenIDUserContext,
    >,
    headers: HeaderMap,
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
        .grant_manual_permission(
            &request.wallet_address,
            &request.permission,
            None,
            request.expires_at,
        )
        .await
    {
        Ok(()) => {
            info!(
                "Granted permission '{}' to wallet: {}",
                request.permission, request.wallet_address
            );

            // Log permission grant to audit trail
            let ctx = AuditCtx::from_wallet(&user_ctx.wallet_address, &headers);
            app_state.audit.log(
                ctx,
                AuditEntry::new("permission", "grant", "permission")
                    .id(&request.wallet_address)
                    .after(serde_json::json!({
                        "wallet": request.wallet_address,
                        "permission": request.permission,
                        "expires_at": request.expires_at
                    })),
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
    axum::Extension(user_ctx): axum::Extension<
        crate::web::middleware::bearer_middleware::OpenIDUserContext,
    >,
    headers: HeaderMap,
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

            // Log permission revocation to audit trail
            let ctx = AuditCtx::from_wallet(&user_ctx.wallet_address, &headers);
            app_state.audit.log(
                ctx,
                AuditEntry::new("permission", "revoke", "permission")
                    .id(&request.wallet_address)
                    .after(serde_json::json!({
                        "wallet": request.wallet_address,
                        "permission": request.permission
                    })),
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
        warn!(
            "Invalid wallet address format for permission check: {}",
            wallet_address
        );
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

    match web3_permission_service
        .get_user_permissions(&wallet_address)
        .await
    {
        Ok(permissions) => Ok(Json(permissions)),
        Err(e) => {
            error!("Failed to get user permissions: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::header::COOKIE;
    use chrono::Duration;

    #[test]
    fn jwks_cache_control_is_public_and_bounded() {
        let headers = jwks_response_headers();
        assert_eq!(
            headers
                .get(CACHE_CONTROL)
                .and_then(|value| value.to_str().ok()),
            Some("public, max-age=300, must-revalidate")
        );
        assert!(!JWKS_CACHE_CONTROL.contains("private"));
        assert!(!JWKS_CACHE_CONTROL.contains("no-store"));
    }

    #[test]
    fn logout_request_remains_compatible_with_wallet_only_body() {
        let request: LogoutRequest = serde_json::from_value(json!({
            "wallet_address": "0x1234567890123456789012345678901234567890"
        }))
        .unwrap();

        assert_eq!(
            request.wallet_address.as_deref(),
            Some("0x1234567890123456789012345678901234567890")
        );
        assert!(request.refresh_token.is_none());
    }

    #[test]
    fn verify_request_defaults_client_and_accepts_admin_audience() {
        let legacy: SignatureVerificationRequest = serde_json::from_value(json!({
            "message": "message",
            "signature": "0xsignature",
            "wallet_address": "0x1234567890123456789012345678901234567890",
            "nonce": "nonce"
        }))
        .unwrap();
        assert!(legacy.client_id.is_none());

        let admin: SignatureVerificationRequest = serde_json::from_value(json!({
            "message": "message",
            "signature": "0xsignature",
            "wallet_address": "0x1234567890123456789012345678901234567890",
            "nonce": "nonce",
            "client_id": "epsx-admin"
        }))
        .unwrap();
        assert_eq!(admin.client_id.as_deref(), Some("epsx-admin"));
    }

    #[test]
    fn refresh_request_requires_an_explicit_supported_client() {
        assert!(serde_json::from_value::<TokenRefreshRequest>(json!({
            "refresh_token": "opaque"
        }))
        .is_err());

        for client_id in ["epsx-frontend", "epsx-admin"] {
            let request: TokenRefreshRequest = serde_json::from_value(json!({
                "refresh_token": "opaque",
                "client_id": client_id
            }))
            .unwrap();
            assert_eq!(request.client_id, client_id);
        }
    }

    #[test]
    fn refresh_responses_are_never_cacheable() {
        for response in [
            refresh_json_response(json!({"success": true})),
            refresh_status_response(StatusCode::UNAUTHORIZED, REFRESH_OUTCOME_REJECTED),
            refresh_status_response(StatusCode::SERVICE_UNAVAILABLE, REFRESH_OUTCOME_UNKNOWN),
        ] {
            assert_eq!(
                response
                    .headers()
                    .get(CACHE_CONTROL)
                    .and_then(|value| value.to_str().ok()),
                Some("no-store")
            );
        }
    }

    #[test]
    fn refresh_responses_attest_the_closed_rotation_outcome() {
        let cases = [
            (
                refresh_json_response(json!({"success": true})),
                StatusCode::OK,
                REFRESH_OUTCOME_ROTATED,
            ),
            (
                refresh_status_response(StatusCode::BAD_REQUEST, REFRESH_OUTCOME_NOT_ROTATED),
                StatusCode::BAD_REQUEST,
                REFRESH_OUTCOME_NOT_ROTATED,
            ),
            (
                refresh_status_response(StatusCode::UNAUTHORIZED, REFRESH_OUTCOME_REJECTED),
                StatusCode::UNAUTHORIZED,
                REFRESH_OUTCOME_REJECTED,
            ),
            (
                refresh_status_response(StatusCode::SERVICE_UNAVAILABLE, REFRESH_OUTCOME_UNKNOWN),
                StatusCode::SERVICE_UNAVAILABLE,
                REFRESH_OUTCOME_UNKNOWN,
            ),
        ];

        for (response, status, outcome) in cases {
            assert_eq!(response.status(), status);
            assert_eq!(
                response
                    .headers()
                    .get(REFRESH_OUTCOME_HEADER)
                    .and_then(|value| value.to_str().ok()),
                Some(outcome)
            );
            assert_eq!(
                response
                    .headers()
                    .get(CACHE_CONTROL)
                    .and_then(|value| value.to_str().ok()),
                Some(AUTH_SESSION_CACHE_CONTROL)
            );
        }
    }

    #[test]
    fn logout_prefers_body_refresh_token_over_cookie() {
        let request = LogoutRequest {
            wallet_address: None,
            refresh_token: Some("body-token".to_string()),
        };
        let mut headers = HeaderMap::new();
        headers.insert(COOKIE, "epsx.refresh_token=cookie-token".parse().unwrap());

        assert_eq!(
            logout_refresh_token(Some(&request), &headers).as_deref(),
            Some("body-token")
        );
    }

    #[test]
    fn logout_accepts_each_canonical_refresh_cookie() {
        for cookie in [
            "epsx.refresh_token=plain-token",
            "__Host-epsx.refresh_token=host-token",
        ] {
            let mut headers = HeaderMap::new();
            headers.insert(COOKIE, cookie.parse().unwrap());
            assert!(logout_refresh_token(None, &headers).is_some());
        }
    }

    #[test]
    fn logout_ignores_empty_and_noncanonical_tokens() {
        let request = LogoutRequest {
            wallet_address: None,
            refresh_token: Some("   ".to_string()),
        };
        let mut headers = HeaderMap::new();
        headers.insert(
            COOKIE,
            "epsx_token=legacy; epsx.refresh_token=".parse().unwrap(),
        );

        assert!(logout_refresh_token(Some(&request), &headers).is_none());
    }

    #[test]
    fn logout_success_is_generic_and_never_contains_token_material() {
        let response = logout_success_response(Some("0x1234567890123456789012345678901234567890"));
        let serialized = response.to_string();

        assert_eq!(response["success"], true);
        assert!(!serialized.contains("refresh_token"));
        assert!(!serialized.contains("body-token"));
    }

    #[test]
    fn verify_expiry_is_derived_from_absolute_expiration() {
        let now = Utc::now();
        assert_eq!(
            remaining_access_token_seconds(Some(now + Duration::seconds(3600)), now),
            Some(3600)
        );
        assert_eq!(
            remaining_access_token_seconds(Some(now - Duration::seconds(1)), now),
            Some(0)
        );
        assert_eq!(remaining_access_token_seconds(None, now), None);
    }
}
