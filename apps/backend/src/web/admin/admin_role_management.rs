use axum::{
    extract::{Path, Query, State, Extension},
    response::Json,
};
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

use crate::{
    core::errors::AppError,
    dom::services::admin_module_service::ModuleAssignmentRequest,
    web::auth::{casbin_claims_mapper::CasbinUserClaims, AppState},
};

#[derive(Debug, Serialize, Deserialize)]
pub struct AdminModuleResponse {
    pub code: String,
    pub name: String,
    pub description: String,
    pub category: String,
    pub icon: Option<String>,
    pub color: Option<String>,
    pub sort_order: i32,
    pub is_active: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserAdminModulesResponse {
    pub firebase_uid: String,
    pub modules: Vec<String>,
    pub module_details: Vec<AdminModuleResponse>,
    pub is_admin: bool,
    pub total_modules: usize,
}

#[derive(Debug, Deserialize)]
pub struct AssignModulesRequest {
    pub firebase_uid: String,
    pub module_codes: Vec<String>,
    pub granted_reason: Option<String>,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Deserialize)]
pub struct RevokeModulesRequest {
    pub firebase_uid: String,
    pub module_codes: Vec<String>,
    pub reason: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct AdminRoleOperationResponse {
    pub success: bool,
    pub message: String,
    pub affected_modules: Vec<String>,
    pub firebase_uid: String,
}

#[derive(Debug, Deserialize)]
pub struct AdminRoleQueryParams {
    pub limit: Option<i32>,
    pub offset: Option<i32>,
    pub module_filter: Option<String>,
}

/// Get all available admin modules
pub async fn get_all_admin_modules(
    State(app_state): State<AppState>,
) -> Result<Json<Vec<AdminModuleResponse>>, AppError> {
    let admin_module_service = &app_state.admin_module_service;
    info!("Fetching all admin modules");

    let modules = admin_module_service.get_all_admin_modules().await?;

    let response: Vec<AdminModuleResponse> = modules
        .into_iter()
        .map(|module| AdminModuleResponse {
            code: module.module_code,
            name: module.module_name,
            description: module.description,
            category: module.category,
            icon: module.icon,
            color: module.color,
            sort_order: module.sort_order,
            is_active: module.is_active,
        })
        .collect();

    info!("Retrieved {} admin modules", response.len());
    Ok(Json(response))
}

/// Get user's assigned admin modules
pub async fn get_user_admin_modules(
    State(app_state): State<AppState>,
    Path(firebase_uid): Path<String>,
) -> Result<Json<UserAdminModulesResponse>, AppError> {
    let admin_module_service = &app_state.admin_module_service;
    info!("Fetching admin modules for user: {}", firebase_uid);

    // Get user's module codes
    let user_modules = admin_module_service.get_user_admin_modules(&firebase_uid).await?;
    
    // Get all available modules for details
    let all_modules = admin_module_service.get_all_admin_modules().await?;
    
    // Filter to only user's assigned modules
    let module_details: Vec<AdminModuleResponse> = all_modules
        .into_iter()
        .filter(|module| user_modules.contains(&module.module_code))
        .map(|module| AdminModuleResponse {
            code: module.module_code,
            name: module.module_name,
            description: module.description,
            category: module.category,
            icon: module.icon,
            color: module.color,
            sort_order: module.sort_order,
            is_active: module.is_active,
        })
        .collect();

    let is_admin = !user_modules.is_empty();

    let response = UserAdminModulesResponse {
        firebase_uid: firebase_uid.clone(),
        modules: user_modules.clone(),
        module_details,
        is_admin,
        total_modules: user_modules.len(),
    };

    info!("User {} has {} admin modules", firebase_uid, response.total_modules);
    Ok(Json(response))
}

/// Assign admin modules to a user
pub async fn assign_admin_modules(
    State(app_state): State<AppState>,
    Extension(claims): Extension<CasbinUserClaims>,
    Json(request): Json<AssignModulesRequest>,
) -> Result<Json<AdminRoleOperationResponse>, AppError> {
    let admin_module_service = &app_state.admin_module_service;
    info!("Assigning admin modules to user: {} by admin: {}", request.firebase_uid, claims.user_id);

    // Ensure the admin has permission to assign these modules
    let admin_modules = admin_module_service.get_user_admin_modules(&claims.user_id).await?;
    
    // Check if admin has 'user_operations' or 'permission_admin' module
    let can_assign = admin_modules.contains(&"user_operations".to_string()) ||
                     admin_modules.contains(&"permission_admin".to_string()) ||
                     admin_modules.contains(&"system_admin".to_string());
    
    if !can_assign {
        warn!("Admin {} attempted to assign modules without proper permissions", claims.user_id);
        return Err(AppError::Unauthorized("Insufficient permissions to assign admin modules"));
    }

    let assignment_request = ModuleAssignmentRequest {
        firebase_uid: request.firebase_uid.clone(),
        module_codes: request.module_codes.clone(),
        granted_by: claims.user_id,
        granted_reason: request.granted_reason.unwrap_or_else(|| 
            format!("Module assignment by admin via API")
        ),
        expires_at: request.expires_at,
    };

    let assigned_modules = admin_module_service.assign_admin_modules(&assignment_request).await?;

    let firebase_uid = request.firebase_uid.clone();
    let response = AdminRoleOperationResponse {
        success: true,
        message: format!("Successfully assigned {} modules", assigned_modules.len()),
        affected_modules: assigned_modules,
        firebase_uid: request.firebase_uid,
    };

    info!("Successfully assigned modules to user: {}", firebase_uid);
    Ok(Json(response))
}

/// Revoke admin modules from a user
pub async fn revoke_admin_modules(
    State(app_state): State<AppState>,
    Extension(claims): Extension<CasbinUserClaims>,
    Json(request): Json<RevokeModulesRequest>,
) -> Result<Json<AdminRoleOperationResponse>, AppError> {
    let admin_module_service = &app_state.admin_module_service;
    info!("Revoking admin modules from user: {} by admin: {}", request.firebase_uid, claims.user_id);

    // Ensure the admin has permission to revoke these modules
    let admin_modules = admin_module_service.get_user_admin_modules(&claims.user_id).await?;
    
    let can_revoke = admin_modules.contains(&"user_operations".to_string()) ||
                     admin_modules.contains(&"permission_admin".to_string()) ||
                     admin_modules.contains(&"system_admin".to_string());
    
    if !can_revoke {
        warn!("Admin {} attempted to revoke modules without proper permissions", claims.user_id);
        return Err(AppError::Unauthorized("Insufficient permissions to revoke admin modules"));
    }

    // Prevent admins from revoking their own admin privileges
    if request.firebase_uid == claims.user_id {
        warn!("Admin {} attempted to revoke their own admin modules", claims.user_id);
        return Err(AppError::BadRequest("Cannot revoke your own admin modules"));
    }

    let revoked_modules = admin_module_service.revoke_admin_modules(
        &request.firebase_uid,
        request.module_codes.clone(),
        &claims.user_id,
        &request.reason.unwrap_or_else(|| "Module revocation by admin via API".to_string()),
    ).await?;

    let firebase_uid = request.firebase_uid.clone();
    let response = AdminRoleOperationResponse {
        success: true,
        message: format!("Successfully revoked {} modules", revoked_modules.len()),
        affected_modules: revoked_modules,
        firebase_uid: request.firebase_uid,
    };

    info!("Successfully revoked modules from user: {}", firebase_uid);
    Ok(Json(response))
}

/// Bulk assign ALL admin modules to a user (super admin creation)
pub async fn assign_all_admin_modules(
    State(app_state): State<AppState>,
    Path(firebase_uid): Path<String>,
    Extension(claims): Extension<CasbinUserClaims>,
) -> Result<Json<AdminRoleOperationResponse>, AppError> {
    let admin_module_service = &app_state.admin_module_service;
    info!("Assigning ALL admin modules to user: {} by super admin: {}", firebase_uid, claims.user_id);

    // Only allow users with 'system_admin' module to create super admins
    let admin_modules = admin_module_service.get_user_admin_modules(&claims.user_id).await?;
    
    if !admin_modules.contains(&"system_admin".to_string()) {
        warn!("User {} attempted to create super admin without system_admin module", claims.user_id);
        return Err(AppError::Unauthorized("Only system administrators can assign all admin modules"));
    }

    let assigned_modules = admin_module_service.assign_all_admin_modules(
        &firebase_uid,
        &claims.user_id,
        "Full admin module assignment via API by system administrator"
    ).await?;

    let response = AdminRoleOperationResponse {
        success: true,
        message: format!("Successfully assigned ALL {} admin modules - user is now a super admin", assigned_modules.len()),
        affected_modules: assigned_modules,
        firebase_uid,
    };

    info!("Successfully created super admin user");
    Ok(Json(response))
}

/// Get admin role audit trail for a user
pub async fn get_admin_role_audit(
    State(app_state): State<AppState>,
    Path(firebase_uid): Path<String>,
    Query(params): Query<AdminRoleQueryParams>,
    Extension(claims): Extension<CasbinUserClaims>,
) -> Result<Json<Vec<serde_json::Value>>, AppError> {
    let admin_module_service = &app_state.admin_module_service;
    info!("Fetching admin role audit for user: {} by admin: {}", firebase_uid, claims.user_id);

    // Ensure the admin has permission to view audit trails
    let admin_modules = admin_module_service.get_user_admin_modules(&claims.user_id).await?;
    
    let can_view_audit = admin_modules.contains(&"compliance_audit".to_string()) ||
                         admin_modules.contains(&"system_admin".to_string());
    
    if !can_view_audit {
        warn!("Admin {} attempted to view audit trail without proper permissions", claims.user_id);
        return Err(AppError::Unauthorized("Insufficient permissions to view admin role audit"));
    }

    let limit = params.limit.or(Some(50)).map(|l| l.min(500)); // Max 500 records
    let audit_records = admin_module_service.get_admin_role_audit(&firebase_uid, limit).await?;

    info!("Retrieved {} audit records for user: {}", audit_records.len(), firebase_uid);
    Ok(Json(audit_records))
}

/// Check if user has specific admin module access
pub async fn check_admin_module_access(
    State(app_state): State<AppState>,
    Path((firebase_uid, module_code)): Path<(String, String)>,
) -> Result<Json<serde_json::Value>, AppError> {
    let admin_module_service = &app_state.admin_module_service;
    info!("Checking admin module access: user={}, module={}", firebase_uid, module_code);

    let has_access = admin_module_service.user_has_admin_module(&firebase_uid, &module_code).await?;

    let response = serde_json::json!({
        "firebase_uid": firebase_uid,
        "module_code": module_code,
        "has_access": has_access,
        "checked_at": chrono::Utc::now()
    });

    Ok(Json(response))
}

/// Get detailed admin module assignments for a user
pub async fn get_user_admin_module_details(
    State(app_state): State<AppState>,
    Path(firebase_uid): Path<String>,
    Extension(claims): Extension<CasbinUserClaims>,
) -> Result<Json<Vec<serde_json::Value>>, AppError> {
    let admin_module_service = &app_state.admin_module_service;
    info!("Fetching detailed admin module assignments for user: {} by admin: {}", firebase_uid, claims.user_id);

    // Ensure the admin can view detailed assignments
    let admin_modules = admin_module_service.get_user_admin_modules(&claims.user_id).await?;
    
    let can_view_details = admin_modules.contains(&"user_operations".to_string()) ||
                           admin_modules.contains(&"permission_admin".to_string()) ||
                           admin_modules.contains(&"system_admin".to_string());
    
    if !can_view_details {
        warn!("Admin {} attempted to view detailed assignments without proper permissions", claims.user_id);
        return Err(AppError::Unauthorized("Insufficient permissions to view detailed admin module assignments"));
    }

    let assignments = admin_module_service.get_user_admin_module_details(&firebase_uid).await?;

    let response: Vec<serde_json::Value> = assignments
        .into_iter()
        .map(|assignment| serde_json::json!({
            "id": assignment.id,
            "firebase_uid": assignment.firebase_uid,
            "module_code": assignment.module_code,
            "granted_by": assignment.granted_by,
            "granted_reason": assignment.granted_reason,
            "expires_at": assignment.expires_at,
            "is_active": assignment.is_active,
            "assignment_metadata": assignment.assignment_metadata,
            "created_at": assignment.created_at,
            "updated_at": assignment.updated_at
        }))
        .collect();

    info!("Retrieved {} detailed assignments for user: {}", response.len(), firebase_uid);
    Ok(Json(response))
}

/// Get current authenticated user's admin modules
pub async fn get_current_user_admin_modules(
    State(app_state): State<AppState>,
    Extension(claims): Extension<CasbinUserClaims>,
) -> Result<Json<UserAdminModulesResponse>, AppError> {
    let admin_module_service = &app_state.admin_module_service;
    let firebase_uid = &claims.user_id;
    info!("Fetching admin modules for current authenticated user: {}", firebase_uid);

    // Get user's module codes
    let user_modules = admin_module_service.get_user_admin_modules(firebase_uid).await?;
    
    // Get all available modules for details
    let all_modules = admin_module_service.get_all_admin_modules().await?;
    
    // Filter to only user's assigned modules
    let module_details: Vec<AdminModuleResponse> = all_modules
        .into_iter()
        .filter(|module| user_modules.contains(&module.module_code))
        .map(|module| AdminModuleResponse {
            code: module.module_code,
            name: module.module_name,
            description: module.description,
            category: module.category,
            icon: module.icon,
            color: module.color,
            sort_order: module.sort_order,
            is_active: module.is_active,
        })
        .collect();

    let is_admin = !user_modules.is_empty();

    let response = UserAdminModulesResponse {
        firebase_uid: firebase_uid.clone(),
        modules: user_modules.clone(),
        module_details,
        is_admin,
        total_modules: user_modules.len(),
    };

    info!("Current user {} has {} admin modules", firebase_uid, response.total_modules);
    Ok(Json(response))
}