// Permission Profile CRUD handlers for admin API

use axum::{
    extract::{Path, Query, State},
    response::Json,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use chrono::{DateTime, Utc};

use crate::web::auth::AppState;
use crate::core::errors::{AppError, ErrorKind};
use crate::dom::entities::permission_profile::{
    PermissionProfile,
    PermissionProfileId,
    PermissionProfileQuery,
    PermissionProfileCategory,
};
use crate::dom::values::identifiers::UserId;
use crate::dom::entities::iam::{Permission, PackageTier};

// Request/Response DTOs for permission profile CRUD operations

#[derive(Debug, Serialize, Deserialize)]
pub struct CreatePermissionProfileRequest {
    pub name: String,
    pub description: Option<String>,
    pub category: String,
    pub permissions: Vec<PermissionDto>,
    pub target_tier: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdatePermissionProfileRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub category: Option<String>,
    pub permissions: Option<Vec<PermissionDto>>,
    pub target_tier: Option<String>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PermissionDto {
    pub resource: String,
    pub action: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ListPermissionProfilesQuery {
    pub page: Option<u32>,
    pub limit: Option<u32>,
    pub category: Option<String>,
    pub active_only: Option<bool>,
    pub name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PermissionProfileResponse {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: String,
    pub permissions: Vec<PermissionDto>,
    pub target_tier: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: String,
    pub assignment_count: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ListPermissionProfilesResponse {
    pub profiles: Vec<PermissionProfileResponse>,
    pub total_count: u64,
    pub page: u32,
    pub limit: u32,
    pub has_more: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UnassignProfileRequest {
    pub user_id: String,
    pub profile_id: String,
    pub reason: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UnassignProfileResponse {
    pub success: bool,
    pub message: String,
    pub user_id: String,
    pub profile_id: String,
    pub unassigned_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ValidateAssignmentRequest {
    pub user_id: String,
    pub profile_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ValidateAssignmentResponse {
    pub is_valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub conflicts: Vec<String>,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BulkValidateAssignmentRequest {
    pub user_ids: Vec<String>,
    pub profile_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BulkValidateAssignmentResponse {
    pub profile_id: String,
    pub profile_name: String,
    pub total_users: usize,
    pub valid_assignments: usize,
    pub invalid_assignments: usize,
    pub results: Vec<BulkValidationResult>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BulkValidationResult {
    pub user_id: String,
    pub is_valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

// Helper functions

fn map_permission_profile_to_response(
    profile: PermissionProfile,
    assignment_count: Option<u32>,
) -> PermissionProfileResponse {
    let permissions: Vec<PermissionDto> = profile
        .default_permissions()
        .iter()
        .map(|p| PermissionDto {
            resource: p.resource().to_string(),
            action: p.action().to_string(),
        })
        .collect();

    PermissionProfileResponse {
        id: profile.id().value().to_string(),
        name: profile.name().to_string(),
        description: profile.description().to_string(),
        category: map_category_to_string(profile.category()),
        permissions,
        target_tier: map_tier_to_string(profile.target_tier()),
        is_active: profile.is_active(),
        created_at: *profile.created_at(),
        updated_at: *profile.updated_at(),
        created_by: profile.created_by().value().to_string(),
        assignment_count,
    }
}

fn map_category_from_string(category: &str) -> PermissionProfileCategory {
    match category.to_lowercase().as_str() {
        "user" => PermissionProfileCategory::User,
        "moderator" => PermissionProfileCategory::Moderator,
        "admin" => PermissionProfileCategory::Admin,
        "custom" => PermissionProfileCategory::Custom,
        "system" => PermissionProfileCategory::System,
        "business" => PermissionProfileCategory::Business,
        "technical" => PermissionProfileCategory::Technical,
        "administrative" => PermissionProfileCategory::Administrative,
        "compliance" => PermissionProfileCategory::Compliance,
        _ => PermissionProfileCategory::Custom,
    }
}

fn map_category_to_string(category: &PermissionProfileCategory) -> String {
    match category {
        PermissionProfileCategory::User => "user".to_string(),
        PermissionProfileCategory::Moderator => "moderator".to_string(),
        PermissionProfileCategory::Admin => "admin".to_string(),
        PermissionProfileCategory::Custom => "custom".to_string(),
        PermissionProfileCategory::System => "system".to_string(),
        PermissionProfileCategory::Business => "business".to_string(),
        PermissionProfileCategory::Technical => "technical".to_string(),
        PermissionProfileCategory::Administrative => "administrative".to_string(),
        PermissionProfileCategory::Compliance => "compliance".to_string(),
    }
}

fn map_tier_from_string(tier: &str) -> PackageTier {
    match tier.to_lowercase().as_str() {
        "free" => PackageTier::Free,
        "bronze" => PackageTier::Bronze,
        "silver" => PackageTier::Silver,
        "gold" => PackageTier::Gold,
        "platinum" => PackageTier::Platinum,
        "admin" => PackageTier::Admin,
        "superadmin" => PackageTier::SuperAdmin,
        _ => PackageTier::Free,
    }
}

fn map_tier_to_string(tier: &PackageTier) -> String {
    match tier {
        PackageTier::Free => "free".to_string(),
        PackageTier::Bronze => "bronze".to_string(),
        PackageTier::Silver => "silver".to_string(),
        PackageTier::Gold => "gold".to_string(),
        PackageTier::Platinum => "platinum".to_string(),
        PackageTier::Admin => "admin".to_string(),
        PackageTier::SuperAdmin => "superadmin".to_string(),
    }
}

async fn verify_admin_access(app_state: &AppState, resource: &str, action: &str) -> Result<(), AppError> {
    // TODO: Extract user ID from authenticated context
    let user_id = "admin_user"; // Placeholder - in production get from session/token

    match app_state.casbin_service.enforce(user_id, resource, action).await {
        Ok(true) => {
            tracing::debug!("Admin access granted for {} on {}/{}", user_id, resource, action);
            Ok(())
        }
        Ok(false) => {
            tracing::warn!("Admin access denied for {} on {}/{}", user_id, resource, action);
            Err(AppError::new(
                ErrorKind::AuthorizationError,
                format!("Access denied for {}/{}", resource, action),
            ))
        }
        Err(e) => {
            tracing::error!("Failed to check admin permissions: {}", e);
            Err(AppError::new(
                ErrorKind::InternalServerError,
                format!("Failed to check permissions: {}", e),
            ))
        }
    }
}

// CRUD Handlers

/// GET /admin/permission-profiles - List all permission profiles with pagination
pub async fn list_permission_profiles_handler(
    State(app_state): State<AppState>,
    Query(query): Query<ListPermissionProfilesQuery>,
) -> Result<Json<ListPermissionProfilesResponse>, AppError> {
    verify_admin_access(&app_state, "permission_profiles", "read").await?;

    let page = query.page.unwrap_or(1);
    let limit = query.limit.unwrap_or(20).min(100); // Max 100 items per page
    let offset = (page - 1) * limit;

    let profile_query = PermissionProfileQuery {
        name: query.name,
        category: query.category.as_ref().map(|c| map_category_from_string(c)),
        target_tier: None, // Not filtering by tier in list endpoint
        tags: Vec::new(), // Not filtering by tags in list endpoint
        active_only: query.active_only.unwrap_or(true),
        limit: Some(limit),
        offset: Some(offset),
    };

    // Get profiles and total count
    let profiles = app_state
        .permission_profile_repo
        .search(&profile_query)
        .await
        .map_err(|e| AppError::new(ErrorKind::InternalServerError, format!("Failed to fetch profiles: {}", e)))?;

    let total_count = app_state
        .permission_profile_repo
        .count(&profile_query)
        .await
        .map_err(|e| AppError::new(ErrorKind::InternalServerError, format!("Failed to count profiles: {}", e)))?;

    // Get assignment counts for each profile
    let mut profile_responses = Vec::new();
    for profile in profiles {
        let assignment_count = app_state
            .permission_profile_repo
            .get_assignment_count(&profile.id())
            .await
            .ok();
        
        profile_responses.push(map_permission_profile_to_response(profile, assignment_count));
    }

    let has_more = (page * limit) < total_count as u32;

    let response = ListPermissionProfilesResponse {
        profiles: profile_responses,
        total_count,
        page,
        limit,
        has_more,
    };

    Ok(Json(response))
}

/// GET /admin/permission-profiles/:id - Get a specific permission profile
pub async fn get_permission_profile_handler(
    State(app_state): State<AppState>,
    Path(profile_id): Path<String>,
) -> Result<Json<PermissionProfileResponse>, AppError> {
    verify_admin_access(&app_state, "permission_profiles", "read").await?;

    let profile_id = PermissionProfileId::new(profile_id);
    
    let profile = app_state
        .permission_profile_repo
        .get(&profile_id)
        .await
        .map_err(|e| AppError::new(ErrorKind::InternalServerError, format!("Failed to fetch profile: {}", e)))?
        .ok_or_else(|| AppError::new(ErrorKind::AggregateNotFound, "Permission profile not found"))?;

    let assignment_count = app_state
        .permission_profile_repo
        .get_assignment_count(&profile_id)
        .await
        .ok();

    let response = map_permission_profile_to_response(profile, assignment_count);
    Ok(Json(response))
}

/// POST /admin/permission-profiles - Create a new permission profile
pub async fn create_permission_profile_handler(
    State(app_state): State<AppState>,
    Json(request): Json<CreatePermissionProfileRequest>,
) -> Result<Json<PermissionProfileResponse>, AppError> {
    verify_admin_access(&app_state, "permission_profiles", "create").await?;

    // Validate input
    if request.name.trim().is_empty() {
        return Err(AppError::new(ErrorKind::ValidationError, "Profile name is required"));
    }

    // TODO: Get actual admin user ID from authenticated context
    let admin_user_id = UserId::new("admin_user".to_string());

    // Convert permissions
    let permissions: Vec<Permission> = request
        .permissions
        .iter()
        .map(|p| Permission::new(p.resource.clone(), p.action.clone()))
        .collect();

    // Create profile
    let mut profile = PermissionProfile::new(
        request.name.trim().to_string(),
        request.description.unwrap_or_default(),
        request.target_tier.as_ref().map(|t| map_tier_from_string(t)).unwrap_or(PackageTier::Free),
        map_category_from_string(&request.category),
        admin_user_id,
    );

    // Set custom permissions if provided
    profile.set_default_permissions(permissions);

    // Save to database
    let created_profile = app_state
        .permission_profile_repo
        .create(profile)
        .await
        .map_err(|e| AppError::new(ErrorKind::InternalServerError, format!("Failed to create profile: {}", e)))?;

    let response = map_permission_profile_to_response(created_profile, Some(0));
    Ok(Json(response))
}

/// PUT /admin/permission-profiles/:id - Update an existing permission profile
pub async fn update_permission_profile_handler(
    State(app_state): State<AppState>,
    Path(profile_id): Path<String>,
    Json(request): Json<UpdatePermissionProfileRequest>,
) -> Result<Json<PermissionProfileResponse>, AppError> {
    verify_admin_access(&app_state, "permission_profiles", "update").await?;

    let profile_id = PermissionProfileId::new(profile_id);
    
    // Get existing profile
    let mut profile = app_state
        .permission_profile_repo
        .get(&profile_id)
        .await
        .map_err(|e| AppError::new(ErrorKind::InternalServerError, format!("Failed to fetch profile: {}", e)))?
        .ok_or_else(|| AppError::new(ErrorKind::AggregateNotFound, "Permission profile not found"))?;

    // Update fields if provided
    if let Some(name) = &request.name {
        if !name.trim().is_empty() {
            profile.set_name(name.trim().to_string());
        }
    }

    if let Some(description) = &request.description {
        profile.set_description(description.clone());
    }

    if let Some(category) = &request.category {
        profile.set_category(map_category_from_string(category));
    }

    if let Some(permissions) = &request.permissions {
        let new_permissions: Vec<Permission> = permissions
            .iter()
            .map(|p| Permission::new(p.resource.clone(), p.action.clone()))
            .collect();
        profile.set_default_permissions(new_permissions);
    }

    if let Some(target_tier) = &request.target_tier {
        profile.set_target_tier(map_tier_from_string(target_tier));
    }

    if let Some(is_active) = request.is_active {
        profile.set_active(is_active);
    }

    // Update timestamp
    profile.touch_updated_at();

    // Save to database
    let updated_profile = app_state
        .permission_profile_repo
        .update(profile)
        .await
        .map_err(|e| AppError::new(ErrorKind::InternalServerError, format!("Failed to update profile: {}", e)))?;

    let assignment_count = app_state
        .permission_profile_repo
        .get_assignment_count(&profile_id)
        .await
        .ok();

    let response = map_permission_profile_to_response(updated_profile, assignment_count);
    Ok(Json(response))
}

/// DELETE /admin/permission-profiles/:id - Soft delete a permission profile
pub async fn delete_permission_profile_handler(
    State(app_state): State<AppState>,
    Path(profile_id): Path<String>,
) -> Result<Json<Value>, AppError> {
    verify_admin_access(&app_state, "permission_profiles", "delete").await?;

    let profile_id = PermissionProfileId::new(profile_id);
    
    // Check if profile exists
    let profile = app_state
        .permission_profile_repo
        .get(&profile_id)
        .await
        .map_err(|e| AppError::new(ErrorKind::InternalServerError, format!("Failed to fetch profile: {}", e)))?
        .ok_or_else(|| AppError::new(ErrorKind::AggregateNotFound, "Permission profile not found"))?;

    // Check assignment count before deletion
    let assignment_count = app_state
        .permission_profile_repo
        .get_assignment_count(&profile_id)
        .await
        .unwrap_or(0);

    if assignment_count > 0 {
        return Err(AppError::new(
            ErrorKind::BusinessRuleViolation,
            format!("Cannot delete profile with {} active assignments. Please remove assignments first.", assignment_count),
        ));
    }

    // Soft delete (marks as inactive)
    app_state
        .permission_profile_repo
        .delete(&profile_id)
        .await
        .map_err(|e| AppError::new(ErrorKind::InternalServerError, format!("Failed to delete profile: {}", e)))?;

    let response = json!({
        "success": true,
        "message": format!("Permission profile '{}' deleted successfully", profile.name()),
        "deleted_at": Utc::now(),
        "profile_id": profile_id.value()
    });

    Ok(Json(response))
}

/// DELETE /admin/permission-profiles/unassign - Remove a permission profile from a user
pub async fn unassign_permission_profile_handler(
    State(app_state): State<AppState>,
    Json(request): Json<UnassignProfileRequest>,
) -> Result<Json<UnassignProfileResponse>, AppError> {
    verify_admin_access(&app_state, "permission_profiles", "unassign").await?;

    let user_id = UserId::new(request.user_id.clone());
    let profile_id = PermissionProfileId::new(request.profile_id.clone());

    // Verify profile exists
    let profile = app_state
        .permission_profile_repo
        .get(&profile_id)
        .await
        .map_err(|e| AppError::new(ErrorKind::InternalServerError, format!("Failed to fetch profile: {}", e)))?
        .ok_or_else(|| AppError::new(ErrorKind::AggregateNotFound, "Permission profile not found"))?;

    // Verify user exists
    let _user = app_state
        .user_repo
        .get(&user_id)
        .await
        .map_err(|e| AppError::new(ErrorKind::InternalServerError, format!("Failed to fetch user: {}", e)))?
        .ok_or_else(|| AppError::new(ErrorKind::AggregateNotFound, "User not found"))?;

    // Revoke the assignment
    app_state
        .permission_profile_repo
        .revoke_assignment(&user_id, &profile_id)
        .await
        .map_err(|e| AppError::new(ErrorKind::InternalServerError, format!("Failed to unassign profile: {}", e)))?;

    // TODO: Remove actual permissions from Casbin
    // This would involve removing the specific permissions that were granted by this profile
    
    let response = UnassignProfileResponse {
        success: true,
        message: format!("Profile '{}' unassigned from user successfully", profile.name()),
        user_id: request.user_id,
        profile_id: request.profile_id,
        unassigned_at: Utc::now(),
    };

    Ok(Json(response))
}

/// GET /admin/permission-profiles/categories - Get available profile categories
pub async fn get_permission_profile_categories_handler(
    State(_app_state): State<AppState>,
) -> Result<Json<Value>, AppError> {
    let categories = json!({
        "categories": [
            {
                "id": "user",
                "name": "User",
                "description": "Standard user permissions"
            },
            {
                "id": "moderator", 
                "name": "Moderator",
                "description": "Content moderation permissions"
            },
            {
                "id": "admin",
                "name": "Administrator",
                "description": "Administrative permissions"
            },
            {
                "id": "custom",
                "name": "Custom",
                "description": "Custom permission sets"
            },
            {
                "id": "system",
                "name": "System",
                "description": "System-level permissions"
            },
            {
                "id": "business",
                "name": "Business",
                "description": "Business-related permissions"
            },
            {
                "id": "technical",
                "name": "Technical", 
                "description": "Technical permissions"
            },
            {
                "id": "administrative",
                "name": "Administrative",
                "description": "Administrative permissions"
            },
            {
                "id": "compliance",
                "name": "Compliance",
                "description": "Compliance-related permissions"
            }
        ]
    });

    Ok(Json(categories))
}

/// POST /admin/permission-profiles/validate-assignment - Validate profile assignment before applying
pub async fn validate_permission_profile_assignment_handler(
    State(app_state): State<AppState>,
    Json(request): Json<ValidateAssignmentRequest>,
) -> Result<Json<ValidateAssignmentResponse>, AppError> {
    verify_admin_access(&app_state, "permission_profiles", "validate").await?;

    let user_id = UserId::new(request.user_id.clone());
    let profile_id = PermissionProfileId::new(request.profile_id.clone());

    // Verify profile exists and is active
    let profile = app_state
        .permission_profile_repo
        .get(&profile_id)
        .await
        .map_err(|e| AppError::new(ErrorKind::InternalServerError, format!("Failed to fetch profile: {}", e)))?
        .ok_or_else(|| AppError::new(ErrorKind::AggregateNotFound, "Permission profile not found"))?;

    if !profile.is_active() {
        return Ok(Json(ValidateAssignmentResponse {
            is_valid: false,
            errors: vec!["Profile is inactive and cannot be assigned".to_string()],
            warnings: vec![],
            conflicts: vec![],
            recommendations: vec!["Choose an active profile instead".to_string()],
        }));
    }

    // Verify user exists
    let user = app_state
        .user_repo
        .get(&user_id)
        .await
        .map_err(|e| AppError::new(ErrorKind::InternalServerError, format!("Failed to fetch user: {}", e)))?
        .ok_or_else(|| AppError::new(ErrorKind::AggregateNotFound, "User not found"))?;

    let mut errors = Vec::new();
    let mut warnings = Vec::new();
    let conflicts = Vec::new();
    let mut recommendations = Vec::new();

    // Check if profile can be applied to user
    match app_state
        .permission_profile_repo
        .can_apply_to_user(&profile_id, &user_id)
        .await
    {
        Ok(true) => {
            // Additional business logic checks
            
            // Check if user already has this profile
            // This would require checking existing assignments
            recommendations.push(format!("Profile '{}' can be safely assigned to user '{}'", profile.name(), user.email()));
            
            // Check for potential conflicts with existing permissions
            // TODO: Implement conflict detection with existing user permissions
            
            // Check prerequisites (example)
            if profile.metadata().requires_approval {
                warnings.push("This profile requires approval for assignment".to_string());
            }
            
            if let Some(max_assignments) = profile.metadata().max_assignments {
                let current_assignments = app_state
                    .permission_profile_repo
                    .get_assignment_count(&profile_id)
                    .await
                    .unwrap_or(0);
                
                if current_assignments >= max_assignments {
                    errors.push(format!("Profile has reached maximum assignments limit ({}/{})", current_assignments, max_assignments));
                }
            }
        }
        Ok(false) => {
            errors.push("Profile cannot be applied to this user due to business rule violations".to_string());
        }
        Err(e) => {
            errors.push(format!("Failed to validate assignment: {}", e));
        }
    }

    let response = ValidateAssignmentResponse {
        is_valid: errors.is_empty(),
        errors,
        warnings,
        conflicts,
        recommendations,
    };

    Ok(Json(response))
}

/// POST /admin/permission-profiles/bulk-validate - Validate bulk assignments
pub async fn bulk_validate_permission_profile_assignment_handler(
    State(app_state): State<AppState>,
    Json(request): Json<BulkValidateAssignmentRequest>,
) -> Result<Json<BulkValidateAssignmentResponse>, AppError> {
    verify_admin_access(&app_state, "permission_profiles", "validate").await?;

    let mut validation_results = Vec::new();

    // Validate the profile exists first
    let profile_id = PermissionProfileId::new(request.profile_id.clone());
    let profile = app_state
        .permission_profile_repo
        .get(&profile_id)
        .await
        .map_err(|e| AppError::new(ErrorKind::InternalServerError, format!("Failed to fetch profile: {}", e)))?
        .ok_or_else(|| AppError::new(ErrorKind::AggregateNotFound, "Permission profile not found"))?;

    // Validate each user assignment
    for user_id_str in request.user_ids {
        let validation_request = ValidateAssignmentRequest {
            user_id: user_id_str.clone(),
            profile_id: request.profile_id.clone(),
        };

        // Reuse single validation logic
        match validate_permission_profile_assignment_handler(
            State(app_state.clone()),
            Json(validation_request),
        ).await {
            Ok(Json(validation_response)) => {
                validation_results.push(BulkValidationResult {
                    user_id: user_id_str,
                    is_valid: validation_response.is_valid,
                    errors: validation_response.errors,
                    warnings: validation_response.warnings,
                });
            }
            Err(e) => {
                validation_results.push(BulkValidationResult {
                    user_id: user_id_str,
                    is_valid: false,
                    errors: vec![format!("Validation failed: {}", e)],
                    warnings: vec![],
                });
            }
        }
    }

    let valid_assignments = validation_results.iter().filter(|r| r.is_valid).count();
    let total_assignments = validation_results.len();

    let response = BulkValidateAssignmentResponse {
        profile_id: request.profile_id,
        profile_name: profile.name().to_string(),
        total_users: total_assignments,
        valid_assignments,
        invalid_assignments: total_assignments - valid_assignments,
        results: validation_results,
    };

    Ok(Json(response))
}

/// GET /admin/permission-profiles/tiers - Get available package tiers
pub async fn get_permission_profile_tiers_handler(
    State(_app_state): State<AppState>,
) -> Result<Json<Value>, AppError> {
    let tiers = json!({
        "tiers": [
            {
                "id": "free",
                "name": "Free",
                "description": "Free tier permissions"
            },
            {
                "id": "bronze",
                "name": "Bronze",
                "description": "Basic tier permissions"
            },
            {
                "id": "silver",
                "name": "Silver", 
                "description": "Silver tier permissions"
            },
            {
                "id": "gold",
                "name": "Gold",
                "description": "Gold tier permissions"
            },
            {
                "id": "platinum",
                "name": "Platinum",
                "description": "Platinum tier permissions"
            },
            {
                "id": "admin",
                "name": "Admin",
                "description": "Administrative tier permissions"
            },
            {
                "id": "superadmin",
                "name": "Super Admin",
                "description": "Super administrative tier permissions"
            }
        ]
    });

    Ok(Json(tiers))
}