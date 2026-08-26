use axum::{
    extract::{Path, Query, State},
    response::IntoResponse,
};
use chrono::{DateTime, Utc};
use sqlx::QueryBuilder;
use uuid::Uuid;

use super::{
    AssignmentResponse, ExpiringAssignmentsQuery, ListAssignmentsQuery, PlanHistoryQuery,
    PlanHistoryResponse,
};
use crate::web::auth::AppState;
use crate::web::responses::{create_pagination, AdminResponse};

/// List wallet-plan assignments with pagination
/// GET /admin/permissions/assignments
pub async fn list_assignments(
    State(app_state): State<AppState>,
    Query(query): Query<ListAssignmentsQuery>,
) -> impl IntoResponse {
    let pg = crate::web::pagination::Pagination::standard(query.page, query.limit);

    #[derive(sqlx::FromRow)]
    struct CountRow {
        count: i64,
    }

    #[derive(sqlx::FromRow)]
    struct AssignmentRow {
        id: Uuid,
        wallet_address: String,
        plan_id: Uuid,
        plan_name: String,
        plan_type: String,
        assigned_at: DateTime<Utc>,
        expires_at: Option<DateTime<Utc>>,
        is_active: bool,
        assignment_source: String,
        assignment_reason: Option<String>,
        assigned_by: Option<String>,
        payment_reference: Option<String>,
        subscription_id: Option<String>,
        auto_renew: bool,
        next_billing_date: Option<DateTime<Utc>>,
        assignment_metadata: serde_json::Value,
    }

    // Get total count
    let total: i64 = match sqlx::query_as::<_, CountRow>(
        "SELECT COUNT(*)::bigint as count FROM wallet_plan_assignments",
    )
    .fetch_one(app_state.db_pool.as_ref())
    .await
    {
        Ok(row) => row.count,
        Err(_) => 0,
    };

    let mut qb: QueryBuilder<sqlx::Postgres> = QueryBuilder::new(
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
        WHERE TRUE
        "#,
    );

    if let Some(ref wallet) = query.wallet_address {
        qb.push(" AND wga.wallet_address = ")
            .push_bind(wallet.to_lowercase());
    }

    if let Some(ref plan_id_str) = query.plan_id {
        if let Ok(plan_uuid) = Uuid::parse_str(plan_id_str) {
            qb.push(" AND wga.plan_id = ").push_bind(plan_uuid);
        }
    }

    if let Some(is_active) = query.is_active {
        qb.push(" AND wga.is_active = ").push_bind(is_active);
    }

    qb.push(" ORDER BY wga.assigned_at DESC LIMIT ")
        .push_bind(pg.limit as i64)
        .push(" OFFSET ")
        .push_bind(pg.offset);

    let rows: Vec<AssignmentRow> = match qb
        .build_query_as::<AssignmentRow>()
        .fetch_all(app_state.db_pool.as_ref())
        .await
    {
        Ok(rows) => rows,
        Err(e) => {
            tracing::error!("Failed to list assignments: {}", e);
            return AdminResponse::server_error("Database query failed").into_response();
        }
    };

    let assignments: Vec<AssignmentResponse> = rows
        .into_iter()
        .map(|row| AssignmentResponse {
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
        })
        .collect();

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

    #[derive(sqlx::FromRow)]
    struct ExpiringRow {
        id: Uuid,
        wallet_address: String,
        plan_id: Uuid,
        plan_name: String,
        assigned_at: DateTime<Utc>,
        expires_at: DateTime<Utc>,
    }

    let rows = match sqlx::query_as::<_, ExpiringRow>(
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
        "#,
    )
    .bind(days)
    .fetch_all(app_state.db_pool.as_ref())
    .await
    {
        Ok(rows) => rows,
        Err(e) => {
            tracing::error!("Failed to fetch expiring assignments: {}", e);
            return AdminResponse::server_error("Database query failed").into_response();
        }
    };

    let expiring: Vec<serde_json::Value> = rows
        .into_iter()
        .map(|row| {
            serde_json::json!({
                "id": row.id.to_string(),
                "wallet_address": row.wallet_address,
                "plan_id": row.plan_id.to_string(),
                "plan_name": row.plan_name,
                "assigned_at": row.assigned_at,
                "expires_at": row.expires_at,
            })
        })
        .collect();

    AdminResponse::success(serde_json::json!({
        "assignments": expiring,
        "count": expiring.len(),
        "days": days
    }))
    .into_response()
}

/// Get assignment history for a wallet
/// GET /admin/permissions/assignments/history/:wallet
pub async fn get_assignment_history(
    State(app_state): State<AppState>,
    Path(wallet): Path<String>,
) -> impl IntoResponse {
    let wallet = wallet.to_lowercase();

    #[derive(sqlx::FromRow)]
    struct HistoryRow {
        id: Uuid,
        wallet_address: String,
        plan_id: Uuid,
        plan_name: String,
        plan_type: String,
        assigned_at: DateTime<Utc>,
        expires_at: Option<DateTime<Utc>>,
        is_active: bool,
        assignment_source: String,
        assignment_reason: Option<String>,
        assigned_by: Option<String>,
        payment_reference: Option<String>,
        subscription_id: Option<String>,
        auto_renew: bool,
        next_billing_date: Option<DateTime<Utc>>,
        assignment_metadata: serde_json::Value,
    }

    let rows = match sqlx::query_as::<_, HistoryRow>(
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
        "#,
    )
    .bind(&wallet)
    .fetch_all(app_state.db_pool.as_ref())
    .await
    {
        Ok(rows) => rows,
        Err(e) => {
            tracing::error!("Failed to fetch assignment history: {}", e);
            return AdminResponse::server_error("Database query failed").into_response();
        }
    };

    let history: Vec<AssignmentResponse> = rows
        .into_iter()
        .map(|row| AssignmentResponse {
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
        })
        .collect();

    AdminResponse::success(serde_json::json!({
        "wallet_address": wallet,
        "assignments": history,
        "count": history.len()
    }))
    .into_response()
}

/// Get plans assigned to a wallet
/// GET /admin/permissions/wallets/:wallet/plans
pub async fn get_wallet_plans(
    State(app_state): State<AppState>,
    Path(wallet): Path<String>,
) -> impl IntoResponse {
    let wallet = wallet.to_lowercase();

    #[derive(sqlx::FromRow)]
    struct PlanRow {
        id: Uuid,
        plan_id: Uuid,
        plan_name: String,
        plan_slug: String,
        plan_type: String,
        assigned_at: DateTime<Utc>,
        expires_at: Option<DateTime<Utc>>,
        is_active: bool,
    }

    let rows = match sqlx::query_as::<_, PlanRow>(
        r#"
        SELECT
            wga.id, wga.plan_id, wga.assigned_at, wga.expires_at, wga.is_active,
            pg.name as plan_name, pg.slug as plan_slug, pg.plan_type
        FROM wallet_plan_assignments wga
        JOIN plans pg ON wga.plan_id = pg.id
        WHERE wga.wallet_address = $1 AND wga.is_active = true
        ORDER BY wga.assigned_at DESC
        "#,
    )
    .bind(&wallet)
    .fetch_all(app_state.db_pool.as_ref())
    .await
    {
        Ok(rows) => rows,
        Err(e) => {
            tracing::error!("Failed to fetch wallet plans: {}", e);
            return AdminResponse::server_error("Database query failed").into_response();
        }
    };

    let plans: Vec<serde_json::Value> = rows
        .into_iter()
        .map(|row| {
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
        })
        .collect();

    AdminResponse::success(serde_json::json!({
        "wallet_address": wallet,
        "plans": plans,
        "count": plans.len()
    }))
    .into_response()
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
    let pool = app_state
        .analytics_db_pool
        .as_ref()
        .unwrap_or(&app_state.db_pool);

    // Whitelist operation_type
    let event_type_filter: Option<String> = query.operation_type.as_ref().map(|op_type| {
        match op_type.as_str() {
            "assign" => "plan_assigned",
            "remove" => "plan_removed",
            "expire" => "expired",
            _ => "plan_assigned",
        }
        .to_string()
    });

    let plan_uuid: Option<Uuid> = query
        .plan_id
        .as_ref()
        .and_then(|gid| Uuid::parse_str(gid).ok());
    let search_pattern: Option<String> = query.user_search.as_ref().map(|s| format!("%{}%", s));

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

    #[derive(sqlx::FromRow)]
    struct AuditRow {
        id: Uuid,
        user_id: String,
        plan_id: Option<Uuid>,
        plan_name: Option<String>,
        event_type: String,
        event_source: String,
        performed_by: Option<String>,
        performed_by_name: Option<String>,
        reason: Option<String>,
        expires_at: Option<DateTime<Utc>>,
        metadata: Option<serde_json::Value>,
        created_at: DateTime<Utc>,
    }

    let date_from: Option<DateTime<Utc>> = query.date_from;
    let date_to: Option<DateTime<Utc>> = query.date_to;

    let result = sqlx::query_as::<_, AuditRow>(safe_sql)
        .bind(page as i64)
        .bind(offset as i64)
        .bind(&event_type_filter)
        .bind(&query.operation_source)
        .bind(plan_uuid)
        .bind(&search_pattern)
        .bind(date_from)
        .bind(date_to)
        .fetch_all(pool.as_ref())
        .await;

    let rows = match result {
        Ok(rows) => rows,
        Err(e) => {
            tracing::error!("Failed to fetch plan history: {}", e);
            return AdminResponse::server_error("Database query failed").into_response();
        }
    };

    let history: Vec<PlanHistoryResponse> = rows
        .into_iter()
        .map(|row| {
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
        })
        .collect();

    let total = history.len() as u64;

    AdminResponse::success(serde_json::json!({
        "history": history,
        "total": total
    }))
    .into_response()
}
