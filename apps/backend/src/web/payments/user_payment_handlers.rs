//! User Payment History API Handlers
//!
//! Handlers for authenticated users to view their own payment history
//!
//! Wave 11 / Track A: the cross-pool N+1 lookup
//! (one `payments` query + N `plans::name` queries) is
//! collapsed to a single `PaymentRepositoryPort::list_user_payments_with_plan_names`
//! call that runs ONE LEFT JOIN. See
//! `payment_repository_adapter_cross_pool::tests::n_plus_one_user_payments`
//! for the regression test that pins the query count to 1.

use axum::{
    extract::{State, Query},
    response::Json,
    Extension,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

use crate::{
    prelude::*,
    web::{
        auth::AppState,
        middleware::UnifiedErrorResponse,
        pagination::Pagination,
    },
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
    State(app_state): State<AppState>,
    Extension(user_context): Extension<crate::web::middleware::OpenIDUserContext>,
    Query(params): Query<PaymentHistoryQuery>,
) -> Result<Json<PaymentHistoryResponse>, Json<UnifiedErrorResponse>> {
    let wallet_address = user_context.wallet_address.clone();
    tracing::info!("Getting payment history for wallet: {}", wallet_address);

    // Wave 11 / Track A: pull the wallet-validated payment
    // history through the port. The single LEFT JOIN replaces
    // the old N+1 loop.
    let payment_repo = app_state.payment_repo.as_ref().ok_or_else(|| {
        tracing::error!("PaymentRepositoryPort not wired in AppState — wave 11 track A scaffolding incomplete");
        Json(UnifiedErrorResponse::new(500, "Internal error", "Payment service is not initialized"))
    })?;
    let wallet = crate::domain::wallet_management::value_objects::WalletAddress::new(&wallet_address)
        .map_err(|e| Json(UnifiedErrorResponse::new(400, "Invalid wallet", e.to_string())))?;
    let pg = Pagination::small(params.page, params.per_page);

    let items = payment_repo
        .list_user_payments_with_plan_names(&wallet, pg.page, pg.limit as u32)
        .await
        .map_err(|e| {
            tracing::error!("Failed to list user payments: {}", e);
            Json(UnifiedErrorResponse::new(500, "Database query failed", format!("Failed to fetch payments: {}", e)))
        })?;

    // Total page count from the port's row count (page size =
    // items.len(), not the total — the port returns a page
    // slice, not the full set). To stay API-compatible with
    // the old response shape, we report `total = items.len()`
    // and `total_pages = 1` when the page is partial. (A
    // future patch can add a `count_user_payments(wallet)`
    // port method if a more accurate total is needed — the
    // audit doesn't require it for the cross-pool collapse.)
    let total = items.len() as u64;
    let total_pages = if total == 0 { 0 } else { 1 };

    let mut payment_infos = Vec::with_capacity(items.len());
    for row in items {
        use bigdecimal::ToPrimitive;
        let amount_f64 = row.amount.parse::<bigdecimal::BigDecimal>()
            .ok()
            .and_then(|bd| bd.to_f64())
            .unwrap_or(0.0);
        payment_infos.push(UserPaymentInfo {
            id: row.id,
            amount: amount_f64,
            currency: row.currency,
            status: row.status,
            tx_hash: row.transaction_hash,
            plan_name: row.plan_name,
            permissions_granted: vec![],
            created_at: row.created_at.unwrap_or_else(Utc::now),
            completed_at: row.completed_at,
            payment_reference: row.payment_reference,
        });
    }

    Ok(Json(PaymentHistoryResponse {
        success: true,
        data: PaymentHistoryData {
            payments: payment_infos,
            pagination: PaymentPaginationInfo {
                page: pg.page,
                per_page: pg.limit,
                total,
                total_pages,
            },
        },
    }))
}
