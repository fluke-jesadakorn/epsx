//! Admin Payment CRUD Handlers
//!
//! BIG-BANG: migrated to sqlx (real). All diesel DSL replaced with raw SQL.

use axum::{
    extract::{Path, Query, State},
    response::Json,
};
use chrono::Utc;
use sqlx::QueryBuilder;
use std::collections::HashMap;
use tracing::{error, info};
use uuid::Uuid;

use crate::web::{middleware::UnifiedErrorResponse, pagination::Pagination};

use super::types::*;

/// Get all payments with filtering and pagination
pub async fn admin_list_payments_handler(
    State(app_state): State<crate::web::auth::AppState>,
    Query(params): Query<AdminPaymentListParams>,
) -> Result<Json<AdminPaymentListResponse>, Json<UnifiedErrorResponse>> {
    use crate::infrastructure::database::get_payments_pool;
    use crate::infrastructure::models::payment::PaymentDb;

    info!("Admin listing payments with params: {:?}", params);

    let payments_pool = get_payments_pool().await.map_err(|e| {
        error!("Failed to get payments database pool: {}", e);
        Json(UnifiedErrorResponse::new(
            500,
            "Database connection failed",
            "Failed to get payments database pool",
        ))
    })?;

    // Build query with filters using sqlx::QueryBuilder
    let mut qb: QueryBuilder<sqlx::Postgres> = QueryBuilder::new(
        "SELECT id, payment_reference, transaction_hash, wallet_address, amount, currency, \
                method, status, plan_id, contract_address, token_address, block_number, \
                confirmations, created_at, updated_at, expires_at, completed_at, metadata, \
                last_checked_at, error_message, network \
         FROM payments WHERE TRUE",
    );
    if let Some(ref status) = params.status {
        qb.push(" AND status = ").push_bind(status.clone());
    }
    if let Some(ref wallet_addr) = params.wallet_address {
        let pattern = format!("%{}%", wallet_addr);
        qb.push(" AND wallet_address ILIKE ").push_bind(pattern);
    }
    if let Some(ref plan_id) = params.plan_id {
        qb.push(" AND plan_id = ").push_bind(*plan_id);
    }
    if let Some(ref search) = params.search {
        let pattern = format!("%{}%", search);
        qb.push(" AND (payment_reference ILIKE ")
            .push_bind(pattern.clone());
        qb.push(" OR transaction_hash ILIKE ").push_bind(pattern);
        qb.push(")");
    }
    if let Some(ref start_date) = params.start_date {
        if let Ok(parsed) = chrono::NaiveDate::parse_from_str(start_date, "%Y-%m-%d") {
            if let Some(start_dt) = parsed
                .and_hms_opt(0, 0, 0)
                .map(|dt| chrono::DateTime::<Utc>::from_naive_utc_and_offset(dt, Utc))
            {
                qb.push(" AND created_at >= ").push_bind(start_dt);
            }
        }
    }
    if let Some(ref end_date) = params.end_date {
        if let Ok(parsed) = chrono::NaiveDate::parse_from_str(end_date, "%Y-%m-%d") {
            if let Some(end_dt) = parsed
                .and_hms_opt(23, 59, 59)
                .map(|dt| chrono::DateTime::<Utc>::from_naive_utc_and_offset(dt, Utc))
            {
                qb.push(" AND created_at <= ").push_bind(end_dt);
            }
        }
    }
    let pg = Pagination::standard(params.page, params.limit);
    qb.push(" ORDER BY created_at DESC NULLS LAST LIMIT ")
        .push_bind(pg.limit as i64)
        .push(" OFFSET ")
        .push_bind(pg.offset);

    let payment_rows: Vec<PaymentDb> = qb
        .build_query_as()
        .fetch_all(&payments_pool)
        .await
        .map_err(|e| {
            error!("Failed to query payments: {}", e);
            Json(UnifiedErrorResponse::new(
                500,
                "Query failed",
                format!("Failed to load payments: {}", e),
            ))
        })?;

    // Batch fetch plan names to avoid N+1 queries
    let plan_ids: Vec<Uuid> = payment_rows.iter().map(|p| p.plan_id).collect();
    let plans_map: HashMap<Uuid, String> = if plan_ids.is_empty() {
        HashMap::new()
    } else {
        #[derive(sqlx::FromRow)]
        struct PlanNameRow {
            id: Uuid,
            name: String,
        }
        let primary_pool = app_state.db_pool.clone();
        let rows: Vec<PlanNameRow> =
            sqlx::query_as("SELECT id, name FROM plans WHERE id = ANY($1)")
                .bind(&plan_ids)
                .fetch_all(primary_pool.as_ref())
                .await
                .unwrap_or_default();
        rows.into_iter().map(|r| (r.id, r.name)).collect()
    };
    let payments_resp: Vec<AdminPaymentInfo> = payment_rows
        .into_iter()
        .map(|pay_db| {
            let plan_name = plans_map
                .get(&pay_db.plan_id)
                .cloned()
                .unwrap_or_else(|| "Unknown Plan".to_string());
            AdminPaymentInfo::from_db(pay_db, plan_name)
        })
        .collect();

    // Total count via separate query
    let total_row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM payments")
        .fetch_one(&payments_pool)
        .await
        .unwrap_or((0,));
    let total_count = total_row.0;

    let total_pages = pg.total_pages(total_count as u64);
    let pagination = PaginationInfo {
        page: pg.page,
        limit: pg.limit,
        total_count: total_count as u64,
        total_pages,
        has_next: pg.has_next(total_count as u64),
        has_prev: pg.has_prev(),
    };

    // Aggregate summary stats in a single query
    let today_start = Utc::now()
        .date_naive()
        .and_hms_opt(0, 0, 0)
        .map(|dt| chrono::DateTime::<Utc>::from_naive_utc_and_offset(dt, Utc))
        .unwrap_or_else(Utc::now);

    #[derive(sqlx::FromRow)]
    struct PaymentSummaryStats {
        completed_count: i64,
        failed_count: i64,
        pending_count: i64,
        total_amount: Option<bigdecimal::BigDecimal>,
        payments_today: i64,
        revenue_today: Option<bigdecimal::BigDecimal>,
    }

    let stats: PaymentSummaryStats = sqlx::query_as(
        r#"
        SELECT
            COUNT(*) FILTER (WHERE status IN ('completed','confirmed')) as completed_count,
            COUNT(*) FILTER (WHERE status = 'failed') as failed_count,
            COUNT(*) FILTER (WHERE status IN ('pending','created')) as pending_count,
            SUM(amount) FILTER (WHERE status IN ('completed','confirmed')) as total_amount,
            COUNT(*) FILTER (WHERE created_at >= $1) as payments_today,
            SUM(amount) FILTER (WHERE status IN ('completed','confirmed') AND created_at >= $1) as revenue_today
        FROM payments
        "#,
    )
    .bind(today_start)
    .fetch_one(&payments_pool)
    .await
    .unwrap_or(PaymentSummaryStats {
        completed_count: 0,
        failed_count: 0,
        pending_count: 0,
        total_amount: None,
        payments_today: 0,
        revenue_today: None,
    });

    let completed_count = stats.completed_count;
    let failed_count = stats.failed_count;
    let pending_count = stats.pending_count;
    let total_amount_f64 = stats
        .total_amount
        .map(|bd| bd.to_string().parse::<f64>().unwrap_or(0.0))
        .unwrap_or(0.0);
    let payments_today = stats.payments_today;
    let revenue_today_f64 = stats
        .revenue_today
        .map(|bd| bd.to_string().parse::<f64>().unwrap_or(0.0))
        .unwrap_or(0.0);

    let summary = PaymentSummary {
        total_payments: total_count as u64,
        total_amount: total_amount_f64,
        successful_payments: completed_count as u64,
        failed_payments: failed_count as u64,
        pending_payments: pending_count as u64,
        average_payment_amount: if completed_count > 0 {
            total_amount_f64 / completed_count as f64
        } else {
            0.0
        },
        payments_today: payments_today as u64,
        revenue_today: revenue_today_f64,
    };

    info!(
        "Found {} payments (page {} of {})",
        payments_resp.len(),
        pg.page,
        total_pages
    );

    Ok(Json(AdminPaymentListResponse {
        success: true,
        payments: payments_resp,
        pagination,
        summary,
    }))
}

/// Approve a payment
pub async fn admin_approve_payment_handler(
    State(_app_state): State<crate::web::auth::AppState>,
    Path(payment_id): Path<Uuid>,
) -> Result<Json<AdminPaymentActionResponse>, Json<UnifiedErrorResponse>> {
    use crate::infrastructure::database::get_payments_pool;
    let pool = get_payments_pool().await.map_err(|e| {
        error!("Failed to get payments pool: {}", e);
        Json(UnifiedErrorResponse::new(
            500,
            "Database connection failed",
            "Failed to get payments database pool",
        ))
    })?;

    let result = sqlx::query(
        "UPDATE payments SET status = 'confirmed', updated_at = NOW() \
         WHERE id = $1 AND status NOT IN ('cancelled', 'refunded')",
    )
    .bind(payment_id)
    .execute(&pool)
    .await
    .map_err(|e| {
        error!("Failed to approve payment {}: {}", payment_id, e);
        Json(UnifiedErrorResponse::new(
            500,
            "Update failed",
            format!("Failed to approve payment: {}", e),
        ))
    })?;

    Ok(Json(AdminPaymentActionResponse {
        success: true,
        payment_id,
        new_status: "confirmed".to_string(),
        rows_affected: result.rows_affected(),
        message: "Payment confirmed successfully".to_string(),
    }))
}

/// Reject a payment
pub async fn admin_reject_payment_handler(
    State(_app_state): State<crate::web::auth::AppState>,
    Path(payment_id): Path<Uuid>,
) -> Result<Json<AdminPaymentActionResponse>, Json<UnifiedErrorResponse>> {
    use crate::infrastructure::database::get_payments_pool;
    let pool = get_payments_pool().await.map_err(|e| {
        error!("Failed to get payments pool: {}", e);
        Json(UnifiedErrorResponse::new(
            500,
            "Database connection failed",
            "Failed to get payments database pool",
        ))
    })?;

    let result = sqlx::query(
        "UPDATE payments SET status = 'failed', updated_at = NOW() \
         WHERE id = $1 AND status NOT IN ('cancelled', 'refunded')",
    )
    .bind(payment_id)
    .execute(&pool)
    .await
    .map_err(|e| {
        error!("Failed to reject payment {}: {}", payment_id, e);
        Json(UnifiedErrorResponse::new(
            500,
            "Update failed",
            format!("Failed to reject payment: {}", e),
        ))
    })?;

    Ok(Json(AdminPaymentActionResponse {
        success: true,
        payment_id,
        new_status: "failed".to_string(),
        rows_affected: result.rows_affected(),
        message: "Payment rejected successfully".to_string(),
    }))
}

/// Refund a payment
pub async fn admin_refund_payment_handler(
    State(_app_state): State<crate::web::auth::AppState>,
    Path(payment_id): Path<Uuid>,
) -> Result<Json<AdminPaymentActionResponse>, Json<UnifiedErrorResponse>> {
    use crate::infrastructure::database::get_payments_pool;
    let pool = get_payments_pool().await.map_err(|e| {
        error!("Failed to get payments pool: {}", e);
        Json(UnifiedErrorResponse::new(
            500,
            "Database connection failed",
            "Failed to get payments database pool",
        ))
    })?;

    let result = sqlx::query(
        "UPDATE payments SET status = 'refunded', updated_at = NOW(), completed_at = NOW() \
         WHERE id = $1 AND status NOT IN ('cancelled', 'refunded')",
    )
    .bind(payment_id)
    .execute(&pool)
    .await
    .map_err(|e| {
        error!("Failed to refund payment {}: {}", payment_id, e);
        Json(UnifiedErrorResponse::new(
            500,
            "Update failed",
            format!("Failed to refund payment: {}", e),
        ))
    })?;

    Ok(Json(AdminPaymentActionResponse {
        success: true,
        payment_id,
        new_status: "refunded".to_string(),
        rows_affected: result.rows_affected(),
        message: "Payment refunded successfully".to_string(),
    }))
}

/// Get payment details (stub — full impl in cross_pool migration)
pub async fn admin_get_payment_details_handler(
    State(_app_state): State<crate::web::auth::AppState>,
    Path(_payment_id): Path<Uuid>,
) -> Result<
    Json<crate::web::payments::admin_handlers::types::AdminPaymentDetailsResponse>,
    Json<UnifiedErrorResponse>,
> {
    Err(Json(UnifiedErrorResponse::new(
        500,
        "NotImplemented",
        "Payment details endpoint migration in progress",
    )))
}

/// Process refund (stub)
pub async fn admin_process_refund_handler(
    State(_app_state): State<crate::web::auth::AppState>,
    Path(_payment_id): Path<Uuid>,
) -> Result<
    Json<crate::web::payments::admin_handlers::types::AdminPaymentActionResponse>,
    Json<UnifiedErrorResponse>,
> {
    Err(Json(UnifiedErrorResponse::new(
        501,
        "NotImplemented",
        "Process refund endpoint migration in progress",
    )))
}

/// Update payment status (stub)
pub async fn admin_update_payment_status_handler(
    State(_app_state): State<crate::web::auth::AppState>,
    Path(_payment_id): Path<Uuid>,
    _body: Option<axum::extract::Json<serde_json::Value>>,
) -> Result<
    Json<crate::web::payments::admin_handlers::types::AdminPaymentActionResponse>,
    Json<UnifiedErrorResponse>,
> {
    Err(Json(UnifiedErrorResponse::new(
        501,
        "NotImplemented",
        "Update payment status endpoint migration in progress",
    )))
}
