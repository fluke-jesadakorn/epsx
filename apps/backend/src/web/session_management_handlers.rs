use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{debug, error, info, warn};

use crate::application::auth::AuthUC;
use crate::auth::{
    SessionCleanupService, 
    CleanupStats, CleanupHealthStatus,
    refresh_token_service::{RefreshTokenService, DeviceInfo},
    session_security_service::{SessionSecurityService, DeviceFingerprint, GeoLocation, SecurityAnalysisResult, SecurityEvent},
};

/// Request to refresh an access token
#[derive(Debug, Deserialize)]
pub struct RefreshTokenRequest {
    pub refresh_token: String,
    pub device_info: Option<DeviceInfo>,
}

/// Response from token refresh
#[derive(Debug, Serialize)]
pub struct RefreshTokenResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: i64,
    pub token_type: String,
}

/// Request to revoke a token
#[derive(Debug, Deserialize)]
pub struct RevokeTokenRequest {
    pub token: String,
    pub token_type_hint: Option<String>, // "access_token" or "refresh_token"
    pub reason: Option<String>,
}

/// Response from token revocation
#[derive(Debug, Serialize)]
pub struct RevokeTokenResponse {
    pub success: bool,
    pub message: String,
}

/// Query parameters for session listing
#[derive(Debug, Deserialize)]
pub struct SessionListQuery {
    pub limit: Option<usize>,
    pub include_inactive: Option<bool>,
}

/// Session information for API response
#[derive(Debug, Serialize)]
pub struct SessionInfo {
    pub session_id: String,
    pub created_at: String,
    pub last_activity: String,
    pub ip_address: String,
    pub device_info: Option<DeviceInfo>,
    pub geo_location: Option<GeoLocation>,
    pub is_active: bool,
}

/// Request for session analysis
#[derive(Debug, Deserialize)]
pub struct AnalyzeSessionRequest {
    pub ip_address: String,
    pub device_fingerprint: DeviceFingerprint,
    pub geo_location: Option<GeoLocation>,
}

/// Query parameters for security events
#[derive(Debug, Deserialize)]
pub struct SecurityEventsQuery {
    pub limit: Option<usize>,
    pub event_type: Option<String>,
}

/// Session management handlers
pub struct SessionManagementHandlers {
    auth_uc: Arc<AuthUC>,
    refresh_token_service: Arc<RefreshTokenService>,
    cleanup_service: Arc<SessionCleanupService>,
    security_service: Arc<SessionSecurityService>,
}

impl SessionManagementHandlers {
    pub fn new(
        auth_uc: Arc<AuthUC>,
        refresh_token_service: Arc<RefreshTokenService>,
        cleanup_service: Arc<SessionCleanupService>,
        security_service: Arc<SessionSecurityService>,
    ) -> Self {
        Self {
            auth_uc,
            refresh_token_service,
            cleanup_service,
            security_service,
        }
    }

    /// Refresh an access token using refresh token
    pub async fn refresh_token(
        State(handlers): State<Arc<SessionManagementHandlers>>,
        Json(request): Json<RefreshTokenRequest>,
    ) -> Result<Json<RefreshTokenResponse>, (StatusCode, Json<serde_json::Value>)> {
        info!("Token refresh request received");

        // Validate the refresh token
        let token_record = handlers.refresh_token_service
            .validate_token(&request.refresh_token)
            .await
            .map_err(|e| {
                warn!("Invalid refresh token: {}", e);
                (StatusCode::UNAUTHORIZED, Json(serde_json::json!({
                    "error": "invalid_token",
                    "error_description": "Invalid or expired refresh token"
                })))
            })?;

        // Rotate the refresh token (generate new one)
        let device_info = request.device_info;
        let new_token_response = handlers.refresh_token_service
            .rotate_token(
                &request.refresh_token,
                device_info.clone(),
                None, // IP would come from request headers
                None, // User agent would come from request headers
            )
            .await
            .map_err(|e| {
                error!("Token rotation failed: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                    "error": "server_error",
                    "error_description": "Failed to rotate token"
                })))
            })?;

        // For now, return a mock access token - in real implementation,
        // this would generate a proper JWT access token
        let response = RefreshTokenResponse {
            access_token: "mock_access_token".to_string(),
            refresh_token: new_token_response.token,
            expires_in: 86400, // 24 hours
            token_type: "Bearer".to_string(),
        };

        info!("Token refresh completed successfully for user {}", token_record.user_id);
        Ok(Json(response))
    }

    /// Revoke a token (access or refresh)
    pub async fn revoke_token(
        State(handlers): State<Arc<SessionManagementHandlers>>,
        Json(request): Json<RevokeTokenRequest>,
    ) -> Result<Json<RevokeTokenResponse>, (StatusCode, Json<serde_json::Value>)> {
        info!("Token revocation request received");

        let reason = request.reason.as_deref().unwrap_or("User requested");

        // Try to revoke as refresh token first
        let success = match handlers.refresh_token_service
            .revoke_token(&request.token, reason)
            .await {
            Ok(_) => {
                info!("Successfully revoked refresh token");
                true
            }
            Err(e) => {
                warn!("Failed to revoke as refresh token: {}", e);
                // Could also try to revoke as access token here
                // by adding to JTI blacklist
                false
            }
        };

        let response = RevokeTokenResponse {
            success,
            message: if success {
                "Token revoked successfully".to_string()
            } else {
                "Token not found or already revoked".to_string()
            },
        };

        Ok(Json(response))
    }

    /// Get user sessions
    pub async fn get_user_sessions(
        State(handlers): State<Arc<SessionManagementHandlers>>,
        Path(user_id): Path<String>,
        Query(query): Query<SessionListQuery>,
    ) -> Result<Json<Vec<SessionInfo>>, (StatusCode, Json<serde_json::Value>)> {
        info!("Get sessions request for user: {}", user_id);

        let limit = query.limit.unwrap_or(50);
        let include_inactive = query.include_inactive.unwrap_or(false);

        // Get refresh tokens as a proxy for sessions
        let tokens = handlers.refresh_token_service
            .get_user_tokens(&user_id)
            .await
            .map_err(|e| {
                error!("Failed to get user tokens: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                    "error": "server_error",
                    "error_description": "Failed to retrieve sessions"
                })))
            })?;

        let mut sessions: Vec<SessionInfo> = tokens
            .into_iter()
            .filter(|_t| include_inactive || true) // TODO: Implement is_revoked field check
            .take(limit)
            .map(|token| SessionInfo {
                session_id: token.id.to_string(),
                created_at: token.created_at.to_rfc3339(),
                last_activity: token.created_at.to_rfc3339(), // Would be updated with actual activity
                ip_address: "unknown".to_string(), // TODO: RefreshToken model missing ip_address field
                device_info: None, // TODO: RefreshToken model missing device_info field
                geo_location: None, // Would need to be stored in token data
                is_active: true, // TODO: RefreshToken model missing is_revoked field
            })
            .collect();

        // Sort by creation time (most recent first)
        sessions.sort_by(|a, b| b.created_at.cmp(&a.created_at));

        info!("Found {} sessions for user {}", sessions.len(), user_id);
        Ok(Json(sessions))
    }

    /// Revoke a specific session
    pub async fn revoke_session(
        State(handlers): State<Arc<SessionManagementHandlers>>,
        Path((user_id, session_id)): Path<(String, String)>,
    ) -> Result<Json<RevokeTokenResponse>, (StatusCode, Json<serde_json::Value>)> {
        info!("Revoke session {} for user {}", session_id, user_id);

        // For now, assume session_id corresponds to a refresh token ID
        // In a real implementation, you'd look up the session and revoke associated tokens
        let result = handlers.security_service
            .terminate_session(&session_id, &user_id)
            .await;

        let response = RevokeTokenResponse {
            success: result.is_ok(),
            message: if result.is_ok() {
                "Session revoked successfully".to_string()
            } else {
                "Failed to revoke session".to_string()
            },
        };

        Ok(Json(response))
    }

    /// Analyze session security
    pub async fn analyze_session_security(
        State(handlers): State<Arc<SessionManagementHandlers>>,
        Path(user_id): Path<String>,
        Json(request): Json<AnalyzeSessionRequest>,
    ) -> Result<Json<SecurityAnalysisResult>, (StatusCode, Json<serde_json::Value>)> {
        info!("Security analysis request for user: {}", user_id);

        let ip_address = request.ip_address.parse()
            .map_err(|_| {
                (StatusCode::BAD_REQUEST, Json(serde_json::json!({
                    "error": "invalid_request",
                    "error_description": "Invalid IP address format"
                })))
            })?;

        let analysis = handlers.security_service
            .analyze_session_security(
                &user_id,
                ip_address,
                request.device_fingerprint,
                request.geo_location,
            )
            .await
            .map_err(|e| {
                error!("Security analysis failed: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                    "error": "server_error",
                    "error_description": "Security analysis failed"
                })))
            })?;

        info!("Security analysis completed for user {} with risk score {}", 
              user_id, analysis.risk_score);
        Ok(Json(analysis))
    }

    /// Get security events for a user
    pub async fn get_security_events(
        State(handlers): State<Arc<SessionManagementHandlers>>,
        Path(user_id): Path<String>,
        Query(query): Query<SecurityEventsQuery>,
    ) -> Result<Json<Vec<SecurityEvent>>, (StatusCode, Json<serde_json::Value>)> {
        info!("Get security events for user: {}", user_id);

        let limit = query.limit.unwrap_or(50);

        let events = handlers.security_service
            .get_user_security_events(&user_id, limit)
            .await
            .map_err(|e| {
                error!("Failed to get security events: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                    "error": "server_error",
                    "error_description": "Failed to retrieve security events"
                })))
            })?;

        info!("Found {} security events for user {}", events.len(), user_id);
        Ok(Json(events))
    }

    /// Get cleanup service health status
    pub async fn get_cleanup_health(
        State(handlers): State<Arc<SessionManagementHandlers>>,
    ) -> Result<Json<CleanupHealthStatus>, (StatusCode, Json<serde_json::Value>)> {
        debug!("Cleanup health status request");

        let health = handlers.cleanup_service
            .get_health_status()
            .await
            .map_err(|e| {
                error!("Failed to get cleanup health: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                    "error": "server_error",
                    "error_description": "Failed to get health status"
                })))
            })?;

        Ok(Json(health))
    }

    /// Run manual cleanup
    pub async fn run_manual_cleanup(
        State(handlers): State<Arc<SessionManagementHandlers>>,
    ) -> Result<Json<CleanupStats>, (StatusCode, Json<serde_json::Value>)> {
        info!("Manual cleanup request");

        let stats = handlers.cleanup_service
            .manual_cleanup()
            .await
            .map_err(|e| {
                error!("Manual cleanup failed: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                    "error": "server_error",
                    "error_description": "Cleanup failed"
                })))
            })?;

        info!("Manual cleanup completed: cleaned {} items", stats.total_cleaned);
        Ok(Json(stats))
    }

    /// Revoke all sessions for a user
    pub async fn revoke_all_user_sessions(
        State(handlers): State<Arc<SessionManagementHandlers>>,
        Path(user_id): Path<String>,
    ) -> Result<Json<RevokeTokenResponse>, (StatusCode, Json<serde_json::Value>)> {
        info!("Revoke all sessions for user: {}", user_id);

        let count = handlers.refresh_token_service
            .revoke_user_tokens(&user_id, "All sessions revoked by user")
            .await
            .map_err(|e| {
                error!("Failed to revoke user tokens: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                    "error": "server_error",
                    "error_description": "Failed to revoke sessions"
                })))
            })?;

        let response = RevokeTokenResponse {
            success: true,
            message: format!("Revoked {} sessions successfully", count),
        };

        info!("Revoked {} sessions for user {}", count, user_id);
        Ok(Json(response))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_structs() {
        let refresh_req = RefreshTokenRequest {
            refresh_token: "test_token".to_string(),
            device_info: None,
        };
        assert_eq!(refresh_req.refresh_token, "test_token");

        let revoke_req = RevokeTokenRequest {
            token: "test_token".to_string(),
            token_type_hint: Some("refresh_token".to_string()),
            reason: Some("test".to_string()),
        };
        assert_eq!(revoke_req.token, "test_token");
    }
}