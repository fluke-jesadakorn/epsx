// User Profile Management handlers using PostgreSQL backend

use axum::{
    extract::{State, Path},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use tower_cookies::Cookies;
use chrono::{DateTime, Utc};
use crate::dom::values::{UserId, Role, SubTier};
use crate::dom::entities::User;
use super::super::auth::routes::AppState;

/// User profile response
#[derive(Debug, Serialize)]
pub struct UserProfileResponse {
    pub user_id: String,
    pub email: String,
    pub role: String,
    pub permissions: Vec<String>,
    pub subscription_tier: String,
    pub package_tier: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub display_name: Option<String>,
    pub photo_url: Option<String>,
    pub email_verified: bool,
}

/// User profile update request
#[derive(Debug, Deserialize)]
pub struct UpdateUserProfileRequest {
    pub display_name: Option<String>,
    pub photo_url: Option<String>,
    pub subscription_tier: Option<String>,
}

/// User profile update response
#[derive(Debug, Serialize)]
pub struct UpdateUserProfileResponse {
    pub user_id: String,
    pub message: String,
    pub updated_at: DateTime<Utc>,
}

/// User list response for admin operations
#[derive(Debug, Serialize)]
pub struct UserListResponse {
    pub users: Vec<UserProfileResponse>,
    pub total: u64,
    pub page: u32,
    pub limit: u32,
}

/// Get current user profile
pub async fn get_current_user_handler(
    cookies: Cookies,
    State(app_state): State<AppState>,
) -> Result<Json<UserProfileResponse>, StatusCode> {
    // Extract user ID from session cookie
    let user_id = extract_user_from_session(&cookies)?;
    
    // Get user from repository
    let user = app_state.user_repo.get(&user_id).await
        .map_err(|e| {
            tracing::error!("Failed to get user: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::NOT_FOUND)?;

    let profile = build_user_profile_response(&user);
    Ok(Json(profile))
}

/// Get user profile by ID (admin only)
pub async fn get_user_by_id_handler(
    Path(id): Path<String>,
    cookies: Cookies,
    State(app_state): State<AppState>,
) -> Result<Json<UserProfileResponse>, StatusCode> {
    // Verify admin access
    let current_user_id = extract_user_from_session(&cookies)?;
    verify_admin_access(&app_state, &current_user_id).await?;
    
    // Get requested user
    let user_id = UserId::new(id);
    let user = app_state.user_repo.get(&user_id).await
        .map_err(|e| {
            tracing::error!("Failed to get user: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::NOT_FOUND)?;

    let profile = build_user_profile_response(&user);
    Ok(Json(profile))
}

/// Update current user profile
pub async fn update_user_profile_handler(
    cookies: Cookies,
    State(app_state): State<AppState>,
    Json(payload): Json<UpdateUserProfileRequest>,
) -> Result<Json<UpdateUserProfileResponse>, StatusCode> {
    // Extract user ID from session
    let user_id = extract_user_from_session(&cookies)?;
    
    // Get current user
    let user = app_state.user_repo.get(&user_id).await
        .map_err(|e| {
            tracing::error!("Failed to get user for update: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::NOT_FOUND)?;

    // Update subscription tier if provided and valid
    if let Some(tier_str) = payload.subscription_tier {
        let _new_tier = SubTier::from_string(&tier_str)
            .map_err(|e| {
                tracing::error!("Invalid subscription tier: {:?}", e);
                StatusCode::BAD_REQUEST
            })?;
        
        // Create updated user with new subscription tier
        // Note: In a real implementation, you'd update the user's subscription
        // For now, we'll just log the change
        tracing::info!("User {} requested subscription tier change to: {}", user_id.to_string(), tier_str);
    }

    // Save updated user
    app_state.user_repo.save(&user).await
        .map_err(|e| {
            tracing::error!("Failed to update user: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(UpdateUserProfileResponse {
        user_id: user_id.to_string(),
        message: "Profile updated successfully".to_string(),
        updated_at: Utc::now(),
    }))
}

/// List users (admin only)
pub async fn list_users_handler(
    cookies: Cookies,
    State(app_state): State<AppState>,
) -> Result<Json<UserListResponse>, StatusCode> {
    // Verify admin access
    let current_user_id = extract_user_from_session(&cookies)?;
    verify_admin_access(&app_state, &current_user_id).await?;
    
    // Get users list
    let users = app_state.user_repo.find_all().await
        .map_err(|e| {
            tracing::error!("Failed to list users: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let total = users.len() as u64;
    let user_profiles: Vec<UserProfileResponse> = users
        .into_iter()
        .map(|user| build_user_profile_response(&user))
        .collect();

    Ok(Json(UserListResponse {
        users: user_profiles,
        total,
        page: 1,
        limit: 100, // Default limit
    }))
}

/// Delete user (admin only)
pub async fn delete_user_handler(
    Path(id): Path<String>,
    cookies: Cookies,
    State(app_state): State<AppState>,
) -> Result<StatusCode, StatusCode> {
    // Verify admin access
    let current_user_id = extract_user_from_session(&cookies)?;
    verify_admin_access(&app_state, &current_user_id).await?;
    
    // Prevent self-deletion
    let user_id = UserId::new(id);
    if user_id == current_user_id {
        return Err(StatusCode::FORBIDDEN);
    }
    
    // Delete user
    app_state.user_repo.delete(&user_id).await
        .map_err(|e| {
            tracing::error!("Failed to delete user: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(StatusCode::NO_CONTENT)
}

/// Logout handler
pub async fn logout_handler(
    cookies: Cookies,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Clear session cookie
    let cookie = tower_cookies::Cookie::build(("sess_id", ""))
        .max_age(tower_cookies::cookie::time::Duration::seconds(0))
        .path("/")
        .build();
    
    cookies.add(cookie);
    
    Ok(Json(serde_json::json!({
        "message": "Logged out successfully"
    })))
}

/// Helper function to extract user ID from session cookie
fn extract_user_from_session(cookies: &Cookies) -> Result<UserId, StatusCode> {
    let session_cookie = cookies.get("sess_id")
        .ok_or(StatusCode::UNAUTHORIZED)?;
    
    // In a real implementation, you would validate the session ID
    // For now, we'll just use it as a user ID (development only)
    let user_id = UserId::new(session_cookie.value().to_string());
    Ok(user_id)
}

/// Helper function to verify admin access
async fn verify_admin_access(app_state: &AppState, user_id: &UserId) -> Result<(), StatusCode> {
    let user = app_state.user_repo.get(user_id).await
        .map_err(|e| {
            tracing::error!("Failed to get user for admin check: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::UNAUTHORIZED)?;

    // Check if user has admin role
    match user.role() {
        Role::Admin => Ok(()),
        Role::SuperAdmin => Ok(()),
        _ => {
            tracing::warn!("Non-admin user {} attempted admin operation", user_id.to_string());
            Err(StatusCode::FORBIDDEN)
        }
    }
}

/// Helper function to build user profile response
fn build_user_profile_response(user: &User) -> UserProfileResponse {
    UserProfileResponse {
        user_id: user.id().to_string(),
        email: user.email().value().to_string(),
        role: user.role().to_string(),
        permissions: user.perms().to_vec(),
        subscription_tier: user.sub().tier.to_string(),
        package_tier: user.sub().tier.to_string(), // For now, use same as subscription
        created_at: user.created_at(),
        updated_at: user.updated_at(),
        display_name: Some(user.email().value().split('@').next().unwrap_or("User").to_string()), // Default display name
        photo_url: None, // TODO: Implement photo storage
        email_verified: true, // Backend users are considered verified
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tower_cookies::Cookies;
    use crate::dom::values::Email;
    
    #[test]
    fn should_build_user_profile_response() {
        let email = Email::new("test@example.com").unwrap();
        let user = User::new(email, Role::User);
        
        let response = build_user_profile_response(&user);
        
        assert_eq!(response.email, "test@example.com");
        assert_eq!(response.role, "user");
        assert!(response.email_verified);
        assert_eq!(response.display_name, Some("test".to_string()));
    }
    
    #[test]
    fn should_handle_empty_session_cookie() {
        let cookies = Cookies::default();
        let result = extract_user_from_session(&cookies);
        
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), StatusCode::UNAUTHORIZED);
    }
    
    #[test]
    fn should_deserialize_update_profile_request() {
        let json = r#"{"display_name": "John Doe", "subscription_tier": "premium"}"#;
        let request: UpdateUserProfileRequest = serde_json::from_str(json).unwrap();
        
        assert_eq!(request.display_name, Some("John Doe".to_string()));
        assert_eq!(request.subscription_tier, Some("premium".to_string()));
    }
    
    #[test]
    fn should_serialize_user_profile_response() {
        let response = UserProfileResponse {
            user_id: "user123".to_string(),
            email: "test@example.com".to_string(),
            role: "user".to_string(),
            permissions: vec!["read:own".to_string()],
            subscription_tier: "basic".to_string(),
            package_tier: "basic".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            display_name: Some("Test User".to_string()),
            photo_url: None,
            email_verified: true,
        };
        
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("test@example.com"));
        assert!(json.contains("user123"));
        assert!(json.contains("true")); // email_verified
    }
}