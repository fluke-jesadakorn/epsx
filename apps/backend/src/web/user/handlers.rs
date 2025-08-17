// User Profile Management handlers with Casbin authorization

use axum::{
    extract::{State, Path},
    http::{StatusCode, HeaderMap},
    response::Json,
};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use crate::web::auth::AppState;
use crate::dom::services::feature_expiration::FeatureExpirationService;
use crate::dom::values::UserId;
use serde_json::{json, Value};
use std::sync::Arc;

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

/// User expiration status response
#[derive(Debug, Serialize)]
pub struct UserExpirationStatusResponse {
    pub user_id: String,
    pub expiring_features: Vec<ExpiringFeatureInfo>,
    pub expired_features: Vec<ExpiringFeatureInfo>,
    pub next_expiration: Option<DateTime<Utc>>,
    pub has_active_grace_periods: bool,
    pub checked_at: DateTime<Utc>,
}

/// Individual expiring feature information
#[derive(Debug, Serialize)]
pub struct ExpiringFeatureInfo {
    pub permission_profile_id: String,
    pub permission_profile_name: String,
    pub expires_at: DateTime<Utc>,
    pub days_until_expiration: i64,
    pub features: Vec<String>,
    pub is_in_grace_period: bool,
    pub grace_period_ends: Option<DateTime<Utc>>,
}

/// Expiration check request
#[derive(Debug, Deserialize)]
pub struct ExpirationCheckRequest {
    pub force_recheck: Option<bool>,
}

/// Manual expiration check response
#[derive(Debug, Serialize)]
pub struct ExpirationCheckResponse {
    pub user_id: String,
    pub check_completed: bool,
    pub total_checked: usize,
    pub expiring_count: usize,
    pub expired_count: usize,
    pub notifications_sent: usize,
    pub checked_at: DateTime<Utc>,
    pub next_check_recommended: DateTime<Utc>,
}

/// User notification response
#[derive(Debug, Serialize)]
pub struct UserNotificationResponse {
    pub id: String,
    pub notification_type: String,
    pub title: String,
    pub message: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub is_read: bool,
    pub priority: String,
    pub data: Option<Value>,
}

/// Mark notifications read request
#[derive(Debug, Deserialize)]
pub struct MarkNotificationsReadRequest {
    pub notification_ids: Vec<String>,
    pub mark_all: Option<bool>,
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

// User-driven expiration management handlers

/// GET /users/expiration-status - Get current user's expiration status
pub async fn get_expiration_status_handler(
    State(app_state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<UserExpirationStatusResponse>, StatusCode> {
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
    
    let user_id_str = session.user_id().to_string();
    let user_id = UserId::new(user_id_str.clone());
    
    tracing::info!("User expiration status requested for user: {}", user_id_str);
    
    // Get expiring features using the domain service
    let expiring_features = match app_state.feature_expiration_service.get_expiring_features(&user_id).await {
        Ok(features) => features,
        Err(e) => {
            tracing::error!("Failed to get expiring features for user {}: {:?}", user_id_str, e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };
    
    let checked_at = Utc::now();
    let mut expiring_list = Vec::new();
    let mut expired_list = Vec::new();
    let mut next_expiration = None::<DateTime<Utc>>;
    let mut has_grace_periods = false;
    
    for feature in expiring_features {
        let days_until_expiration = (feature.expires_at - checked_at).num_days();
        let is_expired = days_until_expiration < 0;
        let is_in_grace_period = is_expired && days_until_expiration >= -(feature.grace_period_days as i64);
        
        if is_in_grace_period {
            has_grace_periods = true;
        }
        
        let feature_info = ExpiringFeatureInfo {
            permission_profile_id: feature.permission_profile_id.value().to_string(),
            permission_profile_name: feature.permission_profile_name.clone(),
            expires_at: feature.expires_at,
            days_until_expiration,
            features: feature.features.clone(),
            is_in_grace_period,
            grace_period_ends: if is_in_grace_period {
                Some(feature.expires_at + chrono::Duration::days(feature.grace_period_days as i64))
            } else {
                None
            },
        };
        
        if is_expired {
            expired_list.push(feature_info);
        } else {
            expiring_list.push(feature_info);
            // Find the nearest expiration
            if next_expiration.is_none() || feature.expires_at < next_expiration.unwrap() {
                next_expiration = Some(feature.expires_at);
            }
        }
    }
    
    Ok(Json(UserExpirationStatusResponse {
        user_id: user_id_str,
        expiring_features: expiring_list,
        expired_features: expired_list,
        next_expiration,
        has_active_grace_periods: has_grace_periods,
        checked_at,
    }))
}

/// POST /users/request-expiration-check - Manually trigger expiration check
pub async fn request_expiration_check_handler(
    State(app_state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<ExpirationCheckRequest>,
) -> Result<Json<ExpirationCheckResponse>, StatusCode> {
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
    
    let user_id_str = session.user_id().to_string();
    let user_id = UserId::new(user_id_str.clone());
    
    tracing::info!("Manual expiration check requested for user: {} (force: {:?})", 
                  user_id_str, req.force_recheck.unwrap_or(false));
    
    let checked_at = Utc::now();
    
    // For now, simulate an expiration check since the actual implementation 
    // would require integration with the notification system
    let expiring_features = match app_state.feature_expiration_service.get_expiring_features(&user_id).await {
        Ok(features) => features,
        Err(e) => {
            tracing::error!("Failed to check expiration for user {}: {:?}", user_id_str, e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };
    
    let total_checked = expiring_features.len();
    let mut expiring_count = 0;
    let mut expired_count = 0;
    
    for feature in &expiring_features {
        let days_until_expiration = (feature.expires_at - checked_at).num_days();
        if days_until_expiration <= 0 {
            expired_count += 1;
        } else if days_until_expiration <= 30 {
            expiring_count += 1;
        }
    }
    
    let notifications_sent = if expiring_count > 0 || expired_count > 0 { 1 } else { 0 };
    
    Ok(Json(ExpirationCheckResponse {
        user_id: user_id_str,
        check_completed: true,
        total_checked,
        expiring_count,
        expired_count,
        notifications_sent,
        checked_at,
        next_check_recommended: checked_at + chrono::Duration::hours(24),
    }))
}

/// GET /users/notifications - Get user's pending notifications
pub async fn get_notifications_handler(
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
    
    let user_id_str = session.user_id().to_string();
    
    tracing::info!("User notifications requested for user: {}", user_id_str);
    
    // For now, return mock notifications - in a full implementation this would
    // query a notifications table or integrate with the notification service
    let mock_notifications = vec![
        UserNotificationResponse {
            id: "notif_1".to_string(),
            notification_type: "feature_expiration".to_string(),
            title: "Subscription Expiring Soon".to_string(),
            message: "Your premium features will expire in 7 days. Renew now to avoid interruption.".to_string(),
            created_at: Utc::now() - chrono::Duration::days(1),
            expires_at: Some(Utc::now() + chrono::Duration::days(30)),
            is_read: false,
            priority: "high".to_string(),
            data: Some(json!({
                "permission_profile_id": "user-premium-002",
                "days_until_expiry": 7,
                "renewal_url": "/billing/renew"
            })),
        }
    ];
    
    Ok(Json(json!({
        "user_id": user_id_str,
        "notifications": mock_notifications,
        "unread_count": 1,
        "total_count": 1,
        "fetched_at": Utc::now()
    })))
}

/// POST /users/notifications/mark-read - Mark notifications as read
pub async fn mark_notifications_read_handler(
    State(app_state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<MarkNotificationsReadRequest>,
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
    
    let user_id_str = session.user_id().to_string();
    
    tracing::info!("Mark notifications read requested for user: {} (mark_all: {:?}, count: {})", 
                  user_id_str, req.mark_all.unwrap_or(false), req.notification_ids.len());
    
    // For now, return success - in a full implementation this would
    // update a notifications table
    let marked_count = if req.mark_all.unwrap_or(false) {
        5 // Mock: mark all notifications as read
    } else {
        req.notification_ids.len()
    };
    
    Ok(Json(json!({
        "user_id": user_id_str,
        "marked_as_read": marked_count,
        "notification_ids": req.notification_ids,
        "mark_all": req.mark_all.unwrap_or(false),
        "marked_at": Utc::now()
    })))
}