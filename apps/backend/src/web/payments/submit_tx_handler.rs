//! Submit Transaction Handler
//!
//! Handles frontend submission of transaction hashes after MetaMask broadcast.
//! Creates a pending payment record and triggers background monitoring.

use axum::{
    extract::State,
    response::Json,
    Extension,
};

use diesel_async::RunQueryDsl;
use serde::{Deserialize, Serialize};
use tracing::{error, info};
use uuid::Uuid;
use bigdecimal::BigDecimal;
use std::str::FromStr;

use crate::{
    prelude::*,
    web::{
        auth::AppState,
        middleware::{ErrorDetails, OpenIDUserContext, UnifiedErrorResponse},
    },
    infrastructure::database::get_payments_pool,
};

// ============================================================================
// REQUEST/RESPONSE TYPES
// ============================================================================

/// Request to submit a transaction for backend monitoring
#[derive(Debug, Deserialize)]
pub struct SubmitTransactionRequest {
    /// Transaction hash from MetaMask
    pub transaction_hash: String,
    /// Plan ID (UUID)
    pub plan_id: String,
    /// Expected payment amount
    pub expected_amount: f64,
    /// Currency (USDT, USDC)
    pub currency: String,
    /// Network name (optional)
    pub network: Option<String>,
}

/// Response after submitting transaction
#[derive(Debug, Serialize)]
pub struct SubmitTransactionResponse {
    pub success: bool,
    pub message: String,
    pub data: Option<SubmitTransactionData>,
}

#[derive(Debug, Serialize)]
pub struct SubmitTransactionData {
    /// Unique payment reference for tracking
    pub payment_reference: String,
    /// Current status: pending, confirming, confirmed, failed
    pub status: String,
    /// Transaction hash being monitored
    pub transaction_hash: String,
}

// ============================================================================
// HANDLER
// ============================================================================

/// POST /api/payments/submit
///
/// Submit a transaction hash for backend monitoring.
/// This creates a pending payment record and the background service
/// will monitor and update the status.
#[axum::debug_handler]
pub async fn submit_transaction_handler(
    State(_app_state): State<AppState>,
    Extension(user_context): Extension<OpenIDUserContext>,
    Json(payload): Json<SubmitTransactionRequest>,
) -> Result<Json<SubmitTransactionResponse>, Json<UnifiedErrorResponse>> {
    let wallet_address = user_context.wallet_address.to_lowercase();
    
    // Explicit debug print (bypasses tracing filters)
    println!("DEBUG: api/payments/submit HIT by wallet: {}", wallet_address);

    info!(
        "📥 Submitting transaction for monitoring: wallet={}, tx_hash={}, plan_id={}",
        wallet_address, payload.transaction_hash, payload.plan_id
    );

    // Validate transaction hash format
    if !payload.transaction_hash.starts_with("0x") || payload.transaction_hash.len() != 66 {
        return Err(Json(UnifiedErrorResponse {
            success: false,
            error: ErrorDetails {
                code: 400,
                message: "Invalid transaction hash".to_string(),
                reason: "Transaction hash must be 66 characters starting with 0x".to_string(),
            },
        }));
    }

    // Parse plan_id as UUID
    let plan_uuid = Uuid::parse_str(&payload.plan_id).map_err(|_| {
        Json(UnifiedErrorResponse {
            success: false,
            error: ErrorDetails {
                code: 400,
                message: "Invalid plan ID".to_string(),
                reason: "Plan ID must be a valid UUID".to_string(),
            },
        })
    })?;

    // Get payments database connection
    let payments_pool = get_payments_pool().await.map_err(|e| {
        error!("Failed to get payments database pool: {}", e);
        Json(UnifiedErrorResponse {
            success: false,
            error: ErrorDetails {
                code: 500,
                message: "Database error".to_string(),
                reason: "Cannot connect to database".to_string(),
            },
        })
    })?;
    
    let mut conn = payments_pool.get().await.map_err(|e| {
        error!("Failed to get database connection: {}", e);
        Json(UnifiedErrorResponse {
            success: false,
            error: ErrorDetails {
                code: 500,
                message: "Database error".to_string(),
                reason: "Cannot establish database connection".to_string(),
            },
        })
    })?;

    // Check if transaction already exists (deduplication)
    let existing: Result<Option<String>, _> = diesel::sql_query(
        "SELECT payment_reference FROM payments WHERE transaction_hash = $1 LIMIT 1"
    )
    .bind::<diesel::sql_types::Text, _>(&payload.transaction_hash)
    .get_result::<crate::web::payments::validation_handlers::PaymentExistsRow>(&mut conn)
    .await
    .optional()
    .map(|opt| opt.map(|row| row.payment_reference));

    if let Ok(Some(existing_ref)) = existing {
        info!("Transaction already submitted: {}", existing_ref);
        return Ok(Json(SubmitTransactionResponse {
            success: true,
            message: "Transaction already being monitored".to_string(),
            data: Some(SubmitTransactionData {
                payment_reference: existing_ref,
                status: "pending".to_string(),
                transaction_hash: payload.transaction_hash,
            }),
        }));
    }

    // Generate payment reference
    let payment_reference = format!("PAY-{}", Uuid::new_v4());
    let network = payload.network.unwrap_or_else(|| "unknown".to_string());

    // Insert pending payment record
    let insert_result = diesel::sql_query(
        r#"
        INSERT INTO payments (
            payment_reference, wallet_address, amount, currency, method, status,
            plan_id, transaction_hash, network, confirmations, metadata, created_at
        )
        VALUES ($1, $2, $3, $4, 'blockchain', 'pending', $5, $6, $7, 0, '{}', NOW())
        "#
    )
    .bind::<diesel::sql_types::Text, _>(&payment_reference)
    .bind::<diesel::sql_types::Text, _>(&wallet_address)
    .bind::<diesel::sql_types::Numeric, _>(
        BigDecimal::from_str(&payload.expected_amount.to_string()).unwrap_or_default()
    )
    .bind::<diesel::sql_types::Text, _>(&payload.currency)
    .bind::<diesel::sql_types::Uuid, _>(plan_uuid)
    .bind::<diesel::sql_types::Text, _>(&payload.transaction_hash)
    .bind::<diesel::sql_types::Text, _>(&network)
    .execute(&mut conn)
    .await;

    match insert_result {
        Ok(_) => {
            info!(
                "✅ Transaction submitted for monitoring: ref={}, tx={}",
                payment_reference, payload.transaction_hash
            );

            Ok(Json(SubmitTransactionResponse {
                success: true,
                message: "Transaction submitted for monitoring".to_string(),
                data: Some(SubmitTransactionData {
                    payment_reference,
                    status: "pending".to_string(),
                    transaction_hash: payload.transaction_hash,
                }),
            }))
        }
        Err(e) => {
            error!("Failed to insert payment record: {}", e);
            Err(Json(UnifiedErrorResponse {
                success: false,
                error: ErrorDetails {
                    code: 500,
                    message: "Failed to submit transaction".to_string(),
                    reason: format!("Database error: {}", e),
                },
            }))
        }
    }
}

use diesel::result::OptionalExtension;
