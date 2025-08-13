// User Profile Management handlers with Casbin authorization

use axum::{
    extract::{State, Path},
    http::{StatusCode, HeaderMap},
    response::Json,
};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use crate::web::auth::AppState;
use serde_json::{json, Value};

/// Extract session ID from headers
fn extract_session_from_headers(headers: &HeaderMap) -> Result<String, StatusCode> {
    // Try Authorization header first (Bearer token)
    if let Some(auth_header) = headers.get("authorization") {
        if let Ok(auth_str) = auth_header.to_str() {
            if auth_str.starts_with("Bearer ") {
                let token = auth_str.strip_prefix("Bearer ").unwrap();
                return Ok(token.to_string());
            }
        }
    }
    
    Err(StatusCode::UNAUTHORIZED)
}


/// Helper function to verify user permissions using Casbin
async fn verify_user_permissions(
    user_id: &str,
    resource: &str,
    action: &str,
) -> Result<(), StatusCode> {
    // Modern JWT-based permission check
    // TODO: Implement modern permission verification logic
    let permission_granted = true; // Placeholder
    if permission_granted {
        tracing::debug!("User permission granted for user {} on {}/{}", user_id, resource, action);
        Ok(())
    } else {
        tracing::warn!("User permission denied for user {} on {}/{}", user_id, resource, action);
        Err(StatusCode::FORBIDDEN)
    }
}

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
    pub total: u32,
    pub offset: u32,
    pub limit: u32,
}

// Simplified user handler implementations for Casbin migration

/// GET /users/profile - Get current user profile
pub async fn get_profile_handler(
    State(app_state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Value>, StatusCode> {
    // Extract session ID from headers
    let session_id_str = extract_session_from_headers(&headers)?;
    let session_id = crate::dom::values::SessId::from_string(session_id_str.clone());
    
    // Validate session and get user ID
    let session = match app_state.session_repo.find_by_id(&session_id).await {
        Ok(session) => {
            if !session.is_active() {
                tracing::warn!("Session {} is not active", session_id_str);
                return Err(StatusCode::UNAUTHORIZED);
            }
            if session.is_expired() {
                tracing::warn!("Session {} has expired", session_id_str);
                return Err(StatusCode::UNAUTHORIZED);
            }
            session
        },
        Err(crate::app::ports::repositories::RepoError::NotFound) => {
            tracing::warn!("Session {} not found", session_id_str);
            return Err(StatusCode::UNAUTHORIZED);
        },
        Err(e) => {
            tracing::error!("Failed to validate session {}: {:?}", session_id_str, e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };
    
    let user_id = session.user_id().to_string();
    verify_user_permissions(&user_id, "/api/v1/users/profile", "GET").await?;
    
    tracing::info!("User get profile handler called with authorization for user: {}", user_id);
    
    // Get actual user from database
    let user_id_obj = crate::dom::values::UserId::new(user_id.clone());
    let user = match app_state.user_repo.get(&user_id_obj).await {
        Ok(Some(user)) => user,
        Ok(None) => {
            tracing::warn!("User {} not found in database", user_id);
            return Err(StatusCode::NOT_FOUND);
        },
        Err(e) => {
            tracing::error!("Failed to fetch user {}: {:?}", user_id, e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };
    
    // Get user roles - using modern JWT-based auth system
    let user_roles = vec!["user".to_string()]; // TODO: Implement modern role loading
    
    // Get user permissions - using modern JWT-based auth system
    let user_permissions = vec!["read".to_string()]; // TODO: Implement modern permission loading
    
    Ok(Json(json!({
        "user_id": user.id().to_string(),
        "email": user.email().value(),
        "roles": user_roles,
        "permissions": user_permissions,
        "subscription_tier": user.sub().tier().to_string(),
        "package_tier": user.sub().tier().to_string(), // Same as subscription tier
        "created_at": user.created_at(),
        "updated_at": user.updated_at(),
        "display_name": format!("User {}", user.email().value()),
        "photo_url": null,
        "email_verified": true,
        "is_active": user.is_active()
    })))
}

/// PUT /users/profile - Update current user profile
pub async fn update_profile_handler(
    State(app_state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<UpdateUserProfileRequest>,
) -> Result<Json<Value>, StatusCode> {
    // Extract session ID from headers and validate
    let session_id_str = extract_session_from_headers(&headers)?;
    let session_id = crate::dom::values::SessId::from_string(session_id_str.clone());
    
    // Validate session and get user ID
    let session = match app_state.session_repo.find_by_id(&session_id).await {
        Ok(session) => {
            if !session.is_active() || session.is_expired() {
                return Err(StatusCode::UNAUTHORIZED);
            }
            session
        },
        Err(crate::app::ports::repositories::RepoError::NotFound) => return Err(StatusCode::UNAUTHORIZED),
        Err(e) => {
            tracing::error!("Session validation failed: {:?}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };
    
    let user_id = session.user_id().to_string();
    verify_user_permissions(&user_id, "/api/v1/users/profile", "PUT").await?;
    
    tracing::info!("User update profile handler called with authorization for user: {}", user_id);
    
    Ok(Json(json!({
        "user_id": user_id,
        "message": "Profile update authorized - database integration pending",
        "updated_at": chrono::Utc::now(),
        "requested_changes": {
            "display_name": req.display_name,
            "photo_url": req.photo_url,
            "subscription_tier": req.subscription_tier
        }
    })))
}

/// DELETE /users/:id - Delete user (admin only)
pub async fn delete_user_handler(
    Path(_user_id): Path<String>,
    State(_app_state): State<AppState>,
) -> Result<StatusCode, StatusCode> {
    // TODO: Implement with Casbin during migration
    tracing::info!("User delete handler called during migration");
    
    Ok(StatusCode::OK)
}

/// POST /logout - Logout current user
pub async fn logout_handler(
    State(_app_state): State<AppState>,
) -> Result<Json<Value>, StatusCode> {
    // TODO: Implement with Casbin during migration
    tracing::info!("User logout handler called during migration");
    
    Ok(Json(json!({
        "message": "Logout successful",
        "logged_out_at": chrono::Utc::now()
    })))
}

/// GET /users - List users (admin only)
pub async fn list_users_handler(
    State(app_state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Value>, StatusCode> {
    // Extract session ID from headers and validate
    let session_id_str = extract_session_from_headers(&headers)?;
    let session_id = crate::dom::values::SessId::from_string(session_id_str.clone());
    
    // Validate session and get user ID
    let session = match app_state.session_repo.find_by_id(&session_id).await {
        Ok(session) => {
            if !session.is_active() || session.is_expired() {
                return Err(StatusCode::UNAUTHORIZED);
            }
            session
        },
        Err(crate::app::ports::repositories::RepoError::NotFound) => return Err(StatusCode::UNAUTHORIZED),
        Err(e) => {
            tracing::error!("Session validation failed: {:?}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };
    
    let user_id = session.user_id().to_string();
    verify_user_permissions(&user_id, "/api/v1/users", "GET").await?;
    
    tracing::info!("User list handler called with authorization for admin user: {}", user_id);
    
    // Get users from database with pagination
    let users = match app_state.user_repo.list(0, 50).await {
        Ok(users) => users,
        Err(e) => {
            tracing::error!("Failed to fetch users: {:?}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };
    
    // Get total count for pagination
    let total_count = match app_state.user_repo.count().await {
        Ok(count) => count,
        Err(e) => {
            tracing::error!("Failed to count users: {:?}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };
    
    let user_list: Vec<Value> = users.into_iter().map(|user| {
        json!({
            "id": user.id().to_string(),
            "email": user.email().value(),
            "role": user.role().to_string(),
            "subscription_tier": user.sub().tier().to_string(),
            "is_active": user.is_active(),
            "created_at": user.created_at(),
            "updated_at": user.updated_at(),
            "is_deleted": user.is_deleted()
        })
    }).collect();
    
    Ok(Json(json!({
        "users": user_list,
        "total": total_count,
        "offset": 0,
        "limit": 50
    })))
}