use axum::{
    extract::{Path, Query, State},
    response::IntoResponse,
};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;

use crate::web::auth::AppState;
use crate::web::responses::{AdminResponse, create_pagination};
use super::{AssignmentResponse, ListAssignmentsQuery, ExpiringAssignmentsQuery, PlanHistoryQuery, PlanHistoryResponse};

/// List wallet-plan assignments with pagination
/// GET /admin/permissions/assignments
pub async fn list_assignments(
    State(app_state): State<AppState>,
    Query(query): Query<ListAssignmentsQuery>,
) -> impl IntoResponse {
    let pg = crate::web::pagination::Pagination::standard(query.page, query.limit);

    // Build query with optional filters
    let mut sql = String::from(
        r#"
        SELECT
            wga.id,
            wga.wallet_address,
            wga.plan_id,
            pg.name as plan_name,
            pg.plan_type,
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
        FROM wallet_plan_assignments wga
        JOIN plans pg ON wga.plan_id = pg.id
        "#
    );

    let mut where_clauses = Vec::new();
    let plan_clause;
    let active_clause;

    if query.wallet_address.is_some() {
        where_clauses.push("wga.wallet_address = $3");
    }
    if query.plan_id.is_some() {
        let clause_idx = if query.wallet_address.is_some() { 4 } else { 3 };
        plan_clause = format!("wga.plan_id = ${}", clause_idx);
        where_clauses.push(&plan_clause);
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
        plan_id: Uuid,
        #[diesel(sql_type = diesel::sql_types::Text)]
        plan_name: String,
        #[diesel(sql_type = diesel::sql_types::Text)]
        plan_type: String,
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
        "SELECT COUNT(*)::bigint as count FROM wallet_plan_assignments"
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
        &query.plan_id.as_ref().and_then(|g| Uuid::parse_str(g).ok()),
        query.is_active,
    ) {
        (Some(wallet), Some(plan_uuid), Some(is_active)) => {
            diesel::sql_query(&sql)
                .bind::<diesel::sql_types::Integer, _>(pg.limit as i32)
                .bind::<diesel::sql_types::Integer, _>(pg.offset as i32)
                .bind::<diesel::sql_types::Text, _>(wallet.to_lowercase())
                .bind::<diesel::sql_types::Uuid, _>(*plan_uuid)
                .bind::<diesel::sql_types::Bool, _>(is_active)
                .load::<AssignmentRow>(&mut conn)
                .await
        }
        (Some(wallet), Some(plan_uuid), None) => {
            diesel::sql_query(&sql)
                .bind::<diesel::sql_types::Integer, _>(pg.limit as i32)
                .bind::<diesel::sql_types::Integer, _>(pg.offset as i32)
                .bind::<diesel::sql_types::Text, _>(wallet.to_lowercase())
                .bind::<diesel::sql_types::Uuid, _>(*plan_uuid)
                .load::<AssignmentRow>(&mut conn)
                .await
        }
        (Some(wallet), None, Some(is_active)) => {
            diesel::sql_query(&sql)
                .bind::<diesel::sql_types::Integer, _>(pg.limit as i32)
                .bind::<diesel::sql_types::Integer, _>(pg.offset as i32)
                .bind::<diesel::sql_types::Text, _>(wallet.to_lowercase())
                .bind::<diesel::sql_types::Bool, _>(is_active)
                .load::<AssignmentRow>(&mut conn)
                .await
        }
        (Some(wallet), None, None) => {
            diesel::sql_query(&sql)
                .bind::<diesel::sql_types::Integer, _>(pg.limit as i32)
                .bind::<diesel::sql_types::Integer, _>(pg.offset as i32)
                .bind::<diesel::sql_types::Text, _>(wallet.to_lowercase())
                .load::<AssignmentRow>(&mut conn)
                .await
        }
        (None, Some(plan_uuid), Some(is_active)) => {
            diesel::sql_query(&sql)
                .bind::<diesel::sql_types::Integer, _>(pg.limit as i32)
                .bind::<diesel::sql_types::Integer, _>(pg.offset as i32)
                .bind::<diesel::sql_types::Uuid, _>(*plan_uuid)
                .bind::<diesel::sql_types::Bool, _>(is_active)
                .load::<AssignmentRow>(&mut conn)
                .await
        }
        (None, Some(plan_uuid), None) => {
            diesel::sql_query(&sql)
                .bind::<diesel::sql_types::Integer, _>(pg.limit as i32)
                .bind::<diesel::sql_types::Integer, _>(pg.offset as i32)
                .bind::<diesel::sql_types::Uuid, _>(*plan_uuid)
                .load::<AssignmentRow>(&mut conn)
                .await
        }
        (None, None, Some(is_active)) => {
            diesel::sql_query(&sql)
                .bind::<diesel::sql_types::Integer, _>(pg.limit as i32)
                .bind::<diesel::sql_types::Integer, _>(pg.offset as i32)
                .bind::<diesel::sql_types::Bool, _>(is_active)
                .load::<AssignmentRow>(&mut conn)
                .await
        }
        (None, None, None) => {
            diesel::sql_query(&sql)
                .bind::<diesel::sql_types::Integer, _>(pg.limit as i32)
                .bind::<diesel::sql_types::Integer, _>(pg.offset as i32)
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
            plan_id: row.plan_id.to_string(),
            plan_name: row.plan_name,
            plan_type: row.plan_type,
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

    let pagination = create_pagination(pg.page, pg.limit, total as u64);
    AdminResponse::success_with_pagination(assignments, pagination).into_response()
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
        plan_id: Uuid,
        #[diesel(sql_type = diesel::sql_types::Text)]
        plan_name: String,
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
            wga.plan_id,
            pg.name as plan_name,
            wga.assigned_at,
            wga.expires_at
        FROM wallet_plan_assignments wga
        JOIN plans pg ON wga.plan_id = pg.id
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
            "plan_id": row.plan_id.to_string(),
            "plan_name": row.plan_name,
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
        plan_id: Uuid,
        #[diesel(sql_type = diesel::sql_types::Text)]
        plan_name: String,
        #[diesel(sql_type = diesel::sql_types::Text)]
        plan_type: String,
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
            wga.plan_id,
            pg.name as plan_name,
            pg.plan_type,
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
        FROM wallet_plan_assignments wga
        JOIN plans pg ON wga.plan_id = pg.id
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
            plan_id: row.plan_id.to_string(),
            plan_name: row.plan_name,
            plan_type: row.plan_type,
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

/// Get plans assigned to a wallet
/// GET /admin/permissions/wallets/:wallet/plans
pub async fn get_wallet_plans(
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
    struct PlanRow {
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        id: Uuid,
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        plan_id: Uuid,
        #[diesel(sql_type = diesel::sql_types::Text)]
        plan_name: String,
        #[diesel(sql_type = diesel::sql_types::Text)]
        plan_slug: String,
        #[diesel(sql_type = diesel::sql_types::Text)]
        plan_type: String,
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
            wga.id, wga.plan_id, wga.assigned_at, wga.expires_at, wga.is_active,
            pg.name as plan_name, pg.slug as plan_slug, pg.plan_type
        FROM wallet_plan_assignments wga
        JOIN plans pg ON wga.plan_id = pg.id
        WHERE wga.wallet_address = $1 AND wga.is_active = true
        ORDER BY wga.assigned_at DESC
        "#
    )
    .bind::<diesel::sql_types::Text, _>(&wallet)
    .load::<PlanRow>(&mut conn)
    .await
    {
        Ok(rows) => rows,
        Err(e) => {
            tracing::error!("Failed to fetch wallet plans: {}", e);
            return AdminResponse::server_error("Database query failed").into_response();
        }
    };

    let plans: Vec<serde_json::Value> = rows.into_iter().map(|row| {
        serde_json::json!({
            "id": row.id.to_string(),
            "plan_id": row.plan_id.to_string(),
            "plan_name": row.plan_name,
            "plan_slug": row.plan_slug,
            "plan_type": row.plan_type,
            "assigned_at": row.assigned_at,
            "expires_at": row.expires_at,
            "is_active": row.is_active,
        })
    }).collect();

    AdminResponse::success(serde_json::json!({
        "wallet_address": wallet,
        "plans": plans,
        "count": plans.len()
    })).into_response()
}

/// Get plan assignment history (audit log)
/// GET /admin/plans/history
pub async fn get_plan_history(
    State(app_state): State<AppState>,
    Query(query): Query<PlanHistoryQuery>,
) -> impl IntoResponse {
    let page = query.limit.unwrap_or(20).clamp(1, 100);
    let offset = query.offset.unwrap_or(0);

    // Use analytics pool if available, otherwise fallback to primary pool (for dev)
    let pool = app_state.analytics_db_pool.as_ref().unwrap_or(&app_state.db_pool);

    let mut conn = match pool.get().await {
        Ok(conn) => conn,
        Err(e) => {
            tracing::error!("Failed to get database connection: {}", e);
            return AdminResponse::server_error("Database connection failed").into_response();
        }
    };

    // Whitelist operation_type
    let event_type_filter: Option<String> = query.operation_type.as_ref().map(|op_type| {
        match op_type.as_str() {
            "assign" => "plan_assigned",
            "remove" => "plan_removed",
            "expire" => "expired",
            _ => "plan_assigned",
        }.to_string()
    });

    let plan_uuid: Option<Uuid> = query.plan_id.as_ref().and_then(|gid| Uuid::parse_str(gid).ok());
    let search_pattern: Option<String> = query.user_search.as_ref().map(|s| format!("%{}%", s));

    // Use fixed param slots with NULL-check pattern to avoid dynamic bind chains
    // $1=limit, $2=offset, $3=event_type, $4=source, $5=plan_id, $6=search, $7=date_from, $8=date_to
    let safe_sql = r#"
        SELECT
            id,
            wallet_address as user_id,
            plan_id,
            plan_name,
            event_type,
            event_source,
            performed_by,
            performed_by_name,
            reason,
            expires_at,
            metadata,
            event_timestamp as created_at
        FROM permission_audit_log
        WHERE event_type IN ('plan_assigned', 'plan_removed', 'plan_updated', 'expired')
          AND ($3::text IS NULL OR event_type = $3)
          AND ($4::text IS NULL OR event_source = $4)
          AND ($5::uuid IS NULL OR plan_id = $5)
          AND ($6::text IS NULL OR (wallet_address ILIKE $6 OR performed_by_name ILIKE $6))
          AND ($7::timestamptz IS NULL OR event_timestamp >= $7)
          AND ($8::timestamptz IS NULL OR event_timestamp <= $8)
        ORDER BY event_timestamp DESC
        LIMIT $1 OFFSET $2
    "#;

    #[derive(QueryableByName)]
    struct AuditRow {
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        id: Uuid,
        #[diesel(sql_type = diesel::sql_types::Text)]
        user_id: String,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Uuid>)]
        plan_id: Option<Uuid>,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
        plan_name: Option<String>,
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

    let date_from_str: Option<String> = query.date_from.map(|d| d.to_rfc3339());
    let date_to_str: Option<String> = query.date_to.map(|d| d.to_rfc3339());

    let result = diesel::sql_query(safe_sql)
        .bind::<diesel::sql_types::Integer, _>(page as i32)
        .bind::<diesel::sql_types::Integer, _>(offset as i32)
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(&event_type_filter)
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(&query.operation_source)
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Uuid>, _>(&plan_uuid)
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(&search_pattern)
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(&date_from_str)
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(&date_to_str)
        .load::<AuditRow>(&mut conn)
        .await;

    let rows = match result {
        Ok(rows) => rows,
        Err(e) => {
            tracing::error!("Failed to fetch plan history: {}", e);
            return AdminResponse::server_error("Database query failed").into_response();
        }
    };

    let history: Vec<PlanHistoryResponse> = rows.into_iter().map(|row| {
        let op_type = match row.event_type.as_str() {
            "plan_assigned" => "assign",
            "plan_removed" => "remove",
            "expired" => "expire",
             _ => "assign",
        };

        PlanHistoryResponse {
            id: row.id.to_string(),
            user_id: row.user_id,
            user_email: None,
            user_name: None,
            plan_id: row.plan_id.map(|g| g.to_string()).unwrap_or_default(),
            plan_name: row.plan_name,
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
