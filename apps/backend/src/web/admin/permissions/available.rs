// Available Permissions Handler
// Provides CRUD operations for permission definitions
//
// BIG-BANG: migrated to sqlx (real). All diesel DSL replaced with raw SQL.

use crate::web::auth::AppState;
use crate::web::responses::AdminResponse;
use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize)]
pub struct PermissionDefinition {
    pub id: Uuid,
    pub permission_string: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub platform: String,
    pub category: Option<String>,
    pub is_system: bool,
    pub is_active: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreatePermissionRequest {
    pub permission: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub platform: Option<String>,
    pub category: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdatePermissionRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub category: Option<String>,
    pub is_active: Option<bool>,
}

#[derive(sqlx::FromRow)]
struct PermRow {
    id: Uuid,
    permission_string: String,
    name: Option<String>,
    description: Option<String>,
    platform: String,
    category: Option<String>,
    is_system: bool,
    is_active: bool,
    created_at: chrono::DateTime<chrono::Utc>,
}

/// List all available permission definitions
/// GET /api/permissions/definitions
pub async fn list_permission_definitions(State(app_state): State<AppState>) -> impl IntoResponse {
    let rows: Result<Vec<PermRow>, _> = sqlx::query_as(
        "SELECT id, permission_string, name, description, platform, category, is_system, is_active, created_at
         FROM permissions
         WHERE is_active = TRUE
         ORDER BY platform, category, permission_string",
    )
    .fetch_all(app_state.db_pool.as_ref())
    .await;

    let permissions: Vec<PermissionDefinition> = match rows {
        Ok(rows) => rows
            .into_iter()
            .map(|r| PermissionDefinition {
                id: r.id,
                permission_string: r.permission_string,
                name: r.name,
                description: r.description,
                platform: r.platform,
                category: r.category,
                is_system: r.is_system,
                is_active: r.is_active,
                created_at: r.created_at,
            })
            .collect(),
        Err(e) => {
            tracing::error!("Failed to get permission definitions: {}", e);
            return AdminResponse::server_error("Database query failed").into_response();
        }
    };

    AdminResponse::success(permissions).into_response()
}

/// List all available unique permission strings (legacy endpoint)
/// GET /admin/permissions/available
pub async fn list_available_permissions(State(app_state): State<AppState>) -> impl IntoResponse {
    let rows: Result<Vec<(String,)>, _> = sqlx::query_as(
        "SELECT DISTINCT permission_string FROM permissions WHERE is_active = TRUE ORDER BY permission_string",
    )
    .fetch_all(app_state.db_pool.as_ref())
    .await;

    let permissions: Vec<String> = match rows {
        Ok(rows) => rows.into_iter().map(|(p,)| p).collect(),
        Err(e) => {
            tracing::error!("Failed to get available permissions: {}", e);
            return AdminResponse::server_error("Database query failed").into_response();
        }
    };

    AdminResponse::success(permissions).into_response()
}

/// Create a new permission definition
/// POST /api/permissions/definitions
pub async fn create_permission_definition(
    State(app_state): State<AppState>,
    Json(req): Json<CreatePermissionRequest>,
) -> impl IntoResponse {
    // Validate permission format: at least 3 colon-separated parts
    let parts: Vec<&str> = req.permission.split(':').collect();
    if parts.len() < 3 {
        return AdminResponse::bad_request(
            "Invalid permission format. Use: platform:resource:action",
        )
        .into_response();
    }

    // Validate each part
    for part in &parts {
        if part.is_empty()
            || (!part
                .chars()
                .all(|c| c.is_alphanumeric() || c == '_' || c == '-' || c == '*'))
        {
            return AdminResponse::bad_request(
                "Permission parts must be alphanumeric, underscore, dash, or wildcard (*)",
            )
            .into_response();
        }
    }

    let permission = req.permission.to_lowercase();
    let platform = req.platform.clone().unwrap_or_else(|| parts[0].to_string());
    let resource = parts[1].to_string();
    let action = parts[2].to_string();
    let category = req.category.clone().or_else(|| Some(parts[1].to_string()));

    // Derive name from permission if not provided
    let name = req.name.clone().or_else(|| {
        Some(
            format!(
                "{} {}",
                parts[1].replace(['_', '-'], " "),
                parts[2].replace(['_', '-'], " ")
            )
            .to_uppercase(),
        )
    });

    let row: Result<(Uuid, String, Option<String>, Option<String>, String, Option<String>, bool, bool, chrono::DateTime<chrono::Utc>), _> = sqlx::query_as(
        "INSERT INTO permissions (permission_string, platform, resource, action, name, description, category, is_system, is_active, permission_type)
         VALUES ($1, $2, $3, $4, $5, $6, $7, FALSE, TRUE, 'manual')
         ON CONFLICT (permission_string) DO UPDATE SET is_active = TRUE, updated_at = NOW()
         RETURNING id, permission_string, name, description, platform, category, is_system, is_active, created_at"
    )
    .bind(&permission)
    .bind(&platform)
    .bind(&resource)
    .bind(&action)
    .bind(&name)
    .bind(&req.description)
    .bind(&category)
    .fetch_one(app_state.db_pool.as_ref())
    .await;

    match row {
        Ok((id, permission_string, name, description, platform, cat, is_system, is_active, created_at)) => {
            AdminResponse::success(PermissionDefinition {
                id,
                permission_string,
                name,
                description,
                platform,
                category: cat,
                is_system,
                is_active,
                created_at,
            })
            .into_response()
        }
        Err(e) => {
            tracing::error!("Failed to create permission definition: {}", e);
            AdminResponse::server_error("Failed to create permission").into_response()
        }
    }
}

/// Update a permission definition
/// PUT /api/permissions/definitions/{id}
pub async fn update_permission_definition(
    State(app_state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdatePermissionRequest>,
) -> impl IntoResponse {
    // Check if permission exists
    let check: Result<Option<(bool,)>, _> = sqlx::query_as(
        "SELECT is_system FROM permissions WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(app_state.db_pool.as_ref())
    .await;

    match check {
        Ok(Some((is_system,))) if is_system => {
            // System permissions can only update display fields
        }
        Ok(None) => {
            return AdminResponse::not_found("Permission not found").into_response();
        }
        Ok(_) => {
            // Non-system permission
        }
        Err(e) => {
            tracing::error!("Failed to check permission: {}", e);
            return AdminResponse::server_error("Database query failed").into_response();
        }
    }

    // Build dynamic UPDATE with COALESCE
    let row: Result<(Uuid, String, Option<String>, Option<String>, String, Option<String>, bool, bool, chrono::DateTime<chrono::Utc>), _> = sqlx::query_as(
        "UPDATE permissions SET 
            name = COALESCE($2, name),
            description = COALESCE($3, description),
            category = COALESCE($4, category),
            is_active = COALESCE($5, is_active),
            updated_at = NOW()
         WHERE id = $1
         RETURNING id, permission_string, name, description, platform, category, is_system, is_active, created_at"
    )
    .bind(id)
    .bind(req.name.clone())
    .bind(req.description.clone())
    .bind(req.category.clone())
    .bind(req.is_active)
    .fetch_one(app_state.db_pool.as_ref())
    .await;

    match row {
        Ok((id, permission_string, name, description, platform, cat, is_system, is_active, created_at)) => {
            AdminResponse::success(PermissionDefinition {
                id,
                permission_string,
                name,
                description,
                platform,
                category: cat,
                is_system,
                is_active,
                created_at,
            })
            .into_response()
        }
        Err(e) => {
            tracing::error!("Failed to update permission definition: {}", e);
            AdminResponse::server_error("Failed to update permission").into_response()
        }
    }
}

/// Delete a permission definition (soft delete by setting is_active = false)
/// DELETE /api/permissions/definitions/{id}
pub async fn delete_permission_definition(
    State(app_state): State<AppState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    // Check if it's a system permission
    let check: Result<Option<(bool,)>, _> = sqlx::query_as(
        "SELECT is_system FROM permissions WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(app_state.db_pool.as_ref())
    .await;

    match check {
        Ok(Some((is_system,))) if is_system => {
            return AdminResponse::bad_request("Cannot delete system permissions")
                .into_response();
        }
        Ok(None) => {
            return AdminResponse::not_found("Permission not found").into_response();
        }
        Err(e) => {
            tracing::error!("Failed to check permission: {}", e);
            return AdminResponse::server_error("Database query failed").into_response();
        }
        _ => {}
    }

    // Soft delete by setting is_active = false
    let result = sqlx::query(
        "UPDATE permissions SET is_active = FALSE, updated_at = NOW() WHERE id = $1 AND is_system = FALSE",
    )
    .bind(id)
    .execute(app_state.db_pool.as_ref())
    .await;

    match result {
        Ok(r) => {
            if r.rows_affected() > 0 {
                AdminResponse::success("Permission deleted").into_response()
            } else {
                AdminResponse::not_found("Permission not found or is a system permission")
                    .into_response()
            }
        }
        Err(e) => {
            tracing::error!("Failed to delete permission: {}", e);
            AdminResponse::server_error("Failed to delete permission").into_response()
        }
    }
}

/// Delete a permission definition by permission string
/// DELETE /api/permissions/definitions/by-name/{permission}
pub async fn delete_permission_by_name(
    State(app_state): State<AppState>,
    Path(permission): Path<String>,
) -> impl IntoResponse {
    let permission = permission.to_lowercase();

    let result = sqlx::query(
        "UPDATE permissions SET is_active = FALSE, updated_at = NOW() WHERE permission_string = $1 AND is_system = FALSE",
    )
    .bind(&permission)
    .execute(app_state.db_pool.as_ref())
    .await;

    match result {
        Ok(r) => {
            if r.rows_affected() > 0 {
                AdminResponse::success("Permission deleted").into_response()
            } else {
                AdminResponse::not_found("Permission not found or is a system permission")
                    .into_response()
            }
        }
        Err(e) => {
            tracing::error!("Failed to delete permission: {}", e);
            AdminResponse::server_error("Failed to delete permission").into_response()
        }
    }
}
