use axum::{
    extract::{Path, Query, State, Extension},
    response::Json,
};
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

use crate::{
    core::errors::AppError,
    dom::services::admin_module_service::AdminModuleAssignRequest,
    web::auth::AppState,
};

// Temporary replacement for CasbinUserClaims
#[derive(Debug, Clone)]
pub struct ModernUserClaims {
    pub user_id: String,
    pub email: String,
}

impl Default for ModernUserClaims {
    fn default() -> Self {
        Self {
            user_id: "system".to_string(),
            email: "system@epsx.com".to_string(),
        }
    }
}

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

    let modules = admin_module_service.get_all_admin_modules().await
        .map_err(|e| AppError::new(crate::core::errors::ErrorKind::DatabaseError, e))?;

    let response: Vec<AdminModuleResponse> = modules
        .into_iter()
        .map(|module_name| AdminModuleResponse {
            code: module_name.clone(),
            name: module_name.clone(),
            description: format!("Admin access for {}", module_name),
            category: "admin".to_string(),
            icon: Some("settings".to_string()),
            color: Some("#6B7280".to_string()),
            sort_order: 1,
            is_active: true,
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
    let user_modules = admin_module_service.get_user_admin_modules(&firebase_uid).await
        .map_err(|e| AppError::new(crate::core::errors::ErrorKind::DatabaseError, e))?;
    
    // Get all available modules for details
    let all_modules = admin_module_service.get_all_admin_modules().await
        .map_err(|e| AppError::new(crate::core::errors::ErrorKind::DatabaseError, e))?;
    
    // Filter to only user's assigned modules
    let module_details: Vec<AdminModuleResponse> = all_modules
        .into_iter()
        .filter(|module_name| user_modules.iter().any(|um| um == module_name))
        .map(|module_name| AdminModuleResponse {
            code: module_name.clone(),
            name: module_name.clone(),
            description: format!("Admin access for {}", module_name),
            category: "admin".to_string(),
            icon: Some("settings".to_string()),
            color: Some("#6B7280".to_string()),
            sort_order: 1,
            is_active: true,
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
    Extension(claims): Extension<ModernUserClaims>,
    Json(request): Json<AssignModulesRequest>,
) -> Result<Json<AdminRoleOperationResponse>, AppError> {
    let admin_module_service = &app_state.admin_module_service;
    info!("Assigning admin modules to user: {} by admin: {}", request.firebase_uid, claims.user_id);

    // Ensure the admin has permission to assign these modules
    let admin_modules = admin_module_service.get_user_admin_modules(&claims.user_id).await
        .map_err(|e| AppError::new(crate::core::errors::ErrorKind::DatabaseError, e))?;
    
    // Check if admin has 'user_operations' or 'permission_admin' module
    let can_assign = admin_modules.iter().any(|am| am == "user_operations") ||
                     admin_modules.iter().any(|am| am == "permission_admin") ||
                     admin_modules.iter().any(|am| am == "system_admin");
    
    if !can_assign {
        warn!("Admin {} attempted to assign modules without proper permissions", claims.user_id);
        return Err(AppError::unauthorized("Insufficient permissions to assign admin modules"));
    }

    let assignment_request = AdminModuleAssignRequest {
        user_id: request.firebase_uid.clone().into(),
        module_name: request.module_codes.first().unwrap_or(&String::new()).clone(),
        access_level: "admin".to_string(),
        expires_at: request.expires_at,
    };

    let assigned_modules = admin_module_service.assign_admin_modules(&assignment_request).await
        .map_err(|e| AppError::new(crate::core::errors::ErrorKind::DatabaseError, e))?;

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
    Extension(claims): Extension<ModernUserClaims>,
    Json(request): Json<RevokeModulesRequest>,
) -> Result<Json<AdminRoleOperationResponse>, AppError> {
    let admin_module_service = &app_state.admin_module_service;
    info!("Revoking admin modules from user: {} by admin: {}", request.firebase_uid, claims.user_id);

    // Ensure the admin has permission to revoke these modules
    let admin_modules = admin_module_service.get_user_admin_modules(&claims.user_id).await
        .map_err(|e| AppError::new(crate::core::errors::ErrorKind::DatabaseError, e))?;
    
    let can_revoke = admin_modules.iter().any(|am| am == "user_operations") ||
                     admin_modules.iter().any(|am| am == "permission_admin") ||
                     admin_modules.iter().any(|am| am == "system_admin");
    
    if !can_revoke {
        warn!("Admin {} attempted to revoke modules without proper permissions", claims.user_id);
        return Err(AppError::unauthorized("Insufficient permissions to revoke admin modules"));
    }

    // Prevent admins from revoking their own admin privileges
    if request.firebase_uid == claims.user_id {
        warn!("Admin {} attempted to revoke their own admin modules", claims.user_id);
        return Err(AppError::bad_request("Cannot revoke your own admin modules"));
    }

    let firebase_uid = request.firebase_uid.clone();
    
    let revoked_modules = admin_module_service.revoke_admin_modules(
        &firebase_uid.clone().into(),
        request.module_codes.clone()
    ).await
    .map_err(|e| AppError::new(crate::core::errors::ErrorKind::DatabaseError, e))?;
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
    Extension(claims): Extension<ModernUserClaims>,
) -> Result<Json<AdminRoleOperationResponse>, AppError> {
    let admin_module_service = &app_state.admin_module_service;
    info!("Assigning ALL admin modules to user: {} by super admin: {}", firebase_uid, claims.user_id);

    // Only allow users with 'system_admin' module to create super admins
    let admin_modules = admin_module_service.get_user_admin_modules(&claims.user_id).await
        .map_err(|e| AppError::new(crate::core::errors::ErrorKind::DatabaseError, e))?;
    
    if !admin_modules.iter().any(|am| am == "system_admin") {
        warn!("User {} attempted to create super admin without system_admin module", claims.user_id);
        return Err(AppError::unauthorized("Only system administrators can assign all admin modules"));
    }

    let assigned_modules = admin_module_service.assign_all_admin_modules(
        &firebase_uid,
        &claims.user_id,
        "Full admin module assignment via API by system administrator"
    ).await
    .map_err(|e| AppError::new(crate::core::errors::ErrorKind::DatabaseError, e))?;

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
    Extension(claims): Extension<ModernUserClaims>,
) -> Result<Json<Vec<serde_json::Value>>, AppError> {
    let admin_module_service = &app_state.admin_module_service;
    info!("Fetching admin role audit for user: {} by admin: {}", firebase_uid, claims.user_id);

    // Ensure the admin has permission to view audit trails
    let admin_modules = admin_module_service.get_user_admin_modules(&claims.user_id).await
        .map_err(|e| AppError::new(crate::core::errors::ErrorKind::DatabaseError, e))?;
    
    let can_view_audit = admin_modules.iter().any(|am| am == "compliance_audit") ||
                         admin_modules.iter().any(|am| am == "system_admin");
    
    if !can_view_audit {
        warn!("Admin {} attempted to view audit trail without proper permissions", claims.user_id);
        return Err(AppError::unauthorized("Insufficient permissions to view admin role audit"));
    }

    let _limit = params.limit.or(Some(50)).map(|l| l.min(500)); // Max 500 records
    let audit_records = admin_module_service.get_admin_role_audit(&firebase_uid).await
        .map_err(|e| AppError::new(crate::core::errors::ErrorKind::DatabaseError, e))?;

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

    let has_access = admin_module_service.user_has_admin_module(&firebase_uid, &module_code).await
        .map_err(|e| AppError::new(crate::core::errors::ErrorKind::DatabaseError, e))?;

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
    Extension(claims): Extension<ModernUserClaims>,
) -> Result<Json<Vec<serde_json::Value>>, AppError> {
    let admin_module_service = &app_state.admin_module_service;
    info!("Fetching detailed admin module assignments for user: {} by admin: {}", firebase_uid, claims.user_id);

    // Ensure the admin can view detailed assignments
    let admin_modules = admin_module_service.get_user_admin_modules(&claims.user_id).await
        .map_err(|e| AppError::new(crate::core::errors::ErrorKind::DatabaseError, e))?;
    
    let can_view_details = admin_modules.iter().any(|am| am == "user_operations") ||
                           admin_modules.iter().any(|am| am == "permission_admin") ||
                           admin_modules.iter().any(|am| am == "system_admin");
    
    if !can_view_details {
        warn!("Admin {} attempted to view detailed assignments without proper permissions", claims.user_id);
        return Err(AppError::unauthorized("Insufficient permissions to view detailed admin module assignments"));
    }

    let assignments = admin_module_service.get_user_admin_modules(&firebase_uid).await
        .map_err(|e| AppError::new(crate::core::errors::ErrorKind::DatabaseError, e))?;

    let response: Vec<serde_json::Value> = assignments
        .into_iter()
        .enumerate()
        .map(|(i, assignment)| serde_json::json!({
            "id": format!("{}-{}", firebase_uid, i),
            "firebase_uid": firebase_uid,
            "module_code": assignment,
            "granted_by": "system",
            "granted_reason": "Simple role system assignment",
            "expires_at": null,
            "is_active": true,
            "assignment_metadata": {},
            "created_at": chrono::Utc::now(),
            "updated_at": chrono::Utc::now()
        }))
        .collect();

    info!("Retrieved {} detailed assignments for user: {}", response.len(), firebase_uid);
    Ok(Json(response))
}

/// Get current authenticated user's admin modules
pub async fn get_current_user_admin_modules(
    State(app_state): State<AppState>,
    Extension(claims): Extension<ModernUserClaims>,
) -> Result<Json<UserAdminModulesResponse>, AppError> {
    let admin_module_service = &app_state.admin_module_service;
    let firebase_uid = &claims.user_id;
    info!("Fetching admin modules for current authenticated user: {}", firebase_uid);

    // Get user's module codes
    let user_modules = admin_module_service.get_user_admin_modules(firebase_uid).await
        .map_err(|e| AppError::new(crate::core::errors::ErrorKind::DatabaseError, e))?;
    
    // Get all available modules for details
    let all_modules = admin_module_service.get_all_admin_modules().await
        .map_err(|e| AppError::new(crate::core::errors::ErrorKind::DatabaseError, e))?;
    
    // Filter to only user's assigned modules
    let module_details: Vec<AdminModuleResponse> = all_modules
        .into_iter()
        .filter(|module_name| user_modules.iter().any(|um| um == module_name))
        .map(|module_name| AdminModuleResponse {
            code: module_name.clone(),
            name: module_name.clone(),
            description: format!("Admin access for {}", module_name),
            category: "admin".to_string(),
            icon: Some("settings".to_string()),
            color: Some("#6B7280".to_string()),
            sort_order: 1,
            is_active: true,
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