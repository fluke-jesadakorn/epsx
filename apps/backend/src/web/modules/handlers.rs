// Module system handlers
// Core handlers for module discovery, management, and access control

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use uuid::Uuid;

use crate::web::{
    auth::AppState,
    middleware::module_auth_middleware::{ModuleAuthCtx, AccessLevel},
};

// ========================================
// REQUEST/RESPONSE TYPES
// ========================================

#[derive(Debug, Deserialize)]
pub struct ModuleFilters {
    pub category: Option<String>,
    pub status: Option<String>,
    pub search: Option<String>,
    pub limit: Option<i32>,
    pub offset: Option<i32>,
}

#[derive(Debug, Serialize)]
pub struct ModuleInfo {
    pub id: String,
    pub name: String,
    pub display_name: String,
    pub description: Option<String>,
    pub category: String,
    pub icon: Option<String>,
    pub status: String,
    pub access_levels: Value,
    pub features: Value,
    pub pricing: Value,
}

#[derive(Debug, Serialize)]
pub struct ModuleAccessInfo {
    pub has_access: bool,
    pub access_level: Option<String>,
    pub quota_status: Option<Value>,
    pub restrictions: Option<Value>,
    pub expires_at: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct QuotaStatus {
    pub module_name: String,
    pub quotas: HashMap<String, QuotaInfo>,
    pub current_usage: HashMap<String, i32>,
    pub utilization: HashMap<String, f64>,
}

#[derive(Debug, Serialize)]
pub struct QuotaInfo {
    pub limit: i32, // -1 for unlimited
    pub used: i32,
    pub remaining: i32,
    pub reset_time: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateModuleRequest {
    pub name: String,
    pub display_name: String,
    pub description: Option<String>,
    pub category: String,
    pub icon: Option<String>,
    pub api_endpoints: Value,
    pub ui_components: Value,
    pub feature_flags: Value,
    pub access_levels: Value,
    pub default_quotas: Value,
    pub pricing_tiers: Value,
    pub dependencies: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct AssignModulesRequest {
    pub modules: Vec<ModuleAssignment>,
    pub reason: String,
}

#[derive(Debug, Deserialize)]
pub struct ModuleAssignment {
    pub module_id: String,
    pub access_level: String,
    pub custom_quotas: Option<Value>,
    pub restrictions: Option<Value>,
    pub expires_at: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct BulkAssignRequest {
    pub user_ids: Vec<String>,
    pub assignments: Vec<ModuleAssignment>,
    pub reason: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateApiKeyRequest {
    pub client_name: String,
    pub client_description: Option<String>,
    pub client_contact_email: Option<String>,
    pub allowed_modules: Vec<ApiKeyModuleAccess>,
    pub ip_restrictions: Vec<String>,
    pub expires_at: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ApiKeyModuleAccess {
    pub module_id: String,
    pub access_level: String,
    pub custom_quotas: Option<Value>,
}

// ========================================
// MODULE DISCOVERY HANDLERS
// ========================================

/// List available modules for the authenticated user
pub async fn list_available_modules(
    auth: ModuleAuthCtx,
    Query(filters): Query<ModuleFilters>,
    State(state): State<AppState>,
) -> Result<Json<Value>, StatusCode> {
    // Get all active modules
    let modules = match state.module_repo.list_active_modules().await {
        Ok(modules) => modules,
        Err(e) => {
            tracing::error!("Failed to list modules: {:?}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    // Filter modules based on user's access and query filters
    let mut filtered_modules = Vec::new();
    for module in modules {
        // Check if user has access to this module
        let has_access = auth.has_module_access(module.name());
        
        // Apply filters
        if let Some(category) = &filters.category {
            if module.category() != category {
                continue;
            }
        }
        
        if let Some(search) = &filters.search {
            let search_lower = search.to_lowercase();
            if !module.name().to_lowercase().contains(&search_lower) &&
               !module.display_name().to_lowercase().contains(&search_lower) {
                continue;
            }
        }

        let module_info = ModuleInfo {
            id: module.id().to_string(),
            name: module.name().to_string(),
            display_name: module.display_name().to_string(),
            description: module.description().map(|s| s.to_string()),
            category: module.category().to_string(),
            icon: module.icon().map(|s| s.to_string()),
            status: if has_access { "accessible".to_string() } else { "restricted".to_string() },
            access_levels: module.access_levels().clone(),
            features: module.feature_flags().clone(),
            pricing: module.pricing_tiers().clone(),
        };
        
        filtered_modules.push(module_info);
    }

    // Apply pagination
    let total = filtered_modules.len();
    let offset = filters.offset.unwrap_or(0) as usize;
    let limit = filters.limit.unwrap_or(50) as usize;
    let paginated_modules: Vec<_> = filtered_modules
        .into_iter()
        .skip(offset)
        .take(limit)
        .collect();

    Ok(Json(json!({
        "modules": paginated_modules,
        "total": total,
        "offset": offset,
        "limit": limit,
        "user_assigned_modules": auth.get_available_modules()
    })))
}

/// Get detailed information about a specific module
pub async fn get_module_details(
    auth: ModuleAuthCtx,
    Path(module_id): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<Value>, StatusCode> {
    let module_uuid = Uuid::parse_str(&module_id)
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    let module = match state.module_repo.get_sub_module(&module_uuid).await {
        Ok(Some(module)) => module,
        Ok(None) => return Err(StatusCode::NOT_FOUND),
        Err(e) => {
            tracing::error!("Failed to get module details: {:?}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    // Check if user has access
    let has_access = auth.has_module_access(module.name());
    let access_level = auth.get_access_level(module.name());

    let module_detail = json!({
        "id": module.id(),
        "name": module.name(),
        "display_name": module.display_name(),
        "description": module.description(),
        "category": module.category(),
        "icon": module.icon(),
        "status": module.status(),
        "version": module.version(),
        "dependencies": module.dependencies(),
        "conflicts": module.conflicts(),
        "api_endpoints": module.api_endpoints(),
        "ui_components": module.ui_components(),
        "feature_flags": module.feature_flags(),
        "access_levels": module.access_levels(),
        "default_quotas": module.default_quotas(),
        "pricing_tiers": module.pricing_tiers(),
        "user_access": {
            "has_access": has_access,
            "access_level": access_level,
            "available_features": if has_access {
                // Extract features available at user's access level
                extract_available_features(module.feature_flags(), access_level)
            } else {
                json!({})
            }
        },
        "created_at": module.created_at(),
        "updated_at": module.updated_at()
    });

    Ok(Json(module_detail))
}

/// Check user's access to a specific module
pub async fn check_module_access(
    auth: ModuleAuthCtx,
    Path(module_name): Path<String>,
    State(_state): State<AppState>,
) -> Result<Json<ModuleAccessInfo>, StatusCode> {
    let has_access = auth.has_module_access(&module_name);
    let access_level = auth.get_access_level(&module_name);
    let quota_status = auth.get_quota_status(&module_name);

    // Get detailed access info if user has access
    let (restrictions, expires_at) = if has_access {
        if let Some(module_access) = auth.assigned_modules.iter().find(|m| m.module_name == module_name) {
            (
                Some(json!(module_access.restrictions)),
                module_access.expires_at.map(|dt| dt.to_rfc3339())
            )
        } else {
            (None, None)
        }
    } else {
        (None, None)
    };

    let access_info = ModuleAccessInfo {
        has_access,
        access_level: access_level.map(|l| l.to_string().to_owned()),
        quota_status: quota_status.map(|q| json!(q)),
        restrictions,
        expires_at,
    };

    Ok(Json(access_info))
}

/// Get quota status for a specific module
pub async fn get_module_quota_status(
    auth: ModuleAuthCtx,
    Path(module_name): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<QuotaStatus>, StatusCode> {
    if !auth.has_module_access(&module_name) {
        return Err(StatusCode::FORBIDDEN);
    }

    let quota_limits = match state.module_repo.get_quota_limits(&auth.user_id, &module_name).await {
        Ok(limits) => limits,
        Err(e) => {
            tracing::error!("Failed to get quota limits: {:?}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    let mut quotas = HashMap::new();
    let mut current_usage = HashMap::new();
    let mut utilization = HashMap::new();

    for (quota_type, limit) in quota_limits {
        let used = match state.module_repo.get_current_usage(&auth.user_id, &module_name, &quota_type).await {
            Ok(usage) => usage,
            Err(_) => 0,
        };

        let remaining = if limit == -1 { -1 } else { limit - used };
        let util = if limit == -1 { 0.0 } else { (used as f64 / limit as f64) * 100.0 };

        quotas.insert(quota_type.clone(), QuotaInfo {
            limit,
            used,
            remaining,
            reset_time: None, // Would be calculated based on quota reset schedule
        });

        current_usage.insert(quota_type.clone(), used);
        utilization.insert(quota_type, util);
    }

    let quota_status = QuotaStatus {
        module_name,
        quotas,
        current_usage,
        utilization,
    };

    Ok(Json(quota_status))
}

// ========================================
// ADMIN MODULE MANAGEMENT HANDLERS
// ========================================

/// Create a new module (admin only)
pub async fn create_module(
    auth: ModuleAuthCtx,
    State(state): State<AppState>,
    Json(request): Json<CreateModuleRequest>,
) -> Result<Json<Value>, StatusCode> {
    // Check admin permissions
    if !matches!(auth.role, crate::dom::values::Role::Admin | crate::dom::values::Role::SuperAdmin) {
        return Err(StatusCode::FORBIDDEN);
    }

    let module = crate::dom::entities::module::SubModule::new(
        request.name,
        request.display_name,
        request.description,
        request.category,
        auth.user_id.clone(),
    );

    match state.module_repo.create_sub_module(&module).await {
        Ok(_) => Ok(Json(json!({
            "id": module.id(),
            "message": "Module created successfully"
        }))),
        Err(e) => {
            tracing::error!("Failed to create module: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Assign modules to a user (admin only)
pub async fn assign_user_modules(
    auth: ModuleAuthCtx,
    Path(user_id): Path<String>,
    State(state): State<AppState>,
    Json(request): Json<AssignModulesRequest>,
) -> Result<Json<Value>, StatusCode> {
    // Check admin permissions
    if !matches!(auth.role, crate::dom::values::Role::Admin | crate::dom::values::Role::SuperAdmin) {
        return Err(StatusCode::FORBIDDEN);
    }

    let target_user_id = crate::dom::values::UserId::from_str(&user_id)
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    let mut assignment_results = Vec::new();

    for assignment in request.modules {
        let module_id = Uuid::parse_str(&assignment.module_id)
            .map_err(|_| StatusCode::BAD_REQUEST)?;

        let user_assignment = crate::dom::entities::module::UserSubModuleAssignment::new(
            target_user_id.clone(),
            module_id,
            assignment.access_level,
            auth.user_id.clone(),
            request.reason.clone(),
        );

        match state.module_repo.create_assignment(&user_assignment).await {
            Ok(_) => {
                assignment_results.push(json!({
                    "module_id": assignment.module_id,
                    "status": "success",
                    "assignment_id": user_assignment.id()
                }));
            }
            Err(e) => {
                tracing::error!("Failed to create assignment: {:?}", e);
                assignment_results.push(json!({
                    "module_id": assignment.module_id,
                    "status": "error",
                    "error": "Assignment failed"
                }));
            }
        }
    }

    Ok(Json(json!({
        "user_id": user_id,
        "assignments": assignment_results,
        "message": "Module assignment completed"
    })))
}

/// Get user's module assignments (admin only)
pub async fn get_user_modules(
    auth: ModuleAuthCtx,
    Path(user_id): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<Value>, StatusCode> {
    // Check admin permissions or self-access
    let target_user_id = crate::dom::values::UserId::from_str(&user_id)
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    if auth.user_id != target_user_id && !matches!(auth.role, crate::dom::values::Role::Admin | crate::dom::values::Role::SuperAdmin) {
        return Err(StatusCode::FORBIDDEN);
    }

    let assignments = match state.module_repo.get_user_module_assignments(&target_user_id).await {
        Ok(assignments) => assignments,
        Err(e) => {
            tracing::error!("Failed to get user assignments: {:?}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    Ok(Json(json!({
        "user_id": user_id,
        "modules": assignments,
        "total": assignments.len()
    })))
}

/// Create API key for third-party access (admin only)
pub async fn create_api_key(
    auth: ModuleAuthCtx,
    State(state): State<AppState>,
    Json(request): Json<CreateApiKeyRequest>,
) -> Result<Json<Value>, StatusCode> {
    // Check admin permissions
    if !matches!(auth.role, crate::dom::values::Role::Admin | crate::dom::values::Role::SuperAdmin) {
        return Err(StatusCode::FORBIDDEN);
    }

    // Generate API key
    let api_key = generate_api_key();
    let key_hash = hash_api_key(&api_key);
    let key_prefix = format!("ak_{}", &api_key[..8]);

    let api_key_entity = crate::dom::entities::module::ApiKey::new(
        key_hash,
        key_prefix.clone(),
        request.client_name,
        auth.user_id.clone(),
    );

    match state.module_repo.create_api_key(&api_key_entity).await {
        Ok(_) => Ok(Json(json!({
            "key_id": api_key_entity.id(),
            "api_key": api_key, // Only returned once during creation
            "key_prefix": key_prefix,
            "message": "API key created successfully. Store this key securely - it won't be shown again."
        }))),
        Err(e) => {
            tracing::error!("Failed to create API key: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// ========================================
// UTILITY FUNCTIONS
// ========================================

fn extract_available_features(feature_flags: &Value, access_level: Option<&AccessLevel>) -> Value {
    if let (Some(flags), Some(level)) = (feature_flags.as_object(), access_level) {
        let level_str = level.to_string();
        let mut available_features = serde_json::Map::new();
        
        for (feature, levels) in flags {
            if let Some(level_obj) = levels.as_object() {
                // Use the string reference directly
                if let Some(enabled) = level_obj.get(&level_str as &str).and_then(|v| v.as_bool()) {
                    available_features.insert(feature.clone(), json!(enabled));
                }
            }
        }
        
        Value::Object(available_features)
    } else {
        json!({})
    }
}

fn generate_api_key() -> String {
    use rand::Rng;
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    const KEY_LEN: usize = 64;
    let mut rng = rand::thread_rng();
    (0..KEY_LEN)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}

fn hash_api_key(api_key: &str) -> String {
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    hasher.update(api_key.as_bytes());
    format!("{:x}", hasher.finalize())
}