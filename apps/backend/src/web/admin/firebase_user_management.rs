use std::collections::HashMap;
use chrono::{DateTime, Utc};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};

use crate::web::auth::routes::AppState;
use crate::domain::shared_kernel::services::FirebaseUserService;
use crate::infrastructure::adapters::services::firebase::FirebaseUser;

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
    State(state): State<AppState>,
    Json(request): Json<AdminCreateUserRequest>,
) -> Result<Json<CreateUserResponse>, (StatusCode, Json<ApiErrorResponse>)> {
    tracing::info!("🏗️ Admin: Creating user with DDD architecture - email: {:?}", request.email);
    
    // Get email from request (required)
    let email = request.email.clone().unwrap_or_else(|| {
        tracing::warn!("No email provided in user creation request");
        format!("user{}@example.com", chrono::Utc::now().timestamp())
    });
    
    // Create Firebase UID (simulate Firebase user creation)
    let firebase_uid = format!("firebase_{}", uuid::Uuid::new_v4());
    
    // Use DDD architecture for user creation
    let user_service = state.ddd_container.user_application_service();
    
    // Create DDD command
    let mut command = crate::application::user_management::CreateUserCommand::new(
        email.clone(), 
        firebase_uid.clone()
    );
    
    // Add initial permissions if role is provided
    if let Some(role) = &request.role {
        let permissions = match role.as_str() {
            "admin" => vec!["admin:users:manage".to_string(), "epsx:analytics:view".to_string()],
            "premium" => vec!["epsx:analytics:view".to_string(), "epsx:premium:access".to_string()],
            _ => vec!["epsx:basic:access".to_string()],
        };
        command = command.with_permissions(permissions);
    }
    
    // Set email as verified if requested
    if let Some(verified) = request.email_verified {
        command = command.with_email_verified(verified);
    }
    
    // Execute command through DDD application service
    match user_service.create_user(command).await {
        Ok(response) => {
            tracing::info!("✅ User created successfully with DDD: {}", response.user_id.to_string());
            
            Ok(Json(CreateUserResponse {
                uid: firebase_uid,
                message: format!("User created successfully: {}", email),
            }))
        }
        Err(e) => {
            tracing::error!("❌ Failed to create user with DDD: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiErrorResponse {
                    error: "user_creation_failed".to_string(),
                    message: "Failed to create user".to_string(),
                    details: Some(e.to_string()),
                }),
            ))
        }
    }
}

/// Get user by Firebase UID
/// GET /admin/users/:uid
pub async fn get_user(
    State(state): State<AppState>,
    Path(firebase_uid): Path<String>,
) -> Result<Json<UserResponse>, (StatusCode, Json<ApiErrorResponse>)> {
    tracing::info!("🔍 Admin: Getting user by UID with DDD architecture: {}", firebase_uid);
    
    // Use DDD architecture for user retrieval
    let user_query_service = state.ddd_container.user_query_service();
    
    // Create query to get user by Firebase UID
    let query = crate::application::user_management::GetUserByFirebaseUidQuery::new(
        crate::domain::user_management::value_objects::FirebaseUid::new(firebase_uid.clone())
            .map_err(|e| {
                tracing::error!("Invalid Firebase UID format: {}", e);
                (
                    StatusCode::BAD_REQUEST,
                    Json(ApiErrorResponse {
                        error: "invalid_uid".to_string(),
                        message: "Invalid Firebase UID format".to_string(),
                        details: Some(e.to_string()),
                    }),
                )
            })?
    );
    
    // Execute query through DDD application service
    match user_query_service.get_user_by_firebase_uid(query).await {
        Ok(user_response) => {
            tracing::info!("✅ User retrieved successfully with DDD: {}", firebase_uid);
            
            // Convert domain User to API response format
            let user_response = UserResponse {
                uid: firebase_uid,
                email: Some(user_response.email.to_string()),
                email_verified: user_response.email_verified,
                display_name: Some("User".to_string()), // TODO: Add display name to domain
                photo_url: None,
                phone_number: None,
                disabled: !user_response.is_active,
                role: if user_response.permissions.iter().any(|p| p.as_str().contains("admin:")) {
                    "admin".to_string()
                } else if user_response.permissions.iter().any(|p| p.as_str().contains("premium:")) {
                    "premium".to_string()
                } else {
                    "user-basic-001".to_string()
                },
                permissions: user_response.permissions.iter().map(|p| p.as_str().to_string()).collect(),
                provider_data: vec![], // TODO: Add provider data to domain if needed
                created_at: chrono::Utc::now().to_rfc3339(), // TODO: Add created_at to domain
                last_login_at: None, // TODO: Add last_login_at to domain if needed
            };
            
            Ok(Json(user_response))
        }
        Err(e) => {
            tracing::error!("❌ Failed to retrieve user with DDD: {}", e);
            
            let status_code = if e.to_string().contains("not found") {
                StatusCode::NOT_FOUND
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            };
            
            Err((
                status_code,
                Json(ApiErrorResponse {
                    error: "user_not_found".to_string(),
                    message: "User not found".to_string(),
                    details: Some(e.to_string()),
                }),
            ))
        }
    }
}

/// Update user
/// PUT /admin/users/:uid
pub async fn update_user(
    State(state): State<AppState>,
    Path(firebase_uid): Path<String>,
    Json(request): Json<AdminUpdateUserRequest>,
) -> Result<Json<UserResponse>, (StatusCode, Json<ApiErrorResponse>)> {
    tracing::info!("✏️ Admin: Updating user with DDD architecture: {}", firebase_uid);
    
    // Use DDD architecture for user updates
    let user_service = state.ddd_container.user_application_service();
    
    // Create Firebase UID value object
    let firebase_uid_vo = crate::domain::user_management::value_objects::FirebaseUid::new(firebase_uid.clone())
        .map_err(|e| {
            tracing::error!("Invalid Firebase UID format: {}", e);
            (
                StatusCode::BAD_REQUEST,
                Json(ApiErrorResponse {
                    error: "invalid_uid".to_string(),
                    message: "Invalid Firebase UID format".to_string(),
                    details: Some(e.to_string()),
                }),
            )
        })?;
    
    // Create DDD update command
    let mut command = crate::application::user_management::UpdateUserCommand::new(firebase_uid_vo);
    
    // Add email if provided
    if let Some(email) = request.email {
        let email_vo = crate::domain::user_management::value_objects::Email::new(email)
            .map_err(|e| {
                tracing::error!("Invalid email format: {}", e);
                (
                    StatusCode::BAD_REQUEST,
                    Json(ApiErrorResponse {
                        error: "invalid_email".to_string(),
                        message: "Invalid email format".to_string(),
                        details: Some(e.to_string()),
                    }),
                )
            })?;
        command = command.with_email(email_vo);
    }
    
    // Add role permissions if provided
    if let Some(role) = &request.role {
        let permissions = match role.as_str() {
            "admin" => vec!["admin:users:manage".to_string(), "epsx:analytics:view".to_string()],
            "premium" => vec!["epsx:analytics:view".to_string(), "epsx:premium:access".to_string()],
            _ => vec!["epsx:basic:access".to_string()],
        };
        command = command.with_permissions(permissions);
    }
    
    // Add active status (inverse of disabled)
    if let Some(disabled) = request.disabled {
        command = command.with_active_status(!disabled);
    }
    
    // Execute command through DDD application service
    match user_service.update_user(command).await {
        Ok(response) => {
            tracing::info!("✅ User updated successfully with DDD: {}", firebase_uid);
            
            // Create response with updated data
            let user_response = UserResponse {
                uid: firebase_uid,
                email: Some(response.email.to_string()),
                email_verified: response.email_verified,
                display_name: Some("User".to_string()), // TODO: Add display name to domain
                photo_url: None,
                phone_number: None,
                disabled: !response.is_active,
                role: if response.permissions.iter().any(|p| p.as_str().contains("admin:")) {
                    "admin".to_string()
                } else if response.permissions.iter().any(|p| p.as_str().contains("premium:")) {
                    "premium".to_string()
                } else {
                    "user-basic-001".to_string()
                },
                permissions: response.permissions.iter().map(|p| p.as_str().to_string()).collect(),
                provider_data: vec![], // TODO: Add provider data to domain if needed
                created_at: chrono::Utc::now().to_rfc3339(), // TODO: Add created_at to domain
                last_login_at: None, // TODO: Add last_login_at to domain if needed
            };
            
            Ok(Json(user_response))
        }
        Err(e) => {
            tracing::error!("❌ Failed to update user with DDD: {}", e);
            
            let status_code = if e.to_string().contains("not found") {
                StatusCode::NOT_FOUND
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            };
            
            Err((
                status_code,
                Json(ApiErrorResponse {
                    error: "user_update_failed".to_string(),
                    message: "Failed to update user".to_string(),
                    details: Some(e.to_string()),
                }),
            ))
        }
    }
}

/// Delete user
/// DELETE /admin/users/:uid
pub async fn delete_user(
    State(state): State<AppState>,
    Path(firebase_uid): Path<String>,
) -> Result<StatusCode, (StatusCode, Json<ApiErrorResponse>)> {
    tracing::info!("🗑️ Admin: Deleting user with DDD architecture: {}", firebase_uid);
    
    // Use DDD architecture for user deletion
    let user_service = state.ddd_container.user_application_service();
    
    // Create Firebase UID value object
    let firebase_uid_vo = crate::domain::user_management::value_objects::FirebaseUid::new(firebase_uid.clone())
        .map_err(|e| {
            tracing::error!("Invalid Firebase UID format: {}", e);
            (
                StatusCode::BAD_REQUEST,
                Json(ApiErrorResponse {
                    error: "invalid_uid".to_string(),
                    message: "Invalid Firebase UID format".to_string(),
                    details: Some(e.to_string()),
                }),
            )
        })?;
    
    // Create DDD delete command
    let command = crate::application::user_management::DeleteUserCommand::new(firebase_uid_vo);
    
    // Execute command through DDD application service
    match user_service.delete_user(command).await {
        Ok(_) => {
            tracing::info!("✅ User deleted successfully with DDD: {}", firebase_uid);
            Ok(StatusCode::NO_CONTENT)
        }
        Err(e) => {
            tracing::error!("❌ Failed to delete user with DDD: {}", e);
            
            let status_code = if e.to_string().contains("not found") {
                StatusCode::NOT_FOUND
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            };
            
            Err((
                status_code,
                Json(ApiErrorResponse {
                    error: "user_deletion_failed".to_string(),
                    message: "Failed to delete user".to_string(),
                    details: Some(e.to_string()),
                }),
            ))
        }
    }
}

/// List users with filtering
/// GET /admin/users
pub async fn list_users(
    State(state): State<AppState>,
    Query(query): Query<UserListQuery>,
) -> Result<Json<UserListResponse>, (StatusCode, Json<ApiErrorResponse>)> {
    tracing::info!("📋 Admin: Listing users with DDD architecture - filters: {:?}", query);
    
    // Use DDD architecture for user listing
    let user_query_service = state.ddd_container.user_query_service();
    
    // Create DDD list users query
    let list_query = crate::application::user_management::ListUsersQuery::new()
        .with_limit(query.page_size.unwrap_or(50) as usize)
        .with_offset(0); // TODO: Implement proper pagination with page tokens
    
    // Apply role filter if specified
    let list_query = if let Some(role_filter) = &query.role_filter {
        let permissions = match role_filter.as_str() {
            "admin" => vec!["admin:users:manage".to_string()],
            "premium" => vec!["epsx:premium:access".to_string()],
            _ => vec!["epsx:basic:access".to_string()],
        };
        list_query.with_permission_filter(permissions)
    } else {
        list_query
    };
    
    // Apply email domain filter if specified
    let list_query = if let Some(email_domain) = &query.email_domain {
        list_query.with_email_domain_filter(email_domain.clone())
    } else {
        list_query
    };
    
    // Execute query through DDD application service
    match user_query_service.list_users(list_query).await {
        Ok(users_response) => {
            tracing::info!("✅ Users listed successfully with DDD: {} users", users_response.users.len());
            
            // Convert domain users to API response format
            let mut user_responses = Vec::new();
            for domain_user in users_response.users {
                let user_response = UserResponse {
                    uid: domain_user.firebase_uid.to_string(),
                    email: Some(domain_user.email.to_string()),
                    email_verified: domain_user.email_verified,
                    display_name: Some("User".to_string()), // TODO: Add display name to domain
                    photo_url: None,
                    phone_number: None,
                    disabled: !domain_user.is_active,
                    role: if domain_user.permissions.iter().any(|p| p.as_str().contains("admin:")) {
                        "admin".to_string()
                    } else if domain_user.permissions.iter().any(|p| p.as_str().contains("premium:")) {
                        "premium".to_string()
                    } else {
                        "user-basic-001".to_string()
                    },
                    permissions: domain_user.permissions.iter().map(|p| p.as_str().to_string()).collect(),
                    provider_data: vec![], // TODO: Add provider data to domain if needed
                    created_at: chrono::Utc::now().to_rfc3339(), // TODO: Add created_at to domain
                    last_login_at: None, // TODO: Add last_login_at to domain if needed
                };
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
            
            Ok(Json(UserListResponse {
                users: user_responses,
                next_page_token: query.page_token, // TODO: Implement proper pagination tokens with DDD
                total_count: Some(users_response.total_count as u32),
            }))
        }
        Err(e) => {
            tracing::error!("❌ Failed to list users with DDD: {}", e);
            
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiErrorResponse {
                    error: "user_list_failed".to_string(),
                    message: "Failed to list users".to_string(),
                    details: Some(e.to_string()),
                }),
            ))
        }
    }
}

/// Set user role
/// POST /admin/users/:uid/role
pub async fn set_user_role(
    State(state): State<AppState>,
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
    
    tracing::info!("🎭 Admin: Setting role '{}' for user with DDD architecture: {}", role, firebase_uid);
    
    // Use DDD architecture for role updates
    let user_service = state.ddd_container.user_application_service();
    
    // Create Firebase UID value object
    let firebase_uid_vo = crate::domain::user_management::value_objects::FirebaseUid::new(firebase_uid.clone())
        .map_err(|e| {
            tracing::error!("Invalid Firebase UID format: {}", e);
            (
                StatusCode::BAD_REQUEST,
                Json(ApiErrorResponse {
                    error: "invalid_uid".to_string(),
                    message: "Invalid Firebase UID format".to_string(),
                    details: Some(e.to_string()),
                }),
            )
        })?;
    
    // Map role to permissions
    let permissions = match role.as_str() {
        "admin" => vec!["admin:users:manage".to_string(), "epsx:analytics:view".to_string()],
        "premium" => vec!["epsx:analytics:view".to_string(), "epsx:premium:access".to_string()],
        _ => vec!["epsx:basic:access".to_string()],
    };
    
    // Create DDD update command for role change
    let command = crate::application::user_management::UpdateUserCommand::new(firebase_uid_vo)
        .with_permissions(permissions);
    
    // Execute command through DDD application service
    match user_service.update_user(command).await {
        Ok(response) => {
            tracing::info!("✅ Role '{}' set successfully with DDD for user: {}", role, firebase_uid);
            
            // Create response with updated data
            let user_response = UserResponse {
                uid: firebase_uid,
                email: Some(response.email.to_string()),
                email_verified: response.email_verified,
                display_name: Some("User".to_string()), // TODO: Add display name to domain
                photo_url: None,
                phone_number: None,
                disabled: !response.is_active,
                role: if response.permissions.iter().any(|p| p.as_str().contains("admin:")) {
                    "admin".to_string()
                } else if response.permissions.iter().any(|p| p.as_str().contains("premium:")) {
                    "premium".to_string()
                } else {
                    "user-basic-001".to_string()
                },
                permissions: response.permissions.iter().map(|p| p.as_str().to_string()).collect(),
                provider_data: vec![], // TODO: Add provider data to domain if needed
                created_at: chrono::Utc::now().to_rfc3339(), // TODO: Add created_at to domain
                last_login_at: None, // TODO: Add last_login_at to domain if needed
            };
            
            Ok(Json(user_response))
        }
        Err(e) => {
            tracing::error!("❌ Failed to set role with DDD: {}", e);
            
            let status_code = if e.to_string().contains("not found") {
                StatusCode::NOT_FOUND
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            };
            
            Err((
                status_code,
                Json(ApiErrorResponse {
                    error: "role_update_failed".to_string(),
                    message: "Failed to update user role".to_string(),
                    details: Some(e.to_string()),
                }),
            ))
        }
    }
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
    // Check admin access (placeholder - Firebase admin validation would be implemented here)
    let has_admin_access = false; // Default to false for safety - would check Firebase custom claims in production
        
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
    
    // FirebaseUser doesn't have provider_data field, provide default
    let provider_data: Vec<UserProviderResponse> = vec![
        UserProviderResponse {
            provider_id: firebase_user.provider_id.clone(),
            uid: firebase_user.uid.clone(),
            email: firebase_user.email.clone(),
            display_name: firebase_user.display_name.clone(),
            photo_url: firebase_user.photo_url.clone(),
        }
    ];
    
    UserResponse {
        uid: firebase_user.uid.clone(),
        email: firebase_user.email.clone(),
        email_verified: firebase_user.email_verified,
        display_name: firebase_user.display_name.clone(),
        photo_url: firebase_user.photo_url.clone(),
        phone_number: None, // FirebaseUser doesn't have phone_number field
        disabled: false, // FirebaseUser doesn't have disabled field, default to false
        role,
        permissions,
        provider_data,
        created_at: chrono::Utc::now().to_rfc3339(), // FirebaseUser doesn't have created_at field, use current time
        last_login_at: None, // FirebaseUser doesn't have last_login_at field
    }
}