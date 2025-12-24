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
use diesel::prelude::*;
use diesel_async::{AsyncConnection, RunQueryDsl};

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

#[derive(Debug, Deserialize)]
pub struct GroupHistoryQuery {
    pub operation_type: Option<String>,
    pub operation_source: Option<String>,
    pub group_id: Option<String>,
    pub user_search: Option<String>,
    pub date_from: Option<DateTime<Utc>>,
    pub date_to: Option<DateTime<Utc>>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

#[derive(Debug, Serialize)]
pub struct GroupHistoryResponse {
    pub id: String,
    pub user_id: String,
    pub user_email: Option<String>, // Not available in blockchain auth, but kept for interface compat
    pub user_name: Option<String>,
    pub group_id: String,
    pub group_name: Option<String>,
    pub operation_type: String,
    pub operation_source: String,
    pub performed_by: Option<String>,
    pub performed_by_name: Option<String>,
    pub reason: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
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

    // Get database connection
    let mut conn = match app_state.db_pool.get().await {
        Ok(conn) => conn,
        Err(e) => {
            tracing::error!("Failed to get database connection: {}", e);
            return AdminResponse::server_error("Database connection failed").into_response();
        }
    };

    #[derive(QueryableByName)]
    struct AssignmentId {
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        id: Uuid,
    }

    #[derive(QueryableByName)]
    struct GroupDetails {
        #[diesel(sql_type = diesel::sql_types::Text)]
        name: String,
        #[diesel(sql_type = diesel::sql_types::Text)]
        group_type: String,
    }

    // Run transaction
    let assignment_metadata = req.assignment_metadata.clone().unwrap_or(serde_json::json!({}));
    let wallet_clone = wallet.clone();
    let wallet_for_user_insert = wallet.clone();
    let assignment_source_clone = req.assignment_source.clone();
    let assignment_reason_clone = req.assignment_reason.clone();
    let payment_reference_clone = req.payment_reference.clone();
    let subscription_id_clone = req.subscription_id.clone();

    let result = conn.transaction::<_, diesel::result::Error, _>(|conn| {
        Box::pin(async move {
            // CRITICAL: Ensure wallet_users entry exists before assignment (FK constraint)
            // This auto-creates a wallet user if it doesn't exist
            diesel::sql_query(
                r#"
                INSERT INTO wallet_users (wallet_address, is_active, tier_level, wallet_metadata)
                VALUES ($1, true, 'Bronze', '{}')
                ON CONFLICT (wallet_address) DO NOTHING
                "#
            )
            .bind::<diesel::sql_types::Text, _>(&wallet_for_user_insert)
            .execute(conn)
            .await?;

            // Insert or update assignment
            let assignment_id = diesel::sql_query(
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
            .bind::<diesel::sql_types::Text, _>(&wallet_clone)
            .bind::<diesel::sql_types::Uuid, _>(group_uuid)
            .bind::<diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>, _>(req.expires_at)
            .bind::<diesel::sql_types::Text, _>(&assignment_source_clone)
            .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(&assignment_reason_clone)
            .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(&payment_reference_clone)
            .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(&subscription_id_clone)
            .bind::<diesel::sql_types::Bool, _>(req.auto_renew.unwrap_or(false))
            .bind::<diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>, _>(req.expires_at)
            .bind::<diesel::sql_types::Jsonb, _>(&assignment_metadata)
            .get_result::<AssignmentId>(conn)
            .await?
            .id;

            // Fetch group details
            let group = diesel::sql_query(
                "SELECT name, group_type FROM groups WHERE id = $1"
            )
            .bind::<diesel::sql_types::Uuid, _>(group_uuid)
            .get_result::<GroupDetails>(conn)
            .await
            .optional()?;

            Ok((assignment_id, group))
        })
    }).await;

    let (assignment_id, group) = match result {
        Ok((id, Some(g))) => (id, g),
        Ok((_, None)) => return AdminResponse::not_found("Permission group").into_response(),
        Err(e) => {
            tracing::error!("Transaction failed: {}", e);
            return AdminResponse::server_error("Failed to create assignment").into_response();
        }
    };

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
        JOIN groups pg ON wga.group_id = pg.id
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

    let mut conn = match app_state.db_pool.get().await {
        Ok(conn) => conn,
        Err(e) => {
            tracing::error!("Failed to get database connection: {}", e);
            return AdminResponse::server_error("Database connection failed").into_response();
        }
    };

    #[derive(QueryableByName)]
    struct CountRow {
        #[diesel(sql_type = diesel::sql_types::BigInt)]
        count: i64,
    }

    #[derive(QueryableByName)]
    struct AssignmentRow {
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        id: Uuid,
        #[diesel(sql_type = diesel::sql_types::Text)]
        wallet_address: String,
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        group_id: Uuid,
        #[diesel(sql_type = diesel::sql_types::Text)]
        group_name: String,
        #[diesel(sql_type = diesel::sql_types::Text)]
        group_type: String,
        #[diesel(sql_type = diesel::sql_types::Timestamptz)]
        assigned_at: DateTime<Utc>,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>)]
        expires_at: Option<DateTime<Utc>>,
        #[diesel(sql_type = diesel::sql_types::Bool)]
        is_active: bool,
        #[diesel(sql_type = diesel::sql_types::Text)]
        assignment_source: String,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
        assignment_reason: Option<String>,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
        assigned_by: Option<String>,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
        payment_reference: Option<String>,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
        subscription_id: Option<String>,
        #[diesel(sql_type = diesel::sql_types::Bool)]
        auto_renew: bool,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>)]
        next_billing_date: Option<DateTime<Utc>>,
        #[diesel(sql_type = diesel::sql_types::Jsonb)]
        assignment_metadata: serde_json::Value,
    }

    // Get total count
    let total: i64 = match diesel::sql_query(
        "SELECT COUNT(*)::bigint as count FROM wallet_group_assignments"
    )
    .get_result::<CountRow>(&mut conn)
    .await
    {
        Ok(row) => row.count,
        Err(_) => 0,
    };

    // Build and execute query with all binds inline
    let result = match (
        &query.wallet_address,
        &query.group_id.as_ref().and_then(|g| Uuid::parse_str(g).ok()),
        query.is_active,
    ) {
        (Some(wallet), Some(group_uuid), Some(is_active)) => {
            diesel::sql_query(&sql)
                .bind::<diesel::sql_types::Integer, _>(limit as i32)
                .bind::<diesel::sql_types::Integer, _>(offset as i32)
                .bind::<diesel::sql_types::Text, _>(wallet.to_lowercase())
                .bind::<diesel::sql_types::Uuid, _>(*group_uuid)
                .bind::<diesel::sql_types::Bool, _>(is_active)
                .load::<AssignmentRow>(&mut conn)
                .await
        }
        (Some(wallet), Some(group_uuid), None) => {
            diesel::sql_query(&sql)
                .bind::<diesel::sql_types::Integer, _>(limit as i32)
                .bind::<diesel::sql_types::Integer, _>(offset as i32)
                .bind::<diesel::sql_types::Text, _>(wallet.to_lowercase())
                .bind::<diesel::sql_types::Uuid, _>(*group_uuid)
                .load::<AssignmentRow>(&mut conn)
                .await
        }
        (Some(wallet), None, Some(is_active)) => {
            diesel::sql_query(&sql)
                .bind::<diesel::sql_types::Integer, _>(limit as i32)
                .bind::<diesel::sql_types::Integer, _>(offset as i32)
                .bind::<diesel::sql_types::Text, _>(wallet.to_lowercase())
                .bind::<diesel::sql_types::Bool, _>(is_active)
                .load::<AssignmentRow>(&mut conn)
                .await
        }
        (Some(wallet), None, None) => {
            diesel::sql_query(&sql)
                .bind::<diesel::sql_types::Integer, _>(limit as i32)
                .bind::<diesel::sql_types::Integer, _>(offset as i32)
                .bind::<diesel::sql_types::Text, _>(wallet.to_lowercase())
                .load::<AssignmentRow>(&mut conn)
                .await
        }
        (None, Some(group_uuid), Some(is_active)) => {
            diesel::sql_query(&sql)
                .bind::<diesel::sql_types::Integer, _>(limit as i32)
                .bind::<diesel::sql_types::Integer, _>(offset as i32)
                .bind::<diesel::sql_types::Uuid, _>(*group_uuid)
                .bind::<diesel::sql_types::Bool, _>(is_active)
                .load::<AssignmentRow>(&mut conn)
                .await
        }
        (None, Some(group_uuid), None) => {
            diesel::sql_query(&sql)
                .bind::<diesel::sql_types::Integer, _>(limit as i32)
                .bind::<diesel::sql_types::Integer, _>(offset as i32)
                .bind::<diesel::sql_types::Uuid, _>(*group_uuid)
                .load::<AssignmentRow>(&mut conn)
                .await
        }
        (None, None, Some(is_active)) => {
            diesel::sql_query(&sql)
                .bind::<diesel::sql_types::Integer, _>(limit as i32)
                .bind::<diesel::sql_types::Integer, _>(offset as i32)
                .bind::<diesel::sql_types::Bool, _>(is_active)
                .load::<AssignmentRow>(&mut conn)
                .await
        }
        (None, None, None) => {
            diesel::sql_query(&sql)
                .bind::<diesel::sql_types::Integer, _>(limit as i32)
                .bind::<diesel::sql_types::Integer, _>(offset as i32)
                .load::<AssignmentRow>(&mut conn)
                .await
        }
    };

    let rows = match result {
        Ok(rows) => rows,
        Err(e) => {
            tracing::error!("Failed to list assignments: {}", e);
            return AdminResponse::server_error("Database query failed").into_response();
        }
    };

    let assignments: Vec<AssignmentResponse> = rows.into_iter().map(|row| {
        AssignmentResponse {
            id: row.id.to_string(),
            wallet_address: row.wallet_address,
            group_id: row.group_id.to_string(),
            group_name: row.group_name,
            group_type: row.group_type,
            assigned_at: row.assigned_at,
            expires_at: row.expires_at,
            is_active: row.is_active,
            assignment_source: row.assignment_source,
            assignment_reason: row.assignment_reason,
            assigned_by: row.assigned_by,
            payment_reference: row.payment_reference,
            subscription_id: row.subscription_id,
            auto_renew: row.auto_renew,
            next_billing_date: row.next_billing_date,
            assignment_metadata: row.assignment_metadata,
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

    let mut conn = match app_state.db_pool.get().await {
        Ok(conn) => conn,
        Err(e) => {
            tracing::error!("Failed to get database connection: {}", e);
            return AdminResponse::server_error("Database connection failed").into_response();
        }
    };

    match diesel::sql_query(
        "UPDATE wallet_group_assignments SET is_active = false, updated_at = NOW() WHERE id = $1"
    )
    .bind::<diesel::sql_types::Uuid, _>(assignment_uuid)
    .execute(&mut conn)
    .await
    {
        Ok(rows) if rows > 0 => {
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

    let mut conn = match app_state.db_pool.get().await {
        Ok(conn) => conn,
        Err(e) => {
            tracing::error!("Failed to get database connection: {}", e);
            return AdminResponse::server_error("Database connection failed").into_response();
        }
    };

    #[derive(QueryableByName)]
    struct ExpiringRow {
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        id: Uuid,
        #[diesel(sql_type = diesel::sql_types::Text)]
        wallet_address: String,
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        group_id: Uuid,
        #[diesel(sql_type = diesel::sql_types::Text)]
        group_name: String,
        #[diesel(sql_type = diesel::sql_types::Timestamptz)]
        assigned_at: DateTime<Utc>,
        #[diesel(sql_type = diesel::sql_types::Timestamptz)]
        expires_at: DateTime<Utc>,
    }

    let rows = match diesel::sql_query(
        r#"
        SELECT
            wga.id,
            wga.wallet_address,
            wga.group_id,
            pg.name as group_name,
            wga.assigned_at,
            wga.expires_at
        FROM wallet_group_assignments wga
        JOIN groups pg ON wga.group_id = pg.id
        WHERE wga.is_active = true
          AND wga.expires_at IS NOT NULL
          AND wga.expires_at BETWEEN NOW() AND NOW() + ($1 || ' days')::interval
        ORDER BY wga.expires_at ASC
        "#
    )
    .bind::<diesel::sql_types::BigInt, _>(days)
    .load::<ExpiringRow>(&mut conn)
    .await
    {
        Ok(rows) => rows,
        Err(e) => {
            tracing::error!("Failed to fetch expiring assignments: {}", e);
            return AdminResponse::server_error("Database query failed").into_response();
        }
    };

    let expiring: Vec<serde_json::Value> = rows.into_iter().map(|row| {
        serde_json::json!({
            "id": row.id.to_string(),
            "wallet_address": row.wallet_address,
            "group_id": row.group_id.to_string(),
            "group_name": row.group_name,
            "assigned_at": row.assigned_at,
            "expires_at": row.expires_at,
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

    let mut conn = match app_state.db_pool.get().await {
        Ok(conn) => conn,
        Err(e) => {
            tracing::error!("Failed to get database connection: {}", e);
            return AdminResponse::server_error("Database connection failed").into_response();
        }
    };

    #[derive(QueryableByName)]
    struct HistoryRow {
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        id: Uuid,
        #[diesel(sql_type = diesel::sql_types::Text)]
        wallet_address: String,
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        group_id: Uuid,
        #[diesel(sql_type = diesel::sql_types::Text)]
        group_name: String,
        #[diesel(sql_type = diesel::sql_types::Text)]
        group_type: String,
        #[diesel(sql_type = diesel::sql_types::Timestamptz)]
        assigned_at: DateTime<Utc>,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>)]
        expires_at: Option<DateTime<Utc>>,
        #[diesel(sql_type = diesel::sql_types::Bool)]
        is_active: bool,
        #[diesel(sql_type = diesel::sql_types::Text)]
        assignment_source: String,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
        assignment_reason: Option<String>,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
        assigned_by: Option<String>,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
        payment_reference: Option<String>,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
        subscription_id: Option<String>,
        #[diesel(sql_type = diesel::sql_types::Bool)]
        auto_renew: bool,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>)]
        next_billing_date: Option<DateTime<Utc>>,
        #[diesel(sql_type = diesel::sql_types::Jsonb)]
        assignment_metadata: serde_json::Value,
    }

    let rows = match diesel::sql_query(
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
        JOIN groups pg ON wga.group_id = pg.id
        WHERE wga.wallet_address = $1
        ORDER BY wga.assigned_at DESC
        "#
    )
    .bind::<diesel::sql_types::Text, _>(&wallet)
    .load::<HistoryRow>(&mut conn)
    .await
    {
        Ok(rows) => rows,
        Err(e) => {
            tracing::error!("Failed to fetch assignment history: {}", e);
            return AdminResponse::server_error("Database query failed").into_response();
        }
    };

    let history: Vec<AssignmentResponse> = rows.into_iter().map(|row| {
        AssignmentResponse {
            id: row.id.to_string(),
            wallet_address: row.wallet_address,
            group_id: row.group_id.to_string(),
            group_name: row.group_name,
            group_type: row.group_type,
            assigned_at: row.assigned_at,
            expires_at: row.expires_at,
            is_active: row.is_active,
            assignment_source: row.assignment_source,
            assignment_reason: row.assignment_reason,
            assigned_by: row.assigned_by,
            payment_reference: row.payment_reference,
            subscription_id: row.subscription_id,
            auto_renew: row.auto_renew,
            next_billing_date: row.next_billing_date,
            assignment_metadata: row.assignment_metadata,
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

    let mut conn = match app_state.db_pool.get().await {
        Ok(conn) => conn,
        Err(e) => {
            tracing::error!("Failed to get database connection: {}", e);
            return AdminResponse::server_error("Database connection failed").into_response();
        }
    };

    #[derive(QueryableByName)]
    struct GroupRow {
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        id: Uuid,
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        group_id: Uuid,
        #[diesel(sql_type = diesel::sql_types::Text)]
        group_name: String,
        #[diesel(sql_type = diesel::sql_types::Text)]
        group_slug: String,
        #[diesel(sql_type = diesel::sql_types::Text)]
        group_type: String,
        #[diesel(sql_type = diesel::sql_types::Timestamptz)]
        assigned_at: DateTime<Utc>,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>)]
        expires_at: Option<DateTime<Utc>>,
        #[diesel(sql_type = diesel::sql_types::Bool)]
        is_active: bool,
    }

    let rows = match diesel::sql_query(
        r#"
        SELECT
            wga.id, wga.group_id, wga.assigned_at, wga.expires_at, wga.is_active,
            pg.name as group_name, pg.slug as group_slug, pg.group_type
        FROM wallet_group_assignments wga
        JOIN groups pg ON wga.group_id = pg.id
        WHERE wga.wallet_address = $1 AND wga.is_active = true
        ORDER BY wga.assigned_at DESC
        "#
    )
    .bind::<diesel::sql_types::Text, _>(&wallet)
    .load::<GroupRow>(&mut conn)
    .await
    {
        Ok(rows) => rows,
        Err(e) => {
            tracing::error!("Failed to fetch wallet groups: {}", e);
            return AdminResponse::server_error("Database query failed").into_response();
        }
    };

    let groups: Vec<serde_json::Value> = rows.into_iter().map(|row| {
        serde_json::json!({
            "id": row.id.to_string(),
            "group_id": row.group_id.to_string(),
            "group_name": row.group_name,
            "group_slug": row.group_slug,
            "group_type": row.group_type,
            "assigned_at": row.assigned_at,
            "expires_at": row.expires_at,
            "is_active": row.is_active,
        })
    }).collect();

    AdminResponse::success(serde_json::json!({
        "wallet_address": wallet,
        "groups": groups,
        "count": groups.len()
    })).into_response()
}

/// Get group assignment history (audit log)
/// GET /admin/groups/history
pub async fn get_group_history(
    State(app_state): State<AppState>,
    Query(query): Query<GroupHistoryQuery>,
) -> impl IntoResponse {
    let page = query.limit.unwrap_or(20).clamp(1, 100);
    let offset = query.offset.unwrap_or(0);

    let mut conn = match app_state.db_pool.get().await {
        Ok(conn) => conn,
        Err(e) => {
            tracing::error!("Failed to get database connection: {}", e);
            return AdminResponse::server_error("Database connection failed").into_response();
        }
    };

    // Base query
    let mut sql = String::from(
        r#"
        SELECT
            id,
            wallet_address as user_id,
            group_id,
            group_name,
            event_type,
            event_source,
            performed_by,
            performed_by_name,
            reason,
            expires_at,
            metadata,
            event_timestamp as created_at
        FROM permission_audit_log
        WHERE event_type IN ('group_assigned', 'group_removed', 'group_updated', 'expired')
        "#
    );

    let mut where_clauses = Vec::new();

    if let Some(op_type) = &query.operation_type {
        let db_event_type = match op_type.as_str() {
            "assign" => "group_assigned",
            "remove" => "group_removed",
            "expire" => "expired",
            _ => "group_assigned", // Default fallback
        };
        where_clauses.push(format!("event_type = '{}'", db_event_type));
    }

    if let Some(source) = &query.operation_source {
        // Sanitize input to prevent injection
        let clean_source = source.replace("'", ""); 
        where_clauses.push(format!("event_source = '{}'", clean_source));
    }

    if let Some(gid) = &query.group_id {
        if let Ok(uuid) = Uuid::parse_str(gid) {
            where_clauses.push(format!("group_id = '{}'", uuid));
        }
    }

    if let Some(search) = &query.user_search {
        let clean_search = search.replace("'", "");
        where_clauses.push(format!("(wallet_address ILIKE '%{}%' OR performed_by_name ILIKE '%{}%')", clean_search, clean_search));
    }

    if let Some(from) = query.date_from {
        where_clauses.push(format!("event_timestamp >= '{}'", from.to_rfc3339()));
    }

    if let Some(to) = query.date_to {
        where_clauses.push(format!("event_timestamp <= '{}'", to.to_rfc3339()));
    }

    if !where_clauses.is_empty() {
        sql.push_str(" AND ");
        sql.push_str(&where_clauses.join(" AND "));
    }

    sql.push_str(" ORDER BY event_timestamp DESC LIMIT $1 OFFSET $2");

    #[derive(QueryableByName)]
    struct AuditRow {
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        id: Uuid,
        #[diesel(sql_type = diesel::sql_types::Text)]
        user_id: String,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Uuid>)]
        group_id: Option<Uuid>,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
        group_name: Option<String>,
        #[diesel(sql_type = diesel::sql_types::Text)]
        event_type: String,
        #[diesel(sql_type = diesel::sql_types::Text)]
        event_source: String,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
        performed_by: Option<String>,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
        performed_by_name: Option<String>,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
        reason: Option<String>,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>)]
        expires_at: Option<DateTime<Utc>>,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Jsonb>)]
        metadata: Option<serde_json::Value>,
        #[diesel(sql_type = diesel::sql_types::Timestamptz)]
        created_at: DateTime<Utc>,
    }
    
    let result = diesel::sql_query(sql)
        .bind::<diesel::sql_types::Integer, _>(page as i32)
        .bind::<diesel::sql_types::Integer, _>(offset as i32)
        .load::<AuditRow>(&mut conn)
        .await;

    let rows = match result {
        Ok(rows) => rows,
        Err(e) => {
            tracing::error!("Failed to fetch group history: {}", e);
            return AdminResponse::server_error("Database query failed").into_response();
        }
    };

    let history: Vec<GroupHistoryResponse> = rows.into_iter().map(|row| {
        let op_type = match row.event_type.as_str() {
            "group_assigned" => "assign",
            "group_removed" => "remove",
            "expired" => "expire",
             _ => "assign",
        };

        GroupHistoryResponse {
            id: row.id.to_string(),
            user_id: row.user_id,
            user_email: None,
            user_name: None,
            group_id: row.group_id.map(|g| g.to_string()).unwrap_or_default(),
            group_name: row.group_name,
            operation_type: op_type.to_string(),
            operation_source: row.event_source,
            performed_by: row.performed_by,
            performed_by_name: row.performed_by_name,
            reason: row.reason,
            expires_at: row.expires_at,
            metadata: row.metadata,
            created_at: row.created_at,
        }
    }).collect();

    let total = history.len() as u64; 

    AdminResponse::success(serde_json::json!({
        "history": history,
        "total": total
    })).into_response()
}
