//! Credit Wallet API Handlers
//!
//! Handlers for credit balance, history, and admin management

use axum::{
    extract::{State, Query, Path},
    response::Json,
    Extension,
};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use tracing::{info, error};
use bigdecimal::{BigDecimal, ToPrimitive};

use crate::{
    prelude::*,
    web::{
        middleware::UnifiedErrorResponse,
        auth::AppState,
    },
    infrastructure::{
        adapters::repositories::CreditRepositoryAdapter,
        models::credit::{
            CreditBalanceResponse, CreditTransactionResponse,
            CreditTransactionFilters, CreditStatsResponse,
            GrantCreditsRequest, RevokeCreditsRequest,
        },
    },
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
    let payments_pool = get_payments_pool().await
        .map_err(|e| {
            error!("Failed to get payments database pool: {}", e);
            Json(UnifiedErrorResponse::new(500, "Database connection failed", "Failed to get database pool"))
        })?;

    let repo = CreditRepositoryAdapter::new(payments_pool);

    // Get or create balance
    let balance = repo.get_or_create_balance(&wallet_address)
        .await
        .map_err(|e| {
            error!("Failed to get credit balance: {}", e);
            Json(UnifiedErrorResponse::new(500, "Failed to retrieve balance", e.to_string()))
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
    let payments_pool = get_payments_pool().await
        .map_err(|e| {
            error!("Failed to get payments database pool: {}", e);
            Json(UnifiedErrorResponse::new(500, "Database connection failed", "Failed to get database pool"))
        })?;

    let repo = CreditRepositoryAdapter::new(payments_pool);

    let filters = CreditTransactionFilters {
        wallet_address: None, // Will be filtered in repository
        tx_type: params.tx_type,
        from_date: params.from_date,
        to_date: params.to_date,
        limit: params.limit,
        offset: params.offset,
    };

    let transactions = repo.get_transactions(&wallet_address, Some(filters))
        .await
        .map_err(|e| {
            error!("Failed to get credit history: {}", e);
            Json(UnifiedErrorResponse::new(500, "Failed to retrieve history", e.to_string()))
        })?;

    let count = transactions.len();
    let data: Vec<CreditTransactionResponse> = transactions.into_iter()
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
) -> Result<Json<serde_json::Value>, Json<UnifiedErrorResponse>> {
    let wallet_address = wallet_address.to_lowercase();
    info!("Admin getting credits for wallet: {}", wallet_address);

    // Get payments database connection
    use crate::infrastructure::database::get_payments_pool;
    let payments_pool = get_payments_pool().await
        .map_err(|e| {
            error!("Failed to get payments database pool: {}", e);
            Json(UnifiedErrorResponse::new(500, "Database connection failed", "Failed to get database pool"))
        })?;

    let repo = CreditRepositoryAdapter::new(payments_pool);

    // Get balance
    let balance = repo.get_or_create_balance(&wallet_address)
        .await
        .map_err(|e| {
            error!("Failed to get credit balance: {}", e);
            Json(UnifiedErrorResponse::new(500, "Failed to retrieve balance", e.to_string()))
        })?;

    // Get transactions
    let filters = CreditTransactionFilters {
        wallet_address: None,
        tx_type: params.tx_type,
        from_date: params.from_date,
        to_date: params.to_date,
        limit: params.limit.or(Some(50)),
        offset: params.offset,
    };

    let transactions = repo.get_transactions(&wallet_address, Some(filters))
        .await
        .map_err(|e| {
            error!("Failed to get credit history: {}", e);
            Json(UnifiedErrorResponse::new(500, "Failed to retrieve history", e.to_string()))
        })?;

    let response = serde_json::json!({
        "success": true,
        "data": {
            "balance": CreditBalanceResponse::from(balance),
            "transactions": transactions.into_iter()
                .map(CreditTransactionResponse::from)
                .collect::<Vec<_>>(),
        }
    });

    Ok(Json(response))
}

/// POST /api/admin/credits/grant
/// Grant credits to a user (admin)
pub async fn admin_grant_credits(
    State(_app_state): State<AppState>,
    Extension(admin_context): Extension<crate::web::middleware::OpenIDUserContext>,
    Json(request): Json<GrantCreditsRequest>,
) -> Result<Json<serde_json::Value>, Json<UnifiedErrorResponse>> {
    let admin_wallet = admin_context.wallet_address.clone();
    info!("Admin {} granting {} credits to {}", admin_wallet, request.amount, request.wallet_address);

    // Validate amount is positive
    if request.amount <= 0 {
        return Err(Json(UnifiedErrorResponse::new(400, "Invalid amount", "Amount must be positive")));
    }

    let wallet_address = request.wallet_address.to_lowercase();

    // Get payments database connection
    use crate::infrastructure::database::get_payments_pool;
    let payments_pool = get_payments_pool().await
        .map_err(|e| {
            error!("Failed to get payments database pool: {}", e);
            Json(UnifiedErrorResponse::new(500, "Database connection failed", "Failed to get database pool"))
        })?;

    let repo = CreditRepositoryAdapter::new(payments_pool);

    // Add grant transaction
    let tx_id = repo.add_transaction(
        &wallet_address,
        request.amount.clone(),
        "grant",
        None,
        Some("admin_action"),
        request.reason.as_deref(),
        Some(&request.granted_by),
        request.expires_at,
        None,
    )
    .await
    .map_err(|e| {
        error!("Failed to grant credits: {}", e);
        Json(UnifiedErrorResponse::new(500, "Failed to grant credits", e.to_string()))
    })?;

    // Get updated balance
    let balance = repo.get_balance(&wallet_address)
        .await
        .map_err(|e| {
            error!("Failed to get updated balance: {}", e);
            Json(UnifiedErrorResponse::new(500, "Failed to retrieve balance", e.to_string()))
        })?
        .ok_or_else(|| {
            error!("Balance not found after granting credits");
            Json(UnifiedErrorResponse::new(500, "Internal error", "Balance not found"))
        })?;

    // Notify user about credits received
    let notif_wallet = wallet_address.clone();
    let notif_amount = request.amount.to_string();
    let notif_state = _app_state.clone();
    tokio::spawn(async move {
        // Wave 10 / R3: route through the NotificationPort (the
        // 8 publisher call sites call the port, not the concrete
        // NotificationService). The HTTP impl in the integration
        // gate can swap in without touching this code.
        use epsx_contracts::notification_port::SendNotificationRequest;
        if let Some(port) = notif_state.notification_port.as_ref() {
            let _ = port
                .send(SendNotificationRequest {
                    recipient_wallet_address: notif_wallet.clone(),
                    notification_type: "payment".to_string(),
                    priority: "normal".to_string(),
                    title: "Credits Received".to_string(),
                    message: format!("You received {} credits", notif_amount),
                    data: Some(serde_json::json!({ "amount": notif_amount, "type": "grant" })),
                    action_url: None,
                })
                .await;
        } else {
            tracing::warn!(
                "notification_port not wired in AppState; credits-received \
                 notification for wallet={} dropped",
                notif_wallet
            );
        }
    });

    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Credits granted successfully",
        "data": {
            "transaction_id": tx_id,
            "new_balance": balance.balance.to_f64().unwrap_or(0.0),
        }
    })))
}

/// POST /api/admin/credits/revoke
/// Revoke credits from a user (admin)
pub async fn admin_revoke_credits(
    State(_app_state): State<AppState>,
    Extension(admin_context): Extension<crate::web::middleware::OpenIDUserContext>,
    Json(request): Json<RevokeCreditsRequest>,
) -> Result<Json<serde_json::Value>, Json<UnifiedErrorResponse>> {
    let admin_wallet = admin_context.wallet_address.clone();
    info!("Admin {} revoking {} credits from {}", admin_wallet, request.amount, request.wallet_address);

    // Validate amount is positive
    if request.amount <= 0 {
        return Err(Json(UnifiedErrorResponse::new(400, "Invalid amount", "Amount must be positive")));
    }

    let wallet_address = request.wallet_address.to_lowercase();

    // Get payments database connection
    use crate::infrastructure::database::get_payments_pool;
    let payments_pool = get_payments_pool().await
        .map_err(|e| {
            error!("Failed to get payments database pool: {}", e);
            Json(UnifiedErrorResponse::new(500, "Database connection failed", "Failed to get database pool"))
        })?;

    let repo = CreditRepositoryAdapter::new(payments_pool);

    // Check if user has sufficient balance
    let current_balance = repo.get_balance(&wallet_address)
        .await
        .map_err(|e| {
            error!("Failed to get current balance: {}", e);
            Json(UnifiedErrorResponse::new(500, "Failed to retrieve balance", e.to_string()))
        })?
        .map(|b| b.balance)
        .unwrap_or_else(|| BigDecimal::from(0));

    if current_balance < request.amount {
        return Err(Json(UnifiedErrorResponse::new(400, "Insufficient balance",
            format!("User only has {} credits available", current_balance))));
    }

    // Add revoke transaction (negative amount)
    let negative_amount = -request.amount.clone();
    let tx_id = repo.add_transaction(
        &wallet_address,
        negative_amount,
        "revoke",
        None,
        Some("admin_action"),
        request.reason.as_deref(),
        Some(&request.granted_by),
        None,
        None,
    )
    .await
    .map_err(|e| {
        error!("Failed to revoke credits: {}", e);
        Json(UnifiedErrorResponse::new(500, "Failed to revoke credits", e.to_string()))
    })?;

    // Get updated balance
    let balance = repo.get_balance(&wallet_address)
        .await
        .map_err(|e| {
            error!("Failed to get updated balance: {}", e);
            Json(UnifiedErrorResponse::new(500, "Failed to retrieve balance", e.to_string()))
        })?
        .ok_or_else(|| {
            error!("Balance not found after revoking credits");
            Json(UnifiedErrorResponse::new(500, "Internal error", "Balance not found"))
        })?;

    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Credits revoked successfully",
        "data": {
            "transaction_id": tx_id,
            "new_balance": balance.balance.to_f64().unwrap_or(0.0),
        }
    })))
}

/// GET /api/admin/credits/stats
/// Get credit system statistics (admin)
pub async fn admin_get_credit_stats(
    State(_app_state): State<AppState>,
    Extension(_admin_context): Extension<crate::web::middleware::OpenIDUserContext>,
) -> Result<Json<CreditStatsResponse>, Json<UnifiedErrorResponse>> {
    info!("Admin getting credit statistics");

    // Get payments database connection
    use crate::infrastructure::database::get_payments_pool;
    let payments_pool = get_payments_pool().await
        .map_err(|e| {
            error!("Failed to get payments database pool: {}", e);
            Json(UnifiedErrorResponse::new(500, "Database connection failed", "Failed to get database pool"))
        })?;

    let repo = CreditRepositoryAdapter::new(payments_pool);

    let stats = repo.get_stats()
        .await
        .map_err(|e| {
            error!("Failed to get credit stats: {}", e);
            Json(UnifiedErrorResponse::new(500, "Failed to retrieve stats", e.to_string()))
        })?;

    Ok(Json(stats))
}
