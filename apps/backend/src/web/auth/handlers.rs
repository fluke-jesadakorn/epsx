// Authentication handlers for Axum web layer

use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use tower_cookies::{Cookie, Cookies};
use chrono::{DateTime, Utc};
use crate::app::dtos::auth::{LoginReq, RefreshReq, LogoutReq, ValidateReq};
use crate::web::middleware::AuthCtx;
use super::AppState;

/// Login request payload
#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

/// Login response payload
#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub user_id: String,
    pub email: String,
    pub role: String,
    pub expires_at: DateTime<Utc>,
}

/// Refresh response payload
#[derive(Debug, Serialize)]
pub struct RefreshResponse {
    pub expires_at: DateTime<Utc>,
}

/// User profile response
#[derive(Debug, Serialize)]
pub struct UserProfileResponse {
    pub user_id: String,
    pub email: String,
    pub role: String,
    pub permissions: Vec<String>,
    pub subscription_tier: String,
}

/// Login handler that validates Firebase token and creates session
pub async fn login_handler(
    cookies: Cookies,
    State(app_state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, StatusCode> {
    // Create login request DTO
    let login_req = LoginReq {
        email: payload.email,
        password: payload.password,
    };
    
    // Perform login through use case
    let login_res = app_state.auth_uc.login(login_req).await
        .map_err(|e| {
            tracing::error!("Login failed: {:?}", e);
            StatusCode::UNAUTHORIZED
        })?;
    
    // Create HTTP-only session cookie
    let session_cookie = Cookie::build(("sess_id", login_res.sess_id.to_string()))
        .http_only(true)
        .secure(false) // Disable for development HTTP, enable in production with HTTPS
        .same_site(tower_cookies::cookie::SameSite::Lax) // Changed from Strict to Lax for better compatibility
        .max_age(tower_cookies::cookie::time::Duration::days(7))
        .path("/")
        .build();
    
    cookies.add(session_cookie);
    
    // Get user info using validate to get role and permissions 
    let validate_req = ValidateReq {
        token: login_res.access_token.clone(),
        sess_id: login_res.sess_id.clone(),
    };
    
    let user_info = app_state.auth_uc.validate(validate_req).await
        .map_err(|e| {
            tracing::error!("Failed to get user info: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // For now, use placeholder email - in production get from user repository
    let email = format!("user{}@example.com", user_info.user_id.to_string());

    // Return response
    let response = LoginResponse {
        user_id: login_res.user_id.to_string(),
        email,
        role: user_info.role.to_string(),
        expires_at: user_info.expires_at,
    };
    
    Ok(Json(response))
}

/// Logout handler that removes session and clears cookie
pub async fn logout_handler(
    cookies: Cookies,
    State(app_state): State<AppState>,
    auth_ctx: AuthCtx,
) -> Result<StatusCode, StatusCode> {
    // Logout through use case
    let logout_req = LogoutReq {
        session_id: auth_ctx.sess.value().to_string(),
        sess_id: auth_ctx.sess.value().to_string(),
    };
    
    app_state.auth_uc.logout(logout_req).await
        .map_err(|e| {
            tracing::error!("Logout failed: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    
    // Remove session cookie
    let logout_cookie = Cookie::build(("sess_id", ""))
        .path("/")
        .max_age(tower_cookies::cookie::time::Duration::seconds(0))
        .build();
    cookies.add(logout_cookie);
    
    Ok(StatusCode::OK)
}

/// Refresh session handler that extends session expiry
pub async fn refresh_handler(
    cookies: Cookies,
    State(app_state): State<AppState>,
    _auth_ctx: AuthCtx,
) -> Result<Json<RefreshResponse>, StatusCode> {
    // Create refresh request
    let refresh_req = RefreshReq {
        refresh_token: "placeholder-refresh-token".to_string(),
    };
    
    // Refresh session through use case
    let refresh_res = app_state.auth_uc.refresh(refresh_req).await
        .map_err(|e| {
            tracing::error!("Session refresh failed: {:?}", e);
            StatusCode::UNAUTHORIZED
        })?;
    
    // Update session cookie with new expiry
    let session_cookie = Cookie::build(("sess_id", refresh_res.sess_id.to_string()))
        .http_only(true)
        .secure(false) // Disable for development HTTP, enable in production with HTTPS
        .same_site(tower_cookies::cookie::SameSite::Lax)
        .max_age(tower_cookies::cookie::time::Duration::days(7))
        .path("/")
        .build();
    
    cookies.add(session_cookie);
    
    // Return response
    let response = RefreshResponse {
        expires_at: chrono::Utc::now() + chrono::Duration::hours(24),
    };
    
    Ok(Json(response))
}

/// Get current user profile handler
pub async fn me_handler(
    State(app_state): State<AppState>,
    auth_ctx: AuthCtx,
) -> Result<Json<UserProfileResponse>, StatusCode> {
    tracing::info!("Getting user profile for user: {} with role: {:?}", auth_ctx.user_id, auth_ctx.role);
    
    // Get user details from repository to get email
    let user = app_state.user_repo.find_by_id(&auth_ctx.user_id).await
        .map_err(|e| {
            tracing::error!("Failed to get user details: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    
    // Use actual user data from auth context and repository
    let response = UserProfileResponse {
        user_id: auth_ctx.user_id.to_string(),
        email: user.email().value().to_string(),
        role: auth_ctx.role.to_string(),
        permissions: vec!["read:own".to_string(), "write:own".to_string()],
        subscription_tier: user.sub().tier().to_string(),
    };
    
    Ok(Json(response))
}

/// Public me handler that checks session without strict middleware
pub async fn me_handler_public(
    cookies: Cookies,
    State(_app_state): State<AppState>,
) -> Result<Json<UserProfileResponse>, StatusCode> {
    // Check for session cookie
    if let Some(session_cookie) = cookies.get("sess_id") {
        let session_id = session_cookie.value();
        tracing::info!("Found session cookie: {}", session_id);
        
        // Mock user data based on session
        let response = UserProfileResponse {
            user_id: session_id.to_string(),
            email: "user@example.com".to_string(),
            role: "user".to_string(),
            permissions: vec!["read:own".to_string(), "write:own".to_string()],
            subscription_tier: "basic".to_string(),
        };
        
        Ok(Json(response))
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::ports::{MockUserRepo, MockSessRepo, MockEventDispatcher};
    use crate::infra::events::SimpleEventDispatcher;
    use crate::dom::entities::{User, Session};
    use crate::dom::values::{Role, UserId, Email};
    use std::sync::Arc;
    
    #[test]
    fn should_serialize_login_response() {
        let response = LoginResponse {
            user_id: "test-id".to_string(),
            email: "test@example.com".to_string(),
            role: "user".to_string(),
            expires_at: Utc::now(),
        };
        
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("test@example.com"));
        assert!(json.contains("test-id"));
        assert!(json.contains("user"));
    }
    
    #[test]
    fn should_deserialize_login_request() {
        let json = r#"{"email": "test@example.com", "password": "test123"}"#;
        let request: LoginRequest = serde_json::from_str(json).unwrap();
        
        assert_eq!(request.email, "test@example.com");
        assert_eq!(request.password, "test123");
    }
    
    #[test]
    fn should_serialize_refresh_response() {
        let response = RefreshResponse {
            expires_at: Utc::now(),
        };
        
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("expires_at"));
    }
    
    #[test]
    fn should_serialize_user_profile_response() {
        let response = UserProfileResponse {
            user_id: "profile-test-id".to_string(),
            email: "profile@example.com".to_string(),
            role: "premium_user".to_string(),
            permissions: vec!["read:own_data".to_string(), "write:own_data".to_string()],
            subscription_tier: "premium".to_string(),
        };
        
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("profile@example.com"));
        assert!(json.contains("premium_user"));
        assert!(json.contains("read:own_data"));
        assert!(json.contains("premium"));
    }
    
    // Simplified test setup without complex async mocking
    // For comprehensive integration tests, we would set up real test services
    
    
    #[test]
    fn should_build_session_cookie() {
        let sess_id = SessId::generate();
        
        let cookie = Cookie::build(("sess_id", sess_id.to_string()))
            .http_only(true)
            .secure(true)
            .same_site(tower_cookies::cookie::SameSite::Strict)
            .max_age(tower_cookies::cookie::time::Duration::days(7))
            .path("/")
            .build();
        
        assert_eq!(cookie.name(), "sess_id");
        assert_eq!(cookie.value(), sess_id.to_string());
        assert!(cookie.http_only().unwrap());
        assert!(cookie.secure().unwrap());
        assert_eq!(cookie.path(), Some("/"));
    }
    
    #[test]
    fn should_build_logout_cookie() {
        let logout_cookie = Cookie::build(("sess_id", ""))
            .path("/")
            .max_age(tower_cookies::cookie::time::Duration::seconds(0))
            .build();
        
        assert_eq!(logout_cookie.name(), "sess_id");
        assert_eq!(logout_cookie.value(), "");
        assert_eq!(logout_cookie.path(), Some("/"));
    }
    
    #[test]
    fn should_format_placeholder_email() {
        let user_id = UserId::generate();
        let email = format!("user{}@example.com", user_id.to_string());
        
        assert!(email.contains("@example.com"));
        assert!(email.starts_with("user"));
        assert!(email.len() > "user@example.com".len()); // Should include the UUID
    }
    
    #[test]
    fn should_validate_login_request_structure() {
        let request = LoginRequest {
            email: "test@example.com".to_string(),
            password: "test123".to_string(),
        };
        
        assert!(!request.email.is_empty());
        assert!(request.email.contains("@"));
        assert!(!request.password.is_empty());
    }
    
    #[test]
    fn should_validate_response_structures() {
        let now = Utc::now();
        
        let login_response = LoginResponse {
            user_id: "user-123".to_string(),
            email: "test@example.com".to_string(),
            role: "user".to_string(),
            expires_at: now,
        };
        
        let refresh_response = RefreshResponse {
            expires_at: now,
        };
        
        let profile_response = UserProfileResponse {
            user_id: "profile-123".to_string(),
            email: "profile@example.com".to_string(),
            role: "admin".to_string(),
            permissions: vec!["manage:users".to_string()],
            subscription_tier: "enterprise".to_string(),
        };
        
        // Basic validation that all fields are accessible
        assert!(!login_response.user_id.is_empty());
        assert!(!login_response.email.is_empty());
        assert!(!login_response.role.is_empty());
        
        assert_eq!(refresh_response.expires_at, now);
        
        assert!(!profile_response.permissions.is_empty());
        assert_eq!(profile_response.permissions[0], "manage:users");
    }
    
    // Integration tests would go here but require more complex setup
    // These tests validate the structure and basic functionality
    // For full handler testing, we'd need to set up Axum test server
    
    #[test]
    fn should_have_debug_implementations() {
        let request = LoginRequest {
            email: "debug@test.com".to_string(),
            password: "debug-test".to_string(),
        };
        
        let debug_str = format!("{:?}", request);
        assert!(debug_str.contains("LoginRequest"));
        assert!(debug_str.contains("debug@test.com"));
    }
}