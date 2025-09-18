use chrono::{DateTime, Utc};
// User Profile Management handlers with Casbin authorization

use axum::{
  extract::{ State, Path, Query },
  http::{ StatusCode, HeaderMap },
  response::Json,
};
use std::sync::Arc;
use serde::{ Deserialize, Serialize };
use crate::web::auth::AppState;
use crate::application::user_management::{GetUserByFirebaseUidQuery, ListUsersQuery};
use crate::infrastructure::integration::AuthenticationError;
use serde_json::{ json, Value };

#[derive(Deserialize)]
pub struct ListUsersParams {
    pub limit: Option<usize>,
    pub offset: Option<usize>,
    pub search: Option<String>,
}

/// Extract session ID from headers
fn extract_session_from_headers(
  headers: &HeaderMap
) -> Result<String, StatusCode> {
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

/// Helper function to verify user permissions using DDD query handlers
async fn verify_user_permissions(
  app_state: &AppState,
  firebase_uid: &str,
  required_permission: &str
) -> Result<(), StatusCode> {
  let firebase_uid_obj = match crate::domain::user_management::value_objects::FirebaseUid::new(firebase_uid.to_string()) {
    Ok(uid) => uid,
    Err(_) => {
      tracing::error!("Invalid Firebase UID: {}", firebase_uid);
      return Err(StatusCode::BAD_REQUEST);
    }
  };
  
  let query = GetUserByFirebaseUidQuery {
    firebase_uid: firebase_uid_obj,
  };
  
  match app_state.ddd_container.user_query_service().get_user_by_firebase_uid(query).await {
    Ok(user_response) => {
      let has_permission = user_response.permissions.iter().any(|p| {
        permission_matches(p.as_str(), required_permission)
      });
      
      if has_permission {
        tracing::debug!(
          "Permission granted for user {} to access {}",
          firebase_uid,
          required_permission
        );
        Ok(())
      } else {
        tracing::warn!(
          "Permission denied for user {} to access {}",
          firebase_uid,
          required_permission
        );
        Err(StatusCode::FORBIDDEN)
      }
    }
    Err(e) => {
      match e {
        crate::application::shared::ApplicationError::NotFound { .. } => {
          tracing::warn!("User {} not found for permission check", firebase_uid);
          Err(StatusCode::NOT_FOUND)
        }
        _ => {
          tracing::error!(
            "Permission check failed for user {} to access {}: {:?}",
            firebase_uid,
            required_permission,
            e
          );
          Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
      }
    }
  }
}

/// Check if user permission matches required permission (supports wildcards and embedded timestamps)
fn permission_matches(user_permission: &str, required_permission: &str) -> bool {
    // First check for embedded timestamp permissions (format: platform:resource:action:timestamp)
    let parts: Vec<&str> = user_permission.split(':').collect();
    if parts.len() == 4 {
        // This is an embedded timestamp permission - check if it's expired
        if let Ok(expiry_timestamp) = parts[3].parse::<i64>() {
            let current_timestamp = chrono::Utc::now().timestamp();
            if current_timestamp > expiry_timestamp {
                // Permission has expired
                tracing::debug!("Embedded permission {} has expired (current: {}, expiry: {})", 
                    user_permission, current_timestamp, expiry_timestamp);
                return false;
            }
            // Check the base permission (without timestamp)
            let base_permission = parts[0..3].join(":");
            return permission_matches(&base_permission, required_permission);
        }
    }
    
    // Exact match (fastest)
    if user_permission == required_permission {
        return true;
    }
    
    // Wildcard matching
    if user_permission.ends_with(":*:*") {
        let prefix = &user_permission[..user_permission.len() - 4];
        return required_permission.starts_with(prefix);
    }
    
    if user_permission.ends_with(":*") {
        let prefix = &user_permission[..user_permission.len() - 2];
        return required_permission.starts_with(prefix);
    }
    
    false
}

/// User profile response
#[derive(Debug, Serialize)]
pub struct UserProfileResponse {
  pub user_id: String,
  pub email: String,
  pub permissions: Vec<String>,
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
  headers: HeaderMap
) -> Result<Json<Value>, StatusCode> {
  // Extract Bearer token from headers (DDD uses token-based auth, not sessions)
  let token = extract_session_from_headers(&headers)?;
  
  // TODO: Replace with proper JWT token validation to get firebase_uid
  // For now, use the token as firebase_uid (this should be replaced with JWT parsing)
  let firebase_uid = token.clone();
  
  tracing::info!("User get profile handler called for firebase_uid: {}", firebase_uid);

  // Verify user permissions using DDD
  verify_user_permissions(&app_state, &firebase_uid, "epsx:profile:view").await?;

  // Get user using DDD query handler
  let firebase_uid_obj = match crate::domain::user_management::value_objects::FirebaseUid::new(firebase_uid.clone()) {
    Ok(uid) => uid,
    Err(_) => {
      tracing::error!("Invalid Firebase UID: {}", firebase_uid);
      return Err(StatusCode::BAD_REQUEST);
    }
  };
  
  let query = GetUserByFirebaseUidQuery {
    firebase_uid: firebase_uid_obj,
  };
  
  let user_response = match app_state.ddd_container.user_query_service().get_user_by_firebase_uid(query).await {
    Ok(response) => response,
    Err(e) => {
      match e {
        crate::application::shared::ApplicationError::NotFound { .. } => {
          tracing::warn!("User {} not found in database", firebase_uid);
          return Err(StatusCode::NOT_FOUND);
        }
        _ => {
          tracing::error!("Failed to fetch user {}: {:?}", firebase_uid, e);
          return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
      }
    }
  };

  // Convert permissions to string format for API compatibility
  let user_permissions: Vec<String> = user_response.permissions.iter()
    .map(|p| p.as_str().to_string())
    .collect();

  Ok(
    Json(
      json!({
        "user_id": user_response.firebase_uid.as_str(),
        "email": user_response.email.as_str(),
        "permissions": user_permissions,
        "package_tier": "premium", // TODO: Get from user subscription value object
        "created_at": chrono::Utc::now(), // TODO: Get from user aggregate
        "updated_at": chrono::Utc::now(), // TODO: Get from user aggregate
        "display_name": format!("User {}", user_response.email.as_str()),
        "photo_url": null,
        "email_verified": user_response.email_verified,
        "is_active": user_response.is_active
    })
    )
  )
}

/// PUT /users/profile - Update current user profile
pub async fn update_profile_handler(
  State(app_state): State<AppState>,
  headers: HeaderMap,
  Json(req): Json<UpdateUserProfileRequest>
) -> Result<Json<Value>, StatusCode> {
  // Extract Bearer token from headers
  let token = extract_session_from_headers(&headers)?;
  
  // TODO: Replace with proper JWT token validation to get firebase_uid
  let firebase_uid = token.clone();

  tracing::info!(
    "User update profile handler called with authorization for firebase_uid: {}",
    firebase_uid
  );

  // Verify user permissions using DDD
  verify_user_permissions(&app_state, &firebase_uid, "epsx:profile:update").await?;

  // TODO: Implement profile update using DDD command handler
  // This would involve creating an UpdateUserProfileCommand and handler
  // For now, return a success response to maintain API compatibility

  Ok(
    Json(
      json!({
        "user_id": firebase_uid,
        "message": "Profile update authorized - DDD implementation pending",
        "updated_at": chrono::Utc::now(),
        "requested_changes": {
            "display_name": req.display_name,
            "photo_url": req.photo_url
        }
    })
    )
  )
}

/// DELETE /users/:id - Delete user (admin only)
pub async fn delete_user_handler(
  Path(_user_id): Path<String>,
  State(_app_state): State<AppState>
) -> Result<StatusCode, StatusCode> {
  // TODO: Implement user deletion logic
  tracing::info!("User delete handler - implementation needed");

  Ok(StatusCode::OK)
}

/// Login request structure
#[derive(Debug, Deserialize)]
pub struct LoginRequest {
  pub email: String,
  pub password: String,
  pub token: String,
}

/// Logout request structure (Bearer token-based - body optional)
#[derive(Debug, Deserialize)]
pub struct LogoutRequest {
  // Optional fields for backward compatibility
  pub session_id: Option<String>,
  pub user_id: Option<String>,
}

/// Validate token request structure (Bearer token-based)
#[derive(Debug, Deserialize)]
pub struct ValidateSessionRequest {
  // Optional fields for backward compatibility
  pub session_id: Option<String>,
  pub token: Option<String>,
}

/// Refresh token request structure
#[derive(Debug, Deserialize)]
pub struct RefreshTokenRequest {
  pub refresh_token: String,
}

/// POST /login - Login user
pub async fn login_handler(
  State(app_state): State<AppState>,
  headers: HeaderMap,
  Json(login_req): Json<LoginRequest>
) -> Result<Json<Value>, StatusCode> {
  tracing::info!("Processing login via DDD authentication service");

  // Extract IP address for security monitoring
  let ip_address = headers.get("x-forwarded-for")
    .or_else(|| headers.get("x-real-ip"))
    .and_then(|h| h.to_str().ok())
    .unwrap_or("unknown")
    .to_string();

  let user_agent = headers.get("user-agent")
    .and_then(|h| h.to_str().ok())
    .map(|s| s.to_string());

  // Use DDD authentication service integration
  let auth_service = app_state.ddd_container.authentication_service_integration()
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
  match auth_service.create_session(&login_req.token, ip_address, user_agent).await {
    Ok(result) => {
      tracing::info!(
        session_id = result.session_id,
        user_id = result.user_id,
        "Login successful via DDD"
      );
      
      Ok(Json(json!({
        "message": "Login successful",
        "user_id": result.user_id,
        "session_id": result.session_id,
        "token": login_req.token,
        "profile": {
          "email": result.profile.email,
          "display_name": result.profile.display_name,
          "is_active": result.profile.is_active,
          "permissions": result.profile.permissions
        },
        "expires_at": result.expires_at,
        "logged_in_at": result.created_at
      })))
    },
    Err(e) => {
      tracing::error!(error = %e, "Login failed via DDD");
      match e {
        AuthenticationError::InvalidToken => {
          Err(StatusCode::UNAUTHORIZED)
        },
        AuthenticationError::UserIdentity(_) => {
          Err(StatusCode::FORBIDDEN)
        },
        _ => Err(StatusCode::INTERNAL_SERVER_ERROR)
      }
    }
  }
}

/// DELETE /logout - Logout current user (Bearer token-based)
pub async fn logout_handler(
  State(_app_state): State<AppState>,
  headers: HeaderMap,
  Json(_logout_req): Json<LogoutRequest>
) -> Result<Json<Value>, StatusCode> {
  tracing::info!("Processing Bearer token logout - stateless");
  
  // Extract user info from Bearer token for logging purposes
  let user_info = if let Some(auth_header) = headers.get("authorization") {
    if let Ok(auth_str) = auth_header.to_str() {
      if auth_str.starts_with("Bearer ") {
        let token = auth_str.strip_prefix("Bearer ").unwrap();
        // Try to decode token for user info (for logging only)
        match crate::auth::jwt::JWT.verify(token).await {
          Ok(claims) => Some(format!("user: {}", claims.sub)),
          Err(_) => Some("unknown user".to_string())
        }
      } else {
        None
      }
    } else {
      None
    }
  } else {
    None
  };
  
  tracing::info!(
    user_info = user_info.as_deref().unwrap_or("anonymous"),
    "Bearer token logout successful - client should delete token"
  );
  
  // For stateless JWT authentication, logout is handled client-side
  // The client should delete the Bearer token from storage
  Ok(Json(json!({
    "message": "Logout successful - please delete Bearer token from client storage",
    "logged_out_at": chrono::Utc::now(),
    "auth_type": "bearer_token",
    "action_required": "client_delete_token"
  })))
}

// Auth.js Integration Handlers

/// Get user claims for JWT token generation
pub async fn get_user_claims(
  State(_pool): State<Arc<crate::infrastructure::adapters::repositories::diesel_types::DbPool>>,
  Json(request): Json<serde_json::Value>
) -> Result<Json<serde_json::Value>, StatusCode> {
  tracing::info!("Getting user claims - stub implementation with request: {:?}", request);
  
  // For now, return a basic claims structure
  // In full implementation, would query user database and generate proper JWT claims
  let response = serde_json::json!({
    "error": "not_implemented",
    "message": "User claims generation not yet implemented",
    "status": "stub",
    "claims": {
      "sub": "stub_user",
      "permissions": [],
      "roles": []
    }
  });
  
  Ok(Json(response))
}

/// Upsert user for OAuth flow
pub async fn upsert_user(
  State(_pool): State<Arc<crate::infrastructure::adapters::repositories::diesel_types::DbPool>>,
  Json(request): Json<serde_json::Value>
) -> Result<Json<serde_json::Value>, StatusCode> {
  tracing::info!("Upserting user - stub implementation with request: {:?}", request);
  
  // For now, return a stub user upsert response
  // In full implementation, would create or update user in database
  let response = serde_json::json!({
    "error": "not_implemented", 
    "message": "User upsert not yet implemented",
    "status": "stub",
    "user": {
      "id": "stubuser_id",
      "email": "stub@example.com",
      "created": false,
      "updated": false
    }
  });
  
  Ok(Json(response))
}

// Additional stub handlers

pub async fn refresh_handler(
  State(app_state): State<AppState>,
  headers: HeaderMap,
  Json(req): Json<RefreshTokenRequest>
) -> Result<Json<Value>, StatusCode> {
  tracing::info!("Refreshing token via DDD authentication service");

  // Extract IP address for security monitoring
  let ip_address = headers.get("x-forwarded-for")
    .or_else(|| headers.get("x-real-ip"))
    .and_then(|h| h.to_str().ok())
    .unwrap_or("unknown")
    .to_string();

  // Use DDD authentication service integration
  let auth_service = app_state.ddd_container.authentication_service_integration()
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
  match auth_service.refresh_token(&req.refresh_token, ip_address).await {
    Ok(result) => {
      tracing::info!("Token refresh successful via DDD");
      
      Ok(Json(json!({
        "access_token": result.access_token,
        "refresh_token": result.refresh_token,
        "expires_at": result.expires_at,
        "token_type": "Bearer"
      })))
    },
    Err(e) => {
      tracing::error!(error = %e, "Token refresh failed via DDD");
      match e {
        AuthenticationError::InvalidRefreshToken => {
          Err(StatusCode::UNAUTHORIZED)
        },
        _ => Err(StatusCode::INTERNAL_SERVER_ERROR)
      }
    }
  }
}

pub async fn register_user() -> Result<Json<Value>, StatusCode> {
  tracing::info!("User registration - stub implementation");
  
  let response = json!({
    "error": "not_implemented",
    "message": "User registration not yet implemented", 
    "status": "stub"
  });
  
  Ok(Json(response))
}

pub async fn check_email_availability() -> Result<Json<Value>, StatusCode> {
  tracing::info!("Email availability check - stub implementation");
  
  let response = json!({
    "error": "not_implemented",
    "message": "Email availability check not yet implemented",
    "status": "stub",
    "available": false
  });
  
  Ok(Json(response))
}

pub async fn check_password_strength() -> Result<Json<Value>, StatusCode> {
  tracing::info!("Password strength check - stub implementation");
  
  let response = json!({
    "error": "not_implemented",
    "message": "Password strength check not yet implemented",
    "status": "stub",
    "strength": "unknown",
    "score": 0
  });
  
  Ok(Json(response))
}

pub async fn validate_session_handler(
  State(_app_state): State<AppState>,
  headers: HeaderMap,
  Json(_req): Json<ValidateSessionRequest>
) -> Result<Json<Value>, StatusCode> {
  tracing::info!("Validating Bearer token - stateless");

  // Extract IP address for security monitoring
  let ip_address = headers.get("x-forwarded-for")
    .or_else(|| headers.get("x-real-ip"))
    .and_then(|h| h.to_str().ok())
    .unwrap_or("unknown")
    .to_string();

  // Extract Bearer token from Authorization header
  let auth_header = headers.get("authorization")
    .and_then(|header| header.to_str().ok())
    .and_then(|header| {
      if header.starts_with("Bearer ") {
        Some(&header[7..])
      } else {
        None
      }
    });

  let token = match auth_header {
    Some(token) => token,
    None => {
      tracing::warn!("No Bearer token found in Authorization header");
      return Ok(Json(json!({
        "valid": false,
        "error": "No Bearer token provided",
        "auth_type": "bearer_token"
      })));
    }
  };

  // Validate JWT Bearer token
  match crate::auth::jwt::JWT.decode_with_permissions(token).await {
    Ok((user, permissions)) => {
      tracing::info!(
        user_id = user.id,
        user_email = user.email,
        ip_address = ip_address,
        "Bearer token validation successful"
      );
      
      Ok(Json(json!({
        "valid": true,
        "user_id": user.id,
        "user_email": user.email,
        "permissions": permissions,
        "auth_type": "bearer_token",
        "validated_at": chrono::Utc::now()
      })))
    },
    Err(e) => {
      tracing::warn!(
        error = %e, 
        ip_address = ip_address,
        "Bearer token validation failed"
      );
      Ok(Json(json!({
        "valid": false,
        "error": format!("Bearer token validation failed: {:?}", e),
        "auth_type": "bearer_token"
      })))
    }
  }
}

pub async fn rotate_session_handler() -> Result<Json<Value>, StatusCode> {
  tracing::info!("Session rotation - stub implementation");
  
  let response = json!({
    "error": "not_implemented",
    "message": "Session rotation not yet implemented",
    "status": "stub"
  });
  
  Ok(Json(response))
}

pub async fn validate_route_access_handler() -> Result<
  Json<Value>,
  StatusCode
> {
  tracing::info!("Route access validation - stub implementation");
  
  let response = json!({
    "error": "not_implemented", 
    "message": "Route access validation not yet implemented",
    "status": "stub",
    "allowed": false
  });
  
  Ok(Json(response))
}

pub async fn validate_bulk_routes_handler() -> Result<Json<Value>, StatusCode> {
  tracing::info!("Bulk routes validation - stub implementation");
  
  let response = json!({
    "error": "not_implemented",
    "message": "Bulk routes validation not yet implemented", 
    "status": "stub",
    "routes": []
  });
  
  Ok(Json(response))
}

pub async fn check_permission_handler() -> Result<Json<Value>, StatusCode> {
  tracing::info!("Permission check - stub implementation");
  
  let response = json!({
    "error": "not_implemented",
    "message": "Permission check not yet implemented",
    "status": "stub", 
    "has_permission": false
  });
  
  Ok(Json(response))
}

pub async fn single_permission_handler() -> Result<Json<Value>, StatusCode> {
  tracing::info!("Single permission check - stub implementation");
  
  let response = json!({
    "error": "not_implemented",
    "message": "Single permission check not yet implemented",
    "status": "stub",
    "permission": null,
    "granted": false
  });
  
  Ok(Json(response))
}

pub async fn navigation_handler() -> Result<Json<Value>, StatusCode> {
  tracing::info!("Navigation handler - stub implementation");
  
  let response = json!({
    "error": "not_implemented",
    "message": "Navigation handler not yet implemented",
    "status": "stub",
    "navigation": []
  });
  
  Ok(Json(response))
}

pub async fn user_features_handler() -> Result<Json<Value>, StatusCode> {
  tracing::info!("User features handler - stub implementation");
  
  let response = json!({
    "error": "not_implemented", 
    "message": "User features handler not yet implemented",
    "status": "stub",
    "features": []
  });
  
  Ok(Json(response))
}

/// GET /users - List users (admin only)
pub async fn list_users_handler(
  State(app_state): State<AppState>,
  Query(params): Query<ListUsersParams>,
  headers: HeaderMap
) -> Result<Json<Value>, StatusCode> {
  // Extract Bearer token from headers
  let token = extract_session_from_headers(&headers)?;
  
  // TODO: Replace with proper JWT token validation to get firebase_uid
  let firebase_uid = token.clone();

  tracing::info!(
    "User list handler called with authorization for admin firebase_uid: {}",
    firebase_uid
  );

  // Verify user has admin permissions using DDD
  verify_user_permissions(&app_state, &firebase_uid, "admin:users:list").await?;

  // Get users using DDD query handler
  let query = ListUsersQuery {
    limit: params.limit.unwrap_or(50),
    offset: params.offset.unwrap_or(0),
    email_domain_filter: None,
    permission_filter: None,
  };
  
  let list_response = match app_state.ddd_container.user_query_service().list_users(query).await {
    Ok(response) => response,
    Err(e) => {
      tracing::error!("Failed to fetch users: {:?}", e);
      return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }
  };

  // Convert users to response format
  let mut user_list = Vec::new();
  for user_summary in list_response.users {
    let user_permissions: Vec<String> = user_summary.permissions.iter()
      .map(|p| p.as_str().to_string())
      .collect();
    
    user_list.push(json!({
      "id": user_summary.firebase_uid.as_str(),
      "email": user_summary.email.as_str(),
      "display_name": user_summary.display_name,
      "permissions": user_permissions,
      "package_tier": user_summary.package_tier,
      "is_active": user_summary.is_active,
      "created_at": user_summary.created_at,
      "updated_at": user_summary.updated_at,
      "last_login_at": user_summary.last_login_at,
    }));
  }

  Ok(
    Json(
      json!({
        "users": user_list,
        "total": list_response.total_count,
        "offset": params.offset.unwrap_or(0),
        "limit": params.limit.unwrap_or(50)
    })
    )
  )
}

// User-driven expiration management handlers

/// GET /users/expiration-status - Get current user's expiration status
pub async fn get_expiration_status_handler(
  State(_app_state): State<AppState>,
  headers: HeaderMap
) -> Result<Json<UserExpirationStatusResponse>, StatusCode> {
  // Extract Bearer token from headers
  let token = extract_session_from_headers(&headers)?;
  
  // TODO: Replace with proper JWT token validation to get firebase_uid
  let firebase_uid = token.clone();

  tracing::info!("User expiration status requested for firebase_uid: {}", firebase_uid);

  // DDD system doesn't have legacy feature expiration - return empty response
  // TODO: Implement expiration logic using DDD if needed
  
  let checked_at = Utc::now();
  let expiring_list = Vec::new();
  let expired_list = Vec::new();
  let next_expiration = None::<DateTime<Utc>>;
  let has_grace_periods = false;

  Ok(
    Json(UserExpirationStatusResponse {
      user_id: firebase_uid, // API contract uses user_id field name but contains firebase_uid
      expiring_features: expiring_list,
      expired_features: expired_list,
      next_expiration,
      has_active_grace_periods: has_grace_periods,
      checked_at,
    })
  )
}

/// POST /users/request-expiration-check - Manually trigger expiration check
pub async fn request_expiration_check_handler(
  State(_app_state): State<AppState>,
  headers: HeaderMap,
  Json(req): Json<ExpirationCheckRequest>
) -> Result<Json<ExpirationCheckResponse>, StatusCode> {
  // Extract Bearer token from headers
  let token = extract_session_from_headers(&headers)?;
  
  // TODO: Replace with proper JWT token validation to get firebase_uid
  let firebase_uid = token.clone();

  tracing::info!(
    "Manual expiration check requested for firebase_uid: {} (force: {:?})",
    firebase_uid,
    req.force_recheck.unwrap_or(false)
  );

  let checked_at = Utc::now();

  // DDD system doesn't have legacy feature expiration - return empty response
  // TODO: Implement expiration check logic using DDD if needed
  
  let total_checked = 0;
  let expiring_count = 0;
  let expired_count = 0;
  let notifications_sent = 0;

  Ok(
    Json(ExpirationCheckResponse {
      user_id: firebase_uid, // API contract uses user_id field name but contains firebase_uid
      check_completed: true,
      total_checked,
      expiring_count,
      expired_count,
      notifications_sent,
      checked_at,
      next_check_recommended: checked_at + chrono::Duration::hours(24),
    })
  )
}

/// GET /users/notifications - Get user's pending notifications
pub async fn get_notifications_handler(
  State(_app_state): State<AppState>,
  headers: HeaderMap
) -> Result<Json<Value>, StatusCode> {
  // Extract Bearer token from headers
  let token = extract_session_from_headers(&headers)?;
  
  // TODO: Replace with proper JWT token validation to get firebase_uid
  let firebase_uid = token.clone();

  tracing::info!("User notifications requested for firebase_uid: {}", firebase_uid);

  // TODO: Implement notifications using DDD Notification bounded context
  // For now, return empty notifications to maintain API compatibility
  let notifications: Vec<UserNotificationResponse> = vec![];

  Ok(
    Json(
      json!({
        "user_id": firebase_uid, // API contract uses user_id field name but contains firebase_uid
        "notifications": notifications,
        "unread_count": 0,
        "total_count": 0,
        "fetched_at": Utc::now()
    })
    )
  )
}

/// POST /users/notifications/mark-read - Mark notifications as read
pub async fn mark_notifications_read_handler(
  State(_app_state): State<AppState>,
  headers: HeaderMap,
  Json(req): Json<MarkNotificationsReadRequest>
) -> Result<Json<Value>, StatusCode> {
  // Extract Bearer token from headers
  let token = extract_session_from_headers(&headers)?;
  
  // TODO: Replace with proper JWT token validation to get firebase_uid
  let firebase_uid = token.clone();

  tracing::info!(
    "Mark notifications read requested for firebase_uid: {} (mark_all: {:?}, count: {})",
    firebase_uid,
    req.mark_all.unwrap_or(false),
    req.notification_ids.len()
  );

  // TODO: Implement using DDD Notification bounded context
  // For now, return success to maintain API compatibility
  let marked_count = if req.mark_all.unwrap_or(false) {
    0 // No notifications to mark in DDD system yet
  } else {
    0 // No notifications to mark in DDD system yet
  };

  Ok(
    Json(
      json!({
        "user_id": firebase_uid, // API contract uses user_id field name but contains firebase_uid
        "marked_as_read": marked_count,
        "notification_ids": req.notification_ids,
        "mark_all": req.mark_all.unwrap_or(false),
        "marked_at": Utc::now()
    })
    )
  )
}

/// GET /api/v1/auth/user - Get current user info
pub async fn me_handler(
  State(app_state): State<AppState>,
  headers: HeaderMap
) -> Result<Json<Value>, StatusCode> {
  // Extract Bearer token from headers
  let token = extract_session_from_headers(&headers)?;
  
  // TODO: Replace with proper JWT token validation to get firebase_uid
  let firebase_uid = token.clone();

  // Get user using DDD query handler
  let firebase_uid_obj = match crate::domain::user_management::value_objects::FirebaseUid::new(firebase_uid.clone()) {
    Ok(uid) => uid,
    Err(_) => {
      tracing::error!("Invalid Firebase UID: {}", firebase_uid);
      return Err(StatusCode::BAD_REQUEST);
    }
  };
  
  let query = GetUserByFirebaseUidQuery {
    firebase_uid: firebase_uid_obj,
  };
  
  let user_response = match app_state.ddd_container.user_query_service().get_user_by_firebase_uid(query).await {
    Ok(response) => response,
    Err(e) => {
      match e {
        crate::application::shared::ApplicationError::NotFound { .. } => {
          tracing::warn!("User {} not found in database", firebase_uid);
          return Err(StatusCode::NOT_FOUND);
        }
        _ => {
          tracing::error!("Failed to fetch user {}: {:?}", firebase_uid, e);
          return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
      }
    }
  };
  
  // Convert permissions to string format for API compatibility
  let user_permissions: Vec<String> = user_response.permissions.iter()
    .map(|p| p.as_str().to_string())
    .collect();

  Ok(
    Json(
      json!({
        "id": user_response.firebase_uid.as_str(),
        "email": user_response.email.as_str(),
        "permissions": user_permissions,
        "package_tier": "premium", // TODO: Get from user subscription
        "is_active": user_response.is_active,
        "created_at": chrono::Utc::now(), // TODO: Get from user aggregate
        "updated_at": chrono::Utc::now() // TODO: Get from user aggregate
    })
    )
  )
}
