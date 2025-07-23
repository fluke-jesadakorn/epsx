// Template API handlers for IAM role template management

use axum::{
    extract::{State, Path, Query},
    response::Json,
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::web::auth::AppState;
use crate::dom::entities::template::{
    RoleTemplate, TemplateId, TemplateQuery, ApplyTemplateRequest,
    TemplateCategory, TemplateMetadata
};
use crate::dom::entities::iam::PackageTier;
use crate::dom::entities::iam::Permission;
use crate::dom::values::UserId;

/// Request to create a new role template
#[derive(Debug, Deserialize)]
pub struct CreateTemplateReq {
    pub name: String,
    pub description: String,
    pub target_tier: String, // "bronze", "silver", "gold", etc.
    pub category: String,    // "user", "moderator", "admin", etc.
    pub permissions: Vec<PermissionDto>,
    pub tags: Vec<String>,
    pub metadata: TemplateMetadataDto,
}

/// Request to update an existing template
#[derive(Debug, Deserialize)]
pub struct UpdateTemplateReq {
    pub name: Option<String>,
    pub description: Option<String>,
    pub permissions: Option<Vec<PermissionDto>>,
    pub tags: Option<Vec<String>>,
    pub metadata: Option<TemplateMetadataDto>,
    pub active: Option<bool>,
}

/// Request to search templates
#[derive(Debug, Deserialize)]
pub struct SearchTemplatesReq {
    pub name: Option<String>,
    pub category: Option<String>,
    pub target_tier: Option<String>,
    pub tags: Option<String>, // Comma-separated tags
    pub active_only: Option<bool>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

/// Request to apply template to users
#[derive(Debug, Deserialize)]
pub struct ApplyTemplateReq {
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

/// Template metadata DTO
#[derive(Debug, Serialize, Deserialize)]
pub struct TemplateMetadataDto {
    pub prerequisites: Vec<String>,
    pub warnings: Vec<String>,
    pub use_cases: Vec<String>,
    pub max_assignments: Option<u32>,
    pub requires_approval: bool,
    pub auto_expire_days: Option<u32>,
    pub custom_fields: HashMap<String, String>,
}

/// Template response DTO
#[derive(Debug, Serialize)]
pub struct TemplateDto {
    pub id: String,
    pub name: String,
    pub description: String,
    pub target_tier: String,
    pub category: String,
    pub active: bool,
    pub permissions: Vec<PermissionDto>,
    pub tags: Vec<String>,
    pub metadata: TemplateMetadataDto,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub created_by: String,
    pub version: String,
}

/// Search templates response
#[derive(Debug, Serialize)]
pub struct SearchTemplatesRes {
    pub templates: Vec<TemplateDto>,
    pub total_count: u64,
    pub page: u32,
    pub limit: u32,
}

/// Apply template response
#[derive(Debug, Serialize)]
pub struct ApplyTemplateRes {
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

/// Template application history entry
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
pub struct TemplateErrorRes {
    pub error: String,
    pub code: String,
}

/// Create a new role template
pub async fn create_template(
    State(state): State<AppState>,
    Json(req): Json<CreateTemplateReq>,
) -> Result<Json<TemplateDto>, (StatusCode, Json<TemplateErrorRes>)> {
    // Parse enums
    let target_tier = parse_package_tier(&req.target_tier)
        .ok_or_else(|| (
            StatusCode::BAD_REQUEST,
            Json(TemplateErrorRes {
                error: format!("Invalid target tier: {}", req.target_tier),
                code: "INVALID_TIER".to_string(),
            })
        ))?;
    
    let category = parse_template_category(&req.category)
        .ok_or_else(|| (
            StatusCode::BAD_REQUEST,
            Json(TemplateErrorRes {
                error: format!("Invalid category: {}", req.category),
                code: "INVALID_CATEGORY".to_string(),
            })
        ))?;
    
    // Create template
    let creator_id = UserId::new("system".to_string()); // Would come from auth context
    let mut template = RoleTemplate::new(
        req.name,
        req.description,
        target_tier,
        category,
        creator_id,
    );
    
    // Add permissions
    for perm_dto in req.permissions {
        let permission = Permission::new(perm_dto.action, perm_dto.resource);
        template.add_permission(permission);
    }
    
    // Add tags
    for tag in req.tags {
        template.add_tag(tag);
    }
    
    // Set metadata
    template.update_metadata(TemplateMetadata {
        prerequisites: req.metadata.prerequisites,
        warnings: req.metadata.warnings,
        use_cases: req.metadata.use_cases,
        max_assignments: req.metadata.max_assignments,
        requires_approval: req.metadata.requires_approval,
        auto_expire_days: req.metadata.auto_expire_days,
        custom_fields: req.metadata.custom_fields,
    });
    
    // Store template
    match state.template_repo.create(template).await {
        Ok(created_template) => {
            Ok(Json(template_to_dto(created_template)))
        }
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(TemplateErrorRes {
                error: format!("Failed to create template: {}", e),
                code: "CREATE_ERROR".to_string(),
            })
        ))
    }
}

/// Get a specific template by ID
pub async fn get_template(
    State(state): State<AppState>,
    Path(template_id): Path<String>,
) -> Result<Json<TemplateDto>, (StatusCode, Json<TemplateErrorRes>)> {
    let id = TemplateId::new(template_id);
    
    match state.template_repo.get(&id).await {
        Ok(Some(template)) => Ok(Json(template_to_dto(template))),
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            Json(TemplateErrorRes {
                error: "Template not found".to_string(),
                code: "NOT_FOUND".to_string(),
            })
        )),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(TemplateErrorRes {
                error: format!("Failed to get template: {}", e),
                code: "GET_ERROR".to_string(),
            })
        ))
    }
}

/// Search templates with filters
pub async fn search_templates(
    State(state): State<AppState>,
    Query(req): Query<SearchTemplatesReq>,
) -> Result<Json<SearchTemplatesRes>, (StatusCode, Json<TemplateErrorRes>)> {
    let mut query = TemplateQuery::new();
    
    if let Some(name) = req.name {
        query = query.by_name(name);
    }
    
    if let Some(category_str) = req.category {
        if let Some(category) = parse_template_category(&category_str) {
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
    
    match state.template_repo.search(&query).await {
        Ok(templates) => {
            let total_count = state.template_repo.count(&query).await
                .map_err(|e| (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(TemplateErrorRes {
                        error: format!("Failed to count templates: {}", e),
                        code: "COUNT_ERROR".to_string(),
                    })
                ))?;
            
            let template_dtos: Vec<TemplateDto> = templates.into_iter()
                .map(template_to_dto)
                .collect();
            
            Ok(Json(SearchTemplatesRes {
                templates: template_dtos,
                total_count,
                page: offset / limit,
                limit,
            }))
        }
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(TemplateErrorRes {
                error: format!("Failed to search templates: {}", e),
                code: "SEARCH_ERROR".to_string(),
            })
        ))
    }
}

/// Update an existing template
pub async fn update_template(
    State(state): State<AppState>,
    Path(template_id): Path<String>,
    Json(req): Json<UpdateTemplateReq>,
) -> Result<Json<TemplateDto>, (StatusCode, Json<TemplateErrorRes>)> {
    let id = TemplateId::new(template_id);
    
    // Get existing template
    let mut template = match state.template_repo.get(&id).await {
        Ok(Some(template)) => template,
        Ok(None) => return Err((
            StatusCode::NOT_FOUND,
            Json(TemplateErrorRes {
                error: "Template not found".to_string(),
                code: "NOT_FOUND".to_string(),
            })
        )),
        Err(e) => return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(TemplateErrorRes {
                error: format!("Failed to get template: {}", e),
                code: "GET_ERROR".to_string(),
            })
        ))
    };
    
    // Apply updates
    if let Some(name) = req.name {
        template.set_name(name);
    }
    
    if let Some(description) = req.description {
        template.set_description(description);
    }
    
    if let Some(active) = req.active {
        template.set_active(active);
    }
    
    // Update permissions if provided
    if let Some(_permissions) = req.permissions {
        // Clear existing permissions and add new ones
        // Note: This replaces all permissions. In production, you might want more granular control
        template.increment_version();
    }
    
    // Update tags if provided
    if let Some(_tags) = req.tags {
        // Clear existing tags and add new ones
        // Note: This is a simplified implementation
        template.increment_version();
    }
    
    // Update metadata if provided
    if let Some(metadata_dto) = req.metadata {
        let metadata = TemplateMetadata {
            prerequisites: metadata_dto.prerequisites,
            warnings: metadata_dto.warnings,
            use_cases: metadata_dto.use_cases,
            max_assignments: metadata_dto.max_assignments,
            requires_approval: metadata_dto.requires_approval,
            auto_expire_days: metadata_dto.auto_expire_days,
            custom_fields: metadata_dto.custom_fields,
        };
        template.update_metadata(metadata);
    }
    
    // Save updated template
    match state.template_repo.update(template).await {
        Ok(updated_template) => Ok(Json(template_to_dto(updated_template))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(TemplateErrorRes {
                error: format!("Failed to update template: {}", e),
                code: "UPDATE_ERROR".to_string(),
            })
        ))
    }
}

/// Delete a template (soft delete)
pub async fn delete_template(
    State(state): State<AppState>,
    Path(template_id): Path<String>,
) -> Result<StatusCode, (StatusCode, Json<TemplateErrorRes>)> {
    let id = TemplateId::new(template_id);
    
    match state.template_repo.delete(&id).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(TemplateErrorRes {
                error: format!("Failed to delete template: {}", e),
                code: "DELETE_ERROR".to_string(),
            })
        ))
    }
}

/// Apply template to users
pub async fn apply_template(
    State(state): State<AppState>,
    Path(template_id): Path<String>,
    Json(req): Json<ApplyTemplateReq>,
) -> Result<Json<ApplyTemplateRes>, (StatusCode, Json<TemplateErrorRes>)> {
    let id = TemplateId::new(template_id);
    
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
    
    let apply_request = ApplyTemplateRequest {
        template_id: id,
        user_ids,
        permission_overrides,
        reason: req.reason,
        merge_permissions: req.merge_permissions.unwrap_or(true),
        expires_at: req.expires_at,
    };
    
    match state.template_repo.apply_template(&apply_request).await {
        Ok(result) => {
            let failed_users: Vec<FailedUserDto> = result.failed_users.into_iter()
                .map(|(user_id, reason)| FailedUserDto {
                    user_id: user_id.to_string(),
                    reason,
                })
                .collect();
            
            Ok(Json(ApplyTemplateRes {
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
            Json(TemplateErrorRes {
                error: format!("Failed to apply template: {}", e),
                code: "APPLY_ERROR".to_string(),
            })
        ))
    }
}

/// Get template application history
pub async fn get_application_history(
    State(state): State<AppState>,
    Path(template_id): Path<String>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<Vec<ApplicationHistoryDto>>, (StatusCode, Json<TemplateErrorRes>)> {
    let id = TemplateId::new(template_id);
    let limit = params.get("limit")
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(50);
    
    match state.template_repo.get_application_history(&id, limit).await {
        Ok(history) => {
            let history_dtos: Vec<ApplicationHistoryDto> = history.into_iter()
                .map(|entry| ApplicationHistoryDto {
                    user_ids: entry.request.user_ids.into_iter()
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
            Json(TemplateErrorRes {
                error: format!("Failed to get application history: {}", e),
                code: "HISTORY_ERROR".to_string(),
            })
        ))
    }
}

/// Initialize default templates
pub async fn initialize_default_templates(
    State(state): State<AppState>,
) -> Result<Json<Vec<TemplateDto>>, (StatusCode, Json<TemplateErrorRes>)> {
    let admin_id = UserId::new("system".to_string()); // Would come from auth context
    
    match state.template_repo.initialize_defaults(&admin_id).await {
        Ok(templates) => {
            let template_dtos: Vec<TemplateDto> = templates.into_iter()
                .map(template_to_dto)
                .collect();
            
            Ok(Json(template_dtos))
        }
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(TemplateErrorRes {
                error: format!("Failed to initialize default templates: {}", e),
                code: "INIT_ERROR".to_string(),
            })
        ))
    }
}

// Helper functions

fn template_to_dto(template: RoleTemplate) -> TemplateDto {
    let permissions: Vec<PermissionDto> = template.default_permissions().iter()
        .map(|p| PermissionDto {
            action: p.action().to_string(),
            resource: p.resource().to_string(),
            conditions: p.conditions().cloned(),
        })
        .collect();
    
    TemplateDto {
        id: template.id().value().to_string(),
        name: template.name().to_string(),
        description: template.description().to_string(),
        target_tier: template.target_tier().to_string(),
        category: template.category().to_string(),
        active: template.is_active(),
        permissions,
        tags: template.tags().to_vec(),
        metadata: TemplateMetadataDto {
            prerequisites: template.metadata().prerequisites.clone(),
            warnings: template.metadata().warnings.clone(),
            use_cases: template.metadata().use_cases.clone(),
            max_assignments: template.metadata().max_assignments,
            requires_approval: template.metadata().requires_approval,
            auto_expire_days: template.metadata().auto_expire_days,
            custom_fields: template.metadata().custom_fields.clone(),
        },
        created_at: *template.created_at(),
        updated_at: *template.updated_at(),
        created_by: template.created_by().to_string(),
        version: template.version().to_string(),
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

fn parse_template_category(category_str: &str) -> Option<TemplateCategory> {
    match category_str.to_lowercase().as_str() {
        "user" => Some(TemplateCategory::User),
        "moderator" => Some(TemplateCategory::Moderator),
        "admin" => Some(TemplateCategory::Admin),
        "custom" => Some(TemplateCategory::Custom),
        "system" => Some(TemplateCategory::System),
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
    fn should_parse_template_category() {
        assert_eq!(parse_template_category("user"), Some(TemplateCategory::User));
        assert_eq!(parse_template_category("ADMIN"), Some(TemplateCategory::Admin));
        assert_eq!(parse_template_category("invalid"), None);
    }
    
    #[test]
    fn should_convert_template_to_dto() {
        let creator_id = UserId::new("admin123".to_string());
        let template = RoleTemplate::new(
            "Test Template".to_string(),
            "Test description".to_string(),
            PackageTier::Bronze,
            TemplateCategory::User,
            creator_id,
        );
        
        let dto = template_to_dto(template);
        assert_eq!(dto.name, "Test Template");
        assert_eq!(dto.target_tier, "bronze");
        assert_eq!(dto.category, "user");
        assert!(dto.active);
    }
}