// Permission Plan CRUD Operations
// Consolidates plan management from permission_plan_handlers.rs

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
use std::collections::HashMap;

use crate::web::auth::AppState;
use crate::web::responses::{AdminResponse, create_pagination};
use crate::domain::permission_management::{
    PlanSlug, PermissionString, Plan, PlanId,
    aggregates::plan::{CreatePlanParams, UpdatePlanParams},
    repository_ports::PlanRepositoryPort,
};
use crate::domain::shared_kernel::aggregate_root::AggregateRoot;

// ============================================================================
// REQUEST/RESPONSE TYPES
// ============================================================================

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CreatePlanRequest {
    pub name: String,
    pub slug: String,
    pub description: String,
    pub plan_type: String,
    pub permissions: Vec<String>,
    #[schema(value_type = Option<f64>)]
    pub price: Option<BigDecimal>,
    pub currency: Option<String>,
    pub billing_cycle: Option<String>,
    pub is_active: Option<bool>,
    pub is_promoted: Option<bool>,
    pub tier_level: Option<i32>,
    pub max_members: Option<i32>,
    pub auto_assign_enabled: Option<bool>,
    pub plan_metadata: Option<serde_json::Value>,
    pub is_public: Option<bool>,
    pub default_expiry_days: Option<i32>,
    pub plan_category: Option<String>,
    pub plan_group: Option<String>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct UpdatePlanRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub permissions: Option<Vec<String>>,
    #[schema(value_type = Option<f64>)]
    pub price: Option<BigDecimal>,
    pub currency: Option<String>,
    pub billing_cycle: Option<String>,
    pub is_active: Option<bool>,
    pub is_promoted: Option<bool>,
    pub tier_level: Option<i32>,
    pub max_members: Option<i32>,
    pub auto_assign_enabled: Option<bool>,
    pub plan_metadata: Option<serde_json::Value>,
    pub is_public: Option<bool>,
    pub default_expiry_days: Option<i32>,
    pub grace_period_hours: Option<i32>,
    pub plan_category: Option<String>,
    pub plan_group: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct PlanResponse {
    pub id: String,
    pub name: String,
    pub slug: String,
    pub description: String,
    pub plan_type: String,
    pub permissions: Vec<String>,
    pub price: BigDecimal,
    pub currency: String,
    pub billing_cycle: String,
    pub is_active: bool,
    pub is_promoted: bool,
    pub tier_level: i32,
    pub max_members: Option<i32>,
    pub auto_assign_enabled: bool,
    pub plan_metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub member_count: i32,
    pub is_public: bool,
    pub is_system_plan: bool,
    pub default_expiry_days: Option<i32>,
    pub grace_period_hours: i32,
    pub plan_category: String,
    pub plan_group: String,
}

impl PlanResponse {
    /// Create from domain Plan with member count
    pub fn from_plan(plan: &Plan, member_count: i32) -> Self {
        Self {
            id: plan.id().to_string(),
            name: plan.name().to_string(),
            slug: plan.slug().as_str().to_string(),
            description: plan.description().to_string(),
            plan_type: plan.plan_type().to_string(),
            permissions: plan.permissions().iter().map(|p| p.as_str().to_string()).collect(),
            price: plan.price().to_string().parse::<BigDecimal>().unwrap_or_else(|_| BigDecimal::from(0)),
            currency: plan.currency().to_string(),
            billing_cycle: plan.billing_cycle().to_string(),
            is_active: plan.is_active(),
            is_promoted: plan.is_promoted(),
            tier_level: plan.tier_level(),
            max_members: plan.max_members(),
            auto_assign_enabled: plan.auto_assign_enabled(),
            plan_metadata: plan.metadata().clone(),
            created_at: plan.created_at(),
            updated_at: plan.updated_at(),
            member_count,
            is_public: plan.is_public(),
            is_system_plan: plan.is_system(),
            default_expiry_days: plan.metadata().get("default_expiry_days").and_then(|v| v.as_i64()).map(|v| v as i32),
            grace_period_hours: plan.grace_period_hours(),
            plan_category: plan.plan_category().as_str().to_string(),
            plan_group: plan.plan_group().as_str().to_string(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ListPlansQuery {
    pub page: Option<u32>,
    pub limit: Option<u32>,
    pub plan_type: Option<String>,
    pub is_active: Option<bool>,
    pub plan_group: Option<String>,
}


// ============================================================================
// HANDLERS
// ============================================================================

/// Create a new permission plan
/// POST /admin/permissions/plans
#[utoipa::path(
    post,
    path = "/admin/permissions/plans",
    tag = "admin-permissions",
    request_body = CreatePlanRequest,
    responses(
        (status = 201, description = "Permission plan created successfully"),
        (status = 400, description = "Bad request"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearerAuth" = []))
)]
pub async fn create_plan(
    State(app_state): State<AppState>,
    Json(req): Json<CreatePlanRequest>,
) -> impl IntoResponse {
    // Validate request
    if req.name.is_empty() || req.slug.is_empty() {
        return AdminResponse::bad_request("Name and slug are required").into_response();
    }

    // Parse slug into domain object
    let slug = match PlanSlug::new(req.slug.clone()) {
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
    let mut plan = match Plan::create(CreatePlanParams {
        name: req.name.clone(),
        slug,
        description: req.description.clone(),
        plan_type: req.plan_type.clone(),
        permissions,
        price: Some(price_f64),
        currency: req.currency.clone(),
        billing_cycle: req.billing_cycle.clone(),
        is_active: req.is_active,
        is_promoted: req.is_promoted,
        tier_level: req.tier_level,
        max_members: req.max_members,
        auto_assign_enabled: req.auto_assign_enabled,
        metadata: req.plan_metadata.clone(),
        is_public: req.is_public,
        grace_period_hours: None,
        plan_category: req.plan_category.as_deref().and_then(|s| crate::domain::permission_management::PlanCategory::from_str(s).ok()),
        plan_group: req.plan_group.as_deref().and_then(|s| crate::domain::permission_management::PlanGroup::from_str(s).ok()),
    }) {
        Ok(g) => g,
        Err(e) => {
            let error_msg = e.to_string();
            tracing::error!("Failed to create permission plan aggregate: {}", error_msg);
            return AdminResponse::bad_request(&error_msg).into_response();
        }
    };

    // Save to database using Diesel repository
    if let Err(e) = app_state.plan_repo.save(&plan).await {
        let error_string = e.to_string();
        tracing::error!("Failed to save permission plan: {}", error_string);
        
        // Check for duplicate key constraint violation
        if error_string.contains("duplicate key") || error_string.contains("unique constraint") {
            if error_string.contains("plans_name_key") {
                return AdminResponse::conflict(&format!("A permission plan with the name '{}' already exists", req.name)).into_response();
            } else if error_string.contains("plans_slug_key") {
                return AdminResponse::conflict(&format!("A permission plan with the slug '{}' already exists", req.slug)).into_response();
            }
            return AdminResponse::conflict("A permission plan with this name or slug already exists").into_response();
        }
        
        return AdminResponse::server_error("Failed to create plan").into_response();
    }

    // Inject default_expiry_days into metadata if provided
    if let Some(days) = req.default_expiry_days {
        let mut metadata = plan.metadata().clone();
        if let Some(obj) = metadata.as_object_mut() {
            obj.insert("default_expiry_days".to_string(), serde_json::json!(days));
        }
        // Save again with metadata
        let update_params = UpdatePlanParams {
            metadata: Some(metadata),
            ..Default::default()
        };
        let _ = plan.update(update_params);
    }
    AdminResponse::created(PlanResponse::from_plan(&plan, 0), "Permission plan created successfully").into_response()
}

/// Get a permission plan by ID
/// GET /admin/permissions/plans/:plan_id
#[utoipa::path(
    get,
    path = "/admin/permissions/plans/{plan_id}",
    tag = "admin-permissions",
    responses(
        (status = 200, description = "Successfully retrieved permission plan"),
        (status = 400, description = "Invalid plan ID format"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Permission plan not found"),
        (status = 500, description = "Internal server error")
    ),
    params(
        ("plan_id" = String, Path, description = "UUID of the permission plan")
    ),
    security(("bearerAuth" = []))
)]
pub async fn get_plan(
    State(app_state): State<AppState>,
    Path(plan_id): Path<String>,
) -> impl IntoResponse {
    // Check for constant Free Plan
    // if plan_id == crate::core::constants::FREE_PLAN_ID {
    //    return AdminResponse::success(get_constant_permission_plan()).into_response();
    // }

    let plan_uuid = match Uuid::parse_str(&plan_id) {
        Ok(id) => id,
        Err(_) => return AdminResponse::bad_request("Invalid plan ID format").into_response(),
    };

    let plan_id_obj = PlanId::from_uuid(plan_uuid);

    // Fetch plan using Diesel repository
    let plan = match app_state.plan_repo.find_by_id(&plan_id_obj).await {
        Ok(Some(g)) => g,
        Ok(None) => return AdminResponse::not_found("Permission plan").into_response(),
        Err(e) => {
            tracing::error!("Failed to fetch permission plan: {}", e);
            return AdminResponse::server_error("Database query failed").into_response();
        }
    };

    // Fetch member count
    let member_count = {
        let mut conn = match app_state.db_pool.get().await {
            Ok(conn) => conn,
            Err(e) => {
                tracing::error!("Failed to get database connection: {}", e);
                return AdminResponse::server_error("Database connection failed").into_response();
            }
        };

        #[derive(diesel::QueryableByName)]
        struct Count {
            #[diesel(sql_type = diesel::sql_types::BigInt)]
            count: i64,
        }

        match diesel::sql_query("SELECT COUNT(*) as count FROM wallet_plan_assignments WHERE plan_id = $1 AND is_active = true")
            .bind::<diesel::sql_types::Uuid, _>(*plan.id().value())
            .get_result::<Count>(&mut conn)
            .await 
        {
            Ok(c) => c.count as i32,
            Err(e) => {
                tracing::error!("Failed to fetch plan member count: {}", e);
                0
            }
        }
    };

    AdminResponse::success(PlanResponse::from_plan(&plan, member_count)).into_response()
}

/// List permission plans with pagination
/// GET /admin/permissions/plans
#[utoipa::path(
    get,
    path = "/admin/permissions/plans",
    tag = "admin-permissions",
    responses(
        (status = 200, description = "Successfully retrieved permission plans list"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 500, description = "Internal server error")
    ),
    params(
        ("page" = Option<u32>, Query, description = "Page number"),
        ("limit" = Option<u32>, Query, description = "Items per page"),
        ("plan_type" = Option<String>, Query, description = "Filter by plan type"),
        ("is_active" = Option<bool>, Query, description = "Filter by active status")
    ),
    security(("bearerAuth" = []))
)]
pub async fn list_plans(
    State(app_state): State<AppState>,
    Query(query): Query<ListPlansQuery>,
) -> impl IntoResponse {
    use crate::domain::permission_management::repository_ports::PlanSearchCriteria;

    let pg = crate::web::pagination::Pagination::standard(query.page, query.limit);

    // Build search criteria for Diesel repository
    let criteria = PlanSearchCriteria {
        plan_type: query.plan_type.clone(),
        is_active: query.is_active,
        is_promoted: None,
        plan_group: query.plan_group.clone(),
        search_term: None,
        limit: Some(pg.limit as i64),
        offset: Some(pg.offset),
    };

    // Get total count using the same criteria (without limit/offset)
    let count_criteria = PlanSearchCriteria {
        plan_type: query.plan_type,
        is_active: query.is_active,
        is_promoted: None,
        plan_group: query.plan_group,
        search_term: None,
        limit: None,
        offset: None,
    };

    let total = match app_state.plan_repo.count(count_criteria).await {
        Ok(count) => count,
        Err(e) => {
            tracing::error!("Failed to count permission plans: {}", e);
            return AdminResponse::server_error("Database query failed").into_response();
        }
    };

    // Fetch plans using Diesel repository
    let domain_plans = match app_state.plan_repo.find_all(criteria).await {
        Ok(plans) => plans,
        Err(e) => {
            tracing::error!("Failed to list permission plans: {}", e);
            return AdminResponse::server_error("Database query failed").into_response();
        }
    };

    // Fetch member counts for these plans
    let plan_ids: Vec<Uuid> = domain_plans.iter().map(|g| *g.id().value()).collect();
    
    let member_counts: HashMap<Uuid, i64> = if !plan_ids.is_empty() {
        match app_state.db_pool.get().await {
            Ok(mut conn) => {
                #[derive(diesel::QueryableByName)]
                struct CountRow {
                    #[diesel(sql_type = diesel::sql_types::Uuid)]
                    plan_id: Uuid,
                    #[diesel(sql_type = diesel::sql_types::BigInt)]
                    count: i64,
                }

                let sql = "SELECT plan_id, COUNT(*) as count FROM wallet_plan_assignments WHERE plan_id = ANY($1) AND is_active = true GROUP BY plan_id";

                match diesel::sql_query(sql)
                    .bind::<diesel::sql_types::Array<diesel::sql_types::Uuid>, _>(&plan_ids)
                    .load::<CountRow>(&mut conn)
                    .await
                {
                    Ok(rows) => rows.into_iter().map(|r| (r.plan_id, r.count)).collect(),
                    Err(e) => {
                        tracing::error!("Failed to fetch plan member counts: {}", e);
                        HashMap::new()
                    }
                }
            },
            Err(e) => {
                tracing::error!("Failed to get database connection for counts: {}", e);
                HashMap::new() // Fallback to 0 counts on error
            }
        }
    } else {
        HashMap::new()
    };

    // Convert domain models to response DTOs
    let plans: Vec<PlanResponse> = domain_plans.iter()
        .map(|plan| {
            let count = member_counts.get(plan.id().value()).unwrap_or(&0);
            PlanResponse::from_plan(plan, *count as i32)
        }).collect();

    let pagination = create_pagination(pg.page, pg.limit, total as u64);
    AdminResponse::success_with_pagination(plans, pagination).into_response()
}

/// Update a permission plan
/// PUT /admin/permissions/plans/:plan_id
#[utoipa::path(
    put,
    path = "/admin/permissions/plans/{plan_id}",
    tag = "admin-permissions",
    request_body = UpdatePlanRequest,
    responses(
        (status = 200, description = "Permission plan updated successfully"),
        (status = 400, description = "Invalid plan ID format"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Permission plan not found"),
        (status = 500, description = "Internal server error")
    ),
    params(
        ("plan_id" = String, Path, description = "UUID of the permission plan")
    ),
    security(("bearerAuth" = []))
)]
pub async fn update_plan(
    State(app_state): State<AppState>,
    Path(plan_id): Path<String>,
    Json(req): Json<UpdatePlanRequest>,
) -> impl IntoResponse {
    use crate::core::constants::{FREE_PLAN_ID, is_system_admin_plan};

    // Check for constant Free Plan locking
    if plan_id == FREE_PLAN_ID {
        if req.price.is_some() {
            return AdminResponse::bad_request("Price of the Free Plan is locked and cannot be modified").into_response();
        }
    }

    // System admin plans: block name/slug/category/group changes
    if is_system_admin_plan(&plan_id) {
        if req.name.is_some() || req.plan_category.is_some() || req.plan_group.is_some() {
            return AdminResponse::forbidden("System admin plans cannot be renamed or recategorized").into_response();
        }
    }
    // Block updates to constant Free Plan
    // if plan_id == crate::core::constants::FREE_PLAN_ID {
    //    return AdminResponse::forbidden("Constant Free Plan cannot be modified").into_response();
    // }

    // Parse plan ID
    let plan_uuid = match Uuid::parse_str(&plan_id) {
        Ok(id) => id,
        Err(_) => return AdminResponse::bad_request("Invalid plan ID format").into_response(),
    };

    let plan_id_obj = PlanId::from_uuid(plan_uuid);

    // Fetch existing plan
    let mut plan = match app_state.plan_repo.find_by_id(&plan_id_obj).await {
        Ok(Some(g)) => g,
        Ok(None) => return AdminResponse::not_found("Permission plan").into_response(),
        Err(e) => {
            tracing::error!("Failed to fetch permission plan: {}", e);
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
    let update_params = UpdatePlanParams {
        name: req.name,
        description: req.description,
        permissions,
        price: price_f64,
        currency: req.currency,
        billing_cycle: req.billing_cycle,
        is_active: req.is_active,
        is_promoted: req.is_promoted,
        tier_level: req.tier_level,
        max_members: req.max_members.map(Some), // Wrap in Option
        auto_assign_enabled: req.auto_assign_enabled,
        metadata: req.plan_metadata,
        is_public: req.is_public,
        grace_period_hours: req.grace_period_hours,
        plan_category: req.plan_category.as_deref().and_then(|s| crate::domain::permission_management::PlanCategory::from_str(s).ok()),
        plan_group: req.plan_group.as_deref().and_then(|s| crate::domain::permission_management::PlanGroup::from_str(s).ok()),
    };

    // If default_expiry_days is provided, merge it into metadata
    let mut update_params = update_params;
    if let Some(days) = req.default_expiry_days {
        let mut metadata = update_params.metadata.clone().unwrap_or_else(|| plan.metadata().clone());
        if let Some(obj) = metadata.as_object_mut() {
            obj.insert("default_expiry_days".to_string(), serde_json::json!(days));
        }
        update_params.metadata = Some(metadata);
    }

    // Update the plan
    if let Err(e) = plan.update(update_params) {
        tracing::error!("Failed to update permission plan: {}", e);
        return AdminResponse::bad_request(&e.to_string()).into_response();
    }

    // Save updated plan
    if let Err(e) = app_state.plan_repo.save(&plan).await {
        tracing::error!("Failed to save permission plan: {}", e);
        return AdminResponse::server_error("Failed to update plan").into_response();
    }

    // Clear analytics cache so updated ranking_offset takes effect immediately
    app_state.cache.clear();
    tracing::info!("Cleared analytics cache after plan update: {}", plan_id);

    // Re-read from DB to ensure response matches persisted state
    let saved_plan = match app_state.plan_repo.find_by_id(&plan_id_obj).await {
        Ok(Some(p)) => p,
        Ok(None) => return AdminResponse::not_found("Permission plan").into_response(),
        Err(e) => {
            tracing::error!("Failed to re-read plan after save: {}", e);
            // Fallback to in-memory model
            return AdminResponse::success(PlanResponse::from_plan(&plan, 0)).into_response();
        }
    };

    AdminResponse::success(PlanResponse::from_plan(&saved_plan, 0)).into_response()
}

/// Delete a permission plan
/// DELETE /admin/permissions/plans/:plan_id
#[utoipa::path(
    delete,
    path = "/admin/permissions/plans/{plan_id}",
    tag = "admin-permissions",
    responses(
        (status = 200, description = "Permission plan deleted successfully"),
        (status = 400, description = "Invalid plan ID format"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Permission plan not found"),
        (status = 500, description = "Internal server error")
    ),
    params(
        ("plan_id" = String, Path, description = "UUID of the permission plan")
    ),
    security(("bearerAuth" = []))
)]
pub async fn delete_plan(
    State(app_state): State<AppState>,
    Path(plan_id): Path<String>,
) -> impl IntoResponse {
    // Block deletion of constant Free Plan
    if plan_id == crate::core::constants::FREE_PLAN_ID {
        return AdminResponse::forbidden("Constant Free Plan cannot be deleted").into_response();
    }

    // Block deletion of system admin plans
    if crate::core::constants::is_system_admin_plan(&plan_id) {
        return AdminResponse::forbidden("System admin plans cannot be deleted").into_response();
    }

    // Parse plan ID
    let plan_uuid = match Uuid::parse_str(&plan_id) {
        Ok(id) => id,
        Err(_) => return AdminResponse::bad_request("Invalid plan ID format").into_response(),
    };

    let plan_id_obj = PlanId::from_uuid(plan_uuid);

    // Delete using Diesel repository
    match app_state.plan_repo.delete(&plan_id_obj).await {
        Ok(_) => {
            AdminResponse::success_with_message(serde_json::json!({"deleted": true}), "Plan deleted successfully").into_response()
        },
        Err(e) => {
            tracing::error!("Failed to delete plan: {}", e);
            AdminResponse::server_error("Failed to delete plan").into_response()
        }
    }
}

/// Get members of a permission plan
/// GET /admin/permissions/plans/:plan_id/members
#[utoipa::path(
    get,
    path = "/admin/permissions/plans/{plan_id}/members",
    tag = "admin-permissions",
    responses(
        (status = 200, description = "Successfully retrieved plan members"),
        (status = 400, description = "Invalid plan ID format"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 500, description = "Internal server error")
    ),
    params(
        ("plan_id" = String, Path, description = "UUID of the permission plan")
    ),
    security(("bearerAuth" = []))
)]
pub async fn get_plan_members(
    State(app_state): State<AppState>,
    Path(plan_id): Path<String>,
) -> impl IntoResponse {
    let plan_uuid = match Uuid::parse_str(&plan_id) {
        Ok(id) => id,
        Err(_) => return AdminResponse::bad_request("Invalid plan ID format").into_response(),
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
        "SELECT wallet_address FROM wallet_plan_assignments WHERE plan_id = $1"
    )
    .bind::<diesel::sql_types::Uuid, _>(plan_uuid)
    .load::<WalletAddress>(&mut conn)
    .await
    {
        Ok(rows) => rows.into_iter().map(|r| r.wallet_address).collect(),
        Err(e) => {
            tracing::error!("Failed to get plan members: {}", e);
            return AdminResponse::server_error("Database query failed").into_response();
        }
    };

    AdminResponse::success(serde_json::json!({"members": members, "count": members.len()})).into_response()
}

/// Get permissions of a permission plan
/// GET /permissions/plans/:plan_id/permissions
#[utoipa::path(
    get,
    path = "/permissions/plans/{plan_id}/permissions",
    tag = "permissions",
    responses(
        (status = 200, description = "Successfully retrieved plan permissions"),
        (status = 400, description = "Invalid plan ID format"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Permission plan not found"),
        (status = 500, description = "Internal server error")
    ),
    params(
        ("plan_id" = String, Path, description = "UUID of the permission plan")
    ),
    security(("bearerAuth" = []))
)]
pub async fn get_plan_permissions(
    State(app_state): State<AppState>,
    Path(plan_id): Path<String>,
) -> impl IntoResponse {
    let plan_uuid = match Uuid::parse_str(&plan_id) {
        Ok(id) => id,
        Err(_) => return AdminResponse::bad_request("Invalid plan ID format").into_response(),
    };

    let plan_id_obj = PlanId::from_uuid(plan_uuid);

    // Fetch plan using Diesel repository
    let plan = match app_state.plan_repo.find_by_id(&plan_id_obj).await {
        Ok(Some(g)) => g,
        Ok(None) => return AdminResponse::not_found("Permission plan").into_response(),
        Err(e) => {
            tracing::error!("Failed to fetch permission plan: {}", e);
            return AdminResponse::server_error("Database query failed").into_response();
        }
    };

    // Extract permissions from the plan
    let permissions: Vec<String> = plan.permissions()
        .iter()
        .map(|p| p.as_str().to_string())
        .collect();

    AdminResponse::success(serde_json::json!({
        "plan_id": plan_id,
        "plan_name": plan.name().to_string(),
        "permissions": permissions,
        "count": permissions.len()
    })).into_response()
}

/// Get assignments for a permission plan
/// GET /permissions/assignments/plan/:plan_id
#[utoipa::path(
    get,
    path = "/permissions/assignments/plan/{plan_id}",
    tag = "permissions",
    responses(
        (status = 200, description = "Successfully retrieved plan assignments"),
        (status = 400, description = "Invalid plan ID format"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    ),
    params(
        ("plan_id" = String, Path, description = "UUID of the permission plan")
    ),
    security(("bearerAuth" = []))
)]
pub async fn get_plan_assignments(
    State(app_state): State<AppState>,
    Path(plan_id): Path<String>,
    Query(query): Query<ListPlansQuery>,
) -> impl IntoResponse {
    let plan_uuid = match Uuid::parse_str(&plan_id) {
        Ok(id) => id,
        Err(_) => return AdminResponse::bad_request("Invalid plan ID format").into_response(),
    };

    let pg = crate::web::pagination::Pagination::standard(query.page, query.limit);

    let mut conn = match app_state.db_pool.get().await {
        Ok(conn) => conn,
        Err(e) => {
            tracing::error!("Failed to get database connection: {}", e);
            return AdminResponse::server_error("Database connection failed").into_response();
        }
    };

    #[derive(diesel::QueryableByName, Serialize)]
    struct AssignmentRow {
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        id: Uuid,
        #[diesel(sql_type = diesel::sql_types::Text)]
        wallet_address: String,
        #[diesel(sql_type = diesel::sql_types::Bool)]
        is_active: bool,
        #[diesel(sql_type = diesel::sql_types::Timestamptz)]
        created_at: DateTime<Utc>,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>)]
        expires_at: Option<DateTime<Utc>>,
    }

    // Get total count
    #[derive(diesel::QueryableByName)]
    struct Count {
        #[diesel(sql_type = diesel::sql_types::BigInt)]
        count: i64,
    }

    let total = match diesel::sql_query(
        "SELECT COUNT(*) as count FROM wallet_plan_assignments WHERE plan_id = $1"
    )
    .bind::<diesel::sql_types::Uuid, _>(plan_uuid)
    .get_result::<Count>(&mut conn)
    .await
    {
        Ok(c) => c.count,
        Err(e) => {
            tracing::error!("Failed to count plan assignments: {}", e);
            0
        }
    };

    // Fetch assignments with pagination
    let assignments: Vec<AssignmentRow> = match diesel::sql_query(
        "SELECT id, wallet_address, is_active, created_at, expires_at 
         FROM wallet_plan_assignments 
         WHERE plan_id = $1 
         ORDER BY created_at DESC 
         LIMIT $2 OFFSET $3"
    )
    .bind::<diesel::sql_types::Uuid, _>(plan_uuid)
    .bind::<diesel::sql_types::BigInt, _>(pg.limit as i64)
    .bind::<diesel::sql_types::BigInt, _>(pg.offset)
    .load::<AssignmentRow>(&mut conn)
    .await
    {
        Ok(rows) => rows,
        Err(e) => {
            tracing::error!("Failed to get plan assignments: {}", e);
            return AdminResponse::server_error("Database query failed").into_response();
        }
    };

    let pagination = create_pagination(pg.page, pg.limit, total as u64);
    AdminResponse::success_with_pagination(assignments, pagination).into_response()
}
