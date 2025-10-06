// Wallet-Group Assignment Management
// Consolidates assignment operations from permission_group_handlers.rs and normalized_permission_handlers.rs

use axum::{
    extract::{Path, Query, State},
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use sqlx::Row;

use crate::web::auth::AppState;
use crate::web::responses::{AdminResponse, create_pagination};

// ============================================================================
// REQUEST/RESPONSE TYPES
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateAssignmentRequest {
    pub wallet_address: String,
    pub group_id: String,
    pub assignment_source: String, // "manual" | "payment" | "web3_asset" | "dao_governance" | "admin" | "migration" | "auto_assignment"
    pub assignment_reason: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub auto_renew: Option<bool>,
    pub payment_reference: Option<String>,
    pub subscription_id: Option<String>,
    pub assignment_metadata: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
pub struct AssignmentResponse {
    pub id: String,
    pub wallet_address: String,
    pub group_id: String,
    pub group_name: String,
    pub group_type: String,
    pub assigned_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub is_active: bool,
    pub assignment_source: String,
    pub assignment_reason: Option<String>,
    pub assigned_by: Option<String>,
    pub payment_reference: Option<String>,
    pub subscription_id: Option<String>,
    pub auto_renew: bool,
    pub next_billing_date: Option<DateTime<Utc>>,
    pub assignment_metadata: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub struct ListAssignmentsQuery {
    pub page: Option<u32>,
    pub limit: Option<u32>,
    pub wallet_address: Option<String>,
    pub group_id: Option<String>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct ExpiringAssignmentsQuery {
    pub days: Option<i64>,
}

// ============================================================================
// HANDLERS
// ============================================================================

/// Create a new wallet-group assignment
/// POST /admin/permissions/assignments
pub async fn create_assignment(
    State(app_state): State<AppState>,
    Json(req): Json<CreateAssignmentRequest>,
) -> impl IntoResponse {
    // Validate wallet address format
    let wallet = req.wallet_address.to_lowercase();
    if !wallet.starts_with("0x") || wallet.len() != 42 {
        return AdminResponse::bad_request("Invalid wallet address format (must be 42 characters starting with 0x)").into_response();
    }

    // Parse group ID
    let group_uuid = match Uuid::parse_str(&req.group_id) {
        Ok(id) => id,
        Err(_) => return AdminResponse::bad_request("Invalid group ID format").into_response(),
    };

    // Begin transaction
    let mut tx = match app_state.db_pool.begin().await {
        Ok(tx) => tx,
        Err(e) => {
            tracing::error!("Failed to begin transaction: {}", e);
            return AdminResponse::server_error("Database transaction failed").into_response();
        }
    };

    // Insert or update assignment
    let assignment_metadata = req.assignment_metadata.clone().unwrap_or(serde_json::json!({}));

    let assignment_id = match sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO wallet_group_assignments (
            wallet_address, group_id, assigned_at, expires_at, is_active,
            assignment_source, assignment_reason, payment_reference, subscription_id,
            auto_renew, next_billing_date, assignment_metadata
        )
        VALUES ($1, $2, NOW(), $3, true, $4, $5, $6, $7, $8, $9, $10)
        ON CONFLICT (wallet_address, group_id) DO UPDATE
        SET is_active = true, expires_at = EXCLUDED.expires_at, updated_at = NOW()
        RETURNING id
        "#
    )
    .bind(&wallet)
    .bind(group_uuid)
    .bind(req.expires_at)
    .bind(&req.assignment_source)
    .bind(&req.assignment_reason)
    .bind(&req.payment_reference)
    .bind(&req.subscription_id)
    .bind(req.auto_renew.unwrap_or(false))
    .bind(req.expires_at)
    .bind(&assignment_metadata)
    .fetch_one(&mut *tx)
    .await
    {
        Ok(id) => id,
        Err(e) => {
            tracing::error!("Failed to create assignment: {}", e);
            return AdminResponse::server_error("Failed to create assignment").into_response();
        }
    };

    // Fetch group details
    let group = match sqlx::query!(
        "SELECT name, group_type FROM permission_groups WHERE id = $1",
        group_uuid
    )
    .fetch_optional(&mut *tx)
    .await
    {
        Ok(Some(group)) => group,
        Ok(None) => return AdminResponse::not_found("Permission group").into_response(),
        Err(e) => {
            tracing::error!("Failed to fetch group details: {}", e);
            return AdminResponse::server_error("Database query failed").into_response();
        }
    };

    // Commit transaction
    if let Err(e) = tx.commit().await {
        tracing::error!("Failed to commit transaction: {}", e);
        return AdminResponse::server_error("Failed to save assignment").into_response();
    }

    // Build response
    let response = AssignmentResponse {
        id: assignment_id.to_string(),
        wallet_address: wallet,
        group_id: req.group_id,
        group_name: group.name,
        group_type: group.group_type,
        assigned_at: Utc::now(),
        expires_at: req.expires_at,
        is_active: true,
        assignment_source: req.assignment_source,
        assignment_reason: req.assignment_reason,
        assigned_by: None,
        payment_reference: req.payment_reference,
        subscription_id: req.subscription_id,
        auto_renew: req.auto_renew.unwrap_or(false),
        next_billing_date: req.expires_at,
        assignment_metadata: req.assignment_metadata.unwrap_or(serde_json::json!({})),
    };

    AdminResponse::created(response, "Wallet assigned to permission group successfully").into_response()
}

/// List wallet-group assignments with pagination
/// GET /admin/permissions/assignments
pub async fn list_assignments(
    State(app_state): State<AppState>,
    Query(query): Query<ListAssignmentsQuery>,
) -> impl IntoResponse {
    let page = query.page.unwrap_or(1);
    let limit = query.limit.unwrap_or(20).min(100);
    let offset = (page - 1) * limit;

    // Build query with optional filters
    let mut sql = String::from(
        r#"
        SELECT
            wga.id,
            wga.wallet_address,
            wga.group_id,
            pg.name as group_name,
            pg.group_type,
            wga.assigned_at,
            wga.expires_at,
            wga.is_active,
            wga.assignment_source,
            wga.assignment_reason,
            wga.assigned_by,
            wga.payment_reference,
            wga.subscription_id,
            wga.auto_renew,
            wga.next_billing_date,
            wga.assignment_metadata
        FROM wallet_group_assignments wga
        JOIN permission_groups pg ON wga.group_id = pg.id
        "#
    );

    let mut where_clauses = Vec::new();
    let group_clause;
    let active_clause;

    if query.wallet_address.is_some() {
        where_clauses.push("wga.wallet_address = $3");
    }
    if query.group_id.is_some() {
        let clause_idx = if query.wallet_address.is_some() { 4 } else { 3 };
        group_clause = format!("wga.group_id = ${}", clause_idx);
        where_clauses.push(&group_clause);
    }
    if query.is_active.is_some() {
        let clause_idx = 3 + where_clauses.len();
        active_clause = format!("wga.is_active = ${}", clause_idx);
        where_clauses.push(&active_clause);
    }

    if !where_clauses.is_empty() {
        sql.push_str(" WHERE ");
        sql.push_str(&where_clauses.join(" AND "));
    }

    sql.push_str(" ORDER BY wga.assigned_at DESC LIMIT $1 OFFSET $2");

    // Get total count
    let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM wallet_group_assignments")
        .fetch_one(&*app_state.db_pool)
        .await
        .unwrap_or(0);

    // Execute query
    let mut query_builder = sqlx::query(&sql)
        .bind(limit as i32)
        .bind(offset as i32);

    if let Some(wallet) = &query.wallet_address {
        query_builder = query_builder.bind(wallet.to_lowercase());
    }
    if let Some(group_id) = &query.group_id {
        if let Ok(uuid) = Uuid::parse_str(group_id) {
            query_builder = query_builder.bind(uuid);
        }
    }
    if let Some(is_active) = query.is_active {
        query_builder = query_builder.bind(is_active);
    }

    let rows = match query_builder.fetch_all(&*app_state.db_pool).await {
        Ok(rows) => rows,
        Err(e) => {
            tracing::error!("Failed to list assignments: {}", e);
            return AdminResponse::server_error("Database query failed").into_response();
        }
    };

    let assignments: Vec<AssignmentResponse> = rows.iter().map(|row| {
        AssignmentResponse {
            id: row.get::<Uuid, _>("id").to_string(),
            wallet_address: row.get("wallet_address"),
            group_id: row.get::<Uuid, _>("group_id").to_string(),
            group_name: row.get("group_name"),
            group_type: row.get("group_type"),
            assigned_at: row.get("assigned_at"),
            expires_at: row.get("expires_at"),
            is_active: row.get("is_active"),
            assignment_source: row.get("assignment_source"),
            assignment_reason: row.get("assignment_reason"),
            assigned_by: row.get("assigned_by"),
            payment_reference: row.get("payment_reference"),
            subscription_id: row.get("subscription_id"),
            auto_renew: row.get("auto_renew"),
            next_billing_date: row.get("next_billing_date"),
            assignment_metadata: row.get("assignment_metadata"),
        }
    }).collect();

    let pagination = create_pagination(page, limit, total as u64);
    AdminResponse::success_with_pagination(assignments, pagination).into_response()
}

/// Remove a wallet-group assignment
/// DELETE /admin/permissions/assignments/:assignment_id
pub async fn remove_assignment(
    State(app_state): State<AppState>,
    Path(assignment_id): Path<String>,
) -> impl IntoResponse {
    let assignment_uuid = match Uuid::parse_str(&assignment_id) {
        Ok(id) => id,
        Err(_) => return AdminResponse::bad_request("Invalid assignment ID format").into_response(),
    };

    match sqlx::query(
        "UPDATE wallet_group_assignments SET is_active = false, updated_at = NOW() WHERE id = $1"
    )
    .bind(assignment_uuid)
    .execute(&*app_state.db_pool)
    .await
    {
        Ok(result) if result.rows_affected() > 0 => {
            AdminResponse::success_with_message(
                serde_json::json!({"deleted": true}),
                "Assignment removed successfully"
            ).into_response()
        },
        Ok(_) => AdminResponse::not_found("Assignment").into_response(),
        Err(e) => {
            tracing::error!("Failed to remove assignment: {}", e);
            AdminResponse::server_error("Failed to remove assignment").into_response()
        }
    }
}

/// Get assignments expiring soon
/// GET /admin/permissions/assignments/expiring
pub async fn get_expiring_assignments(
    State(app_state): State<AppState>,
    Query(query): Query<ExpiringAssignmentsQuery>,
) -> impl IntoResponse {
    let days = query.days.unwrap_or(7);

    let rows = match sqlx::query(
        r#"
        SELECT
            wga.id,
            wga.wallet_address,
            wga.group_id,
            pg.name as group_name,
            wga.assigned_at,
            wga.expires_at
        FROM wallet_group_assignments wga
        JOIN permission_groups pg ON wga.group_id = pg.id
        WHERE wga.is_active = true
          AND wga.expires_at IS NOT NULL
          AND wga.expires_at BETWEEN NOW() AND NOW() + ($1 || ' days')::interval
        ORDER BY wga.expires_at ASC
        "#
    )
    .bind(days)
    .fetch_all(&*app_state.db_pool)
    .await
    {
        Ok(rows) => rows,
        Err(e) => {
            tracing::error!("Failed to fetch expiring assignments: {}", e);
            return AdminResponse::server_error("Database query failed").into_response();
        }
    };

    let expiring: Vec<serde_json::Value> = rows.iter().map(|row| {
        serde_json::json!({
            "id": row.get::<Uuid, _>("id").to_string(),
            "wallet_address": row.get::<String, _>("wallet_address"),
            "group_id": row.get::<Uuid, _>("group_id").to_string(),
            "group_name": row.get::<String, _>("group_name"),
            "assigned_at": row.get::<DateTime<Utc>, _>("assigned_at"),
            "expires_at": row.get::<DateTime<Utc>, _>("expires_at"),
        })
    }).collect();

    AdminResponse::success(serde_json::json!({
        "assignments": expiring,
        "count": expiring.len(),
        "days": days
    })).into_response()
}

/// Get assignment history for a wallet
/// GET /admin/permissions/assignments/history/:wallet
pub async fn get_assignment_history(
    State(app_state): State<AppState>,
    Path(wallet): Path<String>,
) -> impl IntoResponse {
    let wallet = wallet.to_lowercase();

    let rows = match sqlx::query(
        r#"
        SELECT
            wga.id,
            wga.wallet_address,
            wga.group_id,
            pg.name as group_name,
            pg.group_type,
            wga.assigned_at,
            wga.expires_at,
            wga.is_active,
            wga.assignment_source,
            wga.assignment_reason,
            wga.assigned_by,
            wga.payment_reference,
            wga.subscription_id,
            wga.auto_renew,
            wga.next_billing_date,
            wga.assignment_metadata,
            wga.updated_at
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
            tracing::error!("Failed to fetch assignment history: {}", e);
            return AdminResponse::server_error("Database query failed").into_response();
        }
    };

    let history: Vec<AssignmentResponse> = rows.iter().map(|row| {
        AssignmentResponse {
            id: row.get::<Uuid, _>("id").to_string(),
            wallet_address: row.get("wallet_address"),
            group_id: row.get::<Uuid, _>("group_id").to_string(),
            group_name: row.get("group_name"),
            group_type: row.get("group_type"),
            assigned_at: row.get("assigned_at"),
            expires_at: row.get("expires_at"),
            is_active: row.get("is_active"),
            assignment_source: row.get("assignment_source"),
            assignment_reason: row.get("assignment_reason"),
            assigned_by: row.get("assigned_by"),
            payment_reference: row.get("payment_reference"),
            subscription_id: row.get("subscription_id"),
            auto_renew: row.get("auto_renew"),
            next_billing_date: row.get("next_billing_date"),
            assignment_metadata: row.get("assignment_metadata"),
        }
    }).collect();

    AdminResponse::success(serde_json::json!({
        "wallet_address": wallet,
        "assignments": history,
        "count": history.len()
    })).into_response()
}

/// Get groups assigned to a wallet
/// GET /admin/permissions/wallets/:wallet/groups
pub async fn get_wallet_groups(
    State(app_state): State<AppState>,
    Path(wallet): Path<String>,
) -> impl IntoResponse {
    let wallet = wallet.to_lowercase();

    let rows = match sqlx::query(
        r#"
        SELECT
            wga.id, wga.group_id, wga.assigned_at, wga.expires_at, wga.is_active,
            pg.name as group_name, pg.slug as group_slug, pg.group_type
        FROM wallet_group_assignments wga
        JOIN permission_groups pg ON wga.group_id = pg.id
        WHERE wga.wallet_address = $1 AND wga.is_active = true
        ORDER BY wga.assigned_at DESC
        "#
    )
    .bind(&wallet)
    .fetch_all(&*app_state.db_pool)
    .await
    {
        Ok(rows) => rows,
        Err(e) => {
            tracing::error!("Failed to fetch wallet groups: {}", e);
            return AdminResponse::server_error("Database query failed").into_response();
        }
    };

    let groups: Vec<serde_json::Value> = rows.iter().map(|row| {
        serde_json::json!({
            "id": row.get::<Uuid, _>("id").to_string(),
            "group_id": row.get::<Uuid, _>("group_id").to_string(),
            "group_name": row.get::<String, _>("group_name"),
            "group_slug": row.get::<String, _>("group_slug"),
            "group_type": row.get::<String, _>("group_type"),
            "assigned_at": row.get::<DateTime<Utc>, _>("assigned_at"),
            "expires_at": row.get::<Option<DateTime<Utc>>, _>("expires_at"),
            "is_active": row.get::<bool, _>("is_active"),
        })
    }).collect();

    AdminResponse::success(serde_json::json!({
        "wallet_address": wallet,
        "groups": groups,
        "count": groups.len()
    })).into_response()
}
