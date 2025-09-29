// ============================================================================
// SESSION VERIFICATION HANDLERS
// Endpoints for verifying active Web3 authentication sessions
// ============================================================================

use axum::{
    extract::{State, Request},
    http::{StatusCode, HeaderMap},
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tracing::{error, info, warn};
use utoipa::ToSchema;

use crate::web::auth::routes::AppState;

/// Session verification request
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct SessionVerificationRequest {
    /// Whether to check for admin context (admin permissions required)
    #[schema(example = true)]
    pub admin_context: Option<bool>,
}

/// Session verification response
#[derive(Debug, Serialize, ToSchema)]
pub struct SessionVerificationResponse {
    /// Whether the verification was successful
    pub success: bool,
    
    /// Whether the user is authenticated
    pub authenticated: Option<bool>,
    
    /// User's wallet address (if authenticated)
    pub wallet_address: Option<String>,
    
    /// User's ID (if authenticated)
    pub user_id: Option<String>,
    
    /// User's permissions (if authenticated)
    pub permissions: Option<Vec<String>>,
    
    /// Whether user has admin permissions
    pub is_admin: Option<bool>,
    
    /// Session expiry (if authenticated)
    pub expires: Option<String>,
    
    /// Error message (if verification failed)
    pub error: Option<String>,
}

/// Extract Bearer token from Authorization header
fn extract_bearer_token(headers: &HeaderMap) -> Option<String> {
    headers
        .get("authorization")
        .and_then(|value| value.to_str().ok())
        .and_then(|auth_header| {
            if auth_header.starts_with("Bearer ") {
                Some(auth_header[7..].to_string())
            } else {
                None
            }
        })
}

/// Check if user has admin permissions
fn has_admin_permissions(permissions: &[String]) -> bool {
    permissions.iter().any(|p| {
        p == "admin:*:*" || 
        p.starts_with("admin:") ||
        p.contains(":admin:") ||
        p.contains(":manage")
    })
}

/// Verify Web3 authentication session
#[utoipa::path(
    post,
    path = "/api/v1/auth/session/verify",
    request_body = SessionVerificationRequest,
    responses(
        (status = 200, description = "Session verification result", body = SessionVerificationResponse),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "Admin permissions required"),
        (status = 500, description = "Internal server error")
    ),
    tag = "session-auth",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn verify_session_handler(
    State(_app_state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<SessionVerificationRequest>,
) -> Result<Json<SessionVerificationResponse>, StatusCode> {
    info!("Processing session verification request");
    
    let admin_context = request.admin_context.unwrap_or(false);
    
    // Extract Bearer token from Authorization header
    let token = match extract_bearer_token(&headers) {
        Some(token) => token,
        None => {
            info!("No Bearer token provided in Authorization header");
            return Ok(Json(SessionVerificationResponse {
                success: false,
                authenticated: Some(false),
                wallet_address: None,
                user_id: None,
                permissions: None,
                is_admin: None,
                expires: None,
                error: Some("No active session".to_string()),
            }));
        }
    };
    
    // TODO: Implement actual JWT token validation here
    // For now, we'll use a simplified validation that checks for valid format
    
    info!("Bearer token received: {}...", &token[..std::cmp::min(20, token.len())]);
    
    // Simplified token validation (in production, this should verify JWT signature and expiry)
    if token.len() < 20 {
        warn!("Invalid token format");
        return Ok(Json(SessionVerificationResponse {
            success: false,
            authenticated: Some(false),
            wallet_address: None,
            user_id: None,
            permissions: None,
            is_admin: None,
            expires: None,
            error: Some("Invalid token format".to_string()),
        }));
    }
    
    // TODO: Replace with actual JWT parsing and validation
    // For now, return mock admin user data for testing
    let mock_wallet_address = "0x742d35Cc6634C0532925a3b8D369D7763F3c45c6";
    let mock_permissions = vec![
        "admin:*:*".to_string(),
        "epsx:analytics:read".to_string(),
        "epsx:analytics:export".to_string(),
        "epsx:users:manage".to_string(),
        "epsx:permissions:manage".to_string(),
    ];
    
    let is_admin = has_admin_permissions(&mock_permissions);
    
    // Check admin context requirements
    if admin_context && !is_admin {
        warn!("Admin permissions required but user does not have admin access");
        return Err(StatusCode::FORBIDDEN);
    }
    
    info!("Session verification successful for wallet: {}", mock_wallet_address);
    
    Ok(Json(SessionVerificationResponse {
        success: true,
        authenticated: Some(true),
        wallet_address: Some(mock_wallet_address.to_string()),
        user_id: Some(mock_wallet_address.to_string()),
        permissions: Some(mock_permissions),
        is_admin: Some(is_admin),
        expires: Some(chrono::Utc::now().checked_add_signed(chrono::Duration::hours(1))
            .unwrap_or_else(chrono::Utc::now)
            .to_rfc3339()),
        error: None,
    }))
}

/// Get current session status (GET version for convenience)
#[utoipa::path(
    get,
    path = "/api/v1/auth/session/status",
    responses(
        (status = 200, description = "Session status", body = SessionVerificationResponse),
        (status = 401, description = "Authentication required"),
        (status = 500, description = "Internal server error")
    ),
    tag = "session-auth",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_session_status_handler(
    State(app_state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<SessionVerificationResponse>, StatusCode> {
    // Call the main verification handler with default request
    let default_request = SessionVerificationRequest {
        admin_context: Some(false),
    };
    
    verify_session_handler(State(app_state), headers, Json(default_request)).await
}