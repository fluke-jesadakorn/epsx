//! Credit Wallet API Handlers
//!
//! Handlers for credit balance, history, and admin management

use crate::infrastructure::adapters::repositories::credit_repository_adapter::admin::{
    valid_wallet, AdminCreditError,
};
use axum::http::{HeaderMap, StatusCode};
use axum::{
    extract::{Path, Query, State},
    response::Json,
    Extension,
};
use bigdecimal::BigDecimal;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tracing::{error, info};

use crate::{
    infrastructure::{
        adapters::repositories::CreditRepositoryAdapter,
        models::credit::{
            CreditBalanceResponse, CreditStatsResponse, CreditTransactionFilters,
            CreditTransactionResponse, GrantCreditsRequest, RevokeCreditsRequest,
        },
    },
    prelude::*,
    web::{auth::AppState, middleware::UnifiedErrorResponse},
};

// ============================================================================
// USER ENDPOINTS (Authenticated)
// ============================================================================

/// GET /api/credits/balance
/// Get authenticated user's credit balance
pub async fn get_credit_balance(
    State(_app_state): State<AppState>,
    Extension(user_context): Extension<crate::web::middleware::OpenIDUserContext>,
) -> Result<Json<CreditBalanceResponse>, Json<UnifiedErrorResponse>> {
    let wallet_address = user_context.wallet_address.clone();
    info!("Getting credit balance for wallet: {}", wallet_address);

    // Get payments database connection
    use crate::infrastructure::database::get_payments_pool;
    let payments_pool = get_payments_pool().await.map_err(|e| {
        error!("Failed to get payments database pool: {}", e);
        Json(UnifiedErrorResponse::new(
            500,
            "Database connection failed",
            "Failed to get database pool",
        ))
    })?;

    let repo = CreditRepositoryAdapter::new(std::sync::Arc::new(payments_pool));

    // Get or create balance
    let balance = repo
        .get_or_create_balance(&wallet_address)
        .await
        .map_err(|e| {
            error!("Failed to get credit balance: {}", e);
            Json(UnifiedErrorResponse::new(
                500,
                "Failed to retrieve balance",
                e.to_string(),
            ))
        })?;

    Ok(Json(CreditBalanceResponse::from(balance)))
}

/// Credit history query parameters
#[derive(Debug, Deserialize)]
pub struct CreditHistoryQuery {
    pub tx_type: Option<String>,
    pub from_date: Option<DateTime<Utc>>,
    pub to_date: Option<DateTime<Utc>>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// Credit history response
#[derive(Debug, Serialize)]
pub struct CreditHistoryResponse {
    pub success: bool,
    pub data: Vec<CreditTransactionResponse>,
    pub count: usize,
}

/// GET /api/credits/history
/// Get authenticated user's credit transaction history
pub async fn get_credit_history(
    State(_app_state): State<AppState>,
    Extension(user_context): Extension<crate::web::middleware::OpenIDUserContext>,
    Query(params): Query<CreditHistoryQuery>,
) -> Result<Json<CreditHistoryResponse>, Json<UnifiedErrorResponse>> {
    let wallet_address = user_context.wallet_address.clone();
    info!("Getting credit history for wallet: {}", wallet_address);

    // Get payments database connection
    use crate::infrastructure::database::get_payments_pool;
    let payments_pool = get_payments_pool().await.map_err(|e| {
        error!("Failed to get payments database pool: {}", e);
        Json(UnifiedErrorResponse::new(
            500,
            "Database connection failed",
            "Failed to get database pool",
        ))
    })?;

    let repo = CreditRepositoryAdapter::new(std::sync::Arc::new(payments_pool));

    let filters = CreditTransactionFilters {
        wallet_address: None, // Will be filtered in repository
        tx_type: params.tx_type,
        from_date: params.from_date,
        to_date: params.to_date,
        limit: params.limit,
        offset: params.offset,
    };

    let transactions = repo
        .get_transactions(&wallet_address, Some(filters))
        .await
        .map_err(|e| {
            error!("Failed to get credit history: {}", e);
            Json(UnifiedErrorResponse::new(
                500,
                "Failed to retrieve history",
                e.to_string(),
            ))
        })?;

    let count = transactions.len();
    let data: Vec<CreditTransactionResponse> = transactions
        .into_iter()
        .map(CreditTransactionResponse::from)
        .collect();

    Ok(Json(CreditHistoryResponse {
        success: true,
        data,
        count,
    }))
}

// ============================================================================
// ADMIN ENDPOINTS (Permission Required: admin:credits:manage)
// ============================================================================

/// GET /api/admin/credits/:wallet
/// Get user's credit balance and history (admin)
pub async fn admin_get_user_credits(
    State(_app_state): State<AppState>,
    Extension(_admin_context): Extension<crate::web::middleware::OpenIDUserContext>,
    Path(wallet_address): Path<String>,
    Query(params): Query<CreditHistoryQuery>,
) -> Result<Json<serde_json::Value>, AdminError> {
    let wallet_address = wallet_address.to_lowercase();
    info!("Admin getting credits for wallet: {}", wallet_address);

    if !valid_wallet(&wallet_address) {
        return Err(admin_error(AdminCreditError::Invalid));
    }
    let payments_pool = crate::infrastructure::database::get_payments_pool()
        .await
        .map_err(|_| admin_error(AdminCreditError::Unavailable))?;
    let repo = CreditRepositoryAdapter::new(std::sync::Arc::new(payments_pool));
    let balance = repo
        .get_or_create_balance(&wallet_address)
        .await
        .map_err(|_| admin_error(AdminCreditError::Unavailable))?;
    let filters = CreditTransactionFilters {
        wallet_address: None,
        tx_type: params.tx_type,
        from_date: params.from_date,
        to_date: params.to_date,
        limit: Some(params.limit.unwrap_or(50).clamp(1, 100)),
        offset: Some(params.offset.unwrap_or(0).max(0)),
    };
    let transactions = repo
        .get_transactions(&wallet_address, Some(filters))
        .await
        .map_err(|_| admin_error(AdminCreditError::Unavailable))?;
    Ok(Json(serde_json::json!({"success":true,"data":{
        "balance":CreditBalanceResponse::from(balance),
        "transactions":transactions.into_iter().map(CreditTransactionResponse::from).collect::<Vec<_>>()
    }})))
}

type AdminError = (StatusCode, Json<UnifiedErrorResponse>);
fn admin_error(error: AdminCreditError) -> AdminError {
    let (status, message) = match error {
        AdminCreditError::Invalid => (
            StatusCode::BAD_REQUEST,
            "Invalid wallet, amount, reason, expiry or idempotency key",
        ),
        AdminCreditError::Conflict => (
            StatusCode::CONFLICT,
            "Idempotency key was already used for a different command",
        ),
        AdminCreditError::InsufficientBalance => {
            (StatusCode::CONFLICT, "Insufficient available credits")
        }
        AdminCreditError::Unavailable => (
            StatusCode::SERVICE_UNAVAILABLE,
            "Credit ledger is unavailable",
        ),
    };
    (
        status,
        Json(UnifiedErrorResponse::new(status.as_u16(), message, message)),
    )
}

pub async fn admin_grant_credits(
    State(state): State<AppState>,
    Extension(context): Extension<crate::web::middleware::OpenIDUserContext>,
    headers: HeaderMap,
    Json(request): Json<GrantCreditsRequest>,
) -> Result<Json<serde_json::Value>, AdminError> {
    adjust(
        &state,
        &context.wallet_address,
        &headers,
        &request.wallet_address,
        request.amount,
        true,
        request.reason,
        request.expires_at,
    )
    .await
}

pub async fn admin_revoke_credits(
    State(state): State<AppState>,
    Extension(context): Extension<crate::web::middleware::OpenIDUserContext>,
    headers: HeaderMap,
    Json(request): Json<RevokeCreditsRequest>,
) -> Result<Json<serde_json::Value>, AdminError> {
    adjust(
        &state,
        &context.wallet_address,
        &headers,
        &request.wallet_address,
        request.amount,
        false,
        request.reason,
        None,
    )
    .await
}

async fn adjust(
    state: &AppState,
    actor: &str,
    headers: &HeaderMap,
    wallet: &str,
    amount: BigDecimal,
    grant: bool,
    reason: Option<String>,
    expires_at: Option<DateTime<Utc>>,
) -> Result<Json<serde_json::Value>, AdminError> {
    let key = headers
        .get("idempotency-key")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| admin_error(AdminCreditError::Invalid))?;
    let pool = crate::infrastructure::database::get_payments_pool()
        .await
        .map_err(|_| admin_error(AdminCreditError::Unavailable))?;
    let result = CreditRepositoryAdapter::new(std::sync::Arc::new(pool))
        .admin_adjust(
            actor,
            key,
            wallet,
            amount.clone(),
            grant,
            reason.as_deref(),
            expires_at,
        )
        .await
        .map_err(admin_error)?;
    if grant && !result.replayed {
        let state = state.clone();
        let wallet = wallet.to_ascii_lowercase();
        let amount = amount.to_string();
        let transaction_id = result.transaction_id;
        tokio::spawn(async move {
            use epsx_contracts::notification_port::SendNotificationRequest;
            if let Some(port) = state.notification_port.as_ref() {
                let _ = port
                    .send_with_event_id_retry(
                        &format!("payment.credit.grant:{transaction_id}"),
                        SendNotificationRequest {
                            recipient_wallet_address: wallet,
                            notification_type: "payment".into(),
                            priority: "normal".into(),
                            title: "Credits Received".into(),
                            message: format!("You received {amount} credits"),
                            data: Some(serde_json::json!({"amount":amount,"type":"grant"})),
                            action_url: None,
                            expires_at: None,
                        },
                    )
                    .await;
            }
        });
    }
    Ok(Json(serde_json::json!({"success":true,"data":{
        "transaction_id":result.transaction_id,"new_balance":result.balance_after.to_string(),"replayed":result.replayed
    }})))
}

/// GET /api/admin/credits/stats
/// Get credit system statistics (admin)
pub async fn admin_get_credit_stats(
    State(_state): State<AppState>,
    Extension(_context): Extension<crate::web::middleware::OpenIDUserContext>,
) -> Result<Json<CreditStatsResponse>, AdminError> {
    let pool = crate::infrastructure::database::get_payments_pool()
        .await
        .map_err(|_| admin_error(AdminCreditError::Unavailable))?;
    CreditRepositoryAdapter::new(std::sync::Arc::new(pool))
        .get_stats()
        .await
        .map(Json)
        .map_err(|_| admin_error(AdminCreditError::Unavailable))
}
