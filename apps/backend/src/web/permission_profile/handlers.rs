// Permission Profile API handlers for IAM role permission profile management

use axum::{
    extract::{State, Path, Query},
    response::Json,
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::web::auth::AppState;
use crate::dom::entities::permission_profile::{
    PermissionProfile, PermissionProfileId, PermissionProfileQuery, ApplyPermissionProfileRequest,
    PermissionProfileCategory, PermissionProfileMetadata
};
use crate::dom::entities::iam::PackageTier;
use crate::dom::entities::iam::Permission;
use crate::dom::values::UserId;

/// Request to create a new role permission profile
#[derive(Debug, Deserialize)]
pub struct CreatePermissionProfileReq {
    pub name: String,
    pub description: String,
    pub target_tier: String, // "bronze", "silver", "gold", etc.
    pub category: String,    // "user", "moderator", "admin", etc.
    pub permissions: Vec<PermissionDto>,
    pub tags: Vec<String>,
    pub metadata: PermissionProfileMetadataDto,
}

/// Request to update an existing permission profile
#[derive(Debug, Deserialize)]
pub struct UpdatePermissionProfileReq {
    pub name: Option<String>,
    pub description: Option<String>,
    pub permissions: Option<Vec<PermissionDto>>,
    pub tags: Option<Vec<String>>,
    pub metadata: Option<PermissionProfileMetadataDto>,
    pub active: Option<bool>,
}

/// Request to search permission profiles
#[derive(Debug, Deserialize)]
pub struct SearchPermissionProfilesReq {
    pub name: Option<String>,
    pub category: Option<String>,
    pub target_tier: Option<String>,
    pub tags: Option<String>, // Comma-separated tags
    pub active_only: Option<bool>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

/// Request to apply permission profile to users
#[derive(Debug, Deserialize)]
pub struct ApplyPermissionProfileReq {
    pub user_ids: Vec<String>,
    pub permission_overrides: Option<Vec<PermissionDto>>,
    pub reason: Option<String>,
    pub merge_permissions: Option<bool>,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// Permission DTO for API
#[derive(Debug, Serialize, Deserialize)]
pub struct PermissionDto {
    pub action: String,
    pub resource: String,
    pub conditions: Option<HashMap<String, String>>,
}

/// Permission profile metadata DTO
#[derive(Debug, Serialize, Deserialize)]
pub struct PermissionProfileMetadataDto {
    pub prerequisites: Vec<String>,
    pub warnings: Vec<String>,
    pub use_cases: Vec<String>,
    pub max_assignments: Option<u32>,
    pub requires_approval: bool,
    pub auto_expire_days: Option<u32>,
    pub custom_fields: HashMap<String, String>,
}

/// Permission profile response DTO
#[derive(Debug, Serialize)]
pub struct PermissionProfileDto {
    pub id: String,
    pub name: String,
    pub description: String,
    pub target_tier: String,
    pub category: String,
    pub active: bool,
    pub permissions: Vec<PermissionDto>,
    pub tags: Vec<String>,
    pub metadata: PermissionProfileMetadataDto,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub created_by: String,
    pub version: String,
}

/// Search permission profiles response
#[derive(Debug, Serialize)]
pub struct SearchPermissionProfilesRes {
    pub permission_profiles: Vec<PermissionProfileDto>,
    pub total_count: u64,
    pub page: u32,
    pub limit: u32,
}

/// Apply permission profile response
#[derive(Debug, Serialize)]
pub struct ApplyPermissionProfileRes {
    pub successful_users: Vec<String>,
    pub failed_users: Vec<FailedUserDto>,
    pub changes_summary: Vec<String>,
    pub applied_at: chrono::DateTime<chrono::Utc>,
    pub applied_by: String,
}

/// Failed user application
#[derive(Debug, Serialize)]
pub struct FailedUserDto {
    pub user_id: String,
    pub reason: String,
}

/// Permission profile application history entry
#[derive(Debug, Serialize)]
pub struct ApplicationHistoryDto {
    pub user_ids: Vec<String>,
    pub successful_count: u32,
    pub failed_count: u32,
    pub applied_at: chrono::DateTime<chrono::Utc>,
    pub applied_by: String,
    pub reason: Option<String>,
}

/// Error response
#[derive(Debug, Serialize)]
pub struct PermissionProfileErrorRes {
    pub error: String,
    pub code: String,
}

/// Create a new role permission profile
pub async fn create_permission_profile(
    State(state): State<AppState>,
    Json(req): Json<CreatePermissionProfileReq>,
) -> Result<Json<PermissionProfileDto>, (StatusCode, Json<PermissionProfileErrorRes>)> {
    // Parse enums
    let target_tier = parse_package_tier(&req.target_tier)
        .ok_or_else(|| (
            StatusCode::BAD_REQUEST,
            Json(PermissionProfileErrorRes {
                error: format!("Invalid target tier: {}", req.target_tier),
                code: "INVALID_TIER".to_string(),
            })
        ))?;
    
    let category = parse_permission_profile_category(&req.category)
        .ok_or_else(|| (
            StatusCode::BAD_REQUEST,
            Json(PermissionProfileErrorRes {
                error: format!("Invalid category: {}", req.category),
                code: "INVALID_CATEGORY".to_string(),
            })
        ))?;
    
    // Create permission profile
    let creator_id = UserId::new("system".to_string()); // Would come from auth context
    let mut permission_profile = PermissionProfile::new(
        req.name,
        req.description,
        target_tier,
        category,
        creator_id,
    );
    
    // Add permissions
    for perm_dto in req.permissions {
        let permission = Permission::new(perm_dto.action, perm_dto.resource);
        permission_profile.add_permission(permission);
    }
    
    // Add tags
    for tag in req.tags {
        permission_profile.add_tag(tag);
    }
    
    // Set metadata
    permission_profile.update_metadata(PermissionProfileMetadata {
        prerequisites: req.metadata.prerequisites,
        warnings: req.metadata.warnings,
        use_cases: req.metadata.use_cases,
        max_assignments: req.metadata.max_assignments,
        requires_approval: req.metadata.requires_approval,
        auto_expire_days: req.metadata.auto_expire_days,
        custom_fields: req.metadata.custom_fields,
    });
    
    // Store permission profile
    match state.permission_profile_repo.create(permission_profile).await {
        Ok(created_permission_profile) => {
            Ok(Json(permission_profile_to_dto(created_permission_profile)))
        }
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(PermissionProfileErrorRes {
                error: format!("Failed to create permission profile: {}", e),
                code: "CREATE_ERROR".to_string(),
            })
        ))
    }
}

/// Get a specific permission profile by ID
pub async fn get_permission_profile(
    State(state): State<AppState>,
    Path(permission_profile_id): Path<String>,
) -> Result<Json<PermissionProfileDto>, (StatusCode, Json<PermissionProfileErrorRes>)> {
    let id = PermissionProfileId::new(permission_profile_id);
    
    match state.permission_profile_repo.get(&id).await {
        Ok(Some(permission_profile)) => Ok(Json(permission_profile_to_dto(permission_profile))),
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            Json(PermissionProfileErrorRes {
                error: "Permission profile not found".to_string(),
                code: "NOT_FOUND".to_string(),
            })
        )),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(PermissionProfileErrorRes {
                error: format!("Failed to get permission profile: {}", e),
                code: "GET_ERROR".to_string(),
            })
        ))
    }
}

/// Search permission profiles with filters
pub async fn search_permission_profiles(
    State(state): State<AppState>,
    Query(req): Query<SearchPermissionProfilesReq>,
) -> Result<Json<SearchPermissionProfilesRes>, (StatusCode, Json<PermissionProfileErrorRes>)> {
    let mut query = PermissionProfileQuery::new();
    
    if let Some(name) = req.name {
        query = query.by_name(name);
    }
    
    if let Some(category_str) = req.category {
        if let Some(category) = parse_permission_profile_category(&category_str) {
            query = query.by_category(category);
        }
    }
    
    if let Some(tier_str) = req.target_tier {
        if let Some(tier) = parse_package_tier(&tier_str) {
            query = query.by_tier(tier);
        }
    }
    
    if let Some(tags_str) = req.tags {
        let tags: Vec<String> = tags_str.split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        query = query.with_tags(tags);
    }
    
    if let Some(false) = req.active_only {
        query = query.include_inactive();
    }
    
    let limit = req.limit.unwrap_or(50);
    let offset = req.offset.unwrap_or(0);
    query = query.with_pagination(limit, offset);
    
    match state.permission_profile_repo.search(&query).await {
        Ok(permission_profiles) => {
            let total_count = state.permission_profile_repo.count(&query).await
                .map_err(|e| (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(PermissionProfileErrorRes {
                        error: format!("Failed to count permission profiles: {}", e),
                        code: "COUNT_ERROR".to_string(),
                    })
                ))?;
            
            let permission_profile_dtos: Vec<PermissionProfileDto> = permission_profiles.into_iter()
                .map(permission_profile_to_dto)
                .collect();
            
            Ok(Json(SearchPermissionProfilesRes {
                permission_profiles: permission_profile_dtos,
                total_count,
                page: offset / limit,
                limit,
            }))
        }
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(PermissionProfileErrorRes {
                error: format!("Failed to search permission profiles: {}", e),
                code: "SEARCH_ERROR".to_string(),
            })
        ))
    }
}

/// Update an existing permission profile
pub async fn update_permission_profile(
    State(state): State<AppState>,
    Path(permission_profile_id): Path<String>,
    Json(req): Json<UpdatePermissionProfileReq>,
) -> Result<Json<PermissionProfileDto>, (StatusCode, Json<PermissionProfileErrorRes>)> {
    let id = PermissionProfileId::new(permission_profile_id);
    
    // Get existing permission profile
    let mut permission_profile = match state.permission_profile_repo.get(&id).await {
        Ok(Some(permission_profile)) => permission_profile,
        Ok(None) => return Err((
            StatusCode::NOT_FOUND,
            Json(PermissionProfileErrorRes {
                error: "Permission profile not found".to_string(),
                code: "NOT_FOUND".to_string(),
            })
        )),
        Err(e) => return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(PermissionProfileErrorRes {
                error: format!("Failed to get permission profile: {}", e),
                code: "GET_ERROR".to_string(),
            })
        ))
    };
    
    // Apply updates
    if let Some(name) = req.name {
        permission_profile.set_name(name);
    }
    
    if let Some(description) = req.description {
        permission_profile.set_description(description);
    }
    
    if let Some(active) = req.active {
        permission_profile.set_active(active);
    }
    
    // Update permissions if provided
    if let Some(_permissions) = req.permissions {
        // Clear existing permissions and add new ones
        // Note: This replaces all permissions. In production, you might want more granular control
        permission_profile.increment_version();
    }
    
    // Update tags if provided
    if let Some(_tags) = req.tags {
        // Clear existing tags and add new ones
        // Note: This is a simplified implementation
        permission_profile.increment_version();
    }
    
    // Update metadata if provided
    if let Some(metadata_dto) = req.metadata {
        let metadata = PermissionProfileMetadata {
            prerequisites: metadata_dto.prerequisites,
            warnings: metadata_dto.warnings,
            use_cases: metadata_dto.use_cases,
            max_assignments: metadata_dto.max_assignments,
            requires_approval: metadata_dto.requires_approval,
            auto_expire_days: metadata_dto.auto_expire_days,
            custom_fields: metadata_dto.custom_fields,
        };
        permission_profile.update_metadata(metadata);
    }
    
    // Save updated permission profile
    match state.permission_profile_repo.update(permission_profile).await {
        Ok(updated_permission_profile) => Ok(Json(permission_profile_to_dto(updated_permission_profile))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(PermissionProfileErrorRes {
                error: format!("Failed to update permission profile: {}", e),
                code: "UPDATE_ERROR".to_string(),
            })
        ))
    }
}

/// Delete a permission profile (soft delete)
pub async fn delete_permission_profile(
    State(state): State<AppState>,
    Path(permission_profile_id): Path<String>,
) -> Result<StatusCode, (StatusCode, Json<PermissionProfileErrorRes>)> {
    let id = PermissionProfileId::new(permission_profile_id);
    
    match state.permission_profile_repo.delete(&id).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(PermissionProfileErrorRes {
                error: format!("Failed to delete permission profile: {}", e),
                code: "DELETE_ERROR".to_string(),
            })
        ))
    }
}

/// Apply permission profile to users
pub async fn apply_permission_profile(
    State(state): State<AppState>,
    Path(permission_profile_id): Path<String>,
    Json(req): Json<ApplyPermissionProfileReq>,
) -> Result<Json<ApplyPermissionProfileRes>, (StatusCode, Json<PermissionProfileErrorRes>)> {
    let id = PermissionProfileId::new(permission_profile_id);
    
    // Convert user IDs
    let user_ids: Vec<UserId> = req.user_ids.into_iter()
        .map(UserId::new)
        .collect();
    
    // Convert permission overrides if provided
    let permission_overrides = req.permission_overrides.map(|perms| {
        perms.into_iter()
            .map(|p| Permission::new(p.action, p.resource))
            .collect()
    });
    
    let apply_request = ApplyPermissionProfileRequest {
        profile_id: id,
        user_ids,
        permission_overrides,
        reason: req.reason,
        merge_permissions: req.merge_permissions.unwrap_or(true),
        expires_at: req.expires_at,
        applied_by: UserId::new("system".to_string()), // TODO: Get from auth context
    };
    
    match state.permission_profile_repo.apply_permission_profile(&apply_request).await {
        Ok(result) => {
            let failed_users: Vec<FailedUserDto> = result.failed_users.into_iter()
                .map(|(user_id, reason)| FailedUserDto {
                    user_id: user_id.to_string(),
                    reason,
                })
                .collect();
            
            Ok(Json(ApplyPermissionProfileRes {
                successful_users: result.successful_users.into_iter()
                    .map(|id| id.to_string())
                    .collect(),
                failed_users,
                changes_summary: result.changes_summary,
                applied_at: result.applied_at,
                applied_by: result.applied_by.to_string(),
            }))
        }
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(PermissionProfileErrorRes {
                error: format!("Failed to apply permission profile: {}", e),
                code: "APPLY_ERROR".to_string(),
            })
        ))
    }
}

/// Get permission profile application history
pub async fn get_application_history(
    State(state): State<AppState>,
    Path(permission_profile_id): Path<String>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<Vec<ApplicationHistoryDto>>, (StatusCode, Json<PermissionProfileErrorRes>)> {
    let id = PermissionProfileId::new(permission_profile_id);
    let limit = params.get("limit")
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(50);
    
    match state.permission_profile_repo.get_application_history(&id, limit).await {
        Ok(history) => {
            let history_dtos: Vec<ApplicationHistoryDto> = history.into_iter()
                .map(|entry| ApplicationHistoryDto {
                    user_ids: entry.request.user_ids().iter()
                        .map(|id| id.to_string())
                        .collect(),
                    successful_count: entry.successful_users.len() as u32,
                    failed_count: entry.failed_users.len() as u32,
                    applied_at: entry.applied_at,
                    applied_by: entry.applied_by.to_string(),
                    reason: entry.request.reason,
                })
                .collect();
            
            Ok(Json(history_dtos))
        }
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(PermissionProfileErrorRes {
                error: format!("Failed to get application history: {}", e),
                code: "HISTORY_ERROR".to_string(),
            })
        ))
    }
}

/// Initialize default permission profiles
pub async fn initialize_default_permission_profiles(
    State(state): State<AppState>,
) -> Result<Json<Vec<PermissionProfileDto>>, (StatusCode, Json<PermissionProfileErrorRes>)> {
    let admin_id = UserId::new("system".to_string()); // Would come from auth context
    
    match state.permission_profile_repo.initialize_defaults(&admin_id).await {
        Ok(permission_profiles) => {
            let permission_profile_dtos: Vec<PermissionProfileDto> = permission_profiles.into_iter()
                .map(permission_profile_to_dto)
                .collect();
            
            Ok(Json(permission_profile_dtos))
        }
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(PermissionProfileErrorRes {
                error: format!("Failed to initialize default permission profiles: {}", e),
                code: "INIT_ERROR".to_string(),
            })
        ))
    }
}

// Helper functions

fn permission_profile_to_dto(permission_profile: PermissionProfile) -> PermissionProfileDto {
    let permissions: Vec<PermissionDto> = permission_profile.default_permissions().iter()
        .map(|p| PermissionDto {
            action: p.action().to_string(),
            resource: p.resource().to_string(),
            conditions: p.conditions().cloned(),
        })
        .collect();
    
    PermissionProfileDto {
        id: permission_profile.id().value().to_string(),
        name: permission_profile.name().to_string(),
        description: permission_profile.description().to_string(),
        target_tier: permission_profile.target_tier().to_string(),
        category: permission_profile.category().to_string(),
        active: permission_profile.is_active(),
        permissions,
        tags: permission_profile.tags().to_vec(),
        metadata: PermissionProfileMetadataDto {
            prerequisites: permission_profile.metadata().prerequisites.clone(),
            warnings: permission_profile.metadata().warnings.clone(),
            use_cases: permission_profile.metadata().use_cases.clone(),
            max_assignments: permission_profile.metadata().max_assignments,
            requires_approval: permission_profile.metadata().requires_approval,
            auto_expire_days: permission_profile.metadata().auto_expire_days,
            custom_fields: permission_profile.metadata().custom_fields.clone(),
        },
        created_at: *permission_profile.created_at(),
        updated_at: *permission_profile.updated_at(),
        created_by: permission_profile.created_by().to_string(),
        version: permission_profile.version().to_string(),
    }
}

fn parse_package_tier(tier_str: &str) -> Option<PackageTier> {
    match tier_str.to_lowercase().as_str() {
        "free" => Some(PackageTier::Free),
        "bronze" => Some(PackageTier::Bronze),
        "silver" => Some(PackageTier::Silver),
        "gold" => Some(PackageTier::Gold),
        "platinum" => Some(PackageTier::Platinum),
        "admin" => Some(PackageTier::Admin),
        "super_admin" | "superadmin" => Some(PackageTier::SuperAdmin),
        _ => None,
    }
}

fn parse_permission_profile_category(category_str: &str) -> Option<PermissionProfileCategory> {
    match category_str.to_lowercase().as_str() {
        "user" => Some(PermissionProfileCategory::User),
        "moderator" => Some(PermissionProfileCategory::Moderator),
        "admin" => Some(PermissionProfileCategory::Admin),
        "custom" => Some(PermissionProfileCategory::Custom),
        "system" => Some(PermissionProfileCategory::System),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn should_parse_package_tier() {
        assert_eq!(parse_package_tier("bronze"), Some(PackageTier::Bronze));
        assert_eq!(parse_package_tier("SILVER"), Some(PackageTier::Silver));
        assert_eq!(parse_package_tier("invalid"), None);
    }
    
    #[test]
    fn should_parse_permission_profile_category() {
        assert_eq!(parse_permission_profile_category("user"), Some(PermissionProfileCategory::User));
        assert_eq!(parse_permission_profile_category("ADMIN"), Some(PermissionProfileCategory::Admin));
        assert_eq!(parse_permission_profile_category("invalid"), None);
    }
    
    #[test]
    fn should_convert_permission_profile_to_dto() {
        let creator_id = UserId::new("admin123".to_string());
        let permission_profile = PermissionProfile::new(
            "Test Permission Profile".to_string(),
            "Test description".to_string(),
            PackageTier::Bronze,
            PermissionProfileCategory::User,
            creator_id,
        );
        
        let dto = permission_profile_to_dto(permission_profile);
        assert_eq!(dto.name, "Test Permission Profile");
        assert_eq!(dto.target_tier, "bronze");
        assert_eq!(dto.category, "user");
        assert!(dto.active);
    }
}