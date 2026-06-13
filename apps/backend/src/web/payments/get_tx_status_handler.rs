//! Transaction Status Handler
//!
//! Allows frontend to poll for transaction status updates.
//! Returns current confirmation status, block number, and any errors.
//!
//! Wave 11 / Track A: the cross-pool `payments_pool` + `get_diesel_pool`
//! lookup for `plans::name` is collapsed to a single
//! `PaymentRepositoryPort::get_tx_status_with_plan_name` call.
//! This handler no longer imports `schemas::primary::plans` or
//! `infrastructure::database::*`.

use axum::{
    extract::{Path, State},
    response::Json,
    Extension,
};
use bigdecimal::ToPrimitive;
use chrono::{DateTime, Utc};
use serde::Serialize;
use tracing::{debug, error};

use crate::{
    prelude::*,
    web::{
        auth::AppState,
        middleware::{OpenIDUserContext, UnifiedErrorResponse},
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
    State(app_state): State<AppState>,
    Extension(user_context): Extension<OpenIDUserContext>,
    Path(tx_hash): Path<String>,
) -> Result<Json<TransactionStatusResponse>, UnifiedErrorResponse> {
    let wallet_address = user_context.wallet_address.clone();

    debug!(
        "Getting transaction status: wallet={}, tx_hash={}",
        wallet_address, tx_hash
    );

    // Validate transaction hash format
    if !tx_hash.starts_with("0x") || tx_hash.len() != 66 {
        return Err(UnifiedErrorResponse::new(400, "Invalid transaction hash", "Transaction hash must be 66 characters starting with 0x"));
    }

    // Wave 11 / Track A: collapse the cross-pool
    // `payments_pool` + `get_diesel_pool()` reacharound to a
    // single port call. The port's
    // `get_tx_status_with_plan_name` returns
    // `PaymentRowWithPlanName` (the flat row + plan name)
    // — the domain `Payment` aggregate does not carry
    // `confirmations` / `block_number` / `error_message` /
    // `last_checked_at`, which the response shape needs.
    let payment_repo = app_state.payment_repo.as_ref().ok_or_else(|| {
        error!("PaymentRepositoryPort not wired in AppState — wave 11 track A scaffolding incomplete");
        UnifiedErrorResponse::new(500, "Internal error", "Payment service is not initialized")
    })?;

    let result = payment_repo
        .get_tx_status_with_plan_name(&tx_hash)
        .await
        .map_err(|e| {
            error!("Failed to query payment via port: {}", e);
            UnifiedErrorResponse::new(500, "Database query failed", format!("Cannot query payment: {}", e))
        })?;

    let row = match result {
        Some(r) => r,
        None => {
            // H6: Uniform error response to prevent tx hash enumeration
            return Err(UnifiedErrorResponse::new(404, "Transaction not found", "Unable to retrieve transaction status"));
        }
    };

    // Wallet-ownership check. The legacy handler double-checked
    // `payments.wallet_address = $wallet`; the port's WHERE
    // clause does the same check, but the result is also
    // verified here in case the port is swapped for an HTTP
    // impl that doesn't filter by wallet.
    if row.wallet_address.to_lowercase() != wallet_address.to_lowercase() {
        return Err(UnifiedErrorResponse::new(404, "Transaction not found", "Unable to retrieve transaction status"));
    }

    if row.status == "confirmed" {
        tracing::info!(
            "Returning CONFIRMED status to frontend for tx: {}",
            tx_hash
        );
    }

    let amount = row
        .amount
        .parse::<bigdecimal::BigDecimal>()
        .ok()
        .and_then(|bd| bd.to_f64());

    Ok(Json(TransactionStatusResponse {
        success: true,
        data: TransactionStatusData {
            transaction_hash: tx_hash,
            status: row.status,
            confirmations: row.confirmations.unwrap_or(0),
            block_number: row.block_number,
            error_message: row.error_message,
            payment_reference: Some(row.payment_reference),
            plan_name: row.plan_name,
            amount,
            currency: Some(row.currency),
            completed_at: row.completed_at,
            last_checked_at: row.last_checked_at,
        },
    }))
}
