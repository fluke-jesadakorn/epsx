// Direct Permission Management
// Consolidates direct permission operations from normalized_permission_handlers.rs and granular_permissions.rs
//
// BIG-BANG: migrated to sqlx (real). All diesel DSL replaced with raw SQL.

use crate::infrastructure::cache::redis_cache::set_perm_invalidated;
use crate::infrastructure::services::audit_service::{AuditCtx, AuditEntry};
use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Json,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::web::auth::AppState;
use crate::web::responses::AdminResponse;

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

/// Grant a direct permission to a wallet
/// POST /admin/permissions/direct
pub async fn grant_permission(
    State(app_state): State<AppState>,
    axum::Extension(user_ctx): axum::Extension<
        crate::web::middleware::bearer_middleware::OpenIDUserContext,
    >,
    headers: axum::http::HeaderMap,
    Json(req): Json<GrantDirectPermissionRequest>,
) -> impl IntoResponse {
    let wallet = req.wallet_address.to_lowercase();

    if !wallet.starts_with("0x") || wallet.len() != 42 {
        return AdminResponse::bad_request("Invalid wallet address format").into_response();
    }

    let parts_owned: Vec<String> = req
        .permission_string
        .split(':')
        .map(|s| s.to_string())
        .collect();
    if parts_owned.len() < 3 {
        return AdminResponse::bad_request(
            "Invalid permission string format (expected platform:resource:action)",
        )
        .into_response();
    }

    let mut tx = match app_state.db_pool.begin().await {
        Ok(t) => t,
        Err(e) => {
            tracing::error!("Failed to begin transaction: {}", e);
            return AdminResponse::server_error("Database transaction failed").into_response();
        }
    };

    #[derive(sqlx::FromRow)]
    struct IdOnly {
        id: Uuid,
    }

    // Get or create permission
    let perm_id: IdOnly = match sqlx::query_as(
        r#"
        INSERT INTO permissions (permission_string, platform, resource, action, permission_type)
        VALUES ($1, $2, $3, $4, 'manual')
        ON CONFLICT (permission_string) DO UPDATE SET permission_string = EXCLUDED.permission_string
        RETURNING id
        "#,
    )
    .bind(&req.permission_string)
    .bind(&parts_owned[0])
    .bind(&parts_owned[1])
    .bind(&parts_owned[2])
    .fetch_one(&mut *tx)
    .await
    {
        Ok(p) => p,
        Err(e) => {
            tracing::error!("Failed to upsert permission: {}", e);
            return AdminResponse::server_error("Failed to upsert permission").into_response();
        }
    };

    // Grant direct permission
    let grant_id: IdOnly = match sqlx::query_as(
        r#"
        INSERT INTO wallet_direct_permissions (wallet_address, permission_id, expires_at)
        VALUES ($1, $2, $3)
        ON CONFLICT (wallet_address, permission_id) DO UPDATE
        SET expires_at = EXCLUDED.expires_at, is_active = true
        RETURNING id
        "#,
    )
    .bind(&wallet)
    .bind(perm_id.id)
    .bind(req.expires_at)
    .fetch_one(&mut *tx)
    .await
    {
        Ok(g) => g,
        Err(e) => {
            tracing::error!("Failed to grant direct permission: {}", e);
            return AdminResponse::server_error("Failed to grant direct permission").into_response();
        }
    };

    if let Err(e) = tx.commit().await {
        tracing::error!("Failed to commit transaction: {}", e);
        return AdminResponse::server_error("Failed to commit transaction").into_response();
    }

    tracing::info!(
        "Granted direct permission '{}' to wallet {}",
        req.permission_string,
        wallet
    );

    let response = DirectPermissionResponse {
        id: grant_id.id.to_string(),
        wallet_address: wallet.clone(),
        permission_id: perm_id.id.to_string(),
        permission_string: req.permission_string.clone(),
        platform: parts_owned[0].clone(),
        resource: parts_owned[1].clone(),
        action: parts_owned[2].clone(),
        granted_at: Utc::now(),
        expires_at: req.expires_at,
        is_active: true,
    };

    let ctx = AuditCtx::from_wallet(&user_ctx.wallet_address, &headers);
    app_state.audit.log(
        ctx,
        AuditEntry::new("permission", "grant", "permission")
            .id(&wallet)
            .after(serde_json::json!({
                "permission": req.permission_string,
                "expires_at": req.expires_at,
                "reason": req.reason,
            })),
    );

    set_perm_invalidated(app_state.cache.as_ref(), &wallet);

    AdminResponse::created(response, "Direct permission granted successfully").into_response()
}

/// Revoke a direct permission from a wallet
/// DELETE /admin/permissions/direct
pub async fn revoke_permission(
    State(app_state): State<AppState>,
    axum::Extension(user_ctx): axum::Extension<
        crate::web::middleware::bearer_middleware::OpenIDUserContext,
    >,
    headers: axum::http::HeaderMap,
    Json(req): Json<RevokeDirectPermissionRequest>,
) -> impl IntoResponse {
    let wallet = req.wallet_address.to_lowercase();

    let result = sqlx::query(
        "DELETE FROM wallet_direct_permissions \
         WHERE wallet_address IN (\
             SELECT wallet_address FROM permissions \
             WHERE permission_string = $1\
         ) AND wallet_address = $2",
    )
    .bind(&req.permission_string)
    .bind(&wallet)
    .execute(&app_state.db_pool)
    .await;

    let result = match result {
        Ok(r) => r,
        Err(e) => {
            tracing::error!("Failed to revoke permission: {}", e);
            return AdminResponse::server_error("Failed to revoke permission").into_response();
        }
    };

    if result.rows_affected() == 0 {
        return AdminResponse::not_found("Direct permission grant").into_response();
    }

    tracing::info!(
        "Revoked direct permission '{}' from wallet {}",
        req.permission_string,
        wallet
    );

    let ctx = AuditCtx::from_wallet(&user_ctx.wallet_address, &headers);
    app_state.audit.log(
        ctx,
        AuditEntry::new("permission", "revoke", "permission")
            .id(&wallet)
            .before(serde_json::json!({ "permission": req.permission_string })),
    );

    set_perm_invalidated(app_state.cache.as_ref(), &wallet);

    AdminResponse::success_with_message(
        serde_json::json!({"deleted": true}),
        "Direct permission revoked successfully",
    )
    .into_response()
}

/// List direct permissions for a wallet
/// GET /admin/permissions/direct/:wallet
pub async fn list_wallet_permissions(
    State(app_state): State<AppState>,
    Path(wallet): Path<String>,
) -> impl IntoResponse {
    let wallet = wallet.to_lowercase();

    #[derive(sqlx::FromRow)]
    struct PermissionRow {
        id: Uuid,
        permission_id: Uuid,
        granted_at: DateTime<Utc>,
        expires_at: Option<DateTime<Utc>>,
        is_active: bool,
        permission_string: String,
        platform: String,
        resource: String,
        action: String,
    }

    let rows = match sqlx::query_as::<_, PermissionRow>(
        r#"
        SELECT
            wdp.id, wdp.permission_id, wdp.granted_at, wdp.expires_at, wdp.is_active,
            p.permission_string, p.platform, p.resource, p.action
        FROM wallet_direct_permissions wdp
        JOIN permissions p ON wdp.permission_id = p.id
        WHERE wdp.wallet_address = $1 AND wdp.is_active = true
        ORDER BY wdp.granted_at DESC
        "#,
    )
    .bind(&wallet)
    .fetch_all(&app_state.db_pool)
    .await
    {
        Ok(rows) => rows,
        Err(e) => {
            tracing::error!("Failed to list direct permissions: {}", e);
            return AdminResponse::server_error("Database query failed").into_response();
        }
    };

    let permissions: Vec<DirectPermissionResponse> = rows
        .into_iter()
        .map(|row| DirectPermissionResponse {
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
        })
        .collect();

    AdminResponse::success(serde_json::json!({
        "wallet_address": wallet,
        "permissions": permissions,
        "count": permissions.len()
    }))
    .into_response()
}

/// Add a permission to a plan
/// POST /admin/permissions/plans/:plan_id/permissions
pub async fn add_permission_to_plan(
    State(app_state): State<AppState>,
    axum::Extension(user_ctx): axum::Extension<
        crate::web::middleware::bearer_middleware::OpenIDUserContext,
    >,
    headers: axum::http::HeaderMap,
    Path(plan_id): Path<String>,
    Json(req): Json<AddPermissionToPlanRequest>,
) -> impl IntoResponse {
    let plan_uuid = match Uuid::parse_str(&plan_id) {
        Ok(id) => id,
        Err(_) => return AdminResponse::bad_request("Invalid plan ID format").into_response(),
    };

    let parts_owned: Vec<String> = req
        .permission_string
        .split(':')
        .map(|s| s.to_string())
        .collect();
    if parts_owned.len() < 3 {
        return AdminResponse::bad_request(
            "Invalid permission string format (expected platform:resource:action)",
        )
        .into_response();
    }

    let mut tx = match app_state.db_pool.begin().await {
        Ok(t) => t,
        Err(e) => {
            tracing::error!("Failed to begin transaction: {}", e);
            return AdminResponse::server_error("Database transaction failed").into_response();
        }
    };

    #[derive(sqlx::FromRow)]
    struct IdOnly {
        id: Uuid,
    }

    // Get or create permission
    let perm_id: IdOnly = match sqlx::query_as(
        r#"
        INSERT INTO permissions (permission_string, platform, resource, action, permission_type)
        VALUES ($1, $2, $3, $4, 'manual')
        ON CONFLICT (permission_string) DO UPDATE SET permission_string = EXCLUDED.permission_string
        RETURNING id
        "#,
    )
    .bind(&req.permission_string)
    .bind(&parts_owned[0])
    .bind(&parts_owned[1])
    .bind(&parts_owned[2])
    .fetch_one(&mut *tx)
    .await
    {
        Ok(p) => p,
        Err(e) => {
            tracing::error!("Failed to upsert permission: {}", e);
            return AdminResponse::server_error("Failed to upsert permission").into_response();
        }
    };

    // Insert into plan_permissions
    #[derive(sqlx::FromRow)]
    struct MembershipRow {
        id: Uuid,
        plan_id: Uuid,
        permission_id: Uuid,
        granted_at: DateTime<Utc>,
    }

    let membership_result: Option<MembershipRow> = match sqlx::query_as(
        r#"
        INSERT INTO plan_permissions (plan_id, permission_id)
        VALUES ($1, $2)
        ON CONFLICT (plan_id, permission_id) DO NOTHING
        RETURNING id, plan_id, permission_id, granted_at
        "#,
    )
    .bind(plan_uuid)
    .bind(perm_id.id)
    .fetch_optional(&mut *tx)
    .await
    {
        Ok(m) => m,
        Err(e) => {
            tracing::error!("Failed to add permission to plan: {}", e);
            return AdminResponse::server_error("Failed to add permission to plan").into_response();
        }
    };

    if let Err(e) = tx.commit().await {
        tracing::error!("Failed to commit transaction: {}", e);
        return AdminResponse::server_error("Failed to commit transaction").into_response();
    }

    if let Some(m) = membership_result {
        let ctx = AuditCtx::from_wallet(&user_ctx.wallet_address, &headers);
        app_state.audit.log(
            ctx,
            AuditEntry::new("plan_permission", "create", "permission")
                .id(&plan_id)
                .after(serde_json::json!({ "permission": req.permission_string })),
        );
        AdminResponse::created(
            serde_json::json!({
                "id": m.id.to_string(),
                "plan_id": m.plan_id.to_string(),
                "permission_id": m.permission_id.to_string(),
                "granted_at": m.granted_at,
            }),
            "Permission added to plan successfully",
        )
        .into_response()
    } else {
        AdminResponse::success_with_message(
            serde_json::json!({"exists": true}),
            "Permission already exists in plan",
        )
        .into_response()
    }
}

/// Remove a permission from a plan
/// DELETE /admin/permissions/plans/:plan_id/permissions/:permission_id
pub async fn remove_permission_from_plan(
    State(app_state): State<AppState>,
    axum::Extension(user_ctx): axum::Extension<
        crate::web::middleware::bearer_middleware::OpenIDUserContext,
    >,
    headers: axum::http::HeaderMap,
    Path((plan_id, permission_id)): Path<(String, String)>,
) -> impl IntoResponse {
    let plan_uuid = match Uuid::parse_str(&plan_id) {
        Ok(id) => id,
        Err(_) => return AdminResponse::bad_request("Invalid plan ID format").into_response(),
    };

    let perm_uuid = match Uuid::parse_str(&permission_id) {
        Ok(id) => id,
        Err(_) => {
            return AdminResponse::bad_request("Invalid permission ID format").into_response()
        }
    };

    let result = sqlx::query(
        "DELETE FROM plan_permissions WHERE plan_id = $1 AND permission_id = $2",
    )
    .bind(plan_uuid)
    .bind(perm_uuid)
    .execute(&app_state.db_pool)
    .await;

    let result = match result {
        Ok(r) => r,
        Err(e) => {
            tracing::error!("Failed to remove permission from plan: {}", e);
            return AdminResponse::server_error("Failed to remove permission from plan").into_response();
        }
    };

    if result.rows_affected() == 0 {
        return AdminResponse::not_found("Permission membership").into_response();
    }

    let ctx = AuditCtx::from_wallet(&user_ctx.wallet_address, &headers);
    app_state.audit.log(
        ctx,
        AuditEntry::new("plan_permission", "delete", "permission")
            .id(&plan_id)
            .before(serde_json::json!({ "permission_id": permission_id })),
    );

    AdminResponse::success_with_message(
        serde_json::json!({"deleted": true}),
        "Permission removed from plan successfully",
    )
    .into_response()
}
