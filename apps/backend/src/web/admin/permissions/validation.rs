// Permission Validation Logic
// Consolidates validation operations from permission_group_handlers.rs and centralized_permission_handlers.rs

use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use sqlx::Row;
use std::collections::HashMap;

use crate::web::auth::AppState;
use crate::web::responses::AdminResponse;

// ============================================================================
// REQUEST/RESPONSE TYPES
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct PermissionValidationRequest {
    pub wallet_address: String,
    pub permission: String,
    pub context: Option<HashMap<String, serde_json::Value>>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub request_id: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct PermissionValidationResponse {
    pub is_valid: bool,
    pub permission: String,
    pub wallet_address: String,
    pub group_id: Option<String>,
    pub validation_result: PermissionValidationResult,
    pub error_details: Option<PermissionError>,
    pub suggestions: Option<Vec<PermissionSuggestion>>,
    pub audit_id: String,
    pub validated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct PermissionValidationResult {
    pub granted: bool,
    pub reason: String,
    pub expires_at: Option<DateTime<Utc>>,
    pub source_group: Option<String>,
    pub next_refresh: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize)]
pub struct PermissionError {
    pub error_code: String,
    pub error_type: String,
    pub user_message: String,
    pub technical_message: String,
    pub retry_after: Option<DateTime<Utc>>,
    pub upgrade_path: Option<UpgradePath>,
}

#[derive(Debug, Serialize)]
pub struct PermissionSuggestion {
    pub suggestion_type: String,
    pub title: String,
    pub description: String,
    pub action_url: Option<String>,
    pub priority: i32,
}

#[derive(Debug, Serialize)]
pub struct UpgradePath {
    pub suggested_group: String,
    pub price_difference: f64,
    pub additional_permissions: Vec<String>,
    pub upgrade_url: String,
}

#[derive(Debug, Deserialize)]
pub struct BulkPermissionValidationRequest {
    pub wallet_address: String,
    pub permissions: Vec<String>,
    pub context: Option<HashMap<String, serde_json::Value>>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub request_id: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct BulkPermissionValidationResponse {
    pub wallet_address: String,
    pub results: HashMap<String, bool>,
    pub overall_valid: bool,
    pub granted_count: usize,
    pub denied_count: usize,
    pub audit_id: String,
    pub validated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct WalletPermissionsResponse {
    pub wallet_address: String,
    pub permission_groups: Vec<PermissionGroupSummary>,
    pub effective_permissions: Vec<String>,
    pub permission_summary: PermissionSummary,
    pub audit_id: String,
    pub retrieved_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct PermissionGroupSummary {
    pub id: String,
    pub name: String,
    pub group_type: String,
    pub is_active: bool,
    pub assigned_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize)]
pub struct PermissionSummary {
    pub total_permissions: i32,
    pub active_permissions: i32,
    pub expiring_soon: i32,
    pub expired_permissions: i32,
    pub highest_group: String,
}

// ============================================================================
// HANDLERS
// ============================================================================

/// Validate a single permission for a wallet
/// POST /admin/permissions/validate
pub async fn validate_permission(
    State(app_state): State<AppState>,
    Json(req): Json<PermissionValidationRequest>,
) -> impl IntoResponse {
    let audit_id = Uuid::new_v4().to_string();
    let wallet = req.wallet_address.to_lowercase();

    // Check if wallet has the permission through any active group
    let has_permission = match check_wallet_permission(&app_state, &wallet, &req.permission).await {
        Ok(result) => result,
        Err(e) => {
            tracing::error!("Failed to check permission: {}", e);
            return AdminResponse::server_error("Permission check failed").into_response();
        }
    };

    let (is_valid, source_group, expires_at) = has_permission;

    // Build validation result
    let validation_result = PermissionValidationResult {
        granted: is_valid,
        reason: if is_valid {
            "Permission granted by permission group".to_string()
        } else {
            "Insufficient permissions".to_string()
        },
        expires_at,
        source_group: source_group.clone(),
        next_refresh: Some(Utc::now() + chrono::Duration::hours(1)),
    };

    // Build error details if permission denied
    let error_details = if !is_valid {
        Some(PermissionError {
            error_code: "INSUFFICIENT_PERMISSIONS".to_string(),
            error_type: "access_denied".to_string(),
            user_message: "You need to upgrade your plan to access this feature.".to_string(),
            technical_message: format!(
                "Permission '{}' not found in wallet's effective permissions",
                req.permission
            ),
            retry_after: None,
            upgrade_path: None,
        })
    } else {
        None
    };

    // Build suggestions if permission denied
    let suggestions = if !is_valid {
        Some(vec![PermissionSuggestion {
            suggestion_type: "upgrade".to_string(),
            title: "Upgrade to Professional".to_string(),
            description: "Unlock advanced analytics and trading features".to_string(),
            action_url: Some("/upgrade/professional-plan".to_string()),
            priority: 1,
        }])
    } else {
        None
    };

    // Log validation
    tracing::info!(
        audit_id = %audit_id,
        wallet_address = %wallet,
        permission = %req.permission,
        granted = is_valid,
        "Permission validation completed"
    );

    let response = PermissionValidationResponse {
        is_valid,
        permission: req.permission,
        wallet_address: wallet,
        group_id: source_group,
        validation_result,
        error_details,
        suggestions,
        audit_id,
        validated_at: Utc::now(),
    };

    AdminResponse::success(response).into_response()
}

/// Validate multiple permissions for a wallet in one request
/// POST /admin/permissions/validate/bulk
pub async fn validate_bulk_permissions(
    State(app_state): State<AppState>,
    Json(req): Json<BulkPermissionValidationRequest>,
) -> impl IntoResponse {
    let audit_id = Uuid::new_v4().to_string();
    let wallet = req.wallet_address.to_lowercase();

    let mut results = HashMap::new();
    let mut granted_count = 0;

    // Validate each permission
    for permission in &req.permissions {
        match check_wallet_permission(&app_state, &wallet, permission).await {
            Ok((is_valid, _, _)) => {
                results.insert(permission.clone(), is_valid);
                if is_valid {
                    granted_count += 1;
                }
            }
            Err(e) => {
                tracing::error!("Failed to check permission {}: {}", permission, e);
                results.insert(permission.clone(), false);
            }
        }
    }

    let denied_count = req.permissions.len() - granted_count;
    let overall_valid = denied_count == 0;

    tracing::info!(
        audit_id = %audit_id,
        wallet_address = %wallet,
        total_permissions = req.permissions.len(),
        granted_count = granted_count,
        "Bulk permission validation completed"
    );

    let response = BulkPermissionValidationResponse {
        wallet_address: wallet,
        results,
        overall_valid,
        granted_count,
        denied_count,
        audit_id,
        validated_at: Utc::now(),
    };

    AdminResponse::success(response).into_response()
}

/// Get all effective permissions for a wallet
/// GET /admin/permissions/wallets/:wallet/permissions
pub async fn get_wallet_permissions(
    State(app_state): State<AppState>,
    Path(wallet): Path<String>,
) -> impl IntoResponse {
    let audit_id = Uuid::new_v4().to_string();
    let wallet = wallet.to_lowercase();

    // Get permission groups assigned to wallet
    let groups = match sqlx::query(
        r#"
        SELECT
            pg.id, pg.name, pg.group_type,
            wga.is_active, wga.assigned_at, wga.expires_at
        FROM wallet_group_assignments wga
        JOIN permission_groups pg ON wga.group_id = pg.id
        WHERE wga.wallet_address = $1
        ORDER BY wga.assigned_at DESC
        "#
    )
    .bind(&wallet)
    .fetch_all(&*app_state.db_pool)
    .await
    {
        Ok(rows) => rows,
        Err(e) => {
            tracing::error!("Failed to fetch permission groups: {}", e);
            return AdminResponse::server_error("Database query failed").into_response();
        }
    };

    let permission_groups: Vec<PermissionGroupSummary> = groups.iter().map(|row| {
        PermissionGroupSummary {
            id: row.get::<Uuid, _>("id").to_string(),
            name: row.get("name"),
            group_type: row.get("group_type"),
            is_active: row.get("is_active"),
            assigned_at: row.get("assigned_at"),
            expires_at: row.get("expires_at"),
        }
    }).collect();

    // Get effective permissions from all active groups
    let permissions = match sqlx::query_scalar::<_, String>(
        r#"
        SELECT DISTINCT p.permission_string
        FROM permissions p
        JOIN permission_group_memberships pgm ON p.id = pgm.permission_id
        JOIN wallet_group_assignments wga ON pgm.group_id = wga.group_id
        WHERE wga.wallet_address = $1
          AND wga.is_active = true
          AND (wga.expires_at IS NULL OR wga.expires_at > NOW())
        ORDER BY p.permission_string
        "#
    )
    .bind(&wallet)
    .fetch_all(&*app_state.db_pool)
    .await
    {
        Ok(perms) => perms,
        Err(e) => {
            tracing::error!("Failed to fetch effective permissions: {}", e);
            return AdminResponse::server_error("Database query failed").into_response();
        }
    };

    // Calculate expiring permissions (within 7 days)
    let expiring_count = match sqlx::query_scalar::<_, i64>(
        r#"
        SELECT COUNT(DISTINCT wga.id)::bigint
        FROM wallet_group_assignments wga
        WHERE wga.wallet_address = $1
          AND wga.is_active = true
          AND wga.expires_at IS NOT NULL
          AND wga.expires_at BETWEEN NOW() AND NOW() + INTERVAL '7 days'
        "#
    )
    .bind(&wallet)
    .fetch_one(&*app_state.db_pool)
    .await
    {
        Ok(count) => count as i32,
        Err(_) => 0,
    };

    // Calculate expired permissions
    let expired_count = match sqlx::query_scalar::<_, i64>(
        r#"
        SELECT COUNT(DISTINCT wga.id)::bigint
        FROM wallet_group_assignments wga
        WHERE wga.wallet_address = $1
          AND wga.expires_at IS NOT NULL
          AND wga.expires_at < NOW()
        "#
    )
    .bind(&wallet)
    .fetch_one(&*app_state.db_pool)
    .await
    {
        Ok(count) => count as i32,
        Err(_) => 0,
    };

    let highest_group = permission_groups
        .first()
        .map(|g| g.name.clone())
        .unwrap_or_else(|| "None".to_string());

    let permission_summary = PermissionSummary {
        total_permissions: permissions.len() as i32,
        active_permissions: permissions.len() as i32,
        expiring_soon: expiring_count,
        expired_permissions: expired_count,
        highest_group,
    };

    tracing::info!(
        audit_id = %audit_id,
        wallet_address = %wallet,
        active_permissions = permission_summary.active_permissions,
        "Wallet permissions retrieved"
    );

    let response = WalletPermissionsResponse {
        wallet_address: wallet,
        permission_groups,
        effective_permissions: permissions,
        permission_summary,
        audit_id,
        retrieved_at: Utc::now(),
    };

    AdminResponse::success(response).into_response()
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

/// Check if wallet has a specific permission
/// Returns (has_permission, source_group_id, expires_at)
async fn check_wallet_permission(
    app_state: &AppState,
    wallet: &str,
    permission: &str,
) -> Result<(bool, Option<String>, Option<DateTime<Utc>>), sqlx::Error> {
    let row = sqlx::query(
        r#"
        SELECT
            pg.id as group_id,
            pg.name as group_name,
            wga.expires_at
        FROM permissions p
        JOIN permission_group_memberships pgm ON p.id = pgm.permission_id
        JOIN permission_groups pg ON pgm.group_id = pg.id
        JOIN wallet_group_assignments wga ON pg.id = wga.group_id
        WHERE wga.wallet_address = $1
          AND p.permission_string = $2
          AND wga.is_active = true
          AND (wga.expires_at IS NULL OR wga.expires_at > NOW())
        LIMIT 1
        "#
    )
    .bind(wallet)
    .bind(permission)
    .fetch_optional(&*app_state.db_pool)
    .await?;

    if let Some(r) = row {
        let group_id: Uuid = r.get("group_id");
        let expires_at: Option<DateTime<Utc>> = r.get("expires_at");
        Ok((true, Some(group_id.to_string()), expires_at))
    } else {
        Ok((false, None, None))
    }
}
