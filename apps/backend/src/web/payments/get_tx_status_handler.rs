//! Transaction Status Handler
//!
//! Allows frontend to poll for transaction status updates.
//! Returns current confirmation status, block number, and any errors.

use axum::{
    extract::{Path, State},
    response::Json,
    Extension,
};
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use serde::Serialize;
use tracing::{debug, error};

use crate::{
    prelude::*,
    infrastructure::database::{get_payments_pool, get_diesel_pool},
    schemas::payments::payments,
    schemas::primary::groups,
    infrastructure::models::payment::PaymentDb,
    web::{
        auth::AppState,
        middleware::{ErrorDetails, OpenIDUserContext, UnifiedErrorResponse},
    },
};

// ============================================================================
// RESPONSE TYPES
// ============================================================================

/// Response for transaction status query
#[derive(Debug, Serialize)]
pub struct TransactionStatusResponse {
    pub success: bool,
    pub data: TransactionStatusData,
}

/// Transaction status details
#[derive(Debug, Serialize)]
pub struct TransactionStatusData {
    /// Transaction hash
    pub transaction_hash: String,
    /// Current status: pending, confirming, confirmed, failed
    pub status: String,
    /// Number of block confirmations
    pub confirmations: i32,
    /// Block number where transaction was included
    pub block_number: Option<i64>,
    /// Error message if failed
    pub error_message: Option<String>,
    /// Payment reference ID
    pub payment_reference: Option<String>,
    /// Plan name if available
    pub plan_name: Option<String>,
    /// Amount paid
    pub amount: Option<f64>,
    /// Currency
    pub currency: Option<String>,
    /// When payment was completed
    pub completed_at: Option<DateTime<Utc>>,
    /// When the status was last checked
    pub last_checked_at: Option<DateTime<Utc>>,
}

// ============================================================================
// HANDLER
// ============================================================================

/// GET /api/payments/status/:tx_hash
///
/// Get the current status of a transaction.
/// Frontend polls this endpoint to track payment progress.
#[axum::debug_handler]
pub async fn get_transaction_status_handler(
    State(_app_state): State<AppState>,
    Extension(user_context): Extension<OpenIDUserContext>,
    Path(tx_hash): Path<String>,
) -> Result<Json<TransactionStatusResponse>, Json<UnifiedErrorResponse>> {
    let wallet_address = user_context.wallet_address.to_lowercase();
    
    debug!(
        "📊 Getting transaction status: wallet={}, tx_hash={}",
        wallet_address, tx_hash
    );

    // Validate transaction hash format
    if !tx_hash.starts_with("0x") || tx_hash.len() != 66 {
        return Err(Json(UnifiedErrorResponse {
            success: false,
            error: ErrorDetails {
                code: 400,
                message: "Invalid transaction hash".to_string(),
                reason: "Transaction hash must be 66 characters starting with 0x".to_string(),
            },
        }));
    }

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

    // Query payment by transaction hash
    let payment: Option<PaymentDb> = payments::table
        .filter(payments::transaction_hash.eq(&tx_hash))
        .filter(payments::wallet_address.eq(&wallet_address))
        .select(PaymentDb::as_select())
        .first::<PaymentDb>(&mut conn)
        .await
        .optional()
        .map_err(|e| {
            error!("Failed to query payment: {}", e);
            Json(UnifiedErrorResponse {
                success: false,
                error: ErrorDetails {
                    code: 500,
                    message: "Database query failed".to_string(),
                    reason: format!("Cannot query payment: {}", e),
                },
            })
        })?;

    match payment {
        Some(payment) => {
            // Get plan name from primary database if plan_id exists
            // Get plan name from primary database
            let plan_name = {
                let primary_pool = get_diesel_pool().await.ok();
                if let Some(pool) = primary_pool {
                    if let Ok(mut primary_conn) = pool.get().await {
                        groups::table
                            .filter(groups::id.eq(payment.plan_id))
                            .select(groups::name)
                            .first::<String>(&mut primary_conn)
                            .await
                            .ok()
                    } else {
                        None
                    }
                } else {
                    None
                }
            };

            // Convert amount
            use bigdecimal::ToPrimitive;
            let amount = payment.amount.to_f64();

            if payment.status == "confirmed" {
                tracing::info!(
                    "✅ Returning CONFIRMED status to frontend for tx: {}", 
                    tx_hash
                );
            }

            Ok(Json(TransactionStatusResponse {
                success: true,
                data: TransactionStatusData {
                    transaction_hash: tx_hash,
                    status: payment.status.clone(),
                    confirmations: payment.confirmations.unwrap_or(0),
                    block_number: payment.block_number,
                    error_message: payment.error_message,
                    payment_reference: Some(payment.payment_reference),
                    plan_name,
                    amount,
                    currency: Some(payment.currency),
                    completed_at: payment.completed_at,
                    last_checked_at: payment.last_checked_at,
                },
            }))
        }
        None => {
            // Transaction not found in database
            Err(Json(UnifiedErrorResponse {
                success: false,
                error: ErrorDetails {
                    code: 404,
                    message: "Transaction not found".to_string(),
                    reason: "No payment record found for this transaction hash. Make sure to submit the transaction first.".to_string(),
                },
            }))
        }
    }
}

use diesel::result::OptionalExtension;
