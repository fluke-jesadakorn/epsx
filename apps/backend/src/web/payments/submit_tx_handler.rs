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
use tracing::{debug, error, info};
use uuid::Uuid;
use bigdecimal::BigDecimal;
use std::str::FromStr;

use crate::{
    prelude::*,
    web::{
        auth::AppState,
        middleware::{OpenIDUserContext, UnifiedErrorResponse},
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

    debug!("api/payments/submit HIT by wallet: {}", wallet_address);

    info!(
        "Submitting transaction for monitoring: wallet={}, tx_hash={}, plan_id={}",
        wallet_address, payload.transaction_hash, payload.plan_id
    );

    // Validate transaction hash format
    if !payload.transaction_hash.starts_with("0x") || payload.transaction_hash.len() != 66 {
        return Err(UnifiedErrorResponse::json(400, "Invalid transaction hash", "Transaction hash must be 66 characters starting with 0x"));
    }

    // Parse plan_id as UUID
    let plan_uuid = Uuid::parse_str(&payload.plan_id)
        .map_err(|_| UnifiedErrorResponse::json(400, "Invalid plan ID", "Plan ID must be a valid UUID"))?;

    // Get payments database connection
    let payments_pool = get_payments_pool().await.map_err(|e| {
        error!("Failed to get payments database pool: {}", e);
        UnifiedErrorResponse::json(500, "Database error", "Cannot connect to database")
    })?;

    let mut conn = payments_pool.get().await.map_err(|e| {
        error!("Failed to get database connection: {}", e);
        UnifiedErrorResponse::json(500, "Database error", "Cannot establish database connection")
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
    let payment_id = Uuid::new_v4();
    let network = payload.network.unwrap_or_else(|| "unknown".to_string());

    // Check and apply credit balance
    use crate::infrastructure::adapters::repositories::CreditRepositoryAdapter;

    let credit_repo = CreditRepositoryAdapter::new(payments_pool);
    let wallet_credit_balance = credit_repo.get_balance(&wallet_address)
        .await
        .map(|opt| opt.map(|b| b.balance))
        .unwrap_or(None)
        .unwrap_or_else(|| BigDecimal::from(0));

    let payment_amount = BigDecimal::from_str(&payload.expected_amount.to_string()).unwrap_or_default();

    // Calculate how much credit to use (min of balance and payment amount)
    let credit_to_use = wallet_credit_balance.clone().min(payment_amount.clone());
    let remaining_amount = &payment_amount - &credit_to_use;

    info!(
        "Credit check: wallet_balance=${}, payment_amount=${}, credit_to_use=${}, remaining_amount=${}",
        wallet_credit_balance, payment_amount, credit_to_use, remaining_amount
    );

    // Deduct credits if applicable
    if credit_to_use > BigDecimal::from(0) {
        let negative_credit = -credit_to_use.clone();
        match credit_repo.add_transaction(
            &wallet_address,
            negative_credit,
            "payment_debit",
            Some(payment_id),
            Some("payment"),
            Some(&format!("Payment for plan {}", payload.plan_id)),
            None,
            None,
            Some(serde_json::json!({
                "payment_reference": payment_reference,
                "plan_id": payload.plan_id,
                "tx_hash": payload.transaction_hash,
            })),
        ).await {
            Ok(tx_id) => {
                info!("Deducted ${} credits (tx: {})", credit_to_use, tx_id);
            }
            Err(e) => {
                error!("Failed to deduct credits: {}. Proceeding with full blockchain payment", e);
                // If credit deduction fails, proceed with full blockchain payment
            }
        }
    }

    // Determine payment status:
    // - If credits cover full amount, mark as "confirmed" (no blockchain payment needed)
    // - Otherwise, mark as "pending" for blockchain verification
    let payment_status = if remaining_amount <= BigDecimal::from(0) {
        "confirmed"
    } else {
        "pending"
    };

    let transaction_hash_value = if remaining_amount <= BigDecimal::from(0) {
        None // No blockchain tx if fully covered by credits
    } else {
        Some(payload.transaction_hash.clone())
    };

    // Insert payment record with ID
    let insert_result = diesel::sql_query(
        r#"
        INSERT INTO payments (
            id, payment_reference, wallet_address, amount, currency, method, status,
            plan_id, transaction_hash, network, confirmations, metadata, created_at, completed_at
        )
        VALUES ($1, $2, $3, $4, $5, 'blockchain', $6, $7, $8, $9, 0, $10, NOW(), $11)
        "#
    )
    .bind::<diesel::sql_types::Uuid, _>(payment_id)
    .bind::<diesel::sql_types::Text, _>(&payment_reference)
    .bind::<diesel::sql_types::Text, _>(&wallet_address)
    .bind::<diesel::sql_types::Numeric, _>(&payment_amount)
    .bind::<diesel::sql_types::Text, _>(&payload.currency)
    .bind::<diesel::sql_types::Text, _>(payment_status)
    .bind::<diesel::sql_types::Uuid, _>(plan_uuid)
    .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(transaction_hash_value.as_ref())
    .bind::<diesel::sql_types::Text, _>(&network)
    .bind::<diesel::sql_types::Jsonb, _>(serde_json::json!({
        "credit_used": credit_to_use.to_string(),
        "original_amount": payment_amount.to_string(),
        "blockchain_amount": remaining_amount.to_string(),
    }))
    .bind::<diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>, _>(
        if payment_status == "confirmed" { Some(chrono::Utc::now()) } else { None }
    )
    .execute(&mut conn)
    .await;

    match insert_result {
        Ok(_) => {
            let message = if payment_status == "confirmed" {
                info!(
                    "Payment fully covered by credits: ref={}, amount=${}",
                    payment_reference, payment_amount
                );

                // Notify user about payment confirmation
                let notif_wallet = wallet_address.clone();
                let notif_ref = payment_reference.clone();
                let notif_state = _app_state.clone();
                tokio::spawn(async move {
                    use crate::infrastructure::services::NotificationService;
                    use crate::web::notifications::{NotificationType, NotificationPriority};
                    let _ = NotificationService::send(
                        &notif_state,
                        &notif_wallet,
                        NotificationType::Payment,
                        NotificationPriority::Normal,
                        "Payment Confirmed",
                        "Your payment has been confirmed",
                        Some(serde_json::json!({ "payment_reference": notif_ref })),
                        None,
                    ).await;
                });

                format!("Payment completed using ${} wallet credits", credit_to_use)
            } else if credit_to_use > BigDecimal::from(0) {
                info!(
                    "Partial credit payment: ref={}, credits_used=${}, blockchain_amount=${}, tx={}",
                    payment_reference, credit_to_use, remaining_amount, payload.transaction_hash
                );
                format!(
                    "Applied ${} credits. Remaining ${} being processed via blockchain",
                    credit_to_use, remaining_amount
                )
            } else {
                info!(
                    "Transaction submitted for monitoring: ref={}, tx={}",
                    payment_reference, payload.transaction_hash
                );
                "Transaction submitted for monitoring".to_string()
            };

            Ok(Json(SubmitTransactionResponse {
                success: true,
                message,
                data: Some(SubmitTransactionData {
                    payment_reference,
                    status: payment_status.to_string(),
                    transaction_hash: transaction_hash_value.unwrap_or_else(|| "N/A (paid with credits)".to_string()),
                }),
            }))
        }
        Err(e) => {
            error!("Failed to insert payment record: {}", e);
            Err(UnifiedErrorResponse::json(500, "Failed to submit transaction", format!("Database error: {}", e)))
        }
    }
}

use diesel::result::OptionalExtension;
