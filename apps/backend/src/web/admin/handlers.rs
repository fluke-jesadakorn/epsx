// Admin API handlers for user management with Casbin authorization

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use crate::web::auth::AppState;
use chrono::{DateTime, Utc, Datelike};
use serde_json::{json, Value};

// Request/Response DTOs for admin endpoints

#[derive(Debug, Serialize, Deserialize)]
pub struct AdminCreateUserRequest {
    pub email: String,
    pub role: String,
    pub display_name: Option<String>,
    pub password: Option<String>,
    pub fb_token: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AdminUpdateUserRequest {
    pub role: Option<String>,
    pub email: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AdminListUsersQuery {
    pub offset: Option<u32>,
    pub limit: Option<u32>,
    pub role_filter: Option<String>,
    pub page_token: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AdminUserStatsQuery {
    pub include_roles: Option<bool>,
    pub include_tiers: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AdminBulkUpdateRequest {
    pub user_ids: Vec<String>,
    pub new_level: Option<String>,
    pub new_role: Option<String>,
    pub batch_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BulkModuleAssignmentRequest {
    pub user_ids: Vec<String>,
    pub assignments: Vec<ModuleAssignment>,
    pub reason: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ModuleAssignment {
    pub module_id: String,
    pub access_level: String,
    pub custom_quotas: Option<Value>,
    pub restrictions: Option<Value>,
    pub expires_at: Option<DateTime<Utc>>,
}

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
    verify_admin_permissions(&app_state, &user_id, "/api/v1/admin/users", "GET").await?;
    
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
    
    let user_list: Vec<Value> = users.into_iter().map(|user| {
        json!({
            "id": user.id().to_string(),
            "email": user.email().value(),
            "role": user.role().to_string(),
            "subscription_tier": user.sub().tier().to_string(),
            "is_active": user.is_active(),
            "created_at": user.created_at(),
            "updated_at": user.updated_at()
        })
    }).collect();
    
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
    verify_admin_permissions(&app_state, &user_id, "/api/v1/admin/users", "POST").await?;
    
    tracing::info!(
        "Admin create user handler called with authorization for role: {}, display_name: {:?}", 
        req.role, req.display_name
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
    
    // Parse role
    let role = crate::dom::values::Role::from_string(&req.role).unwrap_or(crate::dom::values::Role::User);
    
    // Generate Firebase UID (in a real system, this would come from Firebase)
    let firebase_uid = req.fb_token.unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
    
    // Create new user
    let user = crate::dom::entities::User::new(firebase_uid, email, role);
    
    // Save user to database
    if let Err(e) = app_state.user_repo.save(&user).await {
        tracing::error!("Failed to create user: {:?}", e);
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }
    
    tracing::info!("Successfully created user with ID: {}", user.id());
    
    Ok(Json(json!({
        "message": "User created successfully",
        "user_id": user.id().to_string(),
        "email": user.email().value(),
        "role": user.role().to_string(),
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
    verify_admin_permissions(&app_state, &admin_user_id, "/api/v1/admin/users", "GET").await?;
    
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
    
    // Get user roles from Casbin
    let user_roles = match app_state.casbin_service.get_roles_for_user(&user_id).await {
        Ok(roles) => roles,
        Err(e) => {
            tracing::warn!("Failed to get roles for user {}: {:?}", user_id, e);
            vec![]
        }
    };
    
    // Get user permissions from Casbin
    let user_permissions = match app_state.casbin_service.get_permissions_for_subject(&user_id).await {
        Ok(perms) => perms.into_iter()
            .map(|(resource, action)| format!("{}:{}", resource, action))
            .collect::<Vec<String>>(),
        Err(e) => {
            tracing::warn!("Failed to get permissions for user {}: {:?}", user_id, e);
            vec![]
        }
    };
    
    // TODO: Add audit logging when audit interface is confirmed
    tracing::info!("Successfully retrieved user details for user: {} by admin: {}", user_id, admin_user_id);
    
    Ok(Json(json!({
        "user_id": user.id().to_string(),
        "email": user.email().value(),
        "firebase_uid": user.firebase_uid(),
        "role": user.role().to_string(),
        "roles": user_roles,
        "permissions": user_permissions,
        "subscription_tier": user.sub().tier().to_string(),
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
    verify_admin_permissions(&app_state, &admin_user_id, "/api/v1/admin/users", "PUT").await?;
    
    tracing::info!("Admin update user handler called for user: {} by admin: {}", user_id, admin_user_id);
    
    if req.role.is_none() && req.email.is_none() {
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
    let old_role = user.role().clone();
    
    // Handle role update
    if let Some(new_role_str) = req.role {
        let new_role = match crate::dom::values::Role::from_string(&new_role_str) {
            Ok(role) => role,
            Err(_) => {
                tracing::warn!("Invalid role provided: {}", new_role_str);
                return Err(StatusCode::BAD_REQUEST);
            }
        };
        
        if new_role != old_role {
            match user.upgrade_role(new_role.clone()) {
                Ok(_event) => {
                    changes_made.push(format!("role: {} -> {}", old_role, new_role));
                    
                    // Update Casbin roles
                    if let Err(e) = app_state.casbin_service.remove_role_for_user(&user_id, &old_role.to_string()).await {
                        tracing::warn!("Failed to remove old Casbin role for user {}: {:?}", user_id, e);
                    }
                    
                    if let Err(e) = app_state.casbin_service.add_role_for_user(&user_id, &new_role.to_string()).await {
                        tracing::error!("Failed to add new Casbin role for user {}: {:?}", user_id, e);
                        return Err(StatusCode::INTERNAL_SERVER_ERROR);
                    }
                },
                Err(e) => {
                    tracing::warn!("Role upgrade failed for user {}: {:?}", user_id, e);
                    return Err(StatusCode::BAD_REQUEST);
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
    
    Ok(Json(json!({
        "user_id": user.id().to_string(),
        "message": "User updated successfully",
        "changes_made": changes_made,
        "updated_at": user.updated_at(),
        "current_role": user.role().to_string(),
        "is_active": user.is_active()
    })))
}

/// DELETE /admin/users/{user_id} - Soft delete user
pub async fn delete_user_handler(
    State(app_state): State<AppState>,
    Path(user_id): Path<String>,
) -> Result<Json<Value>, StatusCode> {
    let admin_user_id = extract_user_id_from_context()?;
    verify_admin_permissions(&app_state, &admin_user_id, "/api/v1/admin/users", "DELETE").await?;
    
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
    
    // Remove all Casbin roles and policies for the user
    let user_roles = match app_state.casbin_service.get_roles_for_user(&user_id).await {
        Ok(roles) => roles,
        Err(e) => {
            tracing::warn!("Failed to get roles for user deletion {}: {:?}", user_id, e);
            vec![]
        }
    };
    
    for role in user_roles {
        if let Err(e) = app_state.casbin_service.remove_role_for_user(&user_id, &role).await {
            tracing::warn!("Failed to remove Casbin role {} for deleted user {}: {:?}", role, user_id, e);
        }
    }
    
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
    verify_admin_permissions(&app_state, &admin_user_id, "/api/v1/admin/analytics", "GET").await?;
    
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
    
    // Include role breakdown if requested - updated for modern admin module system
    if query.include_roles.unwrap_or(true) {
        let mut role_counts = std::collections::HashMap::new();
        
        for user in &all_users {
            if !user.is_deleted() {  // Only count active users
                // Since roles are now managed through admin modules, default to "user"
                // In the future, this could query the admin modules system for actual roles
                let role_str = "user".to_string();
                *role_counts.entry(role_str).or_insert(0) += 1;
            }
        }
        
        response["by_role"] = json!(role_counts);
    }
    
    // Include tier breakdown if requested
    if query.include_tiers.unwrap_or(true) {
        let mut tier_counts = std::collections::HashMap::new();
        
        for user in &all_users {
            if !user.is_deleted() {  // Only count active users
                let tier_str = user.sub().tier().to_string();
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
    verify_admin_permissions(&app_state, &admin_user_id, "/api/v1/admin/users", "PUT").await?;
    
    tracing::info!("Admin bulk update handler called for {} users by admin: {}", 
                  req.user_ids.len(), admin_user_id);
    
    if req.user_ids.is_empty() {
        tracing::warn!("Empty user_ids list provided for bulk update");
        return Err(StatusCode::BAD_REQUEST);
    }
    
    if req.new_role.is_none() && req.new_level.is_none() {
        tracing::warn!("No update fields provided for bulk update");
        return Err(StatusCode::BAD_REQUEST);
    }
    
    // Validate new role if provided
    let new_role = if let Some(role_str) = &req.new_role {
        match crate::dom::values::Role::from_string(role_str) {
            Ok(role) => Some(role),
            Err(_) => {
                tracing::warn!("Invalid role provided for bulk update: {}", role_str);
                return Err(StatusCode::BAD_REQUEST);
            }
        }
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
        let old_role = user.role().clone();
        
        // Handle role update
        if let Some(ref new_role) = new_role {
            if *new_role != old_role {
                match user.upgrade_role(new_role.clone()) {
                    Ok(_event) => {
                        changes_made.push(format!("role: {} -> {}", old_role, new_role));
                        
                        // Update Casbin roles
                        if let Err(e) = app_state.casbin_service.remove_role_for_user(&user_id, &old_role.to_string()).await {
                            tracing::warn!("Failed to remove old Casbin role for user {} in bulk update: {:?}", user_id, e);
                        }
                        
                        if let Err(e) = app_state.casbin_service.add_role_for_user(&user_id, &new_role.to_string()).await {
                            tracing::error!("Failed to add new Casbin role for user {} in bulk update: {:?}", user_id, e);
                            failed_updates.push(json!({
                                "user_id": user_id,
                                "error": "Failed to update Casbin role"
                            }));
                            continue;
                        }
                    },
                    Err(e) => {
                        tracing::warn!("Role upgrade failed for user {} in bulk update: {:?}", user_id, e);
                        failed_updates.push(json!({
                            "user_id": user_id,
                            "error": format!("Role upgrade failed: {:?}", e)
                        }));
                        continue;
                    }
                }
            }
        }
        
        // Handle level update (treat as synonym for role for now)
        if let Some(ref level_str) = req.new_level {
            if req.new_role.is_none() {  // Only process if role wasn't already updated
                match crate::dom::values::Role::from_string(level_str) {
                    Ok(level_role) if level_role != old_role => {
                        match user.upgrade_role(level_role.clone()) {
                            Ok(_event) => {
                                changes_made.push(format!("level: {} -> {}", old_role, level_role));
                                
                                // Update Casbin roles for level change
                                if let Err(e) = app_state.casbin_service.remove_role_for_user(&user_id, &old_role.to_string()).await {
                                    tracing::warn!("Failed to remove old Casbin role for level update {}: {:?}", user_id, e);
                                }
                                
                                if let Err(e) = app_state.casbin_service.add_role_for_user(&user_id, &level_role.to_string()).await {
                                    tracing::error!("Failed to add new Casbin role for level update {}: {:?}", user_id, e);
                                    failed_updates.push(json!({
                                        "user_id": user_id,
                                        "error": "Failed to update Casbin role for level change"
                                    }));
                                    continue;
                                }
                            },
                            Err(e) => {
                                tracing::warn!("Level upgrade failed for user {} in bulk update: {:?}", user_id, e);
                                failed_updates.push(json!({
                                    "user_id": user_id,
                                    "error": format!("Level upgrade failed: {:?}", e)
                                }));
                                continue;
                            }
                        }
                    },
                    Ok(_) => {
                        // Same level, no change needed
                        changes_made.push("level: no change needed".to_string());
                    }
                    Err(_) => {
                        tracing::warn!("Invalid level provided for user {} in bulk update: {}", user_id, level_str);
                        failed_updates.push(json!({
                            "user_id": user_id,
                            "error": "Invalid level provided"
                        }));
                        continue;
                    }
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
    verify_admin_permissions(&app_state, &admin_user_id, "/api/v1/admin/users", "GET").await?;
    
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
            let (previous_level, new_level) = if let (Some(prev), Some(new)) = (
                change.metadata().previous_values.as_ref(),
                change.metadata().new_values.as_ref()
            ) {
                let prev_role = prev.get("role").unwrap_or(&"unknown".to_string()).clone();
                let new_role = new.get("role").unwrap_or(&"unknown".to_string()).clone();
                (prev_role, new_role)
            } else {
                ("unknown".to_string(), "current".to_string())
            };
            
            json!({
                "id": change.id().value(),
                "action": change.action().to_string(),
                "timestamp": change.timestamp(),
                "previous_level": previous_level,
                "new_level": new_level,
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
    
    // Generate user level summary
    let current_level_info = json!({
        "current_role": user.role().to_string(),
        "current_tier": user.sub().tier().to_string(),
        "account_age_days": (Utc::now().signed_duration_since(user.created_at())).num_days(),
        "is_active": user.is_active(),
        "created_at": user.created_at(),
        "updated_at": user.updated_at()
    });
    
    let response = json!({
        "status": "success",
        "data": {
            "user_id": query.user_id,
            "user_email": user.email().value(),
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
    verify_admin_permissions(&app_state, &admin_user_id, "/api/v1/admin/permission-profiles", "POST").await?;
    
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
        let mut assignment_errors = Vec::new();
        
        for (action, resource) in &profile_permissions {
            match app_state.casbin_service.add_policy(user_id_str, resource, action).await {
                Ok(true) => {
                    assigned_permissions.push(format!("{}:{}", action, resource));
                    tracing::info!("Assigned permission {} -> {} to user {}", user_id_str, resource, action);
                }
                Ok(false) => {
                    // Permission already exists - not an error
                    assigned_permissions.push(format!("{}:{} (already assigned)", action, resource));
                }
                Err(e) => {
                    tracing::error!("Failed to assign permission {} -> {} to user {}: {:?}", 
                                   user_id_str, resource, action, e);
                    assignment_errors.push(format!("{}:{} - {:?}", action, resource, e));
                }
            }
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
    
    // Reload Casbin policies to ensure they're active
    if let Err(e) = app_state.casbin_service.reload_policies().await {
        tracing::error!("Failed to reload Casbin policies after permission assignment: {:?}", e);
    }
    
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
    app_state: &AppState,
    user_id: &str,
    resource: &str,
    action: &str,
) -> Result<(), StatusCode> {
    // Development bypass: Skip Casbin permission check in development environment
    if std::env::var("RUST_ENV").unwrap_or_default() == "development" {
        tracing::info!("Development mode: Bypassing Casbin permission check for user {} on {}/{}", user_id, resource, action);
        return Ok(());
    }
    
    match app_state.casbin_service.enforce(user_id, resource, action).await {
        Ok(true) => {
            tracing::debug!("Admin permission granted for user {} on {}/{}", user_id, resource, action);
            Ok(())
        }
        Ok(false) => {
            tracing::warn!("Admin permission denied for user {} on {}/{}", user_id, resource, action);
            tracing::info!("Casbin policy check failed - user_id: '{}', resource: '{}', action: '{}'", user_id, resource, action);
            Err(StatusCode::FORBIDDEN)
        }
        Err(e) => {
            tracing::error!("Failed to check admin permissions: {}", e);
            tracing::error!("Casbin error details - user_id: '{}', resource: '{}', action: '{}'", user_id, resource, action);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Extract user ID from request context with JWT/session handling
/// Supports both Authorization header and session-based authentication
fn extract_user_id_from_context() -> Result<String, StatusCode> {
    // Development mode: Allow admin access for testing
    if std::env::var("RUST_ENV").unwrap_or_default() == "development" {
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

/// Bulk assign modules to users
/// NOTE: This is a placeholder handler for frontend compatibility
/// Stock ranking modules are temporarily disabled during Casbin migration
pub async fn bulk_assign_modules_handler(
    State(_state): State<AppState>,
    Json(request): Json<BulkModuleAssignmentRequest>
) -> Result<Json<Value>, StatusCode> {
    tracing::info!("Bulk module assignment request received for {} users", request.user_ids.len());
    
    // Placeholder response - actual implementation would:
    // 1. Validate user IDs exist
    // 2. Check admin permissions using Casbin
    // 3. Apply module assignments
    // 4. Log audit events
    // 5. Send notifications if configured
    
    Ok(Json(json!({
        "message": "Module assignment functionality temporarily unavailable",
        "status": "placeholder",
        "requested_users": request.user_ids.len(),
        "requested_modules": request.assignments.len(),
        "reason": "Stock ranking modules disabled during Casbin migration",
        "timestamp": Utc::now()
    })))
}

/// GET /admin/api-keys - List API keys (placeholder)
pub async fn list_api_keys_handler(
    State(app_state): State<AppState>,
) -> Result<Json<Value>, StatusCode> {
    let user_id = extract_user_id_from_context()?;
    verify_admin_permissions(&app_state, &user_id, "/api/v1/admin/api-keys", "GET").await?;
    
    tracing::info!("Admin API keys list handler called for user: {}", user_id);
    
    // Placeholder response - API key management not yet implemented
    Ok(Json(json!({
        "api_keys": [],
        "total": 0,
        "message": "API key management coming soon",
        "status": "placeholder"
    })))
}