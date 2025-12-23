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

use crate::domain::developer_portal::{CreateApiKeyRequest, RevokeApiKeyRequest};
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
    pub group_ids: Vec<String>, // Permission groups to assign
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
    pub client_name: String,
    pub client_description: Option<String>,
    pub status: String,
    pub groups: Vec<PermissionGroupInfo>,
    pub total_requests: i64,
    pub expires_at: Option<String>,
    pub last_used_at: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Serialize)]
pub struct PermissionGroupInfo {
    pub id: String,
    pub name: String,
    pub slug: String,
}

#[derive(Debug, Serialize)]
pub struct AvailableGroupsResponse {
    pub groups: Vec<AvailableGroup>,
}

#[derive(Debug, Serialize)]
pub struct AvailableGroup {
    pub id: String,
    pub name: String,
    pub slug: String,
    pub description: String,
    pub permissions: Vec<String>,
    pub group_type: String,
    pub is_active: bool,
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

/// GET /api/v1/developer-portal/my-keys
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
                    client_name: key.client_name,
                    client_description: key.client_description,
                    status: key.status.to_string(),
                    groups: key.permission_groups.into_iter().map(|g| PermissionGroupInfo {
                        id: g.id.to_string(),
                        name: g.name,
                        slug: g.slug,
                    }).collect(),
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

/// POST /api/v1/developer-portal/my-keys
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

    // Convert group_ids from strings to UUIDs
    let group_ids: Vec<uuid::Uuid> = body.group_ids.iter()
        .filter_map(|id| uuid::Uuid::parse_str(id).ok())
        .collect();

    let request = CreateApiKeyRequest {
        client_name: body.client_name,
        client_description: body.client_description,
        client_contact_email: None,
        wallet_address: wallet_address.clone(),
        allowed_modules: vec![], // Legacy, replaced by permission groups
        group_ids,
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

/// GET /api/v1/developer-portal/my-keys/:id
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
                client_name: api_key.client_name,
                client_description: api_key.client_description,
                status: api_key.status.to_string(),
                groups: api_key.permission_groups.into_iter().map(|g| PermissionGroupInfo {
                    id: g.id.to_string(),
                    name: g.name,
                    slug: g.slug,
                }).collect(),
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

/// DELETE /api/v1/developer-portal/my-keys/:id
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
        Err(_) => {
            return UnifiedApiResponse::error(400, "Invalid UUID", "The provided ID is not a valid UUID")
        }
    };

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

/// GET /api/v1/developer-portal/available-groups
/// List permission groups available for API key assignment
pub async fn list_available_groups_handler(
    State(state): State<AppState>,
) -> impl IntoResponse {
    use diesel::prelude::*;
    use diesel_async::RunQueryDsl;
    use crate::schema::{groups, group_permissions, permissions};

    let pool = *state.db_pool;
    let mut conn = match pool.get().await {
        Ok(c) => c,
        Err(e) => {
            error!("Pool error: {}", e);
            return UnifiedApiResponse::server_error(&e.to_string());
        }
    };

    // Query active groups with their associated permissions
    #[derive(diesel::Queryable)]
    struct GroupRow {
        id: uuid::Uuid,
        name: String,
        slug: String,
        description: String,
        group_type: String,
        is_active: bool,
    }

    let active_groups = match groups::table
        .filter(groups::is_active.eq(true))
        .select((
            groups::id,
            groups::name,
            groups::slug,
            groups::description,
            groups::group_type,
            groups::is_active,
        ))
        .order(groups::name.asc())
        .load::<GroupRow>(&mut conn)
        .await
    {
        Ok(rows) => rows,
        Err(e) => {
            error!("Failed to query groups: {}", e);
            return UnifiedApiResponse::server_error(&e.to_string());
        }
    };

    // For each group, fetch associated permissions
    let mut result_groups = Vec::new();
    for group in active_groups {
        // Query permissions for this group

        let group_perms = match group_permissions::table
            .inner_join(permissions::table.on(permissions::id.eq(group_permissions::permission_id)))
            .filter(group_permissions::group_id.eq(&group.id))
            .select(permissions::permission_string)
            .load::<String>(&mut conn)
            .await
        {
            Ok(perms) => perms,
            Err(_) => vec![], // If no permissions, return empty
        };

        result_groups.push(AvailableGroup {
            id: group.id.to_string(),
            name: group.name,
            slug: group.slug,
            description: group.description,
            permissions: group_perms,
            group_type: group.group_type,
            is_active: group.is_active,
        });
    }

    info!("Returning {} available groups", result_groups.len());
    UnifiedApiResponse::success(AvailableGroupsResponse { groups: result_groups })
}
