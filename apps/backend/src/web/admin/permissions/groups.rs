// Permission Group CRUD Operations
// Consolidates group management from permission_group_handlers.rs

use axum::{
    extract::{Path, Query, State},
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use sqlx::Row;
use bigdecimal::BigDecimal;

use crate::web::auth::AppState;
use crate::web::responses::{AdminResponse, create_pagination};

// ============================================================================
// REQUEST/RESPONSE TYPES
// ============================================================================

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CreateGroupRequest {
    pub name: String,
    pub slug: String,
    pub description: String,
    pub group_type: String,
    pub permissions: Vec<String>,
    #[schema(value_type = Option<f64>)]
    pub price: Option<BigDecimal>,
    pub currency: Option<String>,
    pub billing_cycle: Option<String>,
    pub is_active: Option<bool>,
    pub is_promoted: Option<bool>,
    pub display_order: Option<i32>,
    pub max_members: Option<i32>,
    pub auto_assign_enabled: Option<bool>,
    pub group_metadata: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct UpdateGroupRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub permissions: Option<Vec<String>>,
    #[schema(value_type = Option<f64>)]
    pub price: Option<BigDecimal>,
    pub currency: Option<String>,
    pub billing_cycle: Option<String>,
    pub is_active: Option<bool>,
    pub is_promoted: Option<bool>,
    pub display_order: Option<i32>,
    pub max_members: Option<i32>,
    pub auto_assign_enabled: Option<bool>,
    pub group_metadata: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
pub struct GroupResponse {
    pub id: String,
    pub name: String,
    pub slug: String,
    pub description: String,
    pub group_type: String,
    pub permissions: Vec<String>,
    pub price: BigDecimal,
    pub currency: String,
    pub billing_cycle: String,
    pub is_active: bool,
    pub is_promoted: bool,
    pub display_order: i32,
    pub max_members: Option<i32>,
    pub auto_assign_enabled: bool,
    pub group_metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub member_count: i32,
}

#[derive(Debug, Deserialize)]
pub struct ListGroupsQuery {
    pub page: Option<u32>,
    pub limit: Option<u32>,
    pub group_type: Option<String>,
    pub is_active: Option<bool>,
}

// ============================================================================
// HANDLERS
// ============================================================================

/// Create a new permission group
/// POST /admin/permissions/groups
#[utoipa::path(
    post,
    path = "/admin/permissions/groups",
    tag = "admin-permissions",
    request_body = CreateGroupRequest,
    responses(
        (status = 201, description = "Permission group created successfully"),
        (status = 400, description = "Bad request"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearerAuth" = []))
)]
pub async fn create_group(
    State(app_state): State<AppState>,
    Json(req): Json<CreateGroupRequest>,
) -> impl IntoResponse {
    // Validate request
    if req.name.is_empty() || req.slug.is_empty() {
        return AdminResponse::bad_request("Name and slug are required").into_response();
    }

    // Begin transaction
    let mut tx = match app_state.db_pool.begin().await {
        Ok(tx) => tx,
        Err(e) => {
            tracing::error!("Failed to begin transaction: {}", e);
            return AdminResponse::server_error("Database transaction failed").into_response();
        }
    };

    // Insert permission group
    let group_id = match sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO permission_groups (
            name, slug, description, group_type, price, currency, billing_cycle,
            is_active, is_promoted, display_order, max_members, auto_assign_enabled, group_metadata
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
        RETURNING id
        "#
    )
    .bind(&req.name)
    .bind(&req.slug)
    .bind(&req.description)
    .bind(&req.group_type)
    .bind(req.price.clone().unwrap_or_else(|| BigDecimal::from(0)))
    .bind(req.currency.as_deref().unwrap_or("USD"))
    .bind(req.billing_cycle.as_deref().unwrap_or("monthly"))
    .bind(req.is_active.unwrap_or(true))
    .bind(req.is_promoted.unwrap_or(false))
    .bind(req.display_order.unwrap_or(0))
    .bind(req.max_members)
    .bind(req.auto_assign_enabled.unwrap_or(false))
    .bind(req.group_metadata.as_ref().unwrap_or(&serde_json::json!({})))
    .fetch_one(&mut *tx)
    .await
    {
        Ok(id) => id,
        Err(e) => {
            tracing::error!("Failed to create permission group: {}", e);
            return AdminResponse::server_error("Failed to create group").into_response();
        }
    };

    // Link permissions to group
    for perm_string in &req.permissions {
        let parts: Vec<&str> = perm_string.split(':').collect();
        if parts.len() < 3 {
            continue;
        }

        // Get or create permission
        let perm_id = match sqlx::query_scalar::<_, Uuid>(
            r#"
            INSERT INTO permissions (permission_string, platform, resource, action, permission_type)
            VALUES ($1, $2, $3, $4, 'manual')
            ON CONFLICT (permission_string) DO UPDATE SET permission_string = EXCLUDED.permission_string
            RETURNING id
            "#
        )
        .bind(perm_string)
        .bind(parts[0])
        .bind(parts[1])
        .bind(parts[2])
        .fetch_one(&mut *tx)
        .await
        {
            Ok(id) => id,
            Err(e) => {
                tracing::error!("Failed to create/get permission: {}", e);
                continue;
            }
        };

        // Link permission to group
        let _ = sqlx::query(
            "INSERT INTO permission_group_memberships (group_id, permission_id) VALUES ($1, $2) ON CONFLICT DO NOTHING"
        )
        .bind(group_id)
        .bind(perm_id)
        .execute(&mut *tx)
        .await;
    }

    // Commit transaction
    if let Err(e) = tx.commit().await {
        tracing::error!("Failed to commit transaction: {}", e);
        return AdminResponse::server_error("Failed to save group").into_response();
    }

    // Build response
    let group = GroupResponse {
        id: group_id.to_string(),
        name: req.name,
        slug: req.slug,
        description: req.description,
        group_type: req.group_type,
        permissions: req.permissions,
        price: req.price.unwrap_or_else(|| BigDecimal::from(0)),
        currency: req.currency.unwrap_or_else(|| "USD".to_string()),
        billing_cycle: req.billing_cycle.unwrap_or_else(|| "monthly".to_string()),
        is_active: req.is_active.unwrap_or(true),
        is_promoted: req.is_promoted.unwrap_or(false),
        display_order: req.display_order.unwrap_or(0),
        max_members: req.max_members,
        auto_assign_enabled: req.auto_assign_enabled.unwrap_or(false),
        group_metadata: req.group_metadata.unwrap_or(serde_json::json!({})),
        created_at: Utc::now(),
        updated_at: Utc::now(),
        member_count: 0,
    };

    AdminResponse::created(group, "Permission group created successfully").into_response()
}

/// Get a permission group by ID
/// GET /admin/permissions/groups/:group_id
#[utoipa::path(
    get,
    path = "/admin/permissions/groups/{group_id}",
    tag = "admin-permissions",
    responses(
        (status = 200, description = "Successfully retrieved permission group"),
        (status = 400, description = "Invalid group ID format"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Permission group not found"),
        (status = 500, description = "Internal server error")
    ),
    params(
        ("group_id" = String, Path, description = "UUID of the permission group")
    ),
    security(("bearerAuth" = []))
)]
pub async fn get_group(
    State(app_state): State<AppState>,
    Path(group_id): Path<String>,
) -> impl IntoResponse {
    let group_uuid = match Uuid::parse_str(&group_id) {
        Ok(id) => id,
        Err(_) => return AdminResponse::bad_request("Invalid group ID format").into_response(),
    };

    // Query group with member count
    let row = match sqlx::query(
        r#"
        SELECT pg.*, COUNT(DISTINCT wga.wallet_address) as member_count
        FROM permission_groups pg
        LEFT JOIN wallet_group_assignments wga ON pg.id = wga.group_id
        WHERE pg.id = $1
        GROUP BY pg.id
        "#
    )
    .bind(group_uuid)
    .fetch_optional(&*app_state.db_pool)
    .await
    {
        Ok(Some(row)) => row,
        Ok(None) => return AdminResponse::not_found("Permission group").into_response(),
        Err(e) => {
            tracing::error!("Failed to fetch permission group: {}", e);
            return AdminResponse::server_error("Database query failed").into_response();
        }
    };

    // Get permissions for this group
    let permissions: Vec<String> = sqlx::query_scalar(
        r#"
        SELECT p.permission_string
        FROM permissions p
        JOIN permission_group_memberships pgm ON p.id = pgm.permission_id
        WHERE pgm.group_id = $1
        "#
    )
    .bind(group_uuid)
    .fetch_all(&*app_state.db_pool)
    .await
    .unwrap_or_default();

    let group = GroupResponse {
        id: row.get::<Uuid, _>("id").to_string(),
        name: row.get("name"),
        slug: row.get("slug"),
        description: row.get("description"),
        group_type: row.get("group_type"),
        permissions,
        price: row.get("price"),
        currency: row.get("currency"),
        billing_cycle: row.get("billing_cycle"),
        is_active: row.get("is_active"),
        is_promoted: row.get("is_promoted"),
        display_order: row.get("display_order"),
        max_members: row.get("max_members"),
        auto_assign_enabled: row.get("auto_assign_enabled"),
        group_metadata: row.get("group_metadata"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
        member_count: row.get::<i64, _>("member_count") as i32,
    };

    AdminResponse::success(group).into_response()
}

/// List permission groups with pagination
/// GET /admin/permissions/groups
#[utoipa::path(
    get,
    path = "/admin/permissions/groups",
    tag = "admin-permissions",
    responses(
        (status = 200, description = "Successfully retrieved permission groups list"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 500, description = "Internal server error")
    ),
    params(
        ("page" = Option<u32>, Query, description = "Page number"),
        ("limit" = Option<u32>, Query, description = "Items per page"),
        ("group_type" = Option<String>, Query, description = "Filter by group type"),
        ("is_active" = Option<bool>, Query, description = "Filter by active status")
    ),
    security(("bearerAuth" = []))
)]
pub async fn list_groups(
    State(app_state): State<AppState>,
    Query(query): Query<ListGroupsQuery>,
) -> impl IntoResponse {
    let page = query.page.unwrap_or(1);
    let limit = query.limit.unwrap_or(20).min(100);
    let offset = (page - 1) * limit;

    // Build query with optional filters
    let mut sql = String::from(
        r#"
        SELECT pg.*, COUNT(DISTINCT wga.wallet_address) as member_count
        FROM permission_groups pg
        LEFT JOIN wallet_group_assignments wga ON pg.id = wga.group_id
        "#
    );

    let mut where_clauses = Vec::new();
    let active_clause;

    if query.group_type.is_some() {
        where_clauses.push("pg.group_type = $3");
    }
    if query.is_active.is_some() {
        let clause_idx = if query.group_type.is_some() { 4 } else { 3 };
        active_clause = format!("pg.is_active = ${}", clause_idx);
        where_clauses.push(&active_clause);
    }

    if !where_clauses.is_empty() {
        sql.push_str(" WHERE ");
        sql.push_str(&where_clauses.join(" AND "));
    }

    sql.push_str(" GROUP BY pg.id ORDER BY pg.display_order, pg.created_at DESC LIMIT $1 OFFSET $2");

    // Get total count
    let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM permission_groups")
        .fetch_one(&*app_state.db_pool)
        .await
        .unwrap_or(0);

    // Execute query
    let mut query_builder = sqlx::query(&sql)
        .bind(limit as i32)
        .bind(offset as i32);

    if let Some(group_type) = &query.group_type {
        query_builder = query_builder.bind(group_type);
    }
    if let Some(is_active) = query.is_active {
        query_builder = query_builder.bind(is_active);
    }

    let rows = match query_builder.fetch_all(&*app_state.db_pool).await {
        Ok(rows) => rows,
        Err(e) => {
            tracing::error!("Failed to list permission groups: {}", e);
            return AdminResponse::server_error("Database query failed").into_response();
        }
    };

    let mut groups = Vec::new();
    for row in rows {
        let group_id: Uuid = row.get("id");

        // Get permissions for each group
        let permissions: Vec<String> = sqlx::query_scalar(
            "SELECT p.permission_string FROM permissions p
             JOIN permission_group_memberships pgm ON p.id = pgm.permission_id
             WHERE pgm.group_id = $1"
        )
        .bind(group_id)
        .fetch_all(&*app_state.db_pool)
        .await
        .unwrap_or_default();

        groups.push(GroupResponse {
            id: group_id.to_string(),
            name: row.get("name"),
            slug: row.get("slug"),
            description: row.get("description"),
            group_type: row.get("group_type"),
            permissions,
            price: row.get("price"),
            currency: row.get("currency"),
            billing_cycle: row.get("billing_cycle"),
            is_active: row.get("is_active"),
            is_promoted: row.get("is_promoted"),
            display_order: row.get("display_order"),
            max_members: row.get("max_members"),
            auto_assign_enabled: row.get("auto_assign_enabled"),
            group_metadata: row.get("group_metadata"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            member_count: row.get::<i64, _>("member_count") as i32,
        });
    }

    let pagination = create_pagination(page, limit, total as u64);
    AdminResponse::success_with_pagination(groups, pagination).into_response()
}

/// Update a permission group
/// PUT /admin/permissions/groups/:group_id
#[utoipa::path(
    put,
    path = "/admin/permissions/groups/{group_id}",
    tag = "admin-permissions",
    request_body = UpdateGroupRequest,
    responses(
        (status = 200, description = "Permission group updated successfully"),
        (status = 400, description = "Bad request"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Permission group not found"),
        (status = 500, description = "Internal server error")
    ),
    params(
        ("group_id" = String, Path, description = "UUID of the permission group")
    ),
    security(("bearerAuth" = []))
)]
pub async fn update_group(
    State(app_state): State<AppState>,
    Path(group_id): Path<String>,
    Json(req): Json<UpdateGroupRequest>,
) -> impl IntoResponse {
    let group_uuid = match Uuid::parse_str(&group_id) {
        Ok(id) => id,
        Err(_) => return AdminResponse::bad_request("Invalid group ID format").into_response(),
    };

    // Build dynamic UPDATE query
    let mut updates = Vec::new();
    let mut param_idx = 2; // $1 is group_id

    if req.name.is_some() { updates.push(format!("name = ${}", param_idx)); param_idx += 1; }
    if req.description.is_some() { updates.push(format!("description = ${}", param_idx)); param_idx += 1; }
    if req.price.is_some() { updates.push(format!("price = ${}", param_idx)); param_idx += 1; }
    if req.is_active.is_some() { updates.push(format!("is_active = ${}", param_idx)); }

    if updates.is_empty() {
        return AdminResponse::bad_request("No fields to update").into_response();
    }

    updates.push("updated_at = NOW()".to_string());
    let sql = format!("UPDATE permission_groups SET {} WHERE id = $1", updates.join(", "));

    let mut query = sqlx::query(&sql).bind(group_uuid);
    if let Some(name) = req.name { query = query.bind(name); }
    if let Some(desc) = req.description { query = query.bind(desc); }
    if let Some(price) = req.price { query = query.bind(price); }
    if let Some(active) = req.is_active { query = query.bind(active); }

    match query.execute(&*app_state.db_pool).await {
        Ok(_) => AdminResponse::success_with_message(serde_json::json!({"id": group_id}), "Group updated successfully").into_response(),
        Err(e) => {
            tracing::error!("Failed to update group: {}", e);
            AdminResponse::server_error("Failed to update group").into_response()
        }
    }
}

/// Delete a permission group
/// DELETE /admin/permissions/groups/:group_id
#[utoipa::path(
    delete,
    path = "/admin/permissions/groups/{group_id}",
    tag = "admin-permissions",
    responses(
        (status = 200, description = "Permission group deleted successfully"),
        (status = 400, description = "Invalid group ID format"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Permission group not found"),
        (status = 500, description = "Internal server error")
    ),
    params(
        ("group_id" = String, Path, description = "UUID of the permission group")
    ),
    security(("bearerAuth" = []))
)]
pub async fn delete_group(
    State(app_state): State<AppState>,
    Path(group_id): Path<String>,
) -> impl IntoResponse {
    let group_uuid = match Uuid::parse_str(&group_id) {
        Ok(id) => id,
        Err(_) => return AdminResponse::bad_request("Invalid group ID format").into_response(),
    };

    match sqlx::query("DELETE FROM permission_groups WHERE id = $1")
        .bind(group_uuid)
        .execute(&*app_state.db_pool)
        .await
    {
        Ok(result) if result.rows_affected() > 0 => {
            AdminResponse::success_with_message(serde_json::json!({"deleted": true}), "Group deleted successfully").into_response()
        },
        Ok(_) => AdminResponse::not_found("Permission group").into_response(),
        Err(e) => {
            tracing::error!("Failed to delete group: {}", e);
            AdminResponse::server_error("Failed to delete group").into_response()
        }
    }
}

/// Get members of a permission group
/// GET /admin/permissions/groups/:group_id/members
#[utoipa::path(
    get,
    path = "/admin/permissions/groups/{group_id}/members",
    tag = "admin-permissions",
    responses(
        (status = 200, description = "Successfully retrieved group members"),
        (status = 400, description = "Invalid group ID format"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 500, description = "Internal server error")
    ),
    params(
        ("group_id" = String, Path, description = "UUID of the permission group")
    ),
    security(("bearerAuth" = []))
)]
pub async fn get_group_members(
    State(app_state): State<AppState>,
    Path(group_id): Path<String>,
) -> impl IntoResponse {
    let group_uuid = match Uuid::parse_str(&group_id) {
        Ok(id) => id,
        Err(_) => return AdminResponse::bad_request("Invalid group ID format").into_response(),
    };

    let members: Vec<String> = match sqlx::query_scalar(
        "SELECT wallet_address FROM wallet_group_assignments WHERE group_id = $1"
    )
    .bind(group_uuid)
    .fetch_all(&*app_state.db_pool)
    .await
    {
        Ok(m) => m,
        Err(e) => {
            tracing::error!("Failed to get group members: {}", e);
            return AdminResponse::server_error("Database query failed").into_response();
        }
    };

    AdminResponse::success(serde_json::json!({"members": members, "count": members.len()})).into_response()
}
