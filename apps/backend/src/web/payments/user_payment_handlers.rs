//! User Payment History API Handlers
//!
//! Handlers for authenticated users to view their own payment history

use axum::{
    extract::{State, Query},
    response::Json,
    Extension,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use tracing::{info, error};
use bigdecimal::ToPrimitive;

use crate::{
    prelude::*,
    web::{
        middleware::{UnifiedErrorResponse},
        auth::AppState,
        pagination::Pagination,
    },
    schemas::payments::payments,
    schemas::primary::{plans},
    infrastructure::models::payment::{PaymentDb},
    // domain::payment::repository_ports::{TransactionHistoryProvider},
};

/// Payment history query parameters
#[derive(Debug, Deserialize)]
pub struct PaymentHistoryQuery {
    /// Page number (default: 1)
    pub page: Option<u32>,
    /// Items per page (default: 10, max: 50)
    pub per_page: Option<u32>,
    /// Filter by status
    pub status: Option<String>,
}

/// Payment history response
#[derive(Debug, Serialize)]
pub struct PaymentHistoryResponse {
    pub success: bool,
    pub data: PaymentHistoryData,
}

#[derive(Debug, Serialize)]
pub struct PaymentHistoryData {
    pub payments: Vec<UserPaymentInfo>,
    pub pagination: PaymentPaginationInfo,
}

#[derive(Debug, Serialize)]
pub struct PaymentPaginationInfo {
    pub page: u32,
    pub per_page: u32,
    pub total: u64,
    pub total_pages: u32,
}

/// User payment information
#[derive(Debug, Serialize)]
pub struct UserPaymentInfo {
    pub id: Uuid,
    pub amount: f64,
    pub currency: String,
    pub status: String,
    pub tx_hash: Option<String>,
    pub plan_name: Option<String>,
    pub permissions_granted: Vec<String>, // Placeholder for now
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub payment_reference: String,
}

/// GET /api/payments/history
/// Returns paginated list of authenticated user's payments
pub async fn get_user_payment_history(
    State(_app_state): State<AppState>,
    Extension(user_context): Extension<crate::web::middleware::OpenIDUserContext>,
    Query(params): Query<PaymentHistoryQuery>,
) -> Result<Json<PaymentHistoryResponse>, Json<UnifiedErrorResponse>> {
    let wallet_address = user_context.wallet_address.to_lowercase();
    info!("Getting payment history for wallet: {}", wallet_address);

    // Pagination defaults
    let pg = Pagination::small(params.page, params.per_page);

    // NOTE: Changed to query DATABASE FIRST since confirm_payment_handler saves there
    // Blockchain provider is now supplementary (for historical data not in DB)
    
    // Get payments database connection (payments table is in payments DB, not primary DB)
    use crate::infrastructure::database::get_payments_pool;
    let payments_pool = get_payments_pool().await
        .map_err(|e| {
            error!("Failed to get payments database pool: {}", e);
            Json(UnifiedErrorResponse::new(500, "Database connection failed", "Failed to get payments database pool"))
        })?;
    let mut conn = payments_pool.get().await
        .map_err(|e| {
            error!("Failed to get database connection: {}", e);
            Json(UnifiedErrorResponse::new(500, "Database connection failed", "Failed to establish database connection"))
        })?;

    // 1. Get total count
    let mut count_query = payments::table.into_boxed()
        .filter(payments::wallet_address.eq(&wallet_address));

    if let Some(status) = &params.status {
        count_query = count_query.filter(payments::status.eq(status));
    }

    let total: i64 = count_query
        .count()
        .get_result(&mut conn)
        .await
        .map_err(|e| {
            error!("Failed to count payments: {}", e);
            Json(UnifiedErrorResponse::new(500, "Database query failed", "Failed to count payments"))
        })?;

    // 2. Get payments data
    let mut data_query = payments::table.into_boxed()
        .filter(payments::wallet_address.eq(&wallet_address));

    if let Some(status) = &params.status {
        data_query = data_query.filter(payments::status.eq(status));
    }

    let payments_list: Vec<PaymentDb> = data_query
        .order(payments::created_at.desc())
        .limit(pg.limit as i64)
        .offset(pg.offset)
        .load::<PaymentDb>(&mut conn)
        .await
        .map_err(|e| {
            error!("Failed to fetch payments: {}", e);
            Json(UnifiedErrorResponse::new(500, "Database query failed", "Failed to fetch payments"))
        })?;

    // Fetch plan names - need to use PRIMARY database since plans is there
    use crate::infrastructure::database::get_diesel_pool;
    let primary_pool = get_diesel_pool().await
        .map_err(|e| {
            error!("Failed to get primary database pool: {}", e);
            Json(UnifiedErrorResponse::new(500, "Database connection failed", "Failed to get primary database pool"))
        })?;
    let mut primary_conn = primary_pool.get().await
        .map_err(|e| {
            error!("Failed to get primary database connection: {}", e);
            Json(UnifiedErrorResponse::new(500, "Database connection failed", "Failed to establish primary database connection"))
        })?;

    let mut payment_infos = Vec::new();

    for payment in payments_list {
        // Find plan name if plan_id exists - query from PRIMARY database (where plans table lives)
        // Find plan name - query from PRIMARY database (where plans table lives)
        let plan_name = plans::table
            .filter(plans::id.eq(payment.plan_id))
            .select(plans::name)
            .first::<String>(&mut primary_conn)
            .await
            .ok();

        // Convert BigDecimal to f64 for JSON serialization
        let amount_f64 = payment.amount.to_f64().unwrap_or(0.0);

        payment_infos.push(UserPaymentInfo {
            id: payment.id,
            amount: amount_f64,
            currency: payment.currency,
            status: payment.status,
            tx_hash: payment.transaction_hash,
            plan_name,
            permissions_granted: vec![],
            created_at: payment.created_at.unwrap_or_else(Utc::now),
            completed_at: payment.completed_at,
            payment_reference: payment.payment_reference,
        });
    }

    let total_pages = pg.total_pages(total as u64);

    Ok(Json(PaymentHistoryResponse {
        success: true,
        data: PaymentHistoryData {
            payments: payment_infos,
            pagination: PaymentPaginationInfo {
                page: pg.page,
                per_page: pg.limit,
                total: total as u64,
                total_pages,
            },
        },
    }))
}
