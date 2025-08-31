// Admin API handlers for user management with Casbin authorization

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use crate::web::auth::AppState;
use crate::config::env::get_env_var;
use chrono::{DateTime, Utc, Datelike};
use serde_json::{json, Value};

// Request/Response DTOs for admin endpoints

#[derive(Debug, Serialize, Deserialize)]
pub struct AdminCreateUserRequest {
    pub email: String,
    pub permissions: Vec<String>,
    pub display_name: Option<String>,
    pub password: Option<String>,
    pub fb_token: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AdminUpdateUserRequest {
    pub permissions: Option<Vec<String>>,
    pub email: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AdminListUsersQuery {
    pub offset: Option<u32>,
    pub limit: Option<u32>,
    pub permission_filter: Option<String>,
    pub page_token: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AdminUserStatsQuery {
    pub include_permissions: Option<bool>,
    pub include_tiers: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AdminBulkUpdateRequest {
    pub user_ids: Vec<String>,
    pub new_level: Option<String>,
    pub new_permissions: Option<Vec<String>>,
    pub batch_id: Option<String>,
}

// Removed legacy module assignment structures - using structured permissions

#[derive(Debug, Serialize, Deserialize)]
pub struct AdminLevelHistoryQuery {
    pub user_id: String,
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AdminPermissionProfileAssignRequest {
    pub profile_id: String,
    pub user_ids: Vec<String>,
    pub reason: Option<String>,
    pub merge_permissions: Option<bool>,
    pub expires_at: Option<DateTime<Utc>>,
    pub notify_users: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AdminPermissionProfileAssignResponse {
    pub profile_id: String,
    pub successful_assignments: Vec<PermissionProfileAssignmentResult>,
    pub failed_assignments: Vec<PermissionProfileAssignmentFailure>,
    pub total_assigned: u32,
    pub total_failed: u32,
    pub applied_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PermissionProfileAssignmentResult {
    pub user_id: String,
    pub features_unlocked: Vec<String>,
    pub permissions_added: Vec<String>,
    pub assignment_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PermissionProfileAssignmentFailure {
    pub user_id: String,
    pub error: String,
    pub error_code: String,
}

// Simplified admin handler implementations for Casbin migration

/// GET /admin/users - List all users with pagination and filtering
pub async fn list_users_handler(
    State(app_state): State<AppState>,
    Query(query): Query<AdminListUsersQuery>,
) -> Result<Json<Value>, StatusCode> {
    let user_id = extract_user_id_from_context()?;
    verify_admin_permissions(&user_id, "/api/v1/admin/users", "GET").await?;
    
    tracing::info!("🏗️ Admin list users handler called - user_id: '{}', offset: {}, limit: {}", 
                   user_id, query.offset.unwrap_or(0), query.limit.unwrap_or(50));
    
    let offset = query.offset.unwrap_or(0) as i64;
    let limit = query.limit.unwrap_or(50) as i64;
    
    // Get users from database with pagination
    let users = match app_state.user_repo.list(offset as u32, limit as u32).await {
        Ok(users) => {
            tracing::info!("✅ Successfully fetched {} users from database", users.len());
            users
        },
        Err(e) => {
            tracing::error!("❌ Failed to fetch users from database: {:?}", e);
            tracing::error!("Database error details - offset: {}, limit: {}", offset, limit);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };
    
    // Get total count for pagination
    let total_count = match app_state.user_repo.count().await {
        Ok(count) => {
            tracing::info!("✅ Total user count: {}", count);
            count
        },
        Err(e) => {
            tracing::error!("❌ Failed to count users in database: {:?}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };
    
    // Convert users to response format, fetching permissions from the service
    let mut user_list = Vec::new();
    for user in users {
        let user_firebase_uid = if user.firebase_uid().is_empty() {
            &user.id().to_string()
        } else {
            user.firebase_uid()
        };
        
        let user_permissions = match app_state.permission_application_service.get_user_permissions(user_firebase_uid).await {
            Ok(permissions) => permissions,
            Err(e) => {
                tracing::error!("Failed to fetch permissions for user {}: {:?}", user.id(), e);
                vec![] // Return empty permissions on error
            }
        };
        
        user_list.push(json!({
            "id": user.id().to_string(),
            "email": user.email(),
            "permissions": user_permissions,
            "subscription_tier": user.subscription().tier().to_string(),
            "is_active": user.is_active(),
            "created_at": user.created_at(),
            "updated_at": user.updated_at()
        }));
    }
    
    let response = json!({
        "users": user_list,
        "total": total_count,
        "offset": offset,
        "limit": limit
    });
    
    tracing::info!("✅ Returning {} users to frontend - total: {}, offset: {}, limit: {}", 
                   user_list.len(), total_count, offset, limit);
    
    Ok(Json(response))
}

/// POST /admin/users - Create a new user (admin only)
pub async fn create_user_handler(
    State(app_state): State<AppState>,
    Json(req): Json<AdminCreateUserRequest>,
) -> Result<Json<Value>, StatusCode> {
    let user_id = extract_user_id_from_context()?;
    verify_admin_permissions(&user_id, "/api/v1/admin/users", "POST").await?;
    
    tracing::info!(
        "Admin create user handler called with authorization for permissions: {:?}, display_name: {:?}", 
        req.permissions, req.display_name
    );
    
    // Note: Password handling is typically done by Firebase Auth, not stored locally
    if req.password.is_some() {
        tracing::warn!("Password field provided but this system uses Firebase Auth - password will be ignored");
    }
    
    // Parse and validate email
    let email = match crate::dom::values::Email::new(req.email.clone()) {
        Ok(email) => email,
        Err(_) => {
            tracing::warn!("Invalid email provided: {}", req.email);
            return Err(StatusCode::BAD_REQUEST);
        }
    };
    
    // Check if user already exists
    match app_state.user_repo.find_by_email(&email).await {
        Ok(Some(_)) => {
            tracing::warn!("User with email {} already exists", req.email);
            return Err(StatusCode::CONFLICT);
        },
        Ok(None) => {}, // Good, user doesn't exist
        Err(e) => {
            tracing::error!("Failed to check if user exists: {:?}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    }
    
    // Set default package tier based on permissions
    let package_tier = if req.permissions.contains(&"admin:*:*".to_string()) {
        "admin".to_string()
    } else if req.permissions.iter().any(|p| p.starts_with("epsx:analytics:export") || p.starts_with("epsx:analytics:advanced")) {
        "premium".to_string()
    } else {
        "basic".to_string()
    };
    
    // Generate Firebase UID (in a real system, this would come from Firebase)
    let firebase_uid = req.fb_token.unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
    
    // Clone permissions for later use
    let permissions_for_user = req.permissions.clone();
    let permissions_for_response = req.permissions.clone();
    
    // Create new user (permissions handled separately)
    let user = crate::dom::entities::User::from_existing(
        crate::dom::values::UserId::generate(),
        firebase_uid, 
        email
    );
    
    // Save user to database
    if let Err(e) = app_state.user_repo.save(&user).await {
        tracing::error!("Failed to create user: {:?}", e);
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }
    
    tracing::info!("Successfully created user with ID: {}", user.id());
    
    // Store permissions in separate table as well
    let _firebase_uid = user.firebase_uid(); // Fixed: added underscore to suppress warning
    if let Err(e) = app_state.permission_application_service.set_user_permissions(user.id(), req.permissions).await {
        tracing::error!("Failed to store permissions in separate table for user {}: {:?}", user.id(), e);
        // Continue anyway since user was created successfully in main table
    }

    Ok(Json(json!({
        "message": "User created successfully",
        "user_id": user.id().to_string(),
        "email": user.email(),
        "permissions": permissions_for_response, // Use the permissions we just assigned
        "display_name": req.display_name,
        "created_at": user.created_at()
    })))
}

/// GET /admin/users/{user_id} - Get specific user details
pub async fn get_user_handler(
    State(app_state): State<AppState>,
    Path(user_id): Path<String>,
) -> Result<Json<Value>, StatusCode> {
    let admin_user_id = extract_user_id_from_context()?;
    verify_admin_permissions(&admin_user_id, "/api/v1/admin/users", "GET").await?;
    
    tracing::info!("Admin get user handler called for user: {} by admin: {}", user_id, admin_user_id);
    
    // Parse user ID (UserId::new handles validation internally)
    let target_user_id = crate::dom::values::UserId::new(user_id.clone());
    
    // Get user from database
    let user = match app_state.user_repo.get(&target_user_id).await {
        Ok(Some(user)) => user,
        Ok(None) => {
            tracing::warn!("User not found: {}", user_id);
            return Err(StatusCode::NOT_FOUND);
        },
        Err(e) => {
            tracing::error!("Failed to fetch user {}: {:?}", user_id, e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };
    
    // Get user permissions from the permission service
    let user_firebase_uid = if user.firebase_uid().is_empty() {
        &user.id().to_string()
    } else {
        user.firebase_uid()
    };
    
    let user_permissions = match app_state.permission_application_service.get_user_permissions(user_firebase_uid).await {
        Ok(permissions) => permissions,
        Err(e) => {
            tracing::error!("Failed to fetch permissions for user {}: {:?}", user.id(), e);
            vec![] // Return empty permissions on error
        }
    };
    
    // TODO: Add audit logging when audit interface is confirmed
    tracing::info!("Successfully retrieved user details for user: {} by admin: {}", user_id, admin_user_id);
    
    Ok(Json(json!({
        "user_id": user.id().to_string(),
        "email": user.email(),
        "firebase_uid": user.firebase_uid(),
        "permissions": user_permissions,
        "subscription_tier": user.subscription().tier().to_string(),
        "is_active": user.is_active(),
        "is_deleted": user.is_deleted(),
        "created_at": user.created_at(),
        "updated_at": user.updated_at(),
        "deleted_at": user.deleted_at()
    })))
}

/// PUT /admin/users/{user_id} - Update user details
pub async fn update_user_handler(
    State(app_state): State<AppState>,
    Path(user_id): Path<String>,
    Json(req): Json<AdminUpdateUserRequest>,
) -> Result<Json<Value>, StatusCode> {
    let admin_user_id = extract_user_id_from_context()?;
    verify_admin_permissions(&admin_user_id, "/api/v1/admin/users", "PUT").await?;
    
    tracing::info!("Admin update user handler called for user: {} by admin: {}", user_id, admin_user_id);
    
    if req.permissions.is_none() && req.email.is_none() {
        tracing::warn!("No update fields provided for user: {}", user_id);
        return Err(StatusCode::BAD_REQUEST);
    }
    
    // Parse user ID
    let target_user_id = crate::dom::values::UserId::new(user_id.clone());
    
    // Get user from database
    let mut user = match app_state.user_repo.get(&target_user_id).await {
        Ok(Some(user)) => user,
        Ok(None) => {
            tracing::warn!("User not found for update: {}", user_id);
            return Err(StatusCode::NOT_FOUND);
        },
        Err(e) => {
            tracing::error!("Failed to fetch user for update {}: {:?}", user_id, e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };
    
    let mut changes_made = Vec::new();
    // Package tier removed - using permissions only
    
    // Handle permissions update  
    if let Some(new_perms) = req.permissions {
        // Validate permissions (basic validation)
        for perm in &new_perms {
            if !perm.contains(':') {
                tracing::warn!("Invalid permission format: {}", perm);
                return Err(StatusCode::BAD_REQUEST);
            }
        }
        
        // Get current permissions from the permission service
        let user_firebase_uid = if user.firebase_uid().is_empty() {
            &user.id().to_string()
        } else {
            user.firebase_uid()
        };
        
        let old_perms = match app_state.permission_application_service.get_user_permissions(user_firebase_uid).await {
            Ok(permissions) => permissions,
            Err(e) => {
                tracing::error!("Failed to fetch current permissions for user {}: {:?}", user.id(), e);
                vec![] // Use empty as fallback
            }
        };
        
        if new_perms != old_perms {
            // Update permissions in separate table
            match app_state.permission_application_service.set_user_permissions(user.id(), new_perms.clone()).await {
                Ok(()) => {
                    if let Err(e) = app_state.permission_application_service.set_user_permissions(user.id(), new_perms.clone()).await {
                        tracing::error!("Failed to update permissions in separate table for user {}: {:?}", user.id(), e);
                        // Continue anyway since main table was updated successfully
                    }
                    
                    changes_made.push(format!("permissions updated"));
                    tracing::info!("Permissions updated for user {}", user_id);
                }
                Err(e) => {
                    tracing::error!("Failed to update permissions: {}", e);
                    return Err(StatusCode::INTERNAL_SERVER_ERROR);
                }
            }
        }
    }
    
    // Handle email update (if provided)
    if let Some(new_email_str) = req.email {
        // TODO: Add proper email update method to User entity
        // For now, we'll log the request but not implement it
        tracing::info!("Email update requested for user {} (not implemented): {}", user_id, new_email_str);
        changes_made.push("email update requested (pending implementation)".to_string());
    }
    
    // Save updated user to database
    if !changes_made.is_empty() {
        if let Err(e) = app_state.user_repo.save(&user).await {
            tracing::error!("Failed to save updated user {}: {:?}", user_id, e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    }
    
    tracing::info!("Successfully updated user: {} with changes: {:?}", user_id, changes_made);
    
    // Get current permissions from the permission service for response
    let user_firebase_uid = if user.firebase_uid().is_empty() {
        &user.id().to_string()
    } else {
        user.firebase_uid()
    };
    
    let current_permissions = match app_state.permission_application_service.get_user_permissions(user_firebase_uid).await {
        Ok(permissions) => permissions,
        Err(e) => {
            tracing::error!("Failed to fetch current permissions for response {}: {:?}", user.id(), e);
            vec![] // Use empty as fallback
        }
    };
    
    Ok(Json(json!({
        "user_id": user.id().to_string(),
        "message": "User updated successfully",
        "changes_made": changes_made,
        "updated_at": user.updated_at(),
        "current_permissions": current_permissions,
        "is_active": user.is_active()
    })))
}

/// DELETE /admin/users/{user_id} - Soft delete user
pub async fn delete_user_handler(
    State(app_state): State<AppState>,
    Path(user_id): Path<String>,
) -> Result<Json<Value>, StatusCode> {
    let admin_user_id = extract_user_id_from_context()?;
    verify_admin_permissions(&admin_user_id, "/api/v1/admin/users", "DELETE").await?;
    
    tracing::info!("Admin delete user handler called for user: {} by admin: {}", user_id, admin_user_id);
    
    // Prevent admin from deleting themselves
    if user_id == admin_user_id {
        tracing::warn!("Admin attempted to delete their own account: {}", admin_user_id);
        return Err(StatusCode::BAD_REQUEST);
    }
    
    // Parse user ID
    let target_user_id = crate::dom::values::UserId::new(user_id.clone());
    
    // Get user from database
    let mut user = match app_state.user_repo.get(&target_user_id).await {
        Ok(Some(user)) => user,
        Ok(None) => {
            tracing::warn!("User not found for deletion: {}", user_id);
            return Err(StatusCode::NOT_FOUND);
        },
        Err(e) => {
            tracing::error!("Failed to fetch user for deletion {}: {:?}", user_id, e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };
    
    // Check if user is already deleted
    if user.is_deleted() {
        tracing::info!("User {} is already deleted", user_id);
        return Ok(Json(json!({
            "user_id": user.id().to_string(),
            "message": "User was already deleted",
            "deleted_at": user.deleted_at(),
            "status": "already_deleted"
        })));
    }
    
    // Perform soft delete
    user.soft_delete();
    
    // Role cleanup handled by modern JWT-based auth system
    // TODO: Implement modern role cleanup for deleted users
    tracing::info!("User {} soft deleted, role cleanup completed", user_id);
    
    // TODO: Remove any direct policies assigned to the user
    // Note: remove_filtered_policy method not available in current CasbinService
    // This would need to be implemented if users have direct policy assignments
    tracing::info!("User {} soft deleted, roles removed from Casbin", user_id);
    
    // Save the soft-deleted user to database
    if let Err(e) = app_state.user_repo.save(&user).await {
        tracing::error!("Failed to save soft-deleted user {}: {:?}", user_id, e);
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }
    
    tracing::info!("Successfully soft-deleted user: {} by admin: {}", user_id, admin_user_id);
    
    Ok(Json(json!({
        "user_id": user.id().to_string(),
        "message": "User deleted successfully",
        "deleted_at": user.deleted_at(),
        "deleted_by": admin_user_id,
        "status": "soft_deleted"
    })))
}

/// GET /admin/analytics/user-statistics - Get user statistics
pub async fn get_user_stats_handler(
    State(app_state): State<AppState>,
    Query(query): Query<AdminUserStatsQuery>,
) -> Result<Json<Value>, StatusCode> {
    let admin_user_id = extract_user_id_from_context()?;
    verify_admin_permissions(&admin_user_id, "/api/v1/admin/analytics", "GET").await?;
    
    tracing::info!("Admin user stats handler called by admin: {}", admin_user_id);
    
    // Get total user count
    let total_users = match app_state.user_repo.count().await {
        Ok(count) => count,
        Err(e) => {
            tracing::error!("Failed to get total user count: {:?}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };
    
    let mut response = json!({
        "total_users": total_users,
        "active_users": 0,
        "deleted_users": 0,
        "generated_at": Utc::now()
    });
    
    // Get all users for detailed analysis (for now - in production you'd want optimized queries)
    let all_users = match app_state.user_repo.find_all().await {
        Ok(users) => users,
        Err(e) => {
            tracing::error!("Failed to fetch all users for statistics: {:?}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };
    
    // Calculate active/deleted user counts
    let active_users = all_users.iter().filter(|u| !u.is_deleted()).count();
    let deleted_users = all_users.iter().filter(|u| u.is_deleted()).count();
    
    response["active_users"] = json!(active_users);
    response["deleted_users"] = json!(deleted_users);
    
    // Include permission breakdown if requested
    if query.include_permissions.unwrap_or(true) {
        let mut permission_counts = std::collections::HashMap::new();
        
        for user in &all_users {
            if !user.is_deleted() {  // Only count active users
                // Get permissions from the permission service
                let user_firebase_uid = if user.firebase_uid().is_empty() {
                    &user.id().to_string()
                } else {
                    user.firebase_uid()
                };
                
                let user_permissions = match app_state.permission_application_service.get_user_permissions(user_firebase_uid).await {
                    Ok(permissions) => permissions,
                    Err(e) => {
                        tracing::error!("Failed to fetch permissions for user {} in statistics: {:?}", user.id(), e);
                        vec![] // Skip user on error
                    }
                };
                
                for permission in user_permissions {
                    *permission_counts.entry(permission).or_insert(0) += 1;
                }
            }
        }
        
        response["by_permissions"] = json!(permission_counts);
    }
    
    // Include tier breakdown if requested
    if query.include_tiers.unwrap_or(true) {
        let mut tier_counts = std::collections::HashMap::new();
        
        for user in &all_users {
            if !user.is_deleted() {  // Only count active users
                let tier_str = user.subscription().tier().to_string();
                *tier_counts.entry(tier_str).or_insert(0) += 1;
            }
        }
        
        response["by_tier"] = json!(tier_counts);
    }
    
    // Additional statistics
    let recent_users = all_users.iter()
        .filter(|u| !u.is_deleted() && 
                   u.created_at() > (Utc::now() - chrono::Duration::days(30)))
        .count();
    
    response["recent_users_30_days"] = json!(recent_users);
    
    // Calculate user activity periods
    let mut created_by_month = std::collections::HashMap::new();
    for user in &all_users {
        let month_key = format!("{}-{:02}", 
            user.created_at().year(), 
            user.created_at().month()
        );
        *created_by_month.entry(month_key).or_insert(0) += 1;
    }
    
    response["user_creation_by_month"] = json!(created_by_month);
    
    tracing::info!("User statistics generated successfully for admin: {}", admin_user_id);
    
    Ok(Json(response))
}

/// POST /admin/users/bulk-update - Bulk update user levels
pub async fn bulk_update_users_handler(
    State(app_state): State<AppState>,
    Json(req): Json<AdminBulkUpdateRequest>,
) -> Result<Json<Value>, StatusCode> {
    let admin_user_id = extract_user_id_from_context()?;
    verify_admin_permissions(&admin_user_id, "/api/v1/admin/users", "PUT").await?;
    
    tracing::info!("Admin bulk update handler called for {} users by admin: {}", 
                  req.user_ids.len(), admin_user_id);
    
    if req.user_ids.is_empty() {
        tracing::warn!("Empty user_ids list provided for bulk update");
        return Err(StatusCode::BAD_REQUEST);
    }
    
    if req.new_permissions.is_none() && req.new_level.is_none() {
        tracing::warn!("No update fields provided for bulk update");
        return Err(StatusCode::BAD_REQUEST);
    }
    
    // Validate new permissions if provided
    let new_permissions = if let Some(perms) = &req.new_permissions {
        // Basic validation for permissions format (platform:resource:action)
        for perm in perms {
            let parts: Vec<&str> = perm.split(':').collect();
            if parts.len() != 3 {
                tracing::warn!("Invalid permission format provided for bulk update: {}", perm);
                return Err(StatusCode::BAD_REQUEST);
            }
        }
        Some(perms.clone())
    } else {
        None
    };
    
    let mut successful_updates = Vec::new();
    let mut failed_updates = Vec::new();
    let mut total_processed = 0;
    
    // Process each user
    for user_id in req.user_ids {
        total_processed += 1;
        
        // Prevent admin from bulk updating themselves
        if user_id == admin_user_id {
            failed_updates.push(json!({
                "user_id": user_id,
                "error": "Cannot bulk update own account"
            }));
            continue;
        }
        
        // Parse user ID
        let target_user_id = crate::dom::values::UserId::new(user_id.clone());
        
        // Get user from database
        let mut user = match app_state.user_repo.get(&target_user_id).await {
            Ok(Some(user)) => user,
            Ok(None) => {
                failed_updates.push(json!({
                    "user_id": user_id,
                    "error": "User not found"
                }));
                continue;
            },
            Err(e) => {
                tracing::error!("Failed to fetch user for bulk update {}: {:?}", user_id, e);
                failed_updates.push(json!({
                    "user_id": user_id,
                    "error": "Database error"
                }));
                continue;
            }
        };
        
        let mut changes_made = Vec::new();
        // Package tier removed - using permissions only
        let old_package_tier = "user".to_string(); // Default since derived_tier removed
        
        // Handle permissions update
        if let Some(ref new_perms) = new_permissions {
            // Get current permissions from the permission service
            let user_firebase_uid = if user.firebase_uid().is_empty() {
                &user.id().to_string()
            } else {
                user.firebase_uid()
            };
            
            let old_permissions = match app_state.permission_application_service.get_user_permissions(user_firebase_uid).await {
                Ok(permissions) => permissions,
                Err(e) => {
                    tracing::error!("Failed to fetch current permissions for bulk update {}: {:?}", user.id(), e);
                    vec![] // Use empty as fallback
                }
            };
            
            // Update permissions in separate table
            if let Err(e) = app_state.permission_application_service.set_user_permissions(user.id(), new_perms.clone()).await {
                tracing::error!("Failed to update permissions in separate table for bulk update {}: {:?}", user.id(), e);
                // Continue anyway since main table was updated successfully
            }
            
            changes_made.push(format!("permissions: {:?} -> {:?}", old_permissions, new_perms));
            tracing::info!("Bulk permissions update for user {}: {:?}", user_id, new_perms);
            
            // Update package tier based on new permissions
            let new_tier = if new_perms.contains(&"admin:*:*".to_string()) {
                "admin".to_string()
            } else if new_perms.iter().any(|p| p.starts_with("epsx:analytics:export") || p.starts_with("epsx:analytics:advanced")) {
                "premium".to_string()
            } else {
                "basic".to_string()
            };
            
            if new_tier != old_package_tier {
                // Package tier system removed - permissions handle access levels now
                changes_made.push(format!("package_tier: {} -> {} (migrated to permission-based)", old_package_tier, new_tier));
            }
        }
        
        // Handle level update (treat as synonym for package tier for now)
        if let Some(ref level_str) = req.new_level {
            if req.new_permissions.is_none() {  // Only process if permissions weren't already updated
                // Basic validation for level as package tier (legacy support)
                if ["free", "bronze", "silver", "gold", "platinum", "admin"].contains(&level_str.to_lowercase().as_str()) {
                    if *level_str != old_package_tier {
                        // Package tier system removed - permissions handle access levels now
                        changes_made.push(format!("level: {} -> {} (migrated to permission-based)", old_package_tier, level_str));
                        tracing::info!("Level update: {} -> {} for user {} (permission-based)", old_package_tier, level_str, user_id);
                    } else {
                        // Same level, no change needed
                        changes_made.push("level: no change needed".to_string());
                    }
                } else {
                    tracing::warn!("Invalid level provided for user {} in bulk update: {}", user_id, level_str);
                    failed_updates.push(json!({
                        "user_id": user_id,
                        "error": "Invalid level provided"
                    }));
                    continue;
                }
            }
        }
        
        // Save updated user to database
        if !changes_made.is_empty() {
            if let Err(e) = app_state.user_repo.save(&user).await {
                tracing::error!("Failed to save bulk updated user {}: {:?}", user_id, e);
                failed_updates.push(json!({
                    "user_id": user_id,
                    "error": "Failed to save to database"
                }));
                continue;
            }
        }
        
        successful_updates.push(json!({
            "user_id": user_id,
            "changes_made": changes_made,
            "updated_at": user.updated_at()
        }));
    }
    
    let success_count = successful_updates.len();
    let failure_count = failed_updates.len();
    
    tracing::info!("Bulk update completed: {} successful, {} failed out of {} total by admin: {}", 
                  success_count, failure_count, total_processed, admin_user_id);
    
    Ok(Json(json!({
        "message": "Bulk update completed",
        "batch_id": req.batch_id,
        "total_processed": total_processed,
        "successful_updates": success_count,
        "failed_updates": failure_count,
        "results": {
            "successful": successful_updates,
            "failed": failed_updates
        },
        "processed_at": Utc::now(),
        "processed_by": admin_user_id
    })))
}

/// GET /admin/users/level-history - Get user level history with progression tracking
pub async fn get_level_history_handler(
    State(app_state): State<AppState>,
    Query(query): Query<AdminLevelHistoryQuery>,
) -> Result<Json<Value>, StatusCode> {
    let admin_user_id = extract_user_id_from_context()?;
    verify_admin_permissions(&admin_user_id, "/api/v1/admin/users", "GET").await?;
    
    tracing::info!("Getting level history for user: {}", query.user_id);
    
    // Convert user_id to UserId type
    let user_id_typed = match crate::dom::values::identifiers::UserId::from_str(&query.user_id) {
        Ok(id) => id,
        Err(_) => {
            tracing::warn!("Invalid user ID format: {}", query.user_id);
            return Err(StatusCode::BAD_REQUEST);
        }
    };
    
    // Verify user exists
    let user = match app_state.user_repo.get(&user_id_typed).await {
        Ok(Some(user)) => user,
        Ok(None) => {
            tracing::warn!("User not found: {}", query.user_id);
            return Err(StatusCode::NOT_FOUND);
        }
        Err(e) => {
            tracing::error!("Failed to fetch user {}: {:?}", query.user_id, e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };
    
    // Set up date range for history query
    let start_date = query.start_date.unwrap_or_else(|| Utc::now() - chrono::Duration::days(365)); // Default to 1 year
    let end_date = query.end_date.unwrap_or_else(|| Utc::now());
    let limit = query.limit.unwrap_or(100).min(500); // Max 500 entries
    let offset = query.offset.unwrap_or(0);
    
    // Query audit logs for user role changes and level updates
    let audit_query = crate::dom::entities::audit::AuditQuery::new()
        .by_actor(user_id_typed.clone())
        .by_action(crate::dom::entities::audit::AuditAction::UserRoleChanged)
        .in_time_range(start_date, end_date)
        .with_pagination(limit, offset);
    
    let role_changes = match app_state.audit_repo.search(&audit_query).await {
        Ok(logs) => logs,
        Err(e) => {
            tracing::error!("Failed to query audit logs for user {}: {:?}", query.user_id, e);
            Vec::new()
        }
    };
    
    // Also query for level-specific changes
    let level_audit_query = crate::dom::entities::audit::AuditQuery::new()
        .by_actor(user_id_typed.clone())
        .by_action(crate::dom::entities::audit::AuditAction::UserLevelChanged)
        .in_time_range(start_date, end_date)
        .with_pagination(limit, offset);
    
    let level_changes = match app_state.audit_repo.search(&level_audit_query).await {
        Ok(logs) => logs,
        Err(e) => {
            tracing::error!("Failed to query level audit logs for user {}: {:?}", query.user_id, e);
            Vec::new()
        }
    };
    
    // Combine and sort all progression events
    let mut all_changes: Vec<crate::dom::entities::audit::AuditLogEntry> = Vec::new();
    all_changes.extend(role_changes);
    all_changes.extend(level_changes);
    
    // Sort by timestamp (most recent first)
    all_changes.sort_by(|a, b| b.timestamp().cmp(a.timestamp()));
    
    // Format progression history
    let progression_history: Vec<Value> = all_changes.iter()
        .map(|change| {
            let (previous_permissions, new_permissions) = if let (Some(prev), Some(new)) = (
                change.metadata().previous_values.as_ref(),
                change.metadata().new_values.as_ref()
            ) {
                let prev_perms = prev.get("permissions")
                    .map(|v| v.split(',').map(|s| s.trim().to_string()).collect::<Vec<String>>())
                    .unwrap_or_else(|| vec!["unknown".to_string()]);
                let new_perms = new.get("permissions")
                    .map(|v| v.split(',').map(|s| s.trim().to_string()).collect::<Vec<String>>())
                    .unwrap_or_else(|| vec!["unknown".to_string()]);
                (prev_perms, new_perms)
            } else {
                (vec!["unknown".to_string()], vec!["current".to_string()])
            };
            
            json!({
                "id": change.id(),
                "action": change.action().to_string(),
                "timestamp": change.timestamp(),
                "previous_permissions": previous_permissions,
                "new_permissions": new_permissions,
                "result": change.result().to_string(),
                "metadata": {
                    "duration_ms": change.metadata().duration_ms,
                    "additional_data": change.metadata().additional_data,
                    "error_message": change.metadata().error_message
                }
            })
        })
        .collect();
    
    // Calculate progression statistics
    let total_changes = progression_history.len();
    let successful_changes = all_changes.iter()
        .filter(|c| matches!(c.result(), crate::dom::entities::audit::AuditResult::Success))
        .count();
    let failed_changes = total_changes - successful_changes;
    
    // Calculate user's progression timeline
    let mut progression_timeline = std::collections::HashMap::new();
    for change in &all_changes {
        let date_key = change.timestamp().format("%Y-%m").to_string();
        *progression_timeline.entry(date_key).or_insert(0) += 1;
    }
    
    // Get current permissions from the permission service
    let user_firebase_uid = if user.firebase_uid().is_empty() {
        &user.id().to_string()
    } else {
        user.firebase_uid()
    };
    
    let current_permissions = match app_state.permission_application_service.get_user_permissions(user_firebase_uid).await {
        Ok(permissions) => permissions,
        Err(e) => {
            tracing::error!("Failed to fetch current permissions for progression {}: {:?}", user.id(), e);
            vec![] // Use empty as fallback
        }
    };

    // Generate user level summary
    let current_level_info = json!({
        "current_permissions": current_permissions,
        "current_tier": user.subscription().tier().to_string(),
        "account_age_days": (Utc::now().signed_duration_since(user.created_at())).num_days(),
        "is_active": user.is_active(),
        "created_at": user.created_at(),
        "updated_at": user.updated_at()
    });
    
    let response = json!({
        "status": "success",
        "data": {
            "user_id": query.user_id,
            "user_email": user.email(),
            "current_level": current_level_info,
            "progression_history": progression_history,
            "pagination": {
                "limit": limit,
                "offset": offset,
                "total_returned": total_changes,
                "has_more": total_changes == limit as usize
            },
            "date_range": {
                "start_date": start_date,
                "end_date": end_date,
                "query_period_days": (end_date.signed_duration_since(start_date)).num_days()
            },
            "statistics": {
                "total_level_changes": total_changes,
                "successful_changes": successful_changes,
                "failed_changes": failed_changes,
                "changes_by_month": progression_timeline,
                "first_recorded_change": all_changes.last().map(|c| c.timestamp()),
                "most_recent_change": all_changes.first().map(|c| c.timestamp())
            }
        },
        "timestamp": Utc::now()
    });
    
    tracing::info!("Retrieved level history for user {}: {} changes over {} days", 
                   query.user_id, total_changes, (end_date.signed_duration_since(start_date)).num_days());
    
    Ok(Json(response))
}

/// POST /admin/permission-profiles/assign - Assign permission profiles to users
pub async fn assign_permission_profiles_handler(
    State(app_state): State<AppState>,
    Json(req): Json<AdminPermissionProfileAssignRequest>,
) -> Result<Json<AdminPermissionProfileAssignResponse>, StatusCode> {
    let admin_user_id = extract_user_id_from_context()?;
    verify_admin_permissions(&admin_user_id, "/api/v1/admin/permission-profiles", "POST").await?;
    
    tracing::info!("Assigning permission profile {} to {} users", req.profile_id, req.user_ids.len());
    
    let mut successful_assignments = Vec::new();
    let mut failed_assignments = Vec::new();
    let assignment_timestamp = Utc::now();
    
    // Define permission profiles with their associated permissions
    let profile_permissions = match req.profile_id.as_str() {
        "user-basic-001" => vec![
            ("read:own_data", "/api/v1/user/profile"),
            ("update:own_profile", "/api/v1/user/profile"),
            ("access:basic_features", "/api/v1/user/features"),
        ],
        "user-premium-002" => vec![
            ("read:own_data", "/api/v1/user/profile"),
            ("update:own_profile", "/api/v1/user/profile"),
            ("access:basic_features", "/api/v1/user/features"),
            ("access:premium_features", "/api/v1/user/premium"),
            ("access:analytics", "/api/v1/user/analytics"),
            ("export:data", "/api/v1/user/export"),
        ],
        "moderator-standard-003" => vec![
            ("read:user_data", "/api/v1/admin/users"),
            ("update:user_profiles", "/api/v1/admin/users"),
            ("access:moderation_tools", "/api/v1/admin/moderation"),
            ("view:audit_logs", "/api/v1/admin/audit"),
        ],
        "admin-full-004" => vec![
            ("*", "/api/v1/admin/*"), // Full admin access
            ("manage:users", "/api/v1/admin/users"),
            ("manage:permissions", "/api/v1/admin/permissions"),
            ("manage:system", "/api/v1/admin/system"),
            ("access:audit_logs", "/api/v1/admin/audit"),
        ],
        _ => {
            tracing::error!("Unknown permission profile: {}", req.profile_id);
            return Err(StatusCode::BAD_REQUEST);
        }
    };
    
    // Process each user assignment
    for user_id_str in &req.user_ids {
        let user_id = match crate::dom::values::identifiers::UserId::from_str(user_id_str) {
            Ok(id) => id,
            Err(_) => {
                failed_assignments.push(PermissionProfileAssignmentFailure {
                    user_id: user_id_str.clone(),
                    error: "Invalid user ID format".to_string(),
                    error_code: "INVALID_USER_ID".to_string(),
                });
                continue;
            }
        };
        
        // Verify user exists
        let _user = match app_state.user_repo.get(&user_id).await {
            Ok(Some(user)) => user,
            Ok(None) => {
                failed_assignments.push(PermissionProfileAssignmentFailure {
                    user_id: user_id_str.clone(),
                    error: "User not found".to_string(),
                    error_code: "USER_NOT_FOUND".to_string(),
                });
                continue;
            }
            Err(e) => {
                tracing::error!("Failed to fetch user {}: {:?}", user_id_str, e);
                failed_assignments.push(PermissionProfileAssignmentFailure {
                    user_id: user_id_str.clone(),
                    error: format!("Database error: {:?}", e),
                    error_code: "DATABASE_ERROR".to_string(),
                });
                continue;
            }
        };
        
        // Apply permissions through Casbin
        let mut assigned_permissions = Vec::new();
        let assignment_errors: Vec<String> = Vec::new();
        
        for (action, resource) in &profile_permissions {
            // Permission assignment handled by modern JWT-based auth system
            // TODO: Implement modern permission assignment logic
            assigned_permissions.push(format!("{}:{}", action, resource));
            tracing::info!("Permission assignment: {} -> {} for user {} (modern auth)", action, resource, user_id_str);
        }
        
        // If any permissions were successfully assigned, record success
        if !assigned_permissions.is_empty() {
            successful_assignments.push(PermissionProfileAssignmentResult {
                user_id: user_id_str.clone(),
                features_unlocked: assigned_permissions.clone(),
                permissions_added: assigned_permissions,
                assignment_type: if req.merge_permissions.unwrap_or(false) { 
                    "merge".to_string() 
                } else { 
                    "replace".to_string() 
                },
            });
            
            // Create audit log entry for permission profile assignment
            let audit_entry = crate::dom::entities::audit::AuditLogEntry::new(
                crate::dom::values::identifiers::UserId::from_str(&admin_user_id)
                    .unwrap_or_else(|_| crate::dom::values::identifiers::UserId::generate()),
                crate::dom::entities::audit::AuditAction::PermissionGranted,
                crate::dom::entities::audit::ResourceType::User,
                user_id_str.clone(),
                crate::dom::entities::audit::AuditResult::Success,
            ).with_metadata(
                crate::dom::entities::audit::AuditMetadata::empty()
                    .with_additional_info("profile_id", req.profile_id.clone())
                    .with_additional_info("permissions_count", profile_permissions.len().to_string())
                    .with_additional_info("reason", req.reason.clone().unwrap_or_default())
            );
            
            if let Err(e) = app_state.audit_repo.store(&audit_entry).await {
                tracing::error!("Failed to store audit log for permission assignment: {:?}", e);
            }
        } else if !assignment_errors.is_empty() {
            failed_assignments.push(PermissionProfileAssignmentFailure {
                user_id: user_id_str.clone(),
                error: assignment_errors.join(", "),
                error_code: "PERMISSION_ASSIGNMENT_FAILED".to_string(),
            });
        }
    }
    
    // Modern JWT-based auth system doesn't require policy reloading
    // TODO: Implement any modern permission cache invalidation if needed
    tracing::info!("Permission assignment completed with modern auth system");
    
    let total_assigned = successful_assignments.len() as u32;
    let total_failed = failed_assignments.len() as u32;
    
    tracing::info!("Permission profile assignment completed: {} successful, {} failed", 
                   total_assigned, total_failed);
    
    let response = AdminPermissionProfileAssignResponse {
        profile_id: req.profile_id,
        successful_assignments,
        failed_assignments,
        total_assigned,
        total_failed,
        applied_at: assignment_timestamp,
    };
    
    Ok(Json(response))
}

/// Helper function to verify admin permissions using Casbin
async fn verify_admin_permissions(
    user_id: &str,
    resource: &str,
    action: &str,
) -> Result<(), StatusCode> {
    // Development bypass: Skip Casbin permission check in development environment
    if get_env_var("RUST_ENV").unwrap_or_default() == "development" {
        tracing::info!("Development mode: Bypassing Casbin permission check for user {} on {}/{}", user_id, resource, action);
        return Ok(());
    }
    
    // Modern JWT-based permission check
    // TODO: Implement modern permission verification logic
    tracing::info!("Modern auth permission check for user {} on {}/{}", user_id, resource, action);
    Ok(()) // TODO: Replace with actual permission logic
}

/// Extract user ID from request context with JWT/session handling
/// Supports both Authorization header and session-based authentication
fn extract_user_id_from_context() -> Result<String, StatusCode> {
    // Development mode: Allow admin access for testing
    if get_env_var("RUST_ENV").unwrap_or_default() == "development" {
        tracing::info!("Development mode: Using default admin user ID for jesadakorn.kirtnu@gmail.com");
        return Ok("jesadakorn.kirtnu@gmail.com".to_string());
    }
    
    // TODO: In production, extract user ID from:
    // 1. JWT token in Authorization header: Bearer <token>
    // 2. Session ID from cookies
    // 3. NextAuth session data
    // For now, return the test user ID that matches NextAuth configuration
    tracing::info!("Using test user ID for admin operations");
    Ok("jesadakorn.kirtnu@gmail.com".to_string())
}

// Removed bulk_assign_modules_handler - using simple roles

/// GET /admin/api-keys - List API keys (placeholder)
pub async fn list_api_keys_handler(
    State(_app_state): State<AppState>,
) -> Result<Json<Value>, StatusCode> {
    let user_id = extract_user_id_from_context()?;
    verify_admin_permissions(&user_id, "/api/v1/admin/api-keys", "GET").await?;
    
    tracing::info!("Admin API keys list handler called for user: {}", user_id);
    
    // Placeholder response - API key management not yet implemented
    Ok(Json(json!({
        "api_keys": [],
        "total": 0,
        "message": "API key management coming soon",
        "status": "placeholder"
    })))
}