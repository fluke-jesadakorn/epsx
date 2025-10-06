// Bulk Permission Operations
// Consolidated bulk operations from bulk_permission_handlers.rs

use axum::{
    extract::State,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use sqlx::Row;

use crate::web::auth::AppState;
use crate::web::responses::AdminResponse;

// ============================================================================
// REQUEST/RESPONSE TYPES
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct BulkGrantRequest {
    pub wallet_addresses: Vec<String>,
    pub permission_strings: Vec<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub reason: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct BulkRevokeRequest {
    pub wallet_addresses: Vec<String>,
    pub permission_strings: Vec<String>,
    pub reason: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct BulkAssignRolesRequest {
    pub wallet_addresses: Vec<String>,
    pub group_id: String,
    pub expires_at: Option<DateTime<Utc>>,
    pub assignment_source: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct BulkApplyTemplateRequest {
    pub wallet_addresses: Vec<String>,
    pub template_name: String,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize)]
pub struct BulkValidateRequest {
    pub wallet_addresses: Vec<String>,
    pub check_expired: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct BulkOperationResponse {
    pub successful: Vec<BulkWalletResult>,
    pub failed: Vec<BulkWalletError>,
    pub summary: BulkSummary,
    pub operation: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct BulkWalletResult {
    pub wallet_address: String,
    pub permissions_added: Vec<String>,
    pub permissions_removed: Vec<String>,
    pub groups_assigned: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct BulkWalletError {
    pub wallet_address: String,
    pub error: String,
    pub error_code: String,
}

#[derive(Debug, Serialize)]
pub struct BulkSummary {
    pub total_wallets: i32,
    pub successful_operations: i32,
    pub failed_operations: i32,
    pub permissions_granted: i32,
    pub permissions_revoked: i32,
}

#[derive(Debug, Serialize)]
pub struct BulkValidationResponse {
    pub wallet_validations: Vec<WalletValidation>,
    pub summary: ValidationSummary,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct WalletValidation {
    pub wallet_address: String,
    pub valid_permissions: Vec<String>,
    pub expired_permissions: Vec<String>,
    pub total_permissions: i32,
}

#[derive(Debug, Serialize)]
pub struct ValidationSummary {
    pub total_wallets: i32,
    pub total_permissions: i32,
    pub valid_permissions: i32,
    pub expired_permissions: i32,
}

// ============================================================================
// HANDLERS
// ============================================================================

/// Bulk grant direct permissions to multiple wallets
/// POST /admin/permissions/bulk/grant
pub async fn bulk_grant(
    State(app_state): State<AppState>,
    Json(req): Json<BulkGrantRequest>,
) -> impl IntoResponse {
    if req.wallet_addresses.is_empty() {
        return AdminResponse::bad_request("No wallet addresses provided").into_response();
    }

    if req.permission_strings.is_empty() {
        return AdminResponse::bad_request("No permissions provided").into_response();
    }

    let mut successful = Vec::new();
    let mut failed = Vec::new();
    let mut total_granted = 0;

    for wallet_address in &req.wallet_addresses {
        let wallet = wallet_address.to_lowercase();

        // Validate wallet format
        if !wallet.starts_with("0x") || wallet.len() != 42 {
            failed.push(BulkWalletError {
                wallet_address: wallet.clone(),
                error: "Invalid wallet address format".to_string(),
                error_code: "INVALID_WALLET".to_string(),
            });
            continue;
        }

        let mut added_permissions = Vec::new();

        // Begin transaction for this wallet
        let mut tx = match app_state.db_pool.begin().await {
            Ok(tx) => tx,
            Err(e) => {
                tracing::error!("Failed to begin transaction: {}", e);
                failed.push(BulkWalletError {
                    wallet_address: wallet.clone(),
                    error: "Database transaction failed".to_string(),
                    error_code: "DB_ERROR".to_string(),
                });
                continue;
            }
        };

        // Grant each permission
        for perm_string in &req.permission_strings {
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
                Err(_) => continue,
            };

            // Grant direct permission
            match sqlx::query(
                r#"
                INSERT INTO wallet_direct_permissions (wallet_address, permission_id, expires_at)
                VALUES ($1, $2, $3)
                ON CONFLICT (wallet_address, permission_id) DO UPDATE
                SET expires_at = EXCLUDED.expires_at, is_active = true
                "#
            )
            .bind(&wallet)
            .bind(perm_id)
            .bind(req.expires_at)
            .execute(&mut *tx)
            .await
            {
                Ok(_) => {
                    added_permissions.push(perm_string.clone());
                    total_granted += 1;
                }
                Err(_) => continue,
            }
        }

        // Commit transaction
        if let Err(e) = tx.commit().await {
            tracing::error!("Failed to commit transaction: {}", e);
            failed.push(BulkWalletError {
                wallet_address: wallet.clone(),
                error: "Failed to save permissions".to_string(),
                error_code: "COMMIT_FAILED".to_string(),
            });
            continue;
        }

        successful.push(BulkWalletResult {
            wallet_address: wallet,
            permissions_added: added_permissions,
            permissions_removed: vec![],
            groups_assigned: vec![],
        });
    }

    let summary = BulkSummary {
        total_wallets: req.wallet_addresses.len() as i32,
        successful_operations: successful.len() as i32,
        failed_operations: failed.len() as i32,
        permissions_granted: total_granted,
        permissions_revoked: 0,
    };

    tracing::info!(
        "Bulk grant completed: {} successful, {} failed, {} permissions granted",
        summary.successful_operations,
        summary.failed_operations,
        summary.permissions_granted
    );

    AdminResponse::success(BulkOperationResponse {
        successful,
        failed,
        summary,
        operation: "bulk_grant_permissions".to_string(),
        timestamp: Utc::now(),
    }).into_response()
}

/// Bulk revoke direct permissions from multiple wallets
/// POST /admin/permissions/bulk/revoke
pub async fn bulk_revoke(
    State(app_state): State<AppState>,
    Json(req): Json<BulkRevokeRequest>,
) -> impl IntoResponse {
    if req.wallet_addresses.is_empty() {
        return AdminResponse::bad_request("No wallet addresses provided").into_response();
    }

    if req.permission_strings.is_empty() {
        return AdminResponse::bad_request("No permissions provided").into_response();
    }

    let mut successful = Vec::new();
    let failed = Vec::new();
    let mut total_revoked = 0;

    for wallet_address in &req.wallet_addresses {
        let wallet = wallet_address.to_lowercase();

        let mut removed_permissions = Vec::new();

        // Revoke each permission
        for perm_string in &req.permission_strings {
            // Get permission ID
            let perm_id = match sqlx::query_scalar::<_, Uuid>(
                "SELECT id FROM permissions WHERE permission_string = $1"
            )
            .bind(perm_string)
            .fetch_optional(&*app_state.db_pool)
            .await
            {
                Ok(Some(id)) => id,
                _ => continue,
            };

            // Revoke direct permission
            match sqlx::query(
                "DELETE FROM wallet_direct_permissions WHERE wallet_address = $1 AND permission_id = $2"
            )
            .bind(&wallet)
            .bind(perm_id)
            .execute(&*app_state.db_pool)
            .await
            {
                Ok(result) if result.rows_affected() > 0 => {
                    removed_permissions.push(perm_string.clone());
                    total_revoked += 1;
                }
                _ => continue,
            }
        }

        successful.push(BulkWalletResult {
            wallet_address: wallet,
            permissions_added: vec![],
            permissions_removed: removed_permissions,
            groups_assigned: vec![],
        });
    }

    let summary = BulkSummary {
        total_wallets: req.wallet_addresses.len() as i32,
        successful_operations: successful.len() as i32,
        failed_operations: failed.len() as i32,
        permissions_granted: 0,
        permissions_revoked: total_revoked,
    };

    tracing::info!(
        "Bulk revoke completed: {} successful, {} failed, {} permissions revoked",
        summary.successful_operations,
        summary.failed_operations,
        summary.permissions_revoked
    );

    AdminResponse::success(BulkOperationResponse {
        successful,
        failed,
        summary,
        operation: "bulk_revoke_permissions".to_string(),
        timestamp: Utc::now(),
    }).into_response()
}

/// Bulk assign wallets to a permission group
/// POST /admin/permissions/bulk/assign-roles
pub async fn bulk_assign_roles(
    State(app_state): State<AppState>,
    Json(req): Json<BulkAssignRolesRequest>,
) -> impl IntoResponse {
    let group_uuid = match Uuid::parse_str(&req.group_id) {
        Ok(id) => id,
        Err(_) => return AdminResponse::bad_request("Invalid group ID format").into_response(),
    };

    if req.wallet_addresses.is_empty() {
        return AdminResponse::bad_request("No wallet addresses provided").into_response();
    }

    let mut successful = Vec::new();
    let mut failed = Vec::new();

    for wallet_address in &req.wallet_addresses {
        let wallet = wallet_address.to_lowercase();

        // Validate wallet format
        if !wallet.starts_with("0x") || wallet.len() != 42 {
            failed.push(BulkWalletError {
                wallet_address: wallet.clone(),
                error: "Invalid wallet address format".to_string(),
                error_code: "INVALID_WALLET".to_string(),
            });
            continue;
        }

        // Assign wallet to group
        match sqlx::query(
            r#"
            INSERT INTO wallet_group_assignments (
                wallet_address, group_id, assigned_at, expires_at, is_active, assignment_source
            )
            VALUES ($1, $2, NOW(), $3, true, $4)
            ON CONFLICT (wallet_address, group_id) DO UPDATE
            SET is_active = true, expires_at = EXCLUDED.expires_at, updated_at = NOW()
            "#
        )
        .bind(&wallet)
        .bind(group_uuid)
        .bind(req.expires_at)
        .bind(req.assignment_source.as_deref().unwrap_or("bulk_assignment"))
        .execute(&*app_state.db_pool)
        .await
        {
            Ok(_) => {
                successful.push(BulkWalletResult {
                    wallet_address: wallet,
                    permissions_added: vec![],
                    permissions_removed: vec![],
                    groups_assigned: vec![req.group_id.clone()],
                });
            }
            Err(e) => {
                tracing::error!("Failed to assign wallet to group: {}", e);
                failed.push(BulkWalletError {
                    wallet_address: wallet,
                    error: "Failed to assign to group".to_string(),
                    error_code: "ASSIGNMENT_FAILED".to_string(),
                });
            }
        }
    }

    let summary = BulkSummary {
        total_wallets: req.wallet_addresses.len() as i32,
        successful_operations: successful.len() as i32,
        failed_operations: failed.len() as i32,
        permissions_granted: 0,
        permissions_revoked: 0,
    };

    tracing::info!(
        "Bulk role assignment completed: {} successful, {} failed",
        summary.successful_operations,
        summary.failed_operations
    );

    AdminResponse::success(BulkOperationResponse {
        successful,
        failed,
        summary,
        operation: "bulk_assign_roles".to_string(),
        timestamp: Utc::now(),
    }).into_response()
}

/// Apply a permission template to multiple wallets
/// POST /admin/permissions/bulk/apply-template
pub async fn bulk_apply_template(
    State(app_state): State<AppState>,
    Json(req): Json<BulkApplyTemplateRequest>,
) -> impl IntoResponse {
    // Get template permissions (hardcoded templates for simplicity)
    let template_permissions = match req.template_name.as_str() {
        "basic" => vec![
            "epsx:analytics:view".to_string(),
            "epsx:portfolio:view".to_string(),
        ],
        "professional" => vec![
            "epsx:analytics:view".to_string(),
            "epsx:analytics:export".to_string(),
            "epsx:portfolio:view".to_string(),
            "epsx:portfolio:manage".to_string(),
        ],
        "enterprise" => vec![
            "epsx:analytics:view".to_string(),
            "epsx:analytics:export".to_string(),
            "epsx:portfolio:view".to_string(),
            "epsx:portfolio:manage".to_string(),
            "epsx:trading:advanced".to_string(),
            "epsx:alerts:create".to_string(),
        ],
        _ => return AdminResponse::bad_request("Unknown template name").into_response(),
    };

    // Convert to bulk grant request
    let grant_req = BulkGrantRequest {
        wallet_addresses: req.wallet_addresses.clone(),
        permission_strings: template_permissions,
        expires_at: req.expires_at,
        reason: Some(format!("Applied template: {}", req.template_name)),
    };

    // Reuse bulk grant logic (call and convert return type)
    bulk_grant(State(app_state), Json(grant_req)).await.into_response()
}

/// Validate permissions for multiple wallets
/// POST /admin/permissions/bulk/validate
pub async fn bulk_validate(
    State(app_state): State<AppState>,
    Json(req): Json<BulkValidateRequest>,
) -> impl IntoResponse {
    if req.wallet_addresses.is_empty() {
        return AdminResponse::bad_request("No wallet addresses provided").into_response();
    }

    let mut wallet_validations = Vec::new();
    let mut total_permissions = 0;
    let mut total_valid = 0;
    let mut total_expired = 0;

    for wallet_address in &req.wallet_addresses {
        let wallet = wallet_address.to_lowercase();

        // Get all permissions (direct + groups)
        let permissions = match sqlx::query(
            r#"
            SELECT DISTINCT p.permission_string, wdp.expires_at as direct_expires
            FROM permissions p
            LEFT JOIN wallet_direct_permissions wdp ON p.id = wdp.permission_id AND wdp.wallet_address = $1
            LEFT JOIN permission_group_memberships pgm ON p.id = pgm.permission_id
            LEFT JOIN wallet_group_assignments wga ON pgm.group_id = wga.group_id AND wga.wallet_address = $1
            WHERE (wdp.is_active = true OR wga.is_active = true)
            "#
        )
        .bind(&wallet)
        .fetch_all(&*app_state.db_pool)
        .await
        {
            Ok(rows) => rows,
            Err(_) => continue,
        };

        let mut valid_permissions = Vec::new();
        let mut expired_permissions = Vec::new();

        for row in permissions {
            let perm_string: String = row.get("permission_string");
            let expires_at: Option<DateTime<Utc>> = row.get("direct_expires");

            if let Some(expiry) = expires_at {
                if expiry < Utc::now() {
                    expired_permissions.push(perm_string);
                    total_expired += 1;
                } else {
                    valid_permissions.push(perm_string);
                    total_valid += 1;
                }
            } else {
                valid_permissions.push(perm_string);
                total_valid += 1;
            }
        }

        total_permissions += valid_permissions.len() + expired_permissions.len();

        wallet_validations.push(WalletValidation {
            wallet_address: wallet,
            valid_permissions,
            expired_permissions,
            total_permissions: (total_permissions as i32),
        });
    }

    let summary = ValidationSummary {
        total_wallets: req.wallet_addresses.len() as i32,
        total_permissions: total_permissions as i32,
        valid_permissions: total_valid,
        expired_permissions: total_expired,
    };

    AdminResponse::success(BulkValidationResponse {
        wallet_validations,
        summary,
        timestamp: Utc::now(),
    }).into_response()
}
