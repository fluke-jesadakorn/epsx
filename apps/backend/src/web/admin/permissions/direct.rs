// Direct Permission Management
// Consolidates direct permission operations from normalized_permission_handlers.rs and granular_permissions.rs

use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use diesel::prelude::*;
use diesel_async::{AsyncConnection, RunQueryDsl};
use crate::infrastructure::services::audit_service::{AuditCtx, AuditEntry};

use crate::web::auth::AppState;
use crate::web::responses::AdminResponse;

// ============================================================================
// REQUEST/RESPONSE TYPES
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct GrantDirectPermissionRequest {
    pub wallet_address: String,
    pub permission_string: String,
    pub expires_at: Option<DateTime<Utc>>,
    pub reason: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RevokeDirectPermissionRequest {
    pub wallet_address: String,
    pub permission_string: String,
}

#[derive(Debug, Serialize)]
pub struct DirectPermissionResponse {
    pub id: String,
    pub wallet_address: String,
    pub permission_id: String,
    pub permission_string: String,
    pub platform: String,
    pub resource: String,
    pub action: String,
    pub granted_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub is_active: bool,
}

#[derive(Debug, Deserialize)]
pub struct AddPermissionToPlanRequest {
    pub permission_string: String,
}

// ============================================================================
// HANDLERS
// ============================================================================

/// Grant a direct permission to a wallet
/// POST /admin/permissions/direct
pub async fn grant_permission(
    State(app_state): State<AppState>,
    axum::Extension(user_ctx): axum::Extension<crate::web::middleware::bearer_middleware::OpenIDUserContext>,
    headers: axum::http::HeaderMap,
    Json(req): Json<GrantDirectPermissionRequest>,
) -> impl IntoResponse {
    let wallet = req.wallet_address.to_lowercase();

    // Validate wallet address format
    if !wallet.starts_with("0x") || wallet.len() != 42 {
        return AdminResponse::bad_request("Invalid wallet address format").into_response();
    }

    // Parse permission string into owned parts for use in transaction
    let parts_owned: Vec<String> = req.permission_string.split(':').map(|s| s.to_string()).collect();
    if parts_owned.len() < 3 {
        return AdminResponse::bad_request("Invalid permission string format (expected platform:resource:action)").into_response();
    }

    // Get database connection
    let mut conn = match app_state.db_pool.get().await {
        Ok(conn) => conn,
        Err(e) => {
            tracing::error!("Failed to get database connection: {}", e);
            return AdminResponse::server_error("Database connection failed").into_response();
        }
    };

    #[derive(QueryableByName)]
    struct PermId {
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        id: Uuid,
    }

    // Clone values for use in transaction
    let permission_string_clone = req.permission_string.clone();
    let wallet_clone = wallet.clone();
    let platform = parts_owned[0].clone();
    let resource = parts_owned[1].clone();
    let action = parts_owned[2].clone();
    let expires_at = req.expires_at;

    // Run transaction
    let result = conn.transaction::<_, diesel::result::Error, _>(|conn| {
        Box::pin(async move {
            // Get or create permission
            let perm_id = diesel::sql_query(
                r#"
                INSERT INTO permissions (permission_string, platform, resource, action, permission_type)
                VALUES ($1, $2, $3, $4, 'manual')
                ON CONFLICT (permission_string) DO UPDATE SET permission_string = EXCLUDED.permission_string
                RETURNING id
                "#
            )
            .bind::<diesel::sql_types::Text, _>(&permission_string_clone)
            .bind::<diesel::sql_types::Text, _>(&platform)
            .bind::<diesel::sql_types::Text, _>(&resource)
            .bind::<diesel::sql_types::Text, _>(&action)
            .get_result::<PermId>(conn)
            .await?
            .id;

            // Grant direct permission to wallet
            let grant_id = diesel::sql_query(
                r#"
                INSERT INTO wallet_direct_permissions (wallet_address, permission_id, expires_at)
                VALUES ($1, $2, $3)
                ON CONFLICT (wallet_address, permission_id) DO UPDATE
                SET expires_at = EXCLUDED.expires_at, is_active = true
                RETURNING id
                "#
            )
            .bind::<diesel::sql_types::Text, _>(&wallet_clone)
            .bind::<diesel::sql_types::Uuid, _>(perm_id)
            .bind::<diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>, _>(expires_at)
            .get_result::<PermId>(conn)
            .await?
            .id;

            Ok((perm_id, grant_id))
        })
    }).await;

    let (perm_id, grant_id) = match result {
        Ok(ids) => ids,
        Err(e) => {
            tracing::error!("Transaction failed: {}", e);
            return AdminResponse::server_error("Failed to grant permission").into_response();
        }
    };

    tracing::info!(
        "Granted direct permission '{}' to wallet {}",
        req.permission_string,
        wallet
    );

    let response = DirectPermissionResponse {
        id: grant_id.to_string(),
        wallet_address: wallet.clone(),
        permission_id: perm_id.to_string(),
        permission_string: req.permission_string.clone(),
        platform: parts_owned[0].clone(),
        resource: parts_owned[1].clone(),
        action: parts_owned[2].clone(),
        granted_at: Utc::now(),
        expires_at: req.expires_at,
        is_active: true,
    };

    let ctx = AuditCtx::from_wallet(&user_ctx.wallet_address, &headers);
    app_state.audit.log(ctx, AuditEntry::new("permission", "grant", "permission")
        .id(&wallet)
        .after(serde_json::json!({
            "permission": req.permission_string,
            "expires_at": req.expires_at,
            "reason": req.reason,
        })));

    AdminResponse::created(response, "Direct permission granted successfully").into_response()
}

/// Revoke a direct permission from a wallet
/// DELETE /admin/permissions/direct
pub async fn revoke_permission(
    State(app_state): State<AppState>,
    axum::Extension(user_ctx): axum::Extension<crate::web::middleware::bearer_middleware::OpenIDUserContext>,
    headers: axum::http::HeaderMap,
    Json(req): Json<RevokeDirectPermissionRequest>,
) -> impl IntoResponse {
    let wallet = req.wallet_address.to_lowercase();

    let mut conn = match app_state.db_pool.get().await {
        Ok(conn) => conn,
        Err(e) => {
            tracing::error!("Failed to get database connection: {}", e);
            return AdminResponse::server_error("Database connection failed").into_response();
        }
    };

    #[derive(QueryableByName)]
    struct PermId {
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        id: Uuid,
    }

    // Get permission ID
    let perm_id = match diesel::sql_query("SELECT id FROM permissions WHERE permission_string = $1")
        .bind::<diesel::sql_types::Text, _>(&req.permission_string)
        .get_result::<PermId>(&mut conn)
        .await
        .optional()
    {
        Ok(Some(result)) => result.id,
        Ok(None) => return AdminResponse::not_found("Permission").into_response(),
        Err(e) => {
            tracing::error!("Failed to fetch permission: {}", e);
            return AdminResponse::server_error("Database query failed").into_response();
        }
    };

    // Revoke direct permission
    match diesel::sql_query("DELETE FROM wallet_direct_permissions WHERE wallet_address = $1 AND permission_id = $2")
        .bind::<diesel::sql_types::Text, _>(&wallet)
        .bind::<diesel::sql_types::Uuid, _>(perm_id)
        .execute(&mut conn)
        .await
    {
        Ok(rows) if rows > 0 => {
            tracing::info!(
                "Revoked direct permission '{}' from wallet {}",
                req.permission_string,
                wallet
            );
            let ctx = AuditCtx::from_wallet(&user_ctx.wallet_address, &headers);
            app_state.audit.log(ctx, AuditEntry::new("permission", "revoke", "permission")
                .id(&wallet)
                .before(serde_json::json!({ "permission": req.permission_string })));

            AdminResponse::success_with_message(
                serde_json::json!({"deleted": true}),
                "Direct permission revoked successfully"
            ).into_response()
        },
        Ok(_) => AdminResponse::not_found("Direct permission grant").into_response(),
        Err(e) => {
            tracing::error!("Failed to revoke permission: {}", e);
            AdminResponse::server_error("Failed to revoke permission").into_response()
        }
    }
}

/// List direct permissions for a wallet
/// GET /admin/permissions/direct/:wallet
pub async fn list_wallet_permissions(
    State(app_state): State<AppState>,
    Path(wallet): Path<String>,
) -> impl IntoResponse {
    let wallet = wallet.to_lowercase();

    let mut conn = match app_state.db_pool.get().await {
        Ok(conn) => conn,
        Err(e) => {
            tracing::error!("Failed to get database connection: {}", e);
            return AdminResponse::server_error("Database connection failed").into_response();
        }
    };

    #[derive(QueryableByName)]
    struct PermissionRow {
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        id: Uuid,
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        permission_id: Uuid,
        #[diesel(sql_type = diesel::sql_types::Timestamptz)]
        granted_at: DateTime<Utc>,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>)]
        expires_at: Option<DateTime<Utc>>,
        #[diesel(sql_type = diesel::sql_types::Bool)]
        is_active: bool,
        #[diesel(sql_type = diesel::sql_types::Text)]
        permission_string: String,
        #[diesel(sql_type = diesel::sql_types::Text)]
        platform: String,
        #[diesel(sql_type = diesel::sql_types::Text)]
        resource: String,
        #[diesel(sql_type = diesel::sql_types::Text)]
        action: String,
    }

    let rows = match diesel::sql_query(
        r#"
        SELECT
            wdp.id, wdp.permission_id, wdp.granted_at, wdp.expires_at, wdp.is_active,
            p.permission_string, p.platform, p.resource, p.action
        FROM wallet_direct_permissions wdp
        JOIN permissions p ON wdp.permission_id = p.id
        WHERE wdp.wallet_address = $1 AND wdp.is_active = true
        ORDER BY wdp.granted_at DESC
        "#
    )
    .bind::<diesel::sql_types::Text, _>(&wallet)
    .load::<PermissionRow>(&mut conn)
    .await
    {
        Ok(rows) => rows,
        Err(e) => {
            tracing::error!("Failed to list direct permissions: {}", e);
            return AdminResponse::server_error("Database query failed").into_response();
        }
    };

    let permissions: Vec<DirectPermissionResponse> = rows.into_iter().map(|row| {
        DirectPermissionResponse {
            id: row.id.to_string(),
            wallet_address: wallet.clone(),
            permission_id: row.permission_id.to_string(),
            permission_string: row.permission_string,
            platform: row.platform,
            resource: row.resource,
            action: row.action,
            granted_at: row.granted_at,
            expires_at: row.expires_at,
            is_active: row.is_active,
        }
    }).collect();

    AdminResponse::success(serde_json::json!({
        "wallet_address": wallet,
        "permissions": permissions,
        "count": permissions.len()
    })).into_response()
}

/// Add a permission to a plan
/// POST /admin/permissions/plans/:plan_id/permissions
pub async fn add_permission_to_plan(
    State(app_state): State<AppState>,
    axum::Extension(user_ctx): axum::Extension<crate::web::middleware::bearer_middleware::OpenIDUserContext>,
    headers: axum::http::HeaderMap,
    Path(plan_id): Path<String>,
    Json(req): Json<AddPermissionToPlanRequest>,
) -> impl IntoResponse {
    let plan_uuid = match Uuid::parse_str(&plan_id) {
        Ok(id) => id,
        Err(_) => return AdminResponse::bad_request("Invalid plan ID format").into_response(),
    };

    // Parse permission string into owned parts for use in transaction
    let parts_owned: Vec<String> = req.permission_string.split(':').map(|s| s.to_string()).collect();
    if parts_owned.len() < 3 {
        return AdminResponse::bad_request("Invalid permission string format (expected platform:resource:action)").into_response();
    }

    let mut conn = match app_state.db_pool.get().await {
        Ok(conn) => conn,
        Err(e) => {
            tracing::error!("Failed to get database connection: {}", e);
            return AdminResponse::server_error("Database connection failed").into_response();
        }
    };

    #[derive(QueryableByName)]
    struct PermId {
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        id: Uuid,
    }

    #[derive(QueryableByName)]
    struct MembershipRow {
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        id: Uuid,
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        plan_id: Uuid,
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        permission_id: Uuid,
        #[diesel(sql_type = diesel::sql_types::Timestamptz)]
        granted_at: DateTime<Utc>,
    }

    // Clone values for use in transaction
    let permission_string_clone = req.permission_string.clone();
    let platform = parts_owned[0].clone();
    let resource = parts_owned[1].clone();
    let action = parts_owned[2].clone();

    // Run transaction
    let membership = conn.transaction::<_, diesel::result::Error, _>(|conn| {
        Box::pin(async move {
            // Get or create permission
            let perm_id = diesel::sql_query(
                r#"
                INSERT INTO permissions (permission_string, platform, resource, action, permission_type)
                VALUES ($1, $2, $3, $4, 'manual')
                ON CONFLICT (permission_string) DO UPDATE SET permission_string = EXCLUDED.permission_string
                RETURNING id
                "#
            )
            .bind::<diesel::sql_types::Text, _>(&permission_string_clone)
            .bind::<diesel::sql_types::Text, _>(&platform)
            .bind::<diesel::sql_types::Text, _>(&resource)
            .bind::<diesel::sql_types::Text, _>(&action)
            .get_result::<PermId>(conn)
            .await?
            .id;

            // Add permission to plan
            let membership = diesel::sql_query(
                r#"
                INSERT INTO plan_permissions (plan_id, permission_id)
                VALUES ($1, $2)
                ON CONFLICT (plan_id, permission_id) DO NOTHING
                RETURNING id, plan_id, permission_id, granted_at
                "#
            )
            .bind::<diesel::sql_types::Uuid, _>(plan_uuid)
            .bind::<diesel::sql_types::Uuid, _>(perm_id)
            .get_result::<MembershipRow>(conn)
            .await
            .optional()?;

            Ok(membership)
        })
    }).await;

    let membership = match membership {
        Ok(m) => m,
        Err(e) => {
            tracing::error!("Transaction failed: {}", e);
            return AdminResponse::server_error("Failed to add permission to plan").into_response();
        }
    };

    if let Some(m) = membership {
        let ctx = AuditCtx::from_wallet(&user_ctx.wallet_address, &headers);
        app_state.audit.log(ctx, AuditEntry::new("plan_permission", "create", "permission")
            .id(&plan_id)
            .after(serde_json::json!({ "permission": req.permission_string })));
        AdminResponse::created(
            serde_json::json!({
                "id": m.id.to_string(),
                "plan_id": m.plan_id.to_string(),
                "permission_id": m.permission_id.to_string(),
                "granted_at": m.granted_at,
            }),
            "Permission added to plan successfully"
        ).into_response()
    } else {
        AdminResponse::success_with_message(
            serde_json::json!({"exists": true}),
            "Permission already exists in plan"
        ).into_response()
    }
}

/// Remove a permission from a plan
/// DELETE /admin/permissions/plans/:plan_id/permissions/:permission_id
pub async fn remove_permission_from_plan(
    State(app_state): State<AppState>,
    axum::Extension(user_ctx): axum::Extension<crate::web::middleware::bearer_middleware::OpenIDUserContext>,
    headers: axum::http::HeaderMap,
    Path((plan_id, permission_id)): Path<(String, String)>,
) -> impl IntoResponse {
    let plan_uuid = match Uuid::parse_str(&plan_id) {
        Ok(id) => id,
        Err(_) => return AdminResponse::bad_request("Invalid plan ID format").into_response(),
    };

    let perm_uuid = match Uuid::parse_str(&permission_id) {
        Ok(id) => id,
        Err(_) => return AdminResponse::bad_request("Invalid permission ID format").into_response(),
    };

    let mut conn = match app_state.db_pool.get().await {
        Ok(conn) => conn,
        Err(e) => {
            tracing::error!("Failed to get database connection: {}", e);
            return AdminResponse::server_error("Database connection failed").into_response();
        }
    };

    match diesel::sql_query("DELETE FROM plan_permissions WHERE plan_id = $1 AND permission_id = $2")
        .bind::<diesel::sql_types::Uuid, _>(plan_uuid)
        .bind::<diesel::sql_types::Uuid, _>(perm_uuid)
        .execute(&mut conn)
        .await
    {
        Ok(rows) if rows > 0 => {
            let ctx = AuditCtx::from_wallet(&user_ctx.wallet_address, &headers);
            app_state.audit.log(ctx, AuditEntry::new("plan_permission", "delete", "permission")
                .id(&plan_id)
                .before(serde_json::json!({ "permission_id": permission_id })));
            AdminResponse::success_with_message(
                serde_json::json!({"deleted": true}),
                "Permission removed from plan successfully"
            ).into_response()
        },
        Ok(_) => AdminResponse::not_found("Permission membership").into_response(),
        Err(e) => {
            tracing::error!("Failed to remove permission from plan: {}", e);
            AdminResponse::server_error("Failed to remove permission from plan").into_response()
        }
    }
}
