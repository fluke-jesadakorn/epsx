//! Admin Payment CRUD Handlers

use axum::{
    extract::{State, Query, Path},
    response::Json,
};
use uuid::Uuid;
use chrono::Utc;
use tracing::{info, error};

use crate::{
    web::{
        middleware::UnifiedErrorResponse,
        pagination::Pagination,
    },
    schemas::primary::plans,
};

use super::types::*;

/// Get all payments with filtering and pagination
pub async fn admin_list_payments_handler(
    State(app_state): State<crate::web::auth::AppState>,
    Query(params): Query<AdminPaymentListParams>,
) -> Result<Json<AdminPaymentListResponse>, Json<UnifiedErrorResponse>> {
    use diesel::prelude::*;
    use diesel_async::RunQueryDsl;
    use crate::infrastructure::models::payment::PaymentDb;
    use crate::infrastructure::database::get_payments_pool;
    use crate::schemas::payments::payments;

    info!("Admin listing payments with params: {:?}", params);

    // Get PAYMENTS database connection
    let payments_pool = get_payments_pool().await.map_err(|e| {
        error!("Failed to get payments database pool: {}", e);
        Json(UnifiedErrorResponse::new(500, "Database connection failed", "Failed to get payments database pool"))
    })?;
    let mut payments_conn = payments_pool.get().await
        .map_err(|e| {
            error!("Failed to get payments database connection: {}", e);
            Json(UnifiedErrorResponse::new(500, "Database connection failed", "Failed to establish payments database connection"))
        })?;

    // Get PRIMARY database connection for plan name lookups
    let mut primary_conn = app_state.db_pool.get().await
        .map_err(|e| {
            error!("Failed to get primary database connection: {}", e);
            Json(UnifiedErrorResponse::new(500, "Database connection failed", "Failed to establish primary database connection"))
        })?;

    // Build query with filters
    let mut query = payments::table.into_boxed();

    // Apply status filter
    if let Some(ref status) = params.status {
        query = query.filter(payments::status.eq(status));
    }

    // Apply wallet_address filter
    if let Some(ref wallet_addr) = params.wallet_address {
        query = query.filter(payments::wallet_address.ilike(format!("%{}%", wallet_addr)));
    }

    // Apply plan_id filter
    if let Some(ref plan_id) = params.plan_id {
        query = query.filter(payments::plan_id.eq(plan_id));
    }

    // Apply search filter (transaction hash or reference)
    if let Some(ref search) = params.search {
        query = query.filter(
            payments::payment_reference.ilike(format!("%{}%", search))
                .or(payments::transaction_hash.ilike(format!("%{}%", search)))
        );
    }

    // Apply date range filters
    if let Some(ref start_date) = params.start_date {
        if let Ok(parsed) = chrono::NaiveDate::parse_from_str(start_date, "%Y-%m-%d") {
            let start_datetime = parsed.and_hms_opt(0, 0, 0)
                .map(|dt| chrono::DateTime::<Utc>::from_naive_utc_and_offset(dt, Utc));
            if let Some(start_dt) = start_datetime {
                query = query.filter(payments::created_at.ge(start_dt));
            }
        }
    }

    if let Some(ref end_date) = params.end_date {
        if let Ok(parsed) = chrono::NaiveDate::parse_from_str(end_date, "%Y-%m-%d") {
            let end_datetime = parsed.and_hms_opt(23, 59, 59)
                .map(|dt| chrono::DateTime::<Utc>::from_naive_utc_and_offset(dt, Utc));
            if let Some(end_dt) = end_datetime {
                query = query.filter(payments::created_at.le(end_dt));
            }
        }
    }

    // Apply pagination
    let pg = Pagination::standard(params.page, params.limit);

    // Get total count (before pagination)
    let total_count: i64 = payments::table
        .count()
        .get_result(&mut payments_conn)
        .await
        .unwrap_or(0);

    // Fetch payments with pagination
    let payment_rows = query
        .order(payments::created_at.desc().nulls_last())
        .limit(pg.limit as i64)
        .offset(pg.offset)
        .load::<PaymentDb>(&mut payments_conn)
        .await
        .map_err(|e| {
            error!("Failed to query payments: {}", e);
            Json(UnifiedErrorResponse::new(500, "Query failed", format!("Failed to load payments: {}", e)))
        })?;

    // Batch fetch plan names to avoid N+1 queries
    let plan_ids: Vec<Uuid> = payment_rows.iter().map(|p| p.plan_id).collect();
    let plans_map: std::collections::HashMap<Uuid, String> = if plan_ids.is_empty() {
        std::collections::HashMap::new()
    } else {
        plans::table
            .filter(plans::id.eq_any(&plan_ids))
            .select((plans::id, plans::name))
            .load::<(Uuid, String)>(&mut primary_conn)
            .await
            .unwrap_or_default()
            .into_iter()
            .collect()
    };
    let payments_resp: Vec<AdminPaymentInfo> = payment_rows.into_iter().map(|pay_db| {
        let plan_name = plans_map.get(&pay_db.plan_id)
            .cloned()
            .unwrap_or_else(|| "Unknown Plan".to_string());
        AdminPaymentInfo::from_db(pay_db, plan_name)
    }).collect();

    let total_pages = pg.total_pages(total_count as u64);
    let pagination = PaginationInfo {
        page: pg.page,
        limit: pg.limit,
        total_count: total_count as u64,
        total_pages,
        has_next: pg.has_next(total_count as u64),
        has_prev: pg.has_prev(),
    };

    // Calculate all summary statistics in a single query using conditional aggregation
    let today_start = Utc::now().date_naive().and_hms_opt(0, 0, 0)
        .map(|dt| chrono::DateTime::<Utc>::from_naive_utc_and_offset(dt, Utc))
        .unwrap_or_else(Utc::now);

    #[derive(diesel::QueryableByName)]
    struct PaymentSummaryStats {
        #[diesel(sql_type = diesel::sql_types::BigInt)]
        completed_count: i64,
        #[diesel(sql_type = diesel::sql_types::BigInt)]
        failed_count: i64,
        #[diesel(sql_type = diesel::sql_types::BigInt)]
        pending_count: i64,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Numeric>)]
        total_amount: Option<bigdecimal::BigDecimal>,
        #[diesel(sql_type = diesel::sql_types::BigInt)]
        payments_today: i64,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Numeric>)]
        revenue_today: Option<bigdecimal::BigDecimal>,
    }

    let stats = diesel::sql_query(
        r#"
        SELECT
            COUNT(*) FILTER (WHERE status IN ('completed','confirmed')) as completed_count,
            COUNT(*) FILTER (WHERE status = 'failed') as failed_count,
            COUNT(*) FILTER (WHERE status IN ('pending','created')) as pending_count,
            SUM(amount) FILTER (WHERE status IN ('completed','confirmed')) as total_amount,
            COUNT(*) FILTER (WHERE created_at >= $1) as payments_today,
            SUM(amount) FILTER (WHERE status IN ('completed','confirmed') AND created_at >= $1) as revenue_today
        FROM payments
        "#
    )
    .bind::<diesel::sql_types::Timestamptz, _>(today_start)
    .get_result::<PaymentSummaryStats>(&mut payments_conn)
    .await
    .unwrap_or(PaymentSummaryStats {
        completed_count: 0, failed_count: 0, pending_count: 0,
        total_amount: None, payments_today: 0, revenue_today: None,
    });

    let completed_count = stats.completed_count;
    let failed_count = stats.failed_count;
    let pending_count = stats.pending_count;
    let total_amount_f64 = stats.total_amount
        .map(|bd| bd.to_string().parse::<f64>().unwrap_or(0.0))
        .unwrap_or(0.0);
    let payments_today = stats.payments_today;
    let revenue_today_f64 = stats.revenue_today
        .map(|bd| bd.to_string().parse::<f64>().unwrap_or(0.0))
        .unwrap_or(0.0);

    let summary = PaymentSummary {
        total_payments: total_count as u64,
        total_amount: total_amount_f64,
        successful_payments: completed_count as u64,
        failed_payments: failed_count as u64,
        pending_payments: pending_count as u64,
        average_payment_amount: if completed_count > 0 { total_amount_f64 / completed_count as f64 } else { 0.0 },
        payments_today: payments_today as u64,
        revenue_today: revenue_today_f64,
    };

    info!("Found {} payments (page {} of {})", payments_resp.len(), pg.page, total_pages);

    Ok(Json(AdminPaymentListResponse {
        success: true,
        payments: payments_resp,
        pagination,
        summary,
    }))
}

/// Get payment details with audit log
pub async fn admin_get_payment_details_handler(
    State(app_state): State<crate::web::auth::AppState>,
    Path(payment_id): Path<Uuid>,
) -> Result<Json<AdminPaymentDetailsResponse>, Json<UnifiedErrorResponse>> {
    use diesel::prelude::*;
    use diesel_async::RunQueryDsl;
    use crate::infrastructure::models::payment::PaymentDb;
    use crate::infrastructure::database::get_payments_pool;
    use crate::schemas::payments::{payments, payment_audit_log};

    info!("Admin getting payment details for {}", payment_id);

    // Get PAYMENTS database connection
    let payments_pool = get_payments_pool().await.map_err(|e| {
        error!("Failed to get payments database pool: {}", e);
        Json(UnifiedErrorResponse::new(500, "Database connection failed", "Failed to get payments database pool"))
    })?;
    let mut payments_conn = payments_pool.get().await
        .map_err(|e| {
            error!("Failed to get payments database connection: {}", e);
            Json(UnifiedErrorResponse::new(500, "Database connection failed", "Failed to establish payments database connection"))
        })?;

    // Get PRIMARY database connection for plan name lookup
    let mut primary_conn = app_state.db_pool.get().await
        .map_err(|e| {
            error!("Failed to get primary database connection: {}", e);
            Json(UnifiedErrorResponse::new(500, "Database connection failed", "Failed to establish primary database connection"))
        })?;

    // Fetch payment from database
    let payment_result = payments::table
        .filter(payments::id.eq(payment_id))
        .first::<PaymentDb>(&mut payments_conn)
        .await;

    let payment = match payment_result {
        Ok(pay_db) => {
            // Try to get plan name from plans table (PRIMARY DB)
            let plan_name = plans::table
                .filter(plans::id.eq(pay_db.plan_id))
                .select(plans::name)
                .first::<String>(&mut primary_conn)
                .await
                .unwrap_or_else(|_| "Unknown Plan".to_string());

            Some(AdminPaymentInfo::from_db(pay_db, plan_name))
        }
        Err(diesel::NotFound) => {
            return Err(Json(UnifiedErrorResponse::new(404, "Payment not found", format!("No payment found with ID: {}", payment_id))));
        }
        Err(e) => {
            error!("Failed to query payment: {}", e);
            return Err(Json(UnifiedErrorResponse::new(500, "Query failed", format!("Failed to load payment: {}", e))));
        }
    };

    // Fetch audit logs for this payment
    let audit_log_rows = payment_audit_log::table
        .filter(payment_audit_log::payment_id.eq(payment_id))
        .order(payment_audit_log::created_at.desc().nulls_last())
        .select((
            payment_audit_log::id,
            payment_audit_log::payment_id,
            payment_audit_log::action,
            payment_audit_log::old_status,
            payment_audit_log::new_status,
            payment_audit_log::reason,
            payment_audit_log::performed_by,
            payment_audit_log::created_at,
            payment_audit_log::metadata,
        ))
        .load::<(Uuid, Uuid, String, Option<String>, Option<String>, Option<String>, Option<String>, chrono::DateTime<Utc>, Option<serde_json::Value>)>(&mut payments_conn)
        .await
        .unwrap_or_default();

    let audit_logs: Vec<PaymentAuditLog> = audit_log_rows.into_iter().map(|(id, _payment_id, action, old_status, new_status, reason, performed_by, created_at, metadata)| {
        PaymentAuditLog {
            id,
            action,
            old_status,
            new_status,
            reason,
            performed_by,
            created_at,
            metadata: metadata.unwrap_or(serde_json::json!({})),
        }
    }).collect();

    info!("Found payment {} with {} audit log entries", payment_id, audit_logs.len());

    Ok(Json(AdminPaymentDetailsResponse {
        success: true,
        payment,
        audit_logs,
    }))
}

/// Update payment status
pub async fn admin_update_payment_status_handler(
    State(_app_state): State<crate::web::auth::AppState>,
    axum::Extension(admin_context): axum::Extension<crate::web::middleware::OpenIDUserContext>,
    Path(payment_id): Path<Uuid>,
    Json(request): Json<UpdatePaymentStatusRequest>,
) -> Result<Json<UpdatePaymentStatusResponse>, Json<UnifiedErrorResponse>> {
    use diesel::prelude::*;
    use diesel_async::RunQueryDsl;
    use crate::infrastructure::database::get_payments_pool;
    use crate::schemas::payments::{payments, payment_audit_log};

    let admin_wallet = &admin_context.wallet_address;
    info!(
        "Admin {} updating payment {} status to {}",
        admin_wallet,
        payment_id,
        request.status
    );

    // Validate status transition
    let valid_statuses = ["created", "pending", "confirmed", "completed", "failed", "refunded", "expired", "cancelled"];
    if !valid_statuses.contains(&request.status.as_str()) {
        return Err(Json(UnifiedErrorResponse::new(400, "Invalid status", format!("Status must be one of: {:?}", valid_statuses))));
    }

    // Get PAYMENTS database connection
    let payments_pool = get_payments_pool().await.map_err(|e| {
        error!("Failed to get payments database pool: {}", e);
        Json(UnifiedErrorResponse::new(500, "Database connection failed", "Failed to get payments database pool"))
    })?;
    let mut payments_conn = payments_pool.get().await
        .map_err(|e| {
            error!("Failed to get payments database connection: {}", e);
            Json(UnifiedErrorResponse::new(500, "Database connection failed", "Failed to establish payments database connection"))
        })?;

    // Get current payment status
    let old_status: String = payments::table
        .filter(payments::id.eq(payment_id))
        .select(payments::status)
        .first(&mut payments_conn)
        .await
        .map_err(|e| {
            if matches!(e, diesel::NotFound) {
                Json(UnifiedErrorResponse::new(404, "Payment not found", format!("No payment found with ID: {}", payment_id)))
            } else {
                error!("Failed to get payment status: {}", e);
                Json(UnifiedErrorResponse::new(500, "Query failed", "Failed to get current payment status"))
            }
        })?;

    // Update payment status
    let updated_at = Utc::now();
    let completed_at = if request.status == "completed" || request.status == "confirmed" {
        Some(updated_at)
    } else {
        None
    };

    diesel::update(payments::table.filter(payments::id.eq(payment_id)))
        .set((
            payments::status.eq(&request.status),
            payments::updated_at.eq(updated_at),
            payments::completed_at.eq(completed_at),
        ))
        .execute(&mut payments_conn)
        .await
        .map_err(|e| {
            error!("Failed to update payment status: {}", e);
            Json(UnifiedErrorResponse::new(500, "Update failed", format!("Failed to update payment: {}", e)))
        })?;

    // Create audit log entry with actual admin wallet
    diesel::insert_into(payment_audit_log::table)
        .values((
            payment_audit_log::id.eq(Uuid::new_v4()),
            payment_audit_log::payment_id.eq(payment_id),
            payment_audit_log::action.eq("status_change"),
            payment_audit_log::old_status.eq(&old_status),
            payment_audit_log::new_status.eq(&request.status),
            payment_audit_log::reason.eq(&request.reason),
            payment_audit_log::performed_by.eq(admin_wallet.as_str()),
            payment_audit_log::created_at.eq(updated_at),
            payment_audit_log::metadata.eq(request.metadata.unwrap_or(serde_json::json!({}))),
        ))
        .execute(&mut payments_conn)
        .await
        .map_err(|e| {
            error!("Failed to create audit log: {}", e);
            // Non-fatal error, log and continue
        })
        .ok();

    info!("Admin {} updated payment {} status from {} to {}", admin_wallet, payment_id, old_status, request.status);

    Ok(Json(UpdatePaymentStatusResponse {
        success: true,
        message: "Payment status updated successfully".to_string(),
        old_status,
        new_status: request.status,
        updated_at,
    }))
}

/// Process refund
pub async fn admin_process_refund_handler(
    State(_app_state): State<crate::web::auth::AppState>,
    axum::Extension(admin_ctx): axum::Extension<crate::web::middleware::OpenIDUserContext>,
    Path(payment_id): Path<Uuid>,
    Json(request): Json<RefundPaymentRequest>,
) -> Result<Json<RefundPaymentResponse>, Json<UnifiedErrorResponse>> {
    use diesel::prelude::*;
    use diesel_async::RunQueryDsl;
    use crate::infrastructure::database::get_payments_pool;
    use crate::schemas::payments::{payments, subscriptions, payment_audit_log};

    // H7: Explicit permission check — don't rely solely on router middleware
    if !admin_ctx.permissions.iter().any(|p| p == "admin:payments:refund" || p == "admin:*") {
        return Err(Json(UnifiedErrorResponse::new(
            403,
            "Permission denied",
            "admin:payments:refund permission required",
        )));
    }

    let admin_wallet = &admin_ctx.wallet_address;
    info!(
        "Admin {} processing refund for payment {}, reason: {}",
        admin_wallet,
        payment_id,
        request.reason
    );

    // Get PAYMENTS database connection
    let payments_pool = get_payments_pool().await.map_err(|e| {
        error!("Failed to get payments database pool: {}", e);
        Json(UnifiedErrorResponse::new(500, "Database connection failed", "Failed to get payments database pool"))
    })?;
    let mut payments_conn = payments_pool.get().await
        .map_err(|e| {
            error!("Failed to get payments database connection: {}", e);
            Json(UnifiedErrorResponse::new(500, "Database connection failed", "Failed to establish payments database connection"))
        })?;

    // Get current payment
    let (old_status, payment_amount): (String, bigdecimal::BigDecimal) = payments::table
        .filter(payments::id.eq(payment_id))
        .select((payments::status, payments::amount))
        .first(&mut payments_conn)
        .await
        .map_err(|e| {
            if matches!(e, diesel::NotFound) {
                Json(UnifiedErrorResponse::new(404, "Payment not found", format!("No payment found with ID: {}", payment_id)))
            } else {
                error!("Failed to query payment: {}", e);
                Json(UnifiedErrorResponse::new(500, "Query failed", format!("Failed to load payment: {}", e)))
            }
        })?;

    // Validate refund eligibility
    if !["completed", "confirmed"].contains(&old_status.as_str()) {
        return Err(Json(UnifiedErrorResponse::new(400, "Invalid refund", format!("Cannot refund payment with status: {}", old_status))));
    }

    // Calculate refund amount
    let payment_amount_f64 = payment_amount.to_string().parse::<f64>().unwrap_or(0.0);
    let refund_amount = if request.partial_refund {
        request.refund_amount.unwrap_or(payment_amount_f64)
    } else {
        payment_amount_f64
    };

    // Validate refund amount
    if refund_amount <= 0.0 || refund_amount > payment_amount_f64 {
        return Err(Json(UnifiedErrorResponse::new(400, "Invalid refund amount", format!("Refund amount must be between 0 and {}", payment_amount_f64))));
    }

    let processed_at = Utc::now();
    let refund_id = format!("REF-{}", Uuid::new_v4().to_string()[..8].to_string().to_uppercase());

    // Update payment status to refunded
    diesel::update(payments::table.filter(payments::id.eq(payment_id)))
        .set((
            payments::status.eq("refunded"),
            payments::updated_at.eq(processed_at),
        ))
        .execute(&mut payments_conn)
        .await
        .map_err(|e| {
            error!("Failed to update payment status: {}", e);
            Json(UnifiedErrorResponse::new(500, "Update failed", format!("Failed to update payment: {}", e)))
        })?;

    // Cancel associated subscription if exists
    diesel::update(subscriptions::table.filter(subscriptions::payment_id.eq(payment_id)))
        .set((
            subscriptions::status.eq("cancelled"),
            subscriptions::cancelled_at.eq(Some(processed_at)),
        ))
        .execute(&mut payments_conn)
        .await
        .ok(); // Non-fatal if no subscription found

    // Create audit log entry
    diesel::insert_into(payment_audit_log::table)
        .values((
            payment_audit_log::id.eq(Uuid::new_v4()),
            payment_audit_log::payment_id.eq(payment_id),
            payment_audit_log::action.eq("refund"),
            payment_audit_log::old_status.eq(&old_status),
            payment_audit_log::new_status.eq("refunded"),
            payment_audit_log::reason.eq(&request.reason),
            payment_audit_log::performed_by.eq(admin_wallet.as_str()),
            payment_audit_log::created_at.eq(processed_at),
            payment_audit_log::metadata.eq(serde_json::json!({
                "refund_id": refund_id,
                "refund_amount": refund_amount,
                "partial_refund": request.partial_refund,
            })),
        ))
        .execute(&mut payments_conn)
        .await
        .ok();

    info!("Refund {} processed for payment {}, amount: {}", refund_id, payment_id, refund_amount);

    Ok(Json(RefundPaymentResponse {
        success: true,
        message: "Refund processed successfully".to_string(),
        refund_id: Some(refund_id),
        refund_amount,
        processed_at,
    }))
}
