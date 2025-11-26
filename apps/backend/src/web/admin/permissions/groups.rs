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
use bigdecimal::BigDecimal;
use diesel_async::RunQueryDsl;

use crate::web::auth::AppState;
use crate::web::responses::{AdminResponse, create_pagination};
use crate::domain::permission_management::{
    GroupSlug, PermissionString, PermissionGroup, GroupId,
    aggregates::permission_group::{CreatePermissionGroupParams, UpdatePermissionGroupParams},
    repository_ports::PermissionGroupRepositoryPort,
};
use crate::domain::shared_kernel::aggregate_root::AggregateRoot;

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

    // Parse slug into domain object
    let slug = match GroupSlug::new(req.slug.clone()) {
        Ok(s) => s,
        Err(e) => {
            tracing::error!("Invalid slug: {}", e);
            return AdminResponse::bad_request("Invalid slug format").into_response();
        }
    };

    // Parse permissions into domain objects
    let permissions: Vec<PermissionString> = req.permissions
        .iter()
        .filter_map(|p| PermissionString::new(p.clone()).ok())
        .collect();

    // Convert BigDecimal to f64 for domain model
    let price_f64 = req.price
        .as_ref()
        .and_then(|bd| bd.to_string().parse::<f64>().ok())
        .unwrap_or(0.0);

    // Create domain aggregate
    let group = match PermissionGroup::create(CreatePermissionGroupParams {
        name: req.name.clone(),
        slug,
        description: req.description.clone(),
        group_type: req.group_type.clone(),
        permissions,
        price: Some(price_f64),
        currency: req.currency.clone(),
        billing_cycle: req.billing_cycle.clone(),
        is_active: req.is_active,
        is_promoted: req.is_promoted,
        display_order: req.display_order,
        max_members: req.max_members,
        auto_assign_enabled: req.auto_assign_enabled,
        metadata: req.group_metadata.clone(),
    }) {
        Ok(g) => g,
        Err(e) => {
            let error_msg = e.to_string();
            tracing::error!("Failed to create permission group aggregate: {}", error_msg);
            return AdminResponse::bad_request(&error_msg).into_response();
        }
    };

    // Save to database using Diesel repository
    if let Err(e) = app_state.permission_group_repo.save(&group).await {
        tracing::error!("Failed to save permission group: {}", e);
        return AdminResponse::server_error("Failed to create group").into_response();
    }

    // Build response
    let response = GroupResponse {
        id: group.id().to_string(),
        name: group.name().to_string(),
        slug: group.slug().as_str().to_string(),
        description: group.description().to_string(),
        group_type: group.group_type().to_string(),
        permissions: group.permissions().iter().map(|p| p.as_str().to_string()).collect(),
        price: req.price.unwrap_or_else(|| BigDecimal::from(0)),
        currency: group.currency().to_string(),
        billing_cycle: group.billing_cycle().to_string(),
        is_active: group.is_active(),
        is_promoted: group.is_promoted(),
        display_order: group.display_order(),
        max_members: group.max_members(),
        auto_assign_enabled: group.auto_assign_enabled(),
        group_metadata: group.metadata().clone(),
        created_at: group.created_at(),
        updated_at: group.updated_at(),
        member_count: 0,
    };

    AdminResponse::created(response, "Permission group created successfully").into_response()
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
    // Parse group ID
    let group_uuid = match Uuid::parse_str(&group_id) {
        Ok(id) => id,
        Err(_) => return AdminResponse::bad_request("Invalid group ID format").into_response(),
    };

    let group_id = GroupId::from_uuid(group_uuid);

    // Fetch group using Diesel repository
    let group = match app_state.permission_group_repo.find_by_id(&group_id).await {
        Ok(Some(g)) => g,
        Ok(None) => return AdminResponse::not_found("Permission group").into_response(),
        Err(e) => {
            tracing::error!("Failed to fetch permission group: {}", e);
            return AdminResponse::server_error("Database query failed").into_response();
        }
    };

    // Build response from domain model
    let response = GroupResponse {
        id: group.id().to_string(),
        name: group.name().to_string(),
        slug: group.slug().as_str().to_string(),
        description: group.description().to_string(),
        group_type: group.group_type().to_string(),
        permissions: group.permissions().iter().map(|p| p.as_str().to_string()).collect(),
        price: group.price().to_string().parse::<BigDecimal>().unwrap_or_else(|_| BigDecimal::from(0)),
        currency: group.currency().to_string(),
        billing_cycle: group.billing_cycle().to_string(),
        is_active: group.is_active(),
        is_promoted: group.is_promoted(),
        display_order: group.display_order(),
        max_members: group.max_members(),
        auto_assign_enabled: group.auto_assign_enabled(),
        group_metadata: group.metadata().clone(),
        created_at: group.created_at(),
        updated_at: group.updated_at(),
        member_count: 0, // TODO: Add member count query to repository
    };

    AdminResponse::success(response).into_response()
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
    use crate::domain::permission_management::repository_ports::GroupSearchCriteria;

    let page = query.page.unwrap_or(1);
    let limit = query.limit.unwrap_or(20).min(100);
    let offset = (page - 1) * limit;

    // Build search criteria for Diesel repository
    let criteria = GroupSearchCriteria {
        group_type: query.group_type.clone(),
        is_active: query.is_active,
        is_promoted: None,
        search_term: None,
        limit: Some(limit as i64),
        offset: Some(offset as i64),
    };

    // Get total count using the same criteria (without limit/offset)
    let count_criteria = GroupSearchCriteria {
        group_type: query.group_type,
        is_active: query.is_active,
        is_promoted: None,
        search_term: None,
        limit: None,
        offset: None,
    };

    let total = match app_state.permission_group_repo.count(count_criteria).await {
        Ok(count) => count,
        Err(e) => {
            tracing::error!("Failed to count permission groups: {}", e);
            return AdminResponse::server_error("Database query failed").into_response();
        }
    };

    // Fetch groups using Diesel repository
    let domain_groups = match app_state.permission_group_repo.find_all(criteria).await {
        Ok(groups) => groups,
        Err(e) => {
            tracing::error!("Failed to list permission groups: {}", e);
            return AdminResponse::server_error("Database query failed").into_response();
        }
    };

    // Convert domain models to response DTOs
    let groups: Vec<GroupResponse> = domain_groups.iter().map(|group| {
        GroupResponse {
            id: group.id().to_string(),
            name: group.name().to_string(),
            slug: group.slug().as_str().to_string(),
            description: group.description().to_string(),
            group_type: group.group_type().to_string(),
            permissions: group.permissions().iter().map(|p| p.as_str().to_string()).collect(),
            price: group.price().to_string().parse::<BigDecimal>().unwrap_or_else(|_| BigDecimal::from(0)),
            currency: group.currency().to_string(),
            billing_cycle: group.billing_cycle().to_string(),
            is_active: group.is_active(),
            is_promoted: group.is_promoted(),
            display_order: group.display_order(),
            max_members: group.max_members(),
            auto_assign_enabled: group.auto_assign_enabled(),
            group_metadata: group.metadata().clone(),
            created_at: group.created_at(),
            updated_at: group.updated_at(),
            member_count: 0, // TODO: Add member count query to repository
        }
    }).collect();

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
pub async fn update_group(
    State(app_state): State<AppState>,
    Path(group_id): Path<String>,
    Json(req): Json<UpdateGroupRequest>,
) -> impl IntoResponse {
    // Parse group ID
    let group_uuid = match Uuid::parse_str(&group_id) {
        Ok(id) => id,
        Err(_) => return AdminResponse::bad_request("Invalid group ID format").into_response(),
    };

    let group_id_obj = GroupId::from_uuid(group_uuid);

    // Fetch existing group
    let mut group = match app_state.permission_group_repo.find_by_id(&group_id_obj).await {
        Ok(Some(g)) => g,
        Ok(None) => return AdminResponse::not_found("Permission group").into_response(),
        Err(e) => {
            tracing::error!("Failed to fetch permission group: {}", e);
            return AdminResponse::server_error("Database query failed").into_response();
        }
    };

    // Parse permissions if provided
    let permissions = req.permissions.map(|perms| {
        perms.iter()
            .filter_map(|p| PermissionString::new(p.clone()).ok())
            .collect()
    });

    // Convert BigDecimal to f64 if provided
    let price_f64 = req.price.as_ref()
        .and_then(|bd| bd.to_string().parse::<f64>().ok());

    // Build update parameters
    let update_params = UpdatePermissionGroupParams {
        name: req.name,
        description: req.description,
        permissions,
        price: price_f64,
        currency: req.currency,
        billing_cycle: req.billing_cycle,
        is_active: req.is_active,
        is_promoted: req.is_promoted,
        display_order: req.display_order,
        max_members: req.max_members.map(Some), // Wrap in Option
        auto_assign_enabled: req.auto_assign_enabled,
        metadata: req.group_metadata,
    };

    // Update the group
    if let Err(e) = group.update(update_params) {
        tracing::error!("Failed to update permission group: {}", e);
        return AdminResponse::bad_request(&e.to_string()).into_response();
    }

    // Save updated group
    if let Err(e) = app_state.permission_group_repo.save(&group).await {
        tracing::error!("Failed to save permission group: {}", e);
        return AdminResponse::server_error("Failed to update group").into_response();
    }

    AdminResponse::success_with_message(serde_json::json!({"id": group_id}), "Group updated successfully").into_response()
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
    // Parse group ID
    let group_uuid = match Uuid::parse_str(&group_id) {
        Ok(id) => id,
        Err(_) => return AdminResponse::bad_request("Invalid group ID format").into_response(),
    };

    let group_id_obj = GroupId::from_uuid(group_uuid);

    // Delete using Diesel repository
    match app_state.permission_group_repo.delete(&group_id_obj).await {
        Ok(_) => {
            AdminResponse::success_with_message(serde_json::json!({"deleted": true}), "Group deleted successfully").into_response()
        },
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

    let mut conn = match app_state.db_pool.get().await {
        Ok(conn) => conn,
        Err(e) => {
            tracing::error!("Failed to get database connection: {}", e);
            return AdminResponse::server_error("Database connection failed").into_response();
        }
    };

    #[derive(diesel::QueryableByName)]
    struct WalletAddress {
        #[diesel(sql_type = diesel::sql_types::Text)]
        wallet_address: String,
    }

    let members: Vec<String> = match diesel::sql_query(
        "SELECT wallet_address FROM wallet_group_assignments WHERE group_id = $1"
    )
    .bind::<diesel::sql_types::Uuid, _>(group_uuid)
    .load::<WalletAddress>(&mut conn)
    .await
    {
        Ok(rows) => rows.into_iter().map(|r| r.wallet_address).collect(),
        Err(e) => {
            tracing::error!("Failed to get group members: {}", e);
            return AdminResponse::server_error("Database query failed").into_response();
        }
    };

    AdminResponse::success(serde_json::json!({"members": members, "count": members.len()})).into_response()
}
