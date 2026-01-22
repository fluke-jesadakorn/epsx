//! User Developer Portal Handlers
//!
//! User-facing API endpoints for managing their own API keys.
//! These routes are scoped to the authenticated user's wallet address.

use axum::{
    extract::{Path, Query, State},
    response::IntoResponse,
    Extension, Json,
};
use serde::{Deserialize, Serialize};
use tracing::{error, info};
use uuid::Uuid;

use crate::domain::developer_portal::{CreateApiKeyRequest, RevokeApiKeyRequest, UsageService};
use crate::infrastructure::adapters::repositories::developer_portal::ApiKeyRepository;
use crate::web::auth::AppState;
use crate::web::responses::UnifiedApiResponse;

// ============================================================================
// Request/Response DTOs
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct ListMyKeysQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    pub status: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateMyApiKeyBody {
    pub client_name: String,
    pub client_description: Option<String>,
    pub plan_ids: Vec<String>, // Permission plans to assign
    #[serde(default)]
    pub permissions: Vec<String>, // Individual permission strings
    pub ip_restrictions: Option<Vec<String>>,
    pub expires_at: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RevokeMyApiKeyBody {
    pub reason: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct MyApiKeyListResponse {
    pub api_keys: Vec<MaskedApiKey>,
    pub total: i64,
}

#[derive(Debug, Serialize)]
pub struct MaskedApiKey {
    pub id: String,
    pub key_preview: String, // e.g., "epsx_xxxx...xxxx"
    pub full_key: Option<String>, // Full key for copying (only for owner)
    pub client_name: String,
    pub client_description: Option<String>,
    pub status: String,
    pub plans: Vec<PermissionPlanInfo>,
    /// Individual permissions selected when creating the key
    pub permissions: Vec<String>,
    pub total_requests: i64,
    pub expires_at: Option<String>,
    pub last_used_at: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Serialize)]
pub struct PermissionPlanInfo {
    pub id: String,
    pub name: String,
    pub slug: String,
}

#[derive(Debug, Serialize)]
pub struct AvailablePlansResponse {
    pub plans: Vec<AvailablePlan>,
}

#[derive(Debug, Serialize)]
pub struct AvailablePlan {
    pub id: String,
    pub name: String,
    pub slug: String,
    pub description: String,
    pub permissions: Vec<String>,
    pub plan_type: String,
    pub is_active: bool,
}

/// Response for user's assigned plans
#[derive(Debug, Serialize)]
pub struct MyPlansResponse {
    pub plans: Vec<UserAssignedPlan>,
    pub total_api_keys: i64,
    pub total_requests: i64,
}

/// User's assigned plan with metadata
#[derive(Debug, Serialize)]
pub struct UserAssignedPlan {
    pub id: String,
    pub name: String,
    pub slug: String,
    pub description: String,
    pub plan_type: String,
    pub permissions: Vec<String>,
    /// When this plan assignment expires (None = never)
    pub expires_at: Option<String>,
    /// Rate limit per minute for this plan
    pub rate_limit_per_minute: Option<i32>,
    /// Rate limit per day for this plan
    pub rate_limit_per_day: Option<i32>,
    /// When the user was assigned this plan
    pub assigned_at: String,
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Mask an API key prefix for display (e.g., "epsx_abc123..." -> "epsx_xxx...xxx")
fn mask_key_prefix(prefix: &str) -> String {
    if prefix.len() <= 8 {
        return format!("{}...", prefix);
    }
    let start = &prefix[..4];
    let end = &prefix[prefix.len().saturating_sub(3)..];
    format!("{}...{}", start, end)
}

// ============================================================================
// User API Key Handlers
// ============================================================================

/// GET /api/developer-portal/my-keys
/// List the authenticated user's API keys
pub async fn list_my_keys_handler(
    State(state): State<AppState>,
    Extension(wallet_address): Extension<String>,
    Query(query): Query<ListMyKeysQuery>,
) -> impl IntoResponse {
    let pool = *state.db_pool;
    let repo = ApiKeyRepository::new(pool);

    match repo.list_by_wallet(&wallet_address, query.limit, query.offset, query.status.as_deref()).await {
        Ok((api_keys, total)) => {
            // Convert to masked response
            let masked_keys: Vec<MaskedApiKey> = api_keys
                .into_iter()
                .map(|key| MaskedApiKey {
                    id: key.id.to_string(),
                    key_preview: mask_key_prefix(&key.key_prefix),
                    full_key: key.full_key,
                    client_name: key.client_name,
                    client_description: key.client_description,
                    status: key.status.to_string(),
                    plans: key.permission_plans.into_iter().map(|g| PermissionPlanInfo {
                        id: g.id.to_string(),
                        name: g.name,
                        slug: g.slug,
                    }).collect(),
                    permissions: key.selected_permissions,
                    total_requests: key.total_requests,
                    expires_at: key.expires_at.map(|dt| dt.to_rfc3339()),
                    last_used_at: key.last_used_at.map(|dt| dt.to_rfc3339()),
                    created_at: key.created_at.to_rfc3339(),
                })
                .collect();

            UnifiedApiResponse::success(MyApiKeyListResponse {
                api_keys: masked_keys,
                total,
            })
        }
        Err(e) => {
            error!("Failed to list user API keys: {}", e);
            UnifiedApiResponse::server_error(&e.to_string())
        }
    }
}

/// POST /api/developer-portal/my-keys
/// Create a new API key for the authenticated user
pub async fn create_my_key_handler(
    State(state): State<AppState>,
    Extension(wallet_address): Extension<String>,
    Json(body): Json<CreateMyApiKeyBody>,
) -> impl IntoResponse {
    let pool = *state.db_pool;
    let repo = ApiKeyRepository::new(pool);

    // Parse expires_at if provided
    let expires_at = if let Some(expires_str) = &body.expires_at {
        chrono::DateTime::parse_from_rfc3339(expires_str)
            .map(|dt| dt.with_timezone(&chrono::Utc))
            .ok()
    } else {
        None
    };

    // Convert plan_ids from strings to UUIDs
    let plan_ids: Vec<uuid::Uuid> = body.plan_ids.iter()
        .filter_map(|id| uuid::Uuid::parse_str(id).ok())
        .collect();

    let request = CreateApiKeyRequest {
        client_name: body.client_name,
        client_description: body.client_description,
        client_contact_email: None,
        wallet_address: wallet_address.clone(),
        allowed_modules: vec![], // Legacy, replaced by permission plans
        plan_ids,
        permissions: body.permissions, // Individual permission strings
        ip_restrictions: body.ip_restrictions,
        rate_limit_per_minute: Some(60),
        rate_limit_per_day: Some(10000),
        expires_at,
        created_by: wallet_address,
    };

    match repo.create(request).await {
        Ok(response) => {
            info!("Created API key for user: {}", response.api_key.id);
            UnifiedApiResponse::success(response)
        }
        Err(e) => {
            error!("Failed to create API key: {}", e);
            UnifiedApiResponse::server_error(&e.to_string())
        }
    }
}

/// GET /api/developer-portal/my-keys/:id
/// Get details of a specific API key owned by the user
pub async fn get_my_key_handler(
    State(state): State<AppState>,
    Extension(wallet_address): Extension<String>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let pool = *state.db_pool;
    let repo = ApiKeyRepository::new(pool);

    let uuid = match Uuid::parse_str(&id) {
        Ok(u) => u,
        Err(_) => {
            return UnifiedApiResponse::error(400, "Invalid UUID", "The provided ID is not a valid UUID")
        }
    };

    match repo.get_by_id(uuid).await {
        Ok(Some(api_key)) => {
            // Check ownership
            if api_key.wallet_address.to_lowercase() != wallet_address.to_lowercase() {
                return UnifiedApiResponse::error(
                    403,
                    "Forbidden",
                    "You do not have permission to view this API key",
                );
            }

            // Return masked key
            let masked = MaskedApiKey {
                id: api_key.id.to_string(),
                key_preview: mask_key_prefix(&api_key.key_prefix),
                full_key: api_key.full_key,
                client_name: api_key.client_name,
                client_description: api_key.client_description,
                status: api_key.status.to_string(),
                plans: api_key.permission_plans.into_iter().map(|g| PermissionPlanInfo {
                    id: g.id.to_string(),
                    name: g.name,
                    slug: g.slug,
                }).collect(),
                permissions: api_key.selected_permissions,
                total_requests: api_key.total_requests,
                expires_at: api_key.expires_at.map(|dt| dt.to_rfc3339()),
                last_used_at: api_key.last_used_at.map(|dt| dt.to_rfc3339()),
                created_at: api_key.created_at.to_rfc3339(),
            };

            UnifiedApiResponse::success(masked)
        }
        Ok(None) => UnifiedApiResponse::not_found("API key"),
        Err(e) => {
            error!("Failed to get API key: {}", e);
            UnifiedApiResponse::server_error(&e.to_string())
        }
    }
}

/// DELETE /api/developer-portal/my-keys/:id
/// Revoke an API key owned by the user
pub async fn revoke_my_key_handler(
    State(state): State<AppState>,
    Extension(wallet_address): Extension<String>,
    Path(id): Path<String>,
    Json(body): Json<RevokeMyApiKeyBody>,
) -> impl IntoResponse {
    let pool = *state.db_pool;
    let repo = ApiKeyRepository::new(pool);

    let uuid = match Uuid::parse_str(&id) {
        Ok(u) => u,
        Err(e) => {
            error!("Invalid UUID provided for revocation: {} - error: {}", id, e);
            return UnifiedApiResponse::error(400, "Invalid UUID", "The provided ID is not a valid UUID")
        }
    };
    
    info!("Attempting to revoke key {} for wallet {}", uuid, wallet_address);

    // First verify ownership
    match repo.get_by_id(uuid).await {
        Ok(Some(api_key)) => {
            if api_key.wallet_address.to_lowercase() != wallet_address.to_lowercase() {
                return UnifiedApiResponse::error(
                    403,
                    "Forbidden",
                    "You do not have permission to revoke this API key",
                );
            }
        }
        Ok(None) => return UnifiedApiResponse::not_found("API key"),
        Err(e) => return UnifiedApiResponse::server_error(&e.to_string()),
    }

    let request = RevokeApiKeyRequest {
        reason: body.reason.unwrap_or_else(|| "Revoked by owner".to_string()),
        revoked_by: wallet_address,
    };

    match repo.revoke(uuid, request).await {
        Ok(api_key) => {
            info!("User revoked API key: {}", uuid);
            UnifiedApiResponse::success(serde_json::json!({
                "success": true,
                "message": "API key revoked successfully",
                "id": api_key.id.to_string()
            }))
        }
        Err(e) => {
            error!("Failed to revoke API key: {}", e);
            UnifiedApiResponse::server_error(&e.to_string())
        }
    }
}

/// GET /api/developer-portal/available-plans
/// List permission plans available for API key assignment
pub async fn list_available_plans_handler(
    State(state): State<AppState>,
) -> impl IntoResponse {
    use diesel::prelude::*;
    use diesel_async::RunQueryDsl;
    use crate::schemas::primary::{plans, plan_permissions, permissions};

    let pool = *state.db_pool;
    let mut conn = match pool.get().await {
        Ok(c) => c,
        Err(e) => {
            error!("Pool error: {}", e);
            return UnifiedApiResponse::server_error(&e.to_string());
        }
    };

    // Query active plans with their associated permissions
    #[derive(diesel::Queryable)]
    struct PlanRow {
        id: uuid::Uuid,
        name: String,
        slug: String,
        description: String,
        plan_type: String,
        is_active: bool,
    }

    // Exclude 'free' plan from available plans for API key assignment
    let active_plans = match plans::table
        .filter(plans::is_active.eq(true))
        .filter(plans::slug.ne("free"))
        .select((
            plans::id,
            plans::name,
            plans::slug,
            plans::description,
            plans::plan_type,
            plans::is_active,
        ))
        .order(plans::name.asc())
        .load::<PlanRow>(&mut conn)
        .await
    {
        Ok(rows) => rows,
        Err(e) => {
            error!("Failed to query plans: {}", e);
            return UnifiedApiResponse::server_error(&e.to_string());
        }
    };

    // For each plan, fetch associated permissions
    let mut result_plans = Vec::new();
    for plan in active_plans {
        // Query permissions for this plan

        let plan_perms: Vec<String> = plan_permissions::table
            .inner_join(permissions::table.on(permissions::id.eq(plan_permissions::permission_id)))
            .filter(plan_permissions::plan_id.eq(&plan.id))
            .select(permissions::permission_string)
            .load::<String>(&mut conn)
            .await
            .unwrap_or_default();

        result_plans.push(AvailablePlan {
            id: plan.id.to_string(),
            name: plan.name,
            slug: plan.slug,
            description: plan.description,
            permissions: plan_perms,
            plan_type: plan.plan_type,
            is_active: plan.is_active,
        });
    }

    info!("Returning {} available plans", result_plans.len());
    UnifiedApiResponse::success(AvailablePlansResponse { plans: result_plans })
}

/// GET /api/developer-portal/my-plans
/// Get the authenticated user's assigned permission plans with metadata
pub async fn get_my_plans_handler(
    State(state): State<AppState>,
    Extension(wallet_address): Extension<String>,
) -> impl IntoResponse {
    use diesel::prelude::*;
    use diesel_async::RunQueryDsl;
    use crate::schemas::primary::{plans, plan_permissions, permissions, api_keys, wallet_plan_assignments};

    let pool = *state.db_pool;
    let mut conn = match pool.get().await {
        Ok(c) => c,
        Err(e) => {
            error!("Pool error: {}", e);
            return UnifiedApiResponse::server_error(&e.to_string());
        }
    };

    // Get user's API keys for total stats
    #[derive(diesel::Queryable)]
    #[allow(dead_code)] // Fields needed for Diesel query but not all are read
    struct ApiKeyRow {
        id: uuid::Uuid,
        total_requests: i64,
        expires_at: Option<chrono::DateTime<chrono::Utc>>,
        rate_limit_per_minute: i32,
        rate_limit_per_day: i32,
        created_at: chrono::DateTime<chrono::Utc>,
    }

    // Query ALL API keys for the user (for total stats)
    // Use case-insensitive comparison for wallet address
    let wallet_lower = wallet_address.to_lowercase();
    let all_api_keys = match api_keys::table
        .filter(diesel::dsl::sql::<diesel::sql_types::Bool>(&format!(
            "LOWER(wallet_address) = '{}'", wallet_lower.replace('\'', "''")
        )))
        .select((
            api_keys::id,
            api_keys::total_requests,
            api_keys::expires_at,
            api_keys::rate_limit_per_minute,
            api_keys::rate_limit_per_day,
            api_keys::created_at,
        ))
        .load::<ApiKeyRow>(&mut conn)
        .await
    {
        Ok(keys) => keys,
        Err(e) => {
            error!("Failed to query user API keys: {}", e);
            return UnifiedApiResponse::server_error(&e.to_string());
        }
    };

    let total_api_keys = all_api_keys.len() as i64;
    let total_requests: i64 = all_api_keys.iter().map(|k| k.total_requests).sum();
    
    // Use first active key for rate limits (if any)
    let user_api_keys: Vec<_> = all_api_keys;

    // Get plans assigned directly to the wallet (from wallet_plan_assignments)
    #[derive(diesel::Queryable)]
    struct WalletPlanRow {
        plan_id: uuid::Uuid,
        assigned_at: chrono::DateTime<chrono::Utc>,
        expires_at: Option<chrono::DateTime<chrono::Utc>>,
    }

    let wallet_assignments = wallet_plan_assignments::table
        .filter(wallet_plan_assignments::wallet_address.eq(&wallet_address))
        .filter(wallet_plan_assignments::is_active.eq(true))
        .select((
            wallet_plan_assignments::plan_id,
            wallet_plan_assignments::assigned_at,
            wallet_plan_assignments::expires_at,
        ))
        .load::<WalletPlanRow>(&mut conn)
        .await
        .unwrap_or_default();

    // Get the full plan details for each assigned plan
    #[derive(diesel::Queryable)]
    struct PlanRow {
        id: uuid::Uuid,
        name: String,
        slug: String,
        description: String,
        plan_type: String,
    }

    let plan_ids: Vec<uuid::Uuid> = wallet_assignments.iter().map(|a| a.plan_id).collect();
    
    let assigned_plans = if plan_ids.is_empty() {
        vec![]
    } else {
        plans::table
            .filter(plans::id.eq_any(&plan_ids))
            .filter(plans::is_active.eq(true))
            .select((
                plans::id,
                plans::name,
                plans::slug,
                plans::description,
                plans::plan_type,
            ))
            .load::<PlanRow>(&mut conn)
            .await
            .unwrap_or_default()
    };

    // Build result with permissions for each plan
    let mut result_plans = Vec::new();
    for plan in assigned_plans {
        // Get permissions for this plan
        let plan_perms: Vec<String> = plan_permissions::table
            .inner_join(permissions::table.on(permissions::id.eq(plan_permissions::permission_id)))
            .filter(plan_permissions::plan_id.eq(&plan.id))
            .select(permissions::permission_string)
            .load::<String>(&mut conn)
            .await
            .unwrap_or_default();

        // Find the wallet assignment for this plan to get expiry/assigned_at
        let wallet_assignment = wallet_assignments.iter().find(|a| a.plan_id == plan.id);
        
        // Use first API key for rate limits if available (use get(0) to avoid Diesel trait conflict)
        let first_api_key = user_api_keys.as_slice().first();
        
        result_plans.push(UserAssignedPlan {
            id: plan.id.to_string(),
            name: plan.name,
            slug: plan.slug,
            description: plan.description,
            plan_type: plan.plan_type,
            permissions: plan_perms,
            expires_at: wallet_assignment.and_then(|a| a.expires_at.map(|dt| dt.to_rfc3339())),
            rate_limit_per_minute: first_api_key.map(|k| k.rate_limit_per_minute),
            rate_limit_per_day: first_api_key.map(|k| k.rate_limit_per_day),
            assigned_at: wallet_assignment
                .map(|a| a.assigned_at.to_rfc3339())
                .unwrap_or_else(|| chrono::Utc::now().to_rfc3339()),
        });
    }

    info!("Returning {} assigned plans for wallet {}", result_plans.len(), wallet_address);
    UnifiedApiResponse::success(MyPlansResponse {
        plans: result_plans,
        total_api_keys,
        total_requests,
    })
}

// ============================================================================
// Usage Analytics Handlers
// ============================================================================

/// GET /api/developer-portal/stats
/// Get aggregated usage stats for the authenticated user
pub async fn get_usage_stats_handler(
    State(state): State<AppState>,
    Extension(wallet_address): Extension<String>,
) -> impl IntoResponse {
    let pool = *state.db_pool;
    let service = UsageService::new_core_only(pool);

    match service.get_wallet_stats(&wallet_address).await {
        Ok(stats) => UnifiedApiResponse::success(stats),
        Err(e) => {
            error!("Failed to get usage stats: {}", e);
            UnifiedApiResponse::server_error(&e.to_string())
        }
    }
}

/// GET /api/developer-portal/usage-history
/// Get usage history (time series)
pub async fn get_usage_history_handler(
    State(state): State<AppState>,
    Extension(wallet_address): Extension<String>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> impl IntoResponse {
    let pool = *state.db_pool;
    let service = UsageService::new_core_only(pool);

    let days = params.get("days")
        .and_then(|d| d.parse::<i32>().ok())
        .unwrap_or(7); // Default to 7 days

    match service.get_usage_history(&wallet_address, days).await {
        Ok(points) => UnifiedApiResponse::success(points),
        Err(e) => {
            error!("Failed to get usage history: {}", e);
            UnifiedApiResponse::server_error(&e.to_string())
        }
    }
}

/// GET /api/developer-portal/top-endpoints
/// Get top used endpoints
pub async fn get_top_endpoints_handler(
    State(state): State<AppState>,
    Extension(wallet_address): Extension<String>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> impl IntoResponse {
    let pool = *state.db_pool;
    let service = UsageService::new_core_only(pool);

    let days = params.get("days")
        .and_then(|d| d.parse::<i32>().ok())
        .unwrap_or(7);

    match service.get_top_endpoints(&wallet_address, days).await {
        Ok(endpoints) => UnifiedApiResponse::success(endpoints),
        Err(e) => {
            error!("Failed to get top endpoints: {}", e);
            UnifiedApiResponse::server_error(&e.to_string())
        }
    }
}
