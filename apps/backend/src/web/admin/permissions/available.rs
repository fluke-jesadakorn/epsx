// Available Permissions Handler
// Provides CRUD operations for permission definitions

use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Json,
};
use diesel_async::RunQueryDsl;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::web::auth::AppState;
use crate::web::responses::AdminResponse;

/// Permission definition response
#[derive(Debug, Serialize)]
pub struct PermissionDefinition {
    pub id: Uuid,
    pub permission: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub platform: String,
    pub category: Option<String>,
    pub is_system: bool,
    pub is_active: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Request to create a new permission definition
#[derive(Debug, Deserialize)]
pub struct CreatePermissionRequest {
    pub permission: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub platform: Option<String>,
    pub category: Option<String>,
}

/// Request to update a permission definition
#[derive(Debug, Deserialize)]
pub struct UpdatePermissionRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub category: Option<String>,
    pub is_active: Option<bool>,
}

/// List all available permission definitions
/// GET /api/v1/permissions/definitions
pub async fn list_permission_definitions(
    State(app_state): State<AppState>,
) -> impl IntoResponse {
    let mut conn = match app_state.db_pool.get().await {
        Ok(conn) => conn,
        Err(e) => {
            tracing::error!("Failed to get database connection: {}", e);
            return AdminResponse::server_error("Database connection failed").into_response();
        }
    };

    #[derive(diesel::QueryableByName)]
    struct PermRow {
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        id: Uuid,
        #[diesel(sql_type = diesel::sql_types::Varchar)]
        permission: String,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Varchar>)]
        name: Option<String>,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
        description: Option<String>,
        #[diesel(sql_type = diesel::sql_types::Varchar)]
        platform: String,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Varchar>)]
        category: Option<String>,
        #[diesel(sql_type = diesel::sql_types::Bool)]
        is_system: bool,
        #[diesel(sql_type = diesel::sql_types::Bool)]
        is_active: bool,
        #[diesel(sql_type = diesel::sql_types::Timestamptz)]
        created_at: chrono::DateTime<chrono::Utc>,
    }

    let permissions: Vec<PermissionDefinition> = match diesel::sql_query(
        "SELECT id, permission, name, description, platform, category, is_system, is_active, created_at
         FROM permission_definitions
         WHERE is_active = TRUE
         ORDER BY platform, category, permission"
    )
    .load::<PermRow>(&mut conn)
    .await
    {
        Ok(rows) => rows.into_iter().map(|r| PermissionDefinition {
            id: r.id,
            permission: r.permission,
            name: r.name,
            description: r.description,
            platform: r.platform,
            category: r.category,
            is_system: r.is_system,
            is_active: r.is_active,
            created_at: r.created_at,
        }).collect(),
        Err(e) => {
            tracing::error!("Failed to get permission definitions: {}", e);
            return AdminResponse::server_error("Database query failed").into_response();
        }
    };

    AdminResponse::success(permissions).into_response()
}

/// List all available unique permission strings (legacy endpoint)
/// GET /admin/permissions/available
pub async fn list_available_permissions(
    State(app_state): State<AppState>,
) -> impl IntoResponse {
    let mut conn = match app_state.db_pool.get().await {
        Ok(conn) => conn,
        Err(e) => {
            tracing::error!("Failed to get database connection: {}", e);
            return AdminResponse::server_error("Database connection failed").into_response();
        }
    };

    #[derive(diesel::QueryableByName)]
    struct PermissionStringRow {
        #[diesel(sql_type = diesel::sql_types::Text)]
        permission_string: String,
    }

    // First try to get from permission_definitions table
    let from_definitions: Vec<String> = match diesel::sql_query(
        "SELECT permission as permission_string FROM permission_definitions WHERE is_active = TRUE ORDER BY permission"
    )
    .load::<PermissionStringRow>(&mut conn)
    .await
    {
        Ok(rows) => rows.into_iter().map(|r| r.permission_string).collect(),
        Err(_) => vec![], // Table might not exist yet
    };

    // If we have permissions from definitions table, use those
    if !from_definitions.is_empty() {
        return AdminResponse::success(from_definitions).into_response();
    }

    // Fallback: get from the permissions table (legacy behavior)
    let permissions: Vec<String> = match diesel::sql_query(
        "SELECT DISTINCT permission_string FROM permissions ORDER BY permission_string"
    )
    .load::<PermissionStringRow>(&mut conn)
    .await
    {
        Ok(rows) => rows.into_iter().map(|r| r.permission_string).collect(),
        Err(e) => {
            tracing::error!("Failed to get available permissions: {}", e);
            return AdminResponse::server_error("Database query failed").into_response();
        }
    };

    AdminResponse::success(permissions).into_response()
}

/// Create a new permission definition
/// POST /api/v1/permissions/definitions
pub async fn create_permission_definition(
    State(app_state): State<AppState>,
    Json(req): Json<CreatePermissionRequest>,
) -> impl IntoResponse {
    // Validate permission format: at least 3 colon-separated parts
    let parts: Vec<&str> = req.permission.split(':').collect();
    if parts.len() < 3 {
        return AdminResponse::bad_request("Invalid permission format. Use: platform:resource:action").into_response();
    }

    // Validate each part
    for part in &parts {
        if part.is_empty() || (!part.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-' || c == '*')) {
            return AdminResponse::bad_request("Permission parts must be alphanumeric, underscore, dash, or wildcard (*)").into_response();
        }
    }

    let mut conn = match app_state.db_pool.get().await {
        Ok(conn) => conn,
        Err(e) => {
            tracing::error!("Failed to get database connection: {}", e);
            return AdminResponse::server_error("Database connection failed").into_response();
        }
    };

    let permission = req.permission.to_lowercase();
    let platform = req.platform.unwrap_or_else(|| parts[0].to_string());
    let category = req.category.or_else(|| Some(parts[1].to_string()));
    
    // Derive name from permission if not provided
    let name = req.name.or_else(|| {
        Some(format!("{} {}", 
            parts[1].replace('_', " ").replace('-', " "),
            parts[2].replace('_', " ").replace('-', " ")
        ).to_uppercase())
    });

    #[derive(diesel::QueryableByName)]
    struct NewPermRow {
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        id: Uuid,
        #[diesel(sql_type = diesel::sql_types::Varchar)]
        permission: String,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Varchar>)]
        name: Option<String>,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
        description: Option<String>,
        #[diesel(sql_type = diesel::sql_types::Varchar)]
        platform: String,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Varchar>)]
        category: Option<String>,
        #[diesel(sql_type = diesel::sql_types::Bool)]
        is_system: bool,
        #[diesel(sql_type = diesel::sql_types::Bool)]
        is_active: bool,
        #[diesel(sql_type = diesel::sql_types::Timestamptz)]
        created_at: chrono::DateTime<chrono::Utc>,
    }

    let result = diesel::sql_query(
        "INSERT INTO permission_definitions (permission, name, description, platform, category, is_system, is_active)
         VALUES ($1, $2, $3, $4, $5, FALSE, TRUE)
         ON CONFLICT (permission) DO UPDATE SET is_active = TRUE, updated_at = NOW()
         RETURNING id, permission, name, description, platform, category, is_system, is_active, created_at"
    )
    .bind::<diesel::sql_types::Varchar, _>(&permission)
    .bind::<diesel::sql_types::Nullable<diesel::sql_types::Varchar>, _>(&name)
    .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(&req.description)
    .bind::<diesel::sql_types::Varchar, _>(&platform)
    .bind::<diesel::sql_types::Nullable<diesel::sql_types::Varchar>, _>(&category)
    .get_result::<NewPermRow>(&mut conn)
    .await;

    match result {
        Ok(row) => {
            AdminResponse::success(PermissionDefinition {
                id: row.id,
                permission: row.permission,
                name: row.name,
                description: row.description,
                platform: row.platform,
                category: row.category,
                is_system: row.is_system,
                is_active: row.is_active,
                created_at: row.created_at,
            }).into_response()
        }
        Err(e) => {
            tracing::error!("Failed to create permission definition: {}", e);
            AdminResponse::server_error("Failed to create permission").into_response()
        }
    }
}

/// Delete a permission definition (soft delete by setting is_active = false)
/// DELETE /api/v1/permissions/definitions/{id}
pub async fn delete_permission_definition(
    State(app_state): State<AppState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    let mut conn = match app_state.db_pool.get().await {
        Ok(conn) => conn,
        Err(e) => {
            tracing::error!("Failed to get database connection: {}", e);
            return AdminResponse::server_error("Database connection failed").into_response();
        }
    };

    // Check if it's a system permission
    #[derive(diesel::QueryableByName)]
    struct CheckRow {
        #[diesel(sql_type = diesel::sql_types::Bool)]
        is_system: bool,
    }

    let check_result = diesel::sql_query(
        "SELECT is_system FROM permission_definitions WHERE id = $1"
    )
    .bind::<diesel::sql_types::Uuid, _>(id)
    .get_result::<CheckRow>(&mut conn)
    .await;

    match check_result {
        Ok(row) => {
            if row.is_system {
                return AdminResponse::bad_request("Cannot delete system permissions").into_response();
            }
        }
        Err(diesel::result::Error::NotFound) => {
            return AdminResponse::not_found("Permission not found").into_response();
        }
        Err(e) => {
            tracing::error!("Failed to check permission: {}", e);
            return AdminResponse::server_error("Database query failed").into_response();
        }
    }

    // Soft delete by setting is_active = false
    let result = diesel::sql_query(
        "UPDATE permission_definitions SET is_active = FALSE, updated_at = NOW() WHERE id = $1 AND is_system = FALSE"
    )
    .bind::<diesel::sql_types::Uuid, _>(id)
    .execute(&mut conn)
    .await;

    match result {
        Ok(count) => {
            if count > 0 {
                AdminResponse::success("Permission deleted").into_response()
            } else {
                AdminResponse::not_found("Permission not found or is a system permission").into_response()
            }
        }
        Err(e) => {
            tracing::error!("Failed to delete permission: {}", e);
            AdminResponse::server_error("Failed to delete permission").into_response()
        }
    }
}

/// Delete a permission definition by permission string
/// DELETE /api/v1/permissions/definitions/by-name/{permission}
pub async fn delete_permission_by_name(
    State(app_state): State<AppState>,
    Path(permission): Path<String>,
) -> impl IntoResponse {
    let mut conn = match app_state.db_pool.get().await {
        Ok(conn) => conn,
        Err(e) => {
            tracing::error!("Failed to get database connection: {}", e);
            return AdminResponse::server_error("Database connection failed").into_response();
        }
    };

    let permission = permission.to_lowercase();

    // Soft delete by setting is_active = false
    let result = diesel::sql_query(
        "UPDATE permission_definitions SET is_active = FALSE, updated_at = NOW() WHERE permission = $1 AND is_system = FALSE"
    )
    .bind::<diesel::sql_types::Varchar, _>(&permission)
    .execute(&mut conn)
    .await;

    match result {
        Ok(count) => {
            if count > 0 {
                AdminResponse::success("Permission deleted").into_response()
            } else {
                AdminResponse::not_found("Permission not found or is a system permission").into_response()
            }
        }
        Err(e) => {
            tracing::error!("Failed to delete permission: {}", e);
            AdminResponse::server_error("Failed to delete permission").into_response()
        }
    }
}
