use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::web::auth::routes::AppState;
use crate::dom::services::{FirebaseUserService, FirebaseUserServiceTrait, CreateUserRequest, UpdateUserRequest, UserListFilters};
use crate::infra::firebase_admin::FirebaseUser;

/// User creation request from admin frontend
#[derive(Debug, Deserialize)]
pub struct AdminCreateUserRequest {
    pub email: Option<String>,
    pub password: Option<String>,
    pub display_name: Option<String>,
    pub role: Option<String>,
    pub email_verified: Option<bool>,
    pub disabled: Option<bool>,
}

/// User update request from admin frontend
#[derive(Debug, Deserialize)]
pub struct AdminUpdateUserRequest {
    pub email: Option<String>,
    pub display_name: Option<String>,
    pub role: Option<String>,
    pub disabled: Option<bool>,
}

/// User list query parameters
#[derive(Debug, Deserialize)]
pub struct UserListQuery {
    pub page_size: Option<u32>,
    pub page_token: Option<String>,
    pub role_filter: Option<String>,
    pub email_domain: Option<String>,
    pub search: Option<String>,
}

/// User response for API
#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub uid: String,
    pub email: Option<String>,
    pub email_verified: bool,
    pub display_name: Option<String>,
    pub photo_url: Option<String>,
    pub phone_number: Option<String>,
    pub disabled: bool,
    pub role: String,
    pub permissions: Vec<String>,
    pub provider_data: Vec<UserProviderResponse>,
    pub created_at: String,
    pub last_login_at: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct UserProviderResponse {
    pub provider_id: String,
    pub uid: String,
    pub email: Option<String>,
    pub display_name: Option<String>,
    pub photo_url: Option<String>,
}

/// User list response
#[derive(Debug, Serialize)]
pub struct UserListResponse {
    pub users: Vec<UserResponse>,
    pub next_page_token: Option<String>,
    pub total_count: Option<u32>,
}

/// User creation response
#[derive(Debug, Serialize)]
pub struct CreateUserResponse {
    pub uid: String,
    pub message: String,
}

/// Standard API error response
#[derive(Debug, Serialize)]
pub struct ApiErrorResponse {
    pub error: String,
    pub message: String,
    pub details: Option<String>,
}

/// Create new user via Firebase Admin API
/// POST /admin/users
pub async fn create_user(
    State(_state): State<AppState>,
    Json(request): Json<AdminCreateUserRequest>,
) -> Result<Json<CreateUserResponse>, (StatusCode, Json<ApiErrorResponse>)> {
    tracing::info!("Admin: Creating new user with email: {:?}", request.email);
    
    // Create Firebase user service
    let firebase_user_service = FirebaseUserService::new()
        .await
        .map_err(|e| {
            tracing::error!("Failed to create Firebase user service: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiErrorResponse {
                    error: "service_error".to_string(),
                    message: "Failed to initialize user service".to_string(),
                    details: Some(e.to_string()),
                }),
            )
        })?;
    
    // Create user request
    let create_request = CreateUserRequest {
        email: request.email.clone(),
        password: request.password,
        display_name: request.display_name,
        role: request.role.clone(),
    };
    
    // Create user in Firebase
    let firebase_uid = firebase_user_service
        .create_user(create_request)
        .await
        .map_err(|e| {
            tracing::error!("Failed to create Firebase user: {}", e);
            
            let status_code = match e.to_string().as_str() {
                s if s.contains("already exists") => StatusCode::CONFLICT,
                s if s.contains("invalid") => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };
            
            (
                status_code,
                Json(ApiErrorResponse {
                    error: "user_creation_failed".to_string(),
                    message: e.to_string(),
                    details: None,
                }),
            )
        })?;
    
    // Set additional properties if specified
    if request.disabled == Some(true) || request.email_verified == Some(true) {
        let update_request = UpdateUserRequest {
            email: None,
            display_name: None,
            disabled: request.disabled,
            role: None,
        };
        
        if let Err(e) = firebase_user_service.update_user(&firebase_uid, update_request).await {
            tracing::warn!("Failed to set additional user properties: {}", e);
        }
    }
    
    tracing::info!("Successfully created Firebase user: {}", firebase_uid);
    
    Ok(Json(CreateUserResponse {
        uid: firebase_uid,
        message: "User created successfully".to_string(),
    }))
}

/// Get user by Firebase UID
/// GET /admin/users/:uid
pub async fn get_user(
    State(_state): State<AppState>,
    Path(firebase_uid): Path<String>,
) -> Result<Json<UserResponse>, (StatusCode, Json<ApiErrorResponse>)> {
    tracing::info!("Admin: Getting user by UID: {}", firebase_uid);
    
    let firebase_user_service = FirebaseUserService::new()
        .await
        .map_err(|e| {
            tracing::error!("Failed to create Firebase user service: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiErrorResponse {
                    error: "service_error".to_string(),
                    message: "Failed to initialize user service".to_string(),
                    details: Some(e.to_string()),
                }),
            )
        })?;
    
    let firebase_user = firebase_user_service
        .get_user_by_uid(&firebase_uid)
        .await
        .map_err(|e| {
            tracing::error!("Failed to get Firebase user {}: {}", firebase_uid, e);
            
            let status_code = if e.to_string().contains("not found") {
                StatusCode::NOT_FOUND
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            };
            
            (
                status_code,
                Json(ApiErrorResponse {
                    error: "user_not_found".to_string(),
                    message: e.to_string(),
                    details: None,
                }),
            )
        })?;
    
    let user_response = convert_firebase_user_to_response(&firebase_user, &firebase_user_service).await;
    Ok(Json(user_response))
}

/// Update user
/// PUT /admin/users/:uid
pub async fn update_user(
    State(_state): State<AppState>,
    Path(firebase_uid): Path<String>,
    Json(request): Json<AdminUpdateUserRequest>,
) -> Result<Json<UserResponse>, (StatusCode, Json<ApiErrorResponse>)> {
    tracing::info!("Admin: Updating user: {}", firebase_uid);
    
    let firebase_user_service = FirebaseUserService::new()
        .await
        .map_err(|e| {
            tracing::error!("Failed to create Firebase user service: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiErrorResponse {
                    error: "service_error".to_string(),
                    message: "Failed to initialize user service".to_string(),
                    details: Some(e.to_string()),
                }),
            )
        })?;
    
    // Create update request
    let update_request = UpdateUserRequest {
        email: request.email,
        display_name: request.display_name,
        disabled: request.disabled,
        role: request.role,
    };
    
    // Update user in Firebase
    firebase_user_service
        .update_user(&firebase_uid, update_request)
        .await
        .map_err(|e| {
            tracing::error!("Failed to update Firebase user {}: {}", firebase_uid, e);
            
            let status_code = if e.to_string().contains("not found") {
                StatusCode::NOT_FOUND
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            };
            
            (
                status_code,
                Json(ApiErrorResponse {
                    error: "user_update_failed".to_string(),
                    message: e.to_string(),
                    details: None,
                }),
            )
        })?;
    
    // Get updated user data
    let updated_user = firebase_user_service
        .get_user_by_uid(&firebase_uid)
        .await
        .map_err(|e| {
            tracing::error!("Failed to get updated user data: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiErrorResponse {
                    error: "user_fetch_failed".to_string(),
                    message: "User updated but failed to retrieve updated data".to_string(),
                    details: Some(e.to_string()),
                }),
            )
        })?;
    
    tracing::info!("Successfully updated Firebase user: {}", firebase_uid);
    let user_response = convert_firebase_user_to_response(&updated_user, &firebase_user_service).await;
    Ok(Json(user_response))
}

/// Delete user
/// DELETE /admin/users/:uid
pub async fn delete_user(
    State(_state): State<AppState>,
    Path(firebase_uid): Path<String>,
) -> Result<StatusCode, (StatusCode, Json<ApiErrorResponse>)> {
    tracing::info!("Admin: Deleting user: {}", firebase_uid);
    
    let firebase_user_service = FirebaseUserService::new()
        .await
        .map_err(|e| {
            tracing::error!("Failed to create Firebase user service: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiErrorResponse {
                    error: "service_error".to_string(),
                    message: "Failed to initialize user service".to_string(),
                    details: Some(e.to_string()),
                }),
            )
        })?;
    
    // Delete user from Firebase
    firebase_user_service
        .delete_user(&firebase_uid)
        .await
        .map_err(|e| {
            tracing::error!("Failed to delete Firebase user {}: {}", firebase_uid, e);
            
            let status_code = if e.to_string().contains("not found") {
                StatusCode::NOT_FOUND
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            };
            
            (
                status_code,
                Json(ApiErrorResponse {
                    error: "user_deletion_failed".to_string(),
                    message: e.to_string(),
                    details: None,
                }),
            )
        })?;
    
    tracing::info!("Successfully deleted Firebase user: {}", firebase_uid);
    Ok(StatusCode::NO_CONTENT)
}

/// List users with filtering
/// GET /admin/users
pub async fn list_users(
    State(_state): State<AppState>,
    Query(query): Query<UserListQuery>,
) -> Result<Json<UserListResponse>, (StatusCode, Json<ApiErrorResponse>)> {
    tracing::info!("Admin: Listing users with filters: {:?}", query);
    
    let firebase_user_service = FirebaseUserService::new()
        .await
        .map_err(|e| {
            tracing::error!("Failed to create Firebase user service: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiErrorResponse {
                    error: "service_error".to_string(),
                    message: "Failed to initialize user service".to_string(),
                    details: Some(e.to_string()),
                }),
            )
        })?;
    
    // Create user list filters
    let filters = UserListFilters {
        max_results: query.page_size.or(Some(50)), // Default to 50 users per page
        page_token: query.page_token,
        role_filter: query.role_filter,
        email_domain_filter: query.email_domain,
    };
    
    // Get users from Firebase
    let (firebase_users, next_page_token) = firebase_user_service
        .list_users(filters)
        .await
        .map_err(|e| {
            tracing::error!("Failed to list Firebase users: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiErrorResponse {
                    error: "user_list_failed".to_string(),
                    message: e.to_string(),
                    details: None,
                }),
            )
        })?;
    
    // Convert to API response format
    let mut user_responses = Vec::new();
    for firebase_user in firebase_users {
        let user_response = convert_firebase_user_to_response(&firebase_user, &firebase_user_service).await;
        user_responses.push(user_response);
    }
    
    // Apply search filter if specified (client-side filtering)
    if let Some(search_term) = query.search {
        let search_lower = search_term.to_lowercase();
        user_responses = user_responses.into_iter()
            .filter(|user| {
                user.email.as_ref().map(|e| e.to_lowercase().contains(&search_lower)).unwrap_or(false) ||
                user.display_name.as_ref().map(|n| n.to_lowercase().contains(&search_lower)).unwrap_or(false) ||
                user.uid.to_lowercase().contains(&search_lower)
            })
            .collect();
    }
    
    tracing::info!("Successfully listed {} Firebase users", user_responses.len());
    
    Ok(Json(UserListResponse {
        users: user_responses,
        next_page_token,
        total_count: None, // Firebase doesn't provide total count
    }))
}

/// Set user role
/// POST /admin/users/:uid/role
pub async fn set_user_role(
    State(_state): State<AppState>,
    Path(firebase_uid): Path<String>,
    Json(role_request): Json<HashMap<String, String>>,
) -> Result<Json<UserResponse>, (StatusCode, Json<ApiErrorResponse>)> {
    let role = role_request.get("role")
        .ok_or_else(|| {
            (
                StatusCode::BAD_REQUEST,
                Json(ApiErrorResponse {
                    error: "missing_role".to_string(),
                    message: "Role is required".to_string(),
                    details: None,
                }),
            )
        })?;
    
    tracing::info!("Admin: Setting role '{}' for user: {}", role, firebase_uid);
    
    let firebase_user_service = FirebaseUserService::new()
        .await
        .map_err(|e| {
            tracing::error!("Failed to create Firebase user service: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiErrorResponse {
                    error: "service_error".to_string(),
                    message: "Failed to initialize user service".to_string(),
                    details: Some(e.to_string()),
                }),
            )
        })?;
    
    // Set user role
    firebase_user_service
        .set_user_role(&firebase_uid, role)
        .await
        .map_err(|e| {
            tracing::error!("Failed to set role for user {}: {}", firebase_uid, e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiErrorResponse {
                    error: "role_update_failed".to_string(),
                    message: e.to_string(),
                    details: None,
                }),
            )
        })?;
    
    // Get updated user data
    let updated_user = firebase_user_service
        .get_user_by_uid(&firebase_uid)
        .await
        .map_err(|e| {
            tracing::error!("Failed to get updated user data: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiErrorResponse {
                    error: "user_fetch_failed".to_string(),
                    message: "Role updated but failed to retrieve updated data".to_string(),
                    details: Some(e.to_string()),
                }),
            )
        })?;
    
    tracing::info!("Successfully set role '{}' for user: {}", role, firebase_uid);
    let user_response = convert_firebase_user_to_response(&updated_user, &firebase_user_service).await;
    Ok(Json(user_response))
}

// Helper functions

/// Convert Firebase user to API response format
async fn convert_firebase_user_to_response(
    firebase_user: &FirebaseUser,
    firebase_user_service: &FirebaseUserService,
) -> UserResponse {
    // Get role from custom claims or default
    let role = firebase_user.custom_claims.get("role")
        .and_then(|r| r.as_str())
        .unwrap_or("user-basic-001")
        .to_string();
    
    // Generate permissions based on role
    // Get admin access (validate_admin_access returns bool)
    let has_admin_access = firebase_user_service
        .validate_admin_access(&firebase_user.uid)
        .await
        .unwrap_or(false);
        
    // Generate permissions based on role and admin access
    let permissions = if has_admin_access {
        vec![
            "read:profile".to_string(),
            "update:profile".to_string(),
            "admin:users".to_string(),
            "create:users".to_string(),
            "update:users".to_string(),
            "delete:users".to_string(),
        ]
    } else {
        vec!["read:profile".to_string(), "update:profile".to_string()]
    };
    
    let provider_data = firebase_user.provider_data.iter().map(|provider| {
        UserProviderResponse {
            provider_id: provider.provider_id.clone(),
            uid: provider.uid.clone(),
            email: provider.email.clone(),
            display_name: provider.display_name.clone(),
            photo_url: provider.photo_url.clone(),
        }
    }).collect();
    
    UserResponse {
        uid: firebase_user.uid.clone(),
        email: firebase_user.email.clone(),
        email_verified: firebase_user.email_verified,
        display_name: firebase_user.display_name.clone(),
        photo_url: firebase_user.photo_url.clone(),
        phone_number: firebase_user.phone_number.clone(),
        disabled: firebase_user.disabled,
        role,
        permissions,
        provider_data,
        created_at: firebase_user.created_at.to_rfc3339(),
        last_login_at: firebase_user.last_login_at.map(|dt| dt.to_rfc3339()),
    }
}