//! Admin Payment Management API Handlers
//!
//! Comprehensive admin interface for managing payments, subscriptions, and financial operations
//! Requires admin permissions and provides detailed analytics and management capabilities

use axum::{
    extract::{State, Query, Path},
    response::Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Datelike, Utc};
use tracing::{info, error};

use crate::{
    prelude::*,
    web::{
        middleware::UnifiedErrorResponse,
        pagination::Pagination,
    },
    schemas::primary::{plans},
    schemas::payments::subscriptions,
};

/// Admin payment list query parameters
#[derive(Debug, Deserialize)]
pub struct AdminPaymentListParams {
    /// Page number for pagination
    pub page: Option<u32>,
    /// Number of items per page
    pub limit: Option<u32>,
    /// Filter by payment status
    pub status: Option<String>,
    /// Filter by wallet address
    pub wallet_address: Option<String>,
    /// Filter by plan ID
    pub plan_id: Option<Uuid>,
    /// Filter by date range (start)
    pub start_date: Option<String>,
    /// Filter by date range (end)
    pub end_date: Option<String>,
    /// Search by transaction hash or reference
    pub search: Option<String>,
}

/// Admin payment list response
#[derive(Debug, Serialize)]
pub struct AdminPaymentListResponse {
    pub success: bool,
    pub payments: Vec<AdminPaymentInfo>,
    pub pagination: PaginationInfo,
    pub summary: PaymentSummary,
}

/// Admin payment information
#[derive(Debug, Serialize)]
pub struct AdminPaymentInfo {
    pub id: Uuid,
    pub payment_reference: String,
    pub wallet_address: String,
    pub amount: f64,
    pub currency: String,
    pub status: String,
    pub plan_id: Uuid,
    pub plan_name: String,
    pub transaction_hash: Option<String>,
    pub contract_address: Option<String>,
    pub token_address: Option<String>,
    pub block_number: Option<i64>,
    pub confirmations: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
    pub metadata: serde_json::Value,
}

impl AdminPaymentInfo {
    /// Create from PaymentDb with plan name
    pub fn from_db(pay: crate::infrastructure::models::payment::PaymentDb, plan_name: String) -> Self {
        Self {
            id: pay.id,
            payment_reference: pay.payment_reference,
            wallet_address: pay.wallet_address,
            amount: pay.amount.to_string().parse::<f64>().unwrap_or(0.0),
            currency: pay.currency,
            status: pay.status,
            plan_id: pay.plan_id,
            plan_name,
            transaction_hash: pay.transaction_hash,
            contract_address: pay.contract_address,
            token_address: pay.token_address,
            block_number: pay.block_number,
            confirmations: pay.confirmations.unwrap_or(0),
            created_at: pay.created_at.unwrap_or_else(Utc::now),
            updated_at: pay.updated_at.unwrap_or_else(Utc::now),
            completed_at: pay.completed_at,
            expires_at: pay.expires_at,
            metadata: pay.metadata.unwrap_or(serde_json::json!({})),
        }
    }
}

/// Pagination information
#[derive(Debug, Serialize)]
pub struct PaginationInfo {
    pub page: u32,
    pub limit: u32,
    pub total_count: u64,
    pub total_pages: u32,
    pub has_next: bool,
    pub has_prev: bool,
}

/// Payment summary statistics
#[derive(Debug, Serialize)]
pub struct PaymentSummary {
    pub total_payments: u64,
    pub total_amount: f64,
    pub successful_payments: u64,
    pub failed_payments: u64,
    pub pending_payments: u64,
    pub average_payment_amount: f64,
    pub payments_today: u64,
    pub revenue_today: f64,
}

/// Admin payment details response
#[derive(Debug, Serialize)]
pub struct AdminPaymentDetailsResponse {
    pub success: bool,
    pub payment: Option<AdminPaymentInfo>,
    pub audit_logs: Vec<PaymentAuditLog>,
}

/// Payment audit log entry
#[derive(Debug, Serialize)]
pub struct PaymentAuditLog {
    pub id: Uuid,
    pub action: String,
    pub old_status: Option<String>,
    pub new_status: Option<String>,
    pub reason: Option<String>,
    pub performed_by: Option<String>,
    pub created_at: DateTime<Utc>,
    pub metadata: serde_json::Value,
}

/// Admin subscription list response
#[derive(Debug, Serialize)]
pub struct AdminSubscriptionListResponse {
    pub success: bool,
    pub subscriptions: Vec<AdminSubscriptionInfo>,
    pub pagination: PaginationInfo,
    pub summary: SubscriptionSummary,
}

/// Admin subscription information
#[derive(Debug, Serialize)]
pub struct AdminSubscriptionInfo {
    pub id: Uuid,
    pub wallet_address: String,
    pub plan_id: Uuid,
    pub plan_name: String,
    pub status: String,
    pub payment_id: Uuid,
    pub started_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub cancelled_at: Option<DateTime<Utc>>,
    pub auto_renew: bool,
    pub metadata: serde_json::Value,
}

/// Subscription summary statistics
#[derive(Debug, Serialize)]
pub struct SubscriptionSummary {
    pub total_subscriptions: u64,
    pub active_subscriptions: u64,
    pub expired_subscriptions: u64,
    pub cancelled_subscriptions: u64,
    pub new_subscriptions_today: u64,
    pub expiring_soon: u64, // Subscriptions expiring in next 7 days
    pub monthly_revenue: f64,
}

/// Payment analytics response
#[derive(Debug, Serialize)]
pub struct PaymentAnalyticsResponse {
    pub success: bool,
    pub analytics: PaymentAnalytics,
}

/// Payment analytics data
#[derive(Debug, Serialize)]
pub struct PaymentAnalytics {
    pub daily_revenue: Vec<DailyRevenue>,
    pub plan_breakdown: Vec<PlanBreakdown>,
    pub payment_methods: Vec<PaymentMethodStats>,
    pub trends: PaymentTrends,
}

/// Daily revenue data
#[derive(Debug, Serialize)]
pub struct DailyRevenue {
    pub date: String,
    pub revenue: f64,
    pub payment_count: u32,
}

/// Plan breakdown data
#[derive(Debug, Serialize)]
pub struct PlanBreakdown {
    pub plan_id: Uuid,
    pub plan_name: String,
    pub subscription_count: u32,
    pub revenue: f64,
    pub average_revenue_per_user: f64,
}

/// Payment method statistics
#[derive(Debug, Serialize)]
pub struct PaymentMethodStats {
    pub method: String,
    pub count: u32,
    pub revenue: f64,
    pub success_rate: f64,
}

/// Payment trends
#[derive(Debug, Serialize)]
pub struct PaymentTrends {
    pub growth_rate: f64,
    pub churn_rate: f64,
    pub average_subscription_length: f64,
    pub customer_lifetime_value: f64,
}

/// Refund payment request
#[derive(Debug, Deserialize)]
pub struct RefundPaymentRequest {
    pub reason: String,
    pub refund_amount: Option<f64>,
    pub partial_refund: bool,
    pub notify_user: bool,
}

/// Refund payment response
#[derive(Debug, Serialize)]
pub struct RefundPaymentResponse {
    pub success: bool,
    pub message: String,
    pub refund_id: Option<String>,
    pub refund_amount: f64,
    pub processed_at: DateTime<Utc>,
}

/// Update payment status request
#[derive(Debug, Deserialize)]
pub struct UpdatePaymentStatusRequest {
    pub status: String,
    pub reason: Option<String>,
    pub notify_user: bool,
    pub metadata: Option<serde_json::Value>,
}

/// Update payment status response
#[derive(Debug, Serialize)]
pub struct UpdatePaymentStatusResponse {
    pub success: bool,
    pub message: String,
    pub old_status: String,
    pub new_status: String,
    pub updated_at: DateTime<Utc>,
}

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
    use crate::schemas::primary::plans;

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

    // Map to response format with plan name lookup
    let mut payments_resp: Vec<AdminPaymentInfo> = Vec::new();
    for pay_db in payment_rows {
        // Try to get plan name from plans table (PRIMARY DB)
        let plan_name = plans::table
            .filter(plans::id.eq(pay_db.plan_id))
            .select(plans::name)
            .first::<String>(&mut primary_conn)
            .await
            .unwrap_or_else(|_| "Unknown Plan".to_string());

        payments_resp.push(AdminPaymentInfo::from_db(pay_db, plan_name));
    }

    let total_pages = pg.total_pages(total_count as u64);
    let pagination = PaginationInfo {
        page: pg.page,
        limit: pg.limit,
        total_count: total_count as u64,
        total_pages,
        has_next: pg.has_next(total_count as u64),
        has_prev: pg.has_prev(),
    };

    // Calculate summary statistics from database
    let today_start = Utc::now().date_naive().and_hms_opt(0, 0, 0)
        .map(|dt| chrono::DateTime::<Utc>::from_naive_utc_and_offset(dt, Utc))
        .unwrap_or_else(Utc::now);

    let completed_count: i64 = payments::table
        .filter(payments::status.eq("completed").or(payments::status.eq("confirmed")))
        .count()
        .get_result(&mut payments_conn)
        .await
        .unwrap_or(0);

    let failed_count: i64 = payments::table
        .filter(payments::status.eq("failed"))
        .count()
        .get_result(&mut payments_conn)
        .await
        .unwrap_or(0);

    let pending_count: i64 = payments::table
        .filter(payments::status.eq("pending").or(payments::status.eq("created")))
        .count()
        .get_result(&mut payments_conn)
        .await
        .unwrap_or(0);

    let total_amount: Option<bigdecimal::BigDecimal> = payments::table
        .filter(payments::status.eq("completed").or(payments::status.eq("confirmed")))
        .select(diesel::dsl::sum(payments::amount))
        .first(&mut payments_conn)
        .await
        .unwrap_or(None);

    let payments_today: i64 = payments::table
        .filter(payments::created_at.ge(today_start))
        .count()
        .get_result(&mut payments_conn)
        .await
        .unwrap_or(0);

    let revenue_today: Option<bigdecimal::BigDecimal> = payments::table
        .filter(payments::created_at.ge(today_start))
        .filter(payments::status.eq("completed").or(payments::status.eq("confirmed")))
        .select(diesel::dsl::sum(payments::amount))
        .first(&mut payments_conn)
        .await
        .unwrap_or(None);

    let total_amount_f64 = total_amount
        .map(|bd| bd.to_string().parse::<f64>().unwrap_or(0.0))
        .unwrap_or(0.0);
    let revenue_today_f64 = revenue_today
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
    use crate::schemas::primary::plans;

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
    Path(payment_id): Path<Uuid>,
    Json(request): Json<RefundPaymentRequest>,
) -> Result<Json<RefundPaymentResponse>, Json<UnifiedErrorResponse>> {
    use diesel::prelude::*;
    use diesel_async::RunQueryDsl;
    use crate::infrastructure::database::get_payments_pool;
    use crate::schemas::payments::{payments, subscriptions, payment_audit_log};

    info!(
        "Admin processing refund for payment {}, reason: {}",
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
            payment_audit_log::performed_by.eq("admin"),
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

/// Get all subscriptions
pub async fn admin_list_subscriptions_handler(
    State(app_state): State<crate::web::auth::AppState>,
    Query(params): Query<AdminPaymentListParams>, // Reuse same params
) -> Result<Json<AdminSubscriptionListResponse>, Json<UnifiedErrorResponse>> {
    use diesel::prelude::*;
    use diesel_async::RunQueryDsl;
    use crate::infrastructure::models::payment::SubscriptionDb;
    use crate::infrastructure::database::get_payments_pool;
    use crate::schemas::payments::payments;

    info!("Admin listing subscriptions with params: {:?}", params);

    // Get PAYMENTS database connection (subscriptions table is in payments DB)
    let payments_pool = get_payments_pool().await.map_err(|e| {
        error!("Failed to get payments database pool: {}", e);
        Json(UnifiedErrorResponse::new(500, "Database connection failed", "Failed to get payments database pool"))
    })?;
    let mut payments_conn = payments_pool.get().await
        .map_err(|e| {
            error!("Failed to get payments database connection: {}", e);
            Json(UnifiedErrorResponse::new(500, "Database connection failed", "Failed to establish payments database connection"))
        })?;

    // Get PRIMARY database connection (plans table is in primary DB)
    let mut primary_conn = app_state.db_pool.get().await
        .map_err(|e| {
            error!("Failed to get primary database connection: {}", e);
            Json(UnifiedErrorResponse::new(500, "Database connection failed", "Failed to establish primary database connection"))
        })?;

    // Build query
    let mut query = subscriptions::table.into_boxed();

    // Apply wallet_address filter if provided
    if let Some(ref wallet_addr) = params.wallet_address {
        query = query.filter(subscriptions::wallet_address.eq(wallet_addr));
    }

    // Apply plan_id filter if provided
    if let Some(ref plan_id) = params.plan_id {
        query = query.filter(subscriptions::plan_id.eq(plan_id));
    }

    // Apply status filter if provided
    if let Some(ref status) = params.status {
        query = query.filter(subscriptions::status.eq(status));
    }

    // Apply pagination
    let pg = Pagination::large(params.page, params.limit);

    // Get total count (before pagination) from PAYMENTS DB
    let total_count: i64 = subscriptions::table
        .count()
        .get_result(&mut payments_conn)
        .await
        .unwrap_or(0);

    // Fetch subscriptions with pagination from PAYMENTS DB
    let subscription_rows = query
        .order(subscriptions::started_at.desc().nulls_last())
        .limit(pg.limit as i64)
        .offset(pg.offset)
        .load::<SubscriptionDb>(&mut payments_conn)
        .await
        .map_err(|e| {
            error!("Failed to query subscriptions: {}", e);
            Json(UnifiedErrorResponse::new(500, "Query failed", format!("Failed to load subscriptions: {}", e)))
        })?;

    // Map to response format with plan name lookup from PRIMARY DB
    let mut subscriptions_resp: Vec<AdminSubscriptionInfo> = Vec::new();
    for sub_db in subscription_rows {
        // Try to get plan name from plans table (PRIMARY DB)
        let plan_name = plans::table
            .filter(plans::id.eq(sub_db.plan_id))
            .select(plans::name)
            .first::<String>(&mut primary_conn)
            .await
            .unwrap_or_else(|_| "Unknown Plan".to_string());

        subscriptions_resp.push(AdminSubscriptionInfo {
            id: sub_db.id,
            wallet_address: sub_db.wallet_address,
            plan_id: sub_db.plan_id,
            plan_name,
            status: sub_db.status,
            payment_id: sub_db.payment_id.unwrap_or(Uuid::nil()),
            started_at: sub_db.started_at.unwrap_or_else(Utc::now),
            expires_at: sub_db.expires_at,
            cancelled_at: sub_db.cancelled_at,
            auto_renew: sub_db.auto_renew.unwrap_or(false),
            metadata: sub_db.metadata.unwrap_or(serde_json::json!({})),
        });
    }

    let total_pages = pg.total_pages(total_count as u64);
    let pagination = PaginationInfo {
        page: pg.page,
        limit: pg.limit,
        total_count: total_count as u64,
        total_pages,
        has_next: pg.has_next(total_count as u64),
        has_prev: pg.has_prev(),
    };

    // Calculate summary statistics with real database queries
    let today_start = Utc::now().date_naive().and_hms_opt(0, 0, 0)
        .map(|dt| chrono::DateTime::<Utc>::from_naive_utc_and_offset(dt, Utc))
        .unwrap_or_else(Utc::now);
    let seven_days_from_now = Utc::now() + chrono::Duration::days(7);
    let month_start = Utc::now().date_naive()
        .with_day(1)
        .and_then(|d| d.and_hms_opt(0, 0, 0))
        .map(|dt| chrono::DateTime::<Utc>::from_naive_utc_and_offset(dt, Utc))
        .unwrap_or_else(Utc::now);

    // Get counts from database
    let active_count: i64 = subscriptions::table
        .filter(subscriptions::status.eq("active"))
        .count()
        .get_result(&mut payments_conn)
        .await
        .unwrap_or(0);

    let expired_count: i64 = subscriptions::table
        .filter(subscriptions::status.eq("expired"))
        .count()
        .get_result(&mut payments_conn)
        .await
        .unwrap_or(0);

    let cancelled_count: i64 = subscriptions::table
        .filter(subscriptions::status.eq("cancelled"))
        .count()
        .get_result(&mut payments_conn)
        .await
        .unwrap_or(0);

    // New subscriptions today (started_at >= today_start)
    let new_today: i64 = subscriptions::table
        .filter(subscriptions::started_at.ge(today_start))
        .count()
        .get_result(&mut payments_conn)
        .await
        .unwrap_or(0);

    // Expiring soon (active and expires_at <= 7 days from now)
    let expiring_soon_count: i64 = subscriptions::table
        .filter(subscriptions::status.eq("active"))
        .filter(subscriptions::expires_at.le(seven_days_from_now))
        .filter(subscriptions::expires_at.ge(Utc::now()))
        .count()
        .get_result(&mut payments_conn)
        .await
        .unwrap_or(0);

    // Monthly revenue from payments (this month, completed/confirmed)
    let monthly_revenue_bd: Option<bigdecimal::BigDecimal> = payments::table
        .filter(payments::created_at.ge(month_start))
        .filter(payments::status.eq("completed")
            .or(payments::status.eq("confirmed")))
        .select(diesel::dsl::sum(payments::amount))
        .first(&mut payments_conn)
        .await
        .unwrap_or(None);

    let monthly_revenue = monthly_revenue_bd
        .map(|bd| bd.to_string().parse::<f64>().unwrap_or(0.0))
        .unwrap_or(0.0);

    let summary = SubscriptionSummary {
        total_subscriptions: total_count as u64,
        active_subscriptions: active_count as u64,
        expired_subscriptions: expired_count as u64,
        cancelled_subscriptions: cancelled_count as u64,
        new_subscriptions_today: new_today as u64,
        expiring_soon: expiring_soon_count as u64,
        monthly_revenue,
    };

    info!("Found {} subscriptions (page {} of {})", subscriptions_resp.len(), pg.page, total_pages);

    Ok(Json(AdminSubscriptionListResponse {
        success: true,
        subscriptions: subscriptions_resp,
        pagination,
        summary,
    }))
}

pub async fn admin_get_payment_analytics_handler(
    State(app_state): State<crate::web::auth::AppState>,
) -> Result<Json<PaymentAnalyticsResponse>, Json<UnifiedErrorResponse>> {
    use diesel::prelude::*;
    use diesel_async::RunQueryDsl;
    use crate::infrastructure::database::get_payments_pool;

    info!("Admin getting payment analytics");

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

    // === 1. Daily Revenue (last 30 days) ===
    let thirty_days_ago = Utc::now() - chrono::Duration::days(30);
    
    #[derive(diesel::QueryableByName)]
    struct DailyRevenueRow {
        #[diesel(sql_type = diesel::sql_types::Date)]
        payment_date: chrono::NaiveDate,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Numeric>)]
        daily_revenue: Option<bigdecimal::BigDecimal>,
        #[diesel(sql_type = diesel::sql_types::BigInt)]
        payment_count: i64,
    }

    let daily_revenue_rows = diesel::sql_query(
        r#"
        SELECT 
            DATE(created_at) as payment_date,
            SUM(amount) as daily_revenue,
            COUNT(*) as payment_count
        FROM payments
        WHERE created_at >= $1
          AND (status = 'completed' OR status = 'confirmed')
        GROUP BY DATE(created_at)
        ORDER BY payment_date DESC
        LIMIT 30
        "#
    )
    .bind::<diesel::sql_types::Timestamptz, _>(thirty_days_ago)
    .load::<DailyRevenueRow>(&mut payments_conn)
    .await
    .unwrap_or_default();

    let daily_revenue: Vec<DailyRevenue> = daily_revenue_rows.into_iter().map(|row| {
        DailyRevenue {
            date: row.payment_date.format("%Y-%m-%d").to_string(),
            revenue: row.daily_revenue.map(|bd| bd.to_string().parse::<f64>().unwrap_or(0.0)).unwrap_or(0.0),
            payment_count: row.payment_count as u32,
        }
    }).collect();

    // === 2. Plan Breakdown ===
    #[derive(diesel::QueryableByName)]
    struct PlanBreakdownRow {
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        plan_id: Uuid,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Numeric>)]
        total_revenue: Option<bigdecimal::BigDecimal>,
        #[diesel(sql_type = diesel::sql_types::BigInt)]
        subscription_count: i64,
    }

    let plan_rows = diesel::sql_query(
        r#"
        SELECT 
            plan_id,
            SUM(amount) as total_revenue,
            COUNT(*) as subscription_count
        FROM payments
        WHERE status = 'completed' OR status = 'confirmed'
        GROUP BY plan_id
        ORDER BY total_revenue DESC NULLS LAST
        LIMIT 10
        "#
    )
    .load::<PlanBreakdownRow>(&mut payments_conn)
    .await
    .unwrap_or_default();

    let mut plan_breakdown: Vec<PlanBreakdown> = Vec::new();
    for row in plan_rows {
        // Get plan name from plans table
        let plan_name = plans::table
            .filter(plans::id.eq(row.plan_id))
            .select(plans::name)
            .first::<String>(&mut primary_conn)
            .await
            .unwrap_or_else(|_| "Unknown Plan".to_string());

        let revenue = row.total_revenue.map(|bd| bd.to_string().parse::<f64>().unwrap_or(0.0)).unwrap_or(0.0);
        let count = row.subscription_count as u32;
        let arpu = if count > 0 { revenue / count as f64 } else { 0.0 };

        plan_breakdown.push(PlanBreakdown {
            plan_id: row.plan_id,
            plan_name,
            subscription_count: count,
            revenue,
            average_revenue_per_user: arpu,
        });
    }

    // === 3. Payment Methods (by currency/token) ===
    #[derive(diesel::QueryableByName)]
    struct PaymentMethodRow {
        #[diesel(sql_type = diesel::sql_types::Text)]
        currency: String,
        #[diesel(sql_type = diesel::sql_types::BigInt)]
        payment_count: i64,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Numeric>)]
        total_revenue: Option<bigdecimal::BigDecimal>,
        #[diesel(sql_type = diesel::sql_types::BigInt)]
        successful_count: i64,
    }

    let method_rows = diesel::sql_query(
        r#"
        SELECT 
            currency,
            COUNT(*) as payment_count,
            SUM(CASE WHEN status IN ('completed', 'confirmed') THEN amount ELSE 0 END) as total_revenue,
            SUM(CASE WHEN status IN ('completed', 'confirmed') THEN 1 ELSE 0 END) as successful_count
        FROM payments
        GROUP BY currency
        ORDER BY payment_count DESC
        "#
    )
    .load::<PaymentMethodRow>(&mut payments_conn)
    .await
    .unwrap_or_default();

    let payment_methods: Vec<PaymentMethodStats> = method_rows.into_iter().map(|row| {
        let total = row.payment_count as f64;
        let success = row.successful_count as f64;
        PaymentMethodStats {
            method: row.currency,
            count: row.payment_count as u32,
            revenue: row.total_revenue.map(|bd| bd.to_string().parse::<f64>().unwrap_or(0.0)).unwrap_or(0.0),
            success_rate: if total > 0.0 { (success / total) * 100.0 } else { 0.0 },
        }
    }).collect();

    // === 4. Payment Trends ===
    // Calculate growth rate (compare last 30 days vs previous 30 days)
    let sixty_days_ago = Utc::now() - chrono::Duration::days(60);
    
    #[derive(diesel::QueryableByName)]
    struct PeriodStats {
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Numeric>)]
        total_revenue: Option<bigdecimal::BigDecimal>,
        #[diesel(sql_type = diesel::sql_types::BigInt)]
        total_count: i64,
    }

    let current_period: PeriodStats = diesel::sql_query(
        r#"
        SELECT COALESCE(SUM(amount), 0) as total_revenue, COUNT(*) as total_count
        FROM payments
        WHERE created_at >= $1 AND (status = 'completed' OR status = 'confirmed')
        "#
    )
    .bind::<diesel::sql_types::Timestamptz, _>(thirty_days_ago)
    .get_result(&mut payments_conn)
    .await
    .unwrap_or(PeriodStats { total_revenue: None, total_count: 0 });

    let previous_period: PeriodStats = diesel::sql_query(
        r#"
        SELECT COALESCE(SUM(amount), 0) as total_revenue, COUNT(*) as total_count
        FROM payments
        WHERE created_at >= $1 AND created_at < $2 AND (status = 'completed' OR status = 'confirmed')
        "#
    )
    .bind::<diesel::sql_types::Timestamptz, _>(sixty_days_ago)
    .bind::<diesel::sql_types::Timestamptz, _>(thirty_days_ago)
    .get_result(&mut payments_conn)
    .await
    .unwrap_or(PeriodStats { total_revenue: None, total_count: 0 });

    let current_rev = current_period.total_revenue.map(|bd| bd.to_string().parse::<f64>().unwrap_or(0.0)).unwrap_or(0.0);
    let previous_rev = previous_period.total_revenue.map(|bd| bd.to_string().parse::<f64>().unwrap_or(0.0)).unwrap_or(0.0);
    
    let growth_rate = if previous_rev > 0.0 {
        ((current_rev - previous_rev) / previous_rev) * 100.0
    } else if current_rev > 0.0 {
        100.0 // 100% growth if previous was 0
    } else {
        0.0
    };

    // Calculate average subscription length from subscriptions table
    let avg_sub_length: f64 = subscriptions::table
        .filter(subscriptions::status.eq("active"))
        .select(diesel::dsl::avg(diesel::dsl::sql::<diesel::sql_types::Float>("EXTRACT(EPOCH FROM (expires_at - started_at)) / 86400.0")))
        .first::<Option<f64>>(&mut payments_conn)
        .await
        .ok()
        .flatten()
        .unwrap_or(30.0);

    // Churn rate estimate (cancelled in last 30 days / total active)
    let cancelled_count: i64 = subscriptions::table
        .filter(subscriptions::status.eq("cancelled"))
        .filter(subscriptions::cancelled_at.ge(thirty_days_ago))
        .count()
        .get_result(&mut payments_conn)
        .await
        .unwrap_or(0);

    let active_count: i64 = subscriptions::table
        .filter(subscriptions::status.eq("active"))
        .count()
        .get_result(&mut payments_conn)
        .await
        .unwrap_or(1); // Avoid division by zero

    let churn_rate = (cancelled_count as f64 / active_count.max(1) as f64) * 100.0;

    // Customer lifetime value estimate (average revenue * average subscription length / 30)
    let avg_payment: f64 = if current_period.total_count > 0 {
        current_rev / current_period.total_count as f64
    } else {
        0.0
    };
    let customer_lifetime_value = avg_payment * (avg_sub_length / 30.0);

    let trends = PaymentTrends {
        growth_rate,
        churn_rate,
        average_subscription_length: avg_sub_length,
        customer_lifetime_value,
    };

    let analytics = PaymentAnalytics {
        daily_revenue,
        plan_breakdown,
        payment_methods,
        trends,
    };

    info!("Successfully retrieved payment analytics");

    Ok(Json(PaymentAnalyticsResponse {
        success: true,
        analytics,
    }))
}

