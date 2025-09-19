use axum::{
    extract::{Path, Query, State},
    response::Json,
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use tracing::{error, info};
use uuid::Uuid;

use crate::auth::{HierarchyResolver, PermissionHierarchy, InheritanceType, HierarchyStats};
use crate::web::middleware::clean_auth::AuthenticatedUser;
use crate::web::auth::AppState;

/// Request to create a permission hierarchy
#[derive(Debug, Deserialize)]
pub struct CreateHierarchyRequest {
    pub parent_permission: String,
    pub child_permission: String,
    pub inheritance_type: InheritanceType,
}

/// Request to resolve user permissions
#[derive(Debug, Deserialize)]
pub struct ResolvePermissionsRequest {
    pub user_id: Uuid,
    pub direct_permissions: Vec<String>,
}

/// Response with hierarchy statistics
#[derive(Debug, Serialize)]
pub struct HierarchyStatsResponse {
    pub success: bool,
    pub stats: HierarchyStats,
}

/// Response with permission resolution result
#[derive(Debug, Serialize)]
pub struct PermissionResolutionResponse {
    pub success: bool,
    pub resolution: crate::auth::HierarchyResolution,
}

/// Response with hierarchy tree
#[derive(Debug, Serialize)]
pub struct HierarchyTreeResponse {
    pub success: bool,
    pub hierarchies: Vec<PermissionHierarchy>,
}

/// Response for hierarchy operations
#[derive(Debug, Serialize)]
pub struct HierarchyResponse {
    pub success: bool,
    pub message: String,
    pub hierarchy_id: Option<Uuid>,
}

/// Error response
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub success: bool,
    pub error: String,
    pub details: Option<String>,
}

/// Get permission hierarchy statistics
pub async fn get_hierarchy_stats(
    State(state): State<AppState>,
    user: AuthenticatedUser,
) -> Result<Json<HierarchyStatsResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Check admin permissions
    if !user.valid_permissions.iter().any(|p| p.starts_with("admin:permissions:") || p == "admin:*:*") {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ErrorResponse {
                success: false,
                error: "Insufficient permissions".to_string(),
                details: Some("admin:permissions:view required".to_string()),
            }),
        ));
    }

    let resolver = HierarchyResolver::new(state.db_pool.clone());
    
    match resolver.get_hierarchy_stats().await {
        Ok(stats) => {
            info!("Retrieved hierarchy stats for admin user: {}", user.email);
            Ok(Json(HierarchyStatsResponse {
                success: true,
                stats,
            }))
        }
        Err(e) => {
            error!("Failed to get hierarchy stats: {:?}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    success: false,
                    error: "Failed to retrieve hierarchy statistics".to_string(),
                    details: Some(e.to_string()),
                }),
            ))
        }
    }
}

/// Get permission hierarchy tree
pub async fn get_hierarchy_tree(
    State(state): State<AppState>,
    user: AuthenticatedUser,
) -> Result<Json<HierarchyTreeResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Check admin permissions
    if !user.valid_permissions.iter().any(|p| p.starts_with("admin:permissions:") || p == "admin:*:*") {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ErrorResponse {
                success: false,
                error: "Insufficient permissions".to_string(),
                details: Some("admin:permissions:view required".to_string()),
            }),
        ));
    }

    let resolver = HierarchyResolver::new(state.db_pool.clone());
    
    match resolver.get_hierarchy_tree().await {
        Ok(hierarchies) => {
            info!("Retrieved hierarchy tree for admin user: {}", user.email);
            Ok(Json(HierarchyTreeResponse {
                success: true,
                hierarchies,
            }))
        }
        Err(e) => {
            error!("Failed to get hierarchy tree: {:?}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    success: false,
                    error: "Failed to retrieve hierarchy tree".to_string(),
                    details: Some(e.to_string()),
                }),
            ))
        }
    }
}

/// Create a new permission hierarchy
pub async fn create_hierarchy(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Json(request): Json<CreateHierarchyRequest>,
) -> Result<Json<HierarchyResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Check admin permissions
    if !user.valid_permissions.iter().any(|p| p.starts_with("admin:permissions:") || p == "admin:*:*") {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ErrorResponse {
                success: false,
                error: "Insufficient permissions".to_string(),
                details: Some("admin:permissions:modify required".to_string()),
            }),
        ));
    }

    let resolver = HierarchyResolver::new(state.db_pool.clone());
    
    match resolver.add_hierarchy(
        &request.parent_permission,
        &request.child_permission,
        request.inheritance_type,
        Some(user.user_id),
    ).await {
        Ok(hierarchy_id) => {
            info!(
                "Created permission hierarchy: {} -> {} by admin user: {}",
                request.parent_permission, request.child_permission, user.email
            );
            Ok(Json(HierarchyResponse {
                success: true,
                message: format!(
                    "Successfully created hierarchy: {} -> {}",
                    request.parent_permission, request.child_permission
                ),
                hierarchy_id: Some(hierarchy_id),
            }))
        }
        Err(e) => {
            error!("Failed to create hierarchy: {:?}", e);
            Err((
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    success: false,
                    error: "Failed to create permission hierarchy".to_string(),
                    details: Some(e.to_string()),
                }),
            ))
        }
    }
}

/// Remove a permission hierarchy
pub async fn remove_hierarchy(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Path(hierarchy_id): Path<Uuid>,
) -> Result<Json<HierarchyResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Check admin permissions
    if !user.valid_permissions.iter().any(|p| p.starts_with("admin:permissions:") || p == "admin:*:*") {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ErrorResponse {
                success: false,
                error: "Insufficient permissions".to_string(),
                details: Some("admin:permissions:modify required".to_string()),
            }),
        ));
    }

    let resolver = HierarchyResolver::new(state.db_pool.clone());
    
    match resolver.remove_hierarchy(hierarchy_id).await {
        Ok(()) => {
            info!(
                "Removed permission hierarchy {} by admin user: {}",
                hierarchy_id, user.email
            );
            Ok(Json(HierarchyResponse {
                success: true,
                message: format!("Successfully removed hierarchy {}", hierarchy_id),
                hierarchy_id: Some(hierarchy_id),
            }))
        }
        Err(e) => {
            error!("Failed to remove hierarchy: {:?}", e);
            Err((
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    success: false,
                    error: "Failed to remove permission hierarchy".to_string(),
                    details: Some(e.to_string()),
                }),
            ))
        }
    }
}

/// Resolve user permissions with inheritance
pub async fn resolve_user_permissions(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Json(request): Json<ResolvePermissionsRequest>,
) -> Result<Json<PermissionResolutionResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Check admin permissions
    if !user.valid_permissions.iter().any(|p| p.starts_with("admin:permissions:") || p.starts_with("admin:users:") || p == "admin:*:*") {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ErrorResponse {
                success: false,
                error: "Insufficient permissions".to_string(),
                details: Some("admin:permissions:view or admin:users:view required".to_string()),
            }),
        ));
    }

    let resolver = HierarchyResolver::new(state.db_pool.clone());
    
    match resolver.resolve_user_permissions(request.user_id, &request.direct_permissions).await {
        Ok(resolution) => {
            info!(
                "Resolved permissions for user {} by admin user: {} ({} -> {} total)",
                request.user_id, user.email, 
                request.direct_permissions.len(), 
                resolution.all_permissions.len()
            );
            Ok(Json(PermissionResolutionResponse {
                success: true,
                resolution,
            }))
        }
        Err(e) => {
            error!("Failed to resolve user permissions: {:?}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    success: false,
                    error: "Failed to resolve user permissions".to_string(),
                    details: Some(e.to_string()),
                }),
            ))
        }
    }
}

/// Invalidate permission cache for a user
pub async fn invalidate_user_cache(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Path(user_id): Path<Uuid>,
) -> Result<Json<HierarchyResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Check admin permissions
    if !user.valid_permissions.iter().any(|p| p.starts_with("admin:permissions:") || p.starts_with("admin:users:") || p == "admin:*:*") {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ErrorResponse {
                success: false,
                error: "Insufficient permissions".to_string(),
                details: Some("admin:permissions:modify required".to_string()),
            }),
        ));
    }

    let resolver = HierarchyResolver::new(state.db_pool.clone());
    
    match resolver.invalidate_cache(user_id).await {
        Ok(()) => {
            info!(
                "Invalidated permission cache for user {} by admin user: {}",
                user_id, user.email
            );
            Ok(Json(HierarchyResponse {
                success: true,
                message: format!("Successfully invalidated cache for user {}", user_id),
                hierarchy_id: None,
            }))
        }
        Err(e) => {
            error!("Failed to invalidate user cache: {:?}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    success: false,
                    error: "Failed to invalidate user cache".to_string(),
                    details: Some(e.to_string()),
                }),
            ))
        }
    }
}

/// Test permission hierarchy resolution (for debugging)
pub async fn test_hierarchy_resolution(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<Json<PermissionResolutionResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Check admin permissions
    if !user.valid_permissions.iter().any(|p| p.starts_with("admin:permissions:") || p == "admin:*:*") {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ErrorResponse {
                success: false,
                error: "Insufficient permissions".to_string(),
                details: Some("admin:permissions:view required".to_string()),
            }),
        ));
    }

    // Parse permissions from query parameter
    let permissions_param = params.get("permissions").ok_or_else(|| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                success: false,
                error: "Missing 'permissions' query parameter".to_string(),
                details: Some("Provide comma-separated list of permissions to test".to_string()),
            }),
        )
    })?;

    let test_permissions: Vec<String> = permissions_param
        .split(',')
        .map(|p| p.trim().to_string())
        .filter(|p| !p.is_empty())
        .collect();

    if test_permissions.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                success: false,
                error: "No valid permissions provided".to_string(),
                details: Some("Provide at least one permission to test".to_string()),
            }),
        ));
    }

    let resolver = HierarchyResolver::new(state.db_pool.clone());
    
    // Use a dummy user ID for testing
    let test_user_id = Uuid::new_v4();
    
    match resolver.resolve_user_permissions(test_user_id, &test_permissions).await {
        Ok(resolution) => {
            info!(
                "Tested hierarchy resolution for permissions {:?} by admin user: {}",
                test_permissions, user.email
            );
            Ok(Json(PermissionResolutionResponse {
                success: true,
                resolution,
            }))
        }
        Err(e) => {
            error!("Failed to test hierarchy resolution: {:?}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    success: false,
                    error: "Failed to test hierarchy resolution".to_string(),
                    details: Some(e.to_string()),
                }),
            ))
        }
    }
}