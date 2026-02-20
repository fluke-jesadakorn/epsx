//! Submit Transaction Handler
//!
//! Handles frontend submission of transaction hashes after MetaMask broadcast.
//! Creates a pending payment record and triggers background monitoring.
//! Credit deduction and payment insert are wrapped in a DB transaction.

use axum::{extract::State, response::Json, Extension};

use diesel::result::OptionalExtension;
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
    /// Expected payment amount as string for precision (e.g. "29.99")
    pub expected_amount: serde_json::Value,
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
/// Credit deduction + payment record insert are atomic (single DB transaction).
#[axum::debug_handler]
pub async fn submit_transaction_handler(
    State(_app_state): State<AppState>,
    Extension(user_context): Extension<OpenIDUserContext>,
    Json(payload): Json<SubmitTransactionRequest>,
) -> Result<Json<SubmitTransactionResponse>, Json<UnifiedErrorResponse>> {
    let wallet_address = user_context.wallet_address.clone();

    debug!("api/payments/submit HIT by wallet: {}", wallet_address);

    // Validate transaction hash format
    if !payload.transaction_hash.starts_with("0x") || payload.transaction_hash.len() != 66 {
        return Err(UnifiedErrorResponse::json(
            400,
            "Invalid transaction hash",
            "Transaction hash must be 66 characters starting with 0x",
        ));
    }

    // Parse plan_id as UUID
    let plan_uuid = Uuid::parse_str(&payload.plan_id)
        .map_err(|_| UnifiedErrorResponse::json(400, "Invalid plan ID", "Plan ID must be a valid UUID"))?;

    // Parse expected_amount: accept both f64 and string
    let amount_str = match &payload.expected_amount {
        serde_json::Value::Number(n) => n.to_string(),
        serde_json::Value::String(s) => s.clone(),
        _ => {
            return Err(UnifiedErrorResponse::json(
                400,
                "Invalid amount",
                "expected_amount must be a number or string",
            ));
        }
    };
    let payment_amount = BigDecimal::from_str(&amount_str).map_err(|_| {
        UnifiedErrorResponse::json(400, "Invalid amount", "Cannot parse expected_amount")
    })?;

    if payment_amount <= BigDecimal::from(0) {
        return Err(UnifiedErrorResponse::json(
            400,
            "Invalid amount",
            "Payment amount must be positive",
        ));
    }

    // Get payments database connection
    let payments_pool = get_payments_pool().await.map_err(|e| {
        error!("Failed to get payments database pool: {}", e);
        UnifiedErrorResponse::json(500, "Database error", "Cannot connect to database")
    })?;

    let mut conn = payments_pool.get().await.map_err(|e| {
        error!("Failed to get database connection: {}", e);
        UnifiedErrorResponse::json(500, "Database error", "Cannot establish database connection")
    })?;

    // Check deduplication
    let existing: Result<Option<String>, _> = diesel::sql_query(
        "SELECT payment_reference FROM payments WHERE transaction_hash = $1 LIMIT 1",
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

    let payment_reference = format!("PAY-{}", Uuid::new_v4());
    let payment_id = Uuid::new_v4();
    let network = payload.network.unwrap_or_else(|| "unknown".to_string());

    // Get credit balance
    let wallet_credit_balance: BigDecimal = diesel::sql_query(
        "SELECT COALESCE((SELECT balance FROM wallet_credits WHERE wallet_address = $1), 0) as bal",
    )
    .bind::<diesel::sql_types::Text, _>(&wallet_address)
    .get_result::<BalanceRow>(&mut conn)
    .await
    .map(|r| r.bal)
    .unwrap_or_else(|_| BigDecimal::from(0));

    let credit_to_use = wallet_credit_balance.clone().min(payment_amount.clone());
    let remaining_amount = &payment_amount - &credit_to_use;

    info!(
        "Credit check: balance=${}, amount=${}, credit=${}, remaining=${}",
        wallet_credit_balance, payment_amount, credit_to_use, remaining_amount
    );

    let payment_status = if remaining_amount <= BigDecimal::from(0) {
        "confirmed"
    } else {
        "pending"
    };

    let tx_hash_value = if remaining_amount <= BigDecimal::from(0) {
        None
    } else {
        Some(payload.transaction_hash.clone())
    };

    let completed_at = if payment_status == "confirmed" {
        Some(chrono::Utc::now())
    } else {
        None
    };

    let metadata = serde_json::json!({
        "credit_used": credit_to_use.to_string(),
        "original_amount": payment_amount.to_string(),
        "blockchain_amount": remaining_amount.to_string(),
    });

    // Atomic transaction: credit deduction + payment insert
    // Uses a single SQL DO block to ensure both happen or neither
    let use_credits = credit_to_use > BigDecimal::from(0);

    if use_credits {
        // Wrap credit deduction + payment insert in a transaction
        let result = diesel::sql_query(
            r#"
            WITH credit_deduction AS (
                SELECT add_credit_transaction($1, $2, 'payment_debit', $3, 'payment', $4, NULL, NULL, $5) as tx_id
            )
            INSERT INTO payments (
                id, payment_reference, wallet_address, amount, currency, method, status,
                plan_id, transaction_hash, network, confirmations, metadata, created_at, completed_at
            )
            SELECT $6, $7, $1, $8, $9, 'blockchain', $10, $11, $12, $13, 0, $14, NOW(), $15
            FROM credit_deduction
            "#,
        )
        .bind::<diesel::sql_types::Text, _>(&wallet_address) // $1
        .bind::<diesel::sql_types::Numeric, _>(&(-credit_to_use.clone())) // $2 negative
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Uuid>, _>(Some(payment_id)) // $3
        .bind::<diesel::sql_types::Text, _>(&format!("Payment for plan {}", payload.plan_id)) // $4
        .bind::<diesel::sql_types::Jsonb, _>(serde_json::json!({
            "payment_reference": payment_reference,
            "plan_id": payload.plan_id,
            "tx_hash": payload.transaction_hash,
        })) // $5
        .bind::<diesel::sql_types::Uuid, _>(payment_id) // $6
        .bind::<diesel::sql_types::Text, _>(&payment_reference) // $7
        .bind::<diesel::sql_types::Numeric, _>(&payment_amount) // $8
        .bind::<diesel::sql_types::Text, _>(&payload.currency) // $9
        .bind::<diesel::sql_types::Text, _>(payment_status) // $10
        .bind::<diesel::sql_types::Uuid, _>(plan_uuid) // $11
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(tx_hash_value.as_ref()) // $12
        .bind::<diesel::sql_types::Text, _>(&network) // $13
        .bind::<diesel::sql_types::Jsonb, _>(&metadata) // $14
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>, _>(completed_at) // $15
        .execute(&mut conn)
        .await;

        match result {
            Ok(_) => {
                info!(
                    "Atomic credit+payment: ref={}, credits=${}, remaining=${}",
                    payment_reference, credit_to_use, remaining_amount
                );
            }
            Err(e) => {
                error!("Atomic credit+payment failed: {}", e);
                return Err(UnifiedErrorResponse::json(
                    500,
                    "Failed to submit transaction",
                    format!("Database error: {}", e),
                ));
            }
        }
    } else {
        // No credits - simple insert
        let result = diesel::sql_query(
            r#"
            INSERT INTO payments (
                id, payment_reference, wallet_address, amount, currency, method, status,
                plan_id, transaction_hash, network, confirmations, metadata, created_at, completed_at
            )
            VALUES ($1, $2, $3, $4, $5, 'blockchain', $6, $7, $8, $9, 0, $10, NOW(), $11)
            "#,
        )
        .bind::<diesel::sql_types::Uuid, _>(payment_id)
        .bind::<diesel::sql_types::Text, _>(&payment_reference)
        .bind::<diesel::sql_types::Text, _>(&wallet_address)
        .bind::<diesel::sql_types::Numeric, _>(&payment_amount)
        .bind::<diesel::sql_types::Text, _>(&payload.currency)
        .bind::<diesel::sql_types::Text, _>(payment_status)
        .bind::<diesel::sql_types::Uuid, _>(plan_uuid)
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(tx_hash_value.as_ref())
        .bind::<diesel::sql_types::Text, _>(&network)
        .bind::<diesel::sql_types::Jsonb, _>(&metadata)
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>, _>(completed_at)
        .execute(&mut conn)
        .await;

        if let Err(e) = result {
            error!("Failed to insert payment record: {}", e);
            return Err(UnifiedErrorResponse::json(
                500,
                "Failed to submit transaction",
                format!("Database error: {}", e),
            ));
        }
    }

    // Build response message
    let message = if payment_status == "confirmed" {
        info!(
            "Payment fully covered by credits: ref={}, amount=${}",
            payment_reference, payment_amount
        );

        // Async notification
        let notif_wallet = wallet_address.clone();
        let notif_ref = payment_reference.clone();
        let notif_state = _app_state.clone();
        tokio::spawn(async move {
            use crate::infrastructure::services::NotificationService;
            use crate::web::notifications::{NotificationPriority, NotificationType};
            let _ = NotificationService::send(
                &notif_state,
                &notif_wallet,
                NotificationType::Payment,
                NotificationPriority::Normal,
                "Payment Confirmed",
                "Your payment has been confirmed",
                Some(serde_json::json!({ "payment_reference": notif_ref })),
                None,
            )
            .await;
        });

        format!("Payment completed using ${} wallet credits", credit_to_use)
    } else if use_credits {
        info!(
            "Partial credit: ref={}, credits=${}, blockchain=${}",
            payment_reference, credit_to_use, remaining_amount
        );
        format!(
            "Applied ${} credits. Remaining ${} via blockchain",
            credit_to_use, remaining_amount
        )
    } else {
        info!(
            "Transaction submitted: ref={}, tx={}",
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
            transaction_hash: tx_hash_value
                .unwrap_or_else(|| "N/A (paid with credits)".to_string()),
        }),
    }))
}

/// Helper struct for balance query
#[derive(diesel::QueryableByName)]
struct BalanceRow {
    #[diesel(sql_type = diesel::sql_types::Numeric)]
    bal: BigDecimal,
}
