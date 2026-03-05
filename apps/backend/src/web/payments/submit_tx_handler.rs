//! Submit Transaction Handler
//!
//! Handles frontend submission of transaction hashes after MetaMask broadcast.
//! Creates a pending payment record and triggers background monitoring.
//! Credit deduction and payment insert are wrapped in a DB transaction.

use axum::{extract::State, response::Json, Extension};

use diesel::result::OptionalExtension;
use diesel_async::RunQueryDsl;
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info, warn};
use uuid::Uuid;
use bigdecimal::BigDecimal;
use std::str::FromStr;

use crate::{
    prelude::*,
    web::{
        auth::AppState,
        middleware::{OpenIDUserContext, UnifiedErrorResponse},
    },
    infrastructure::database::{get_payments_pool, get_diesel_pool},
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
) -> Result<Json<SubmitTransactionResponse>, UnifiedErrorResponse> {
    let wallet_address = user_context.wallet_address.clone();

    debug!("api/payments/submit HIT by wallet: {}", wallet_address);

    // H5: Rate limit — max 10 payment submissions per wallet per minute
    {
        use crate::web::middleware::rate_limiter::{UnifiedRateLimiter, RateLimitConfig, ClientId};
        let limiter = UnifiedRateLimiter::new(_app_state.cache.clone());
        let config = RateLimitConfig {
            requests_per_minute: Some(10),
            requests_per_hour: Some(60),
            requests_per_day: Some(200),
        };
        let client = ClientId::User(wallet_address.clone().into());
        match limiter.check_client_rate_limit(&client, "/api/payments/submit", "POST", &config).await {
            Ok(result) if !result.allowed => {
                return Err(UnifiedErrorResponse::new(
                    429,
                    "Too many requests",
                    "Payment submission rate limit exceeded. Please try again later.",
                ));
            }
            Err(e) => {
                error!("Rate limit check failed: {}", e);
                // Allow through on rate limiter failure to avoid blocking legitimate payments
            }
            _ => {}
        }
    }

    // Validate transaction hash format
    if !payload.transaction_hash.starts_with("0x") || payload.transaction_hash.len() != 66 {
        return Err(UnifiedErrorResponse::new(
            400,
            "Invalid transaction hash",
            "Transaction hash must be 66 characters starting with 0x",
        ));
    }

    // Parse plan_id as UUID
    let plan_uuid = Uuid::parse_str(&payload.plan_id)
        .map_err(|_| UnifiedErrorResponse::new(400, "Invalid plan ID", "Plan ID must be a valid UUID"))?;

    // Parse expected_amount: accept both f64 and string
    let amount_str = match &payload.expected_amount {
        serde_json::Value::Number(n) => n.to_string(),
        serde_json::Value::String(s) => s.clone(),
        _ => {
            return Err(UnifiedErrorResponse::new(
                400,
                "Invalid amount",
                "expected_amount must be a number or string",
            ));
        }
    };
    let payment_amount = BigDecimal::from_str(&amount_str).map_err(|_| {
        UnifiedErrorResponse::new(400, "Invalid amount", "Cannot parse expected_amount")
    })?;

    if payment_amount <= 0 {
        return Err(UnifiedErrorResponse::new(
            400,
            "Invalid amount",
            "Payment amount must be positive",
        ));
    }

    // Validate network
    let network = match payload.network.as_deref() {
        Some(n) if ["bsc-mainnet", "bsc-testnet", "localhost"].contains(&n) => n.to_string(),
        Some(_) => {
            return Err(UnifiedErrorResponse::new(
                400,
                "Invalid network",
                "Unsupported network. Must be bsc-mainnet, bsc-testnet, or localhost",
            ));
        }
        None => "unknown".to_string(),
    };

    // C3+C5: Server-side plan price & eligibility validation
    let primary_pool = get_diesel_pool().await.map_err(|e| {
        error!("Failed to get primary database pool: {}", e);
        UnifiedErrorResponse::new(500, "Database error", "Cannot connect to primary database")
    })?;
    let mut primary_conn = primary_pool.get().await.map_err(|e| {
        error!("Failed to get primary connection: {}", e);
        UnifiedErrorResponse::new(500, "Database error", "Cannot establish primary database connection")
    })?;

    #[derive(diesel::QueryableByName)]
    struct PlanCheck {
        #[diesel(sql_type = diesel::sql_types::Numeric)]
        price: BigDecimal,
        #[diesel(sql_type = diesel::sql_types::Bool)]
        is_active: bool,
        #[diesel(sql_type = diesel::sql_types::Text)]
        plan_type: String,
        #[diesel(sql_type = diesel::sql_types::Jsonb)]
        plan_metadata: serde_json::Value,
    }

    let plan_check: Option<PlanCheck> = diesel::sql_query(
        "SELECT COALESCE(price, 0) as price, is_active, COALESCE(plan_type, 'subscription') as plan_type, COALESCE(plan_metadata, '{}'::jsonb) as plan_metadata FROM plans WHERE id = $1"
    )
    .bind::<diesel::sql_types::Uuid, _>(plan_uuid)
    .get_result(&mut primary_conn)
    .await
    .optional()
    .map_err(|e| {
        error!("Failed to query plan: {}", e);
        UnifiedErrorResponse::new(500, "Database error", "Failed to verify plan")
    })?;

    let plan_info = plan_check.ok_or_else(|| {
        UnifiedErrorResponse::new(404, "Plan not found", "The specified plan does not exist")
    })?;

    // C5: Check plan eligibility
    if !plan_info.is_active {
        return Err(UnifiedErrorResponse::new(
            403,
            "Plan unavailable",
            "This plan is not currently available for purchase",
        ));
    }

    if plan_info.plan_type == "system" {
        return Err(UnifiedErrorResponse::new(
            403,
            "Plan unavailable",
            "This plan cannot be purchased directly",
        ));
    }

    // C3: Validate amount matches plan price (allow 1% tolerance for rounding)
    // Check both base price and promotional price (if promotion is active)
    let base_price = &plan_info.price;
    let effective_price = plan_info.plan_metadata.get("promotion")
        .and_then(|promo_val| {
            serde_json::from_value::<crate::domain::subscription_management::promotion::Promotion>(promo_val.clone()).ok()
        })
        .map(|promo| {
            let bp = base_price.to_string().parse::<f64>().unwrap_or(0.0);
            let ep = promo.calculate_effective_price(bp);
            BigDecimal::from_str(&format!("{:.2}", ep)).unwrap_or_else(|_| base_price.clone())
        });

    let price_to_validate = effective_price.as_ref().unwrap_or(base_price);
    let price_diff = (&payment_amount - price_to_validate).abs();
    let tolerance = price_to_validate * BigDecimal::from_str("0.01").unwrap_or_else(|_| BigDecimal::from(0));
    if price_diff > tolerance && *price_to_validate > 0 {
        // Also check against base price in case promotion just expired
        let base_diff = (&payment_amount - base_price).abs();
        let base_tolerance = base_price * BigDecimal::from_str("0.01").unwrap_or_else(|_| BigDecimal::from(0));
        if base_diff > base_tolerance && *base_price > 0 {
            error!(
                "Amount mismatch: submitted={}, plan_price={}, effective_price={:?}, plan_id={}",
                payment_amount, plan_info.price, effective_price, plan_uuid
            );
            return Err(UnifiedErrorResponse::new(
                400,
                "Amount mismatch",
                "Payment amount does not match plan price",
            ));
        }
    }

    // Get payments database connection
    let payments_pool = get_payments_pool().await.map_err(|e| {
        error!("Failed to get payments database pool: {}", e);
        UnifiedErrorResponse::new(500, "Database error", "Cannot connect to database")
    })?;

    let mut conn = payments_pool.get().await.map_err(|e| {
        error!("Failed to get database connection: {}", e);
        UnifiedErrorResponse::new(500, "Database error", "Cannot establish database connection")
    })?;

    // Atomic dedup check — the UNIQUE constraint on transaction_hash prevents races.
    // We still do a quick SELECT first for the fast-path (idempotent retry).
    #[derive(diesel::QueryableByName)]
    struct DedupRow {
        #[diesel(sql_type = diesel::sql_types::Text)]
        payment_reference: String,
        #[diesel(sql_type = diesel::sql_types::Text)]
        status: String,
    }

    let existing: Result<Option<DedupRow>, _> = diesel::sql_query(
        "SELECT payment_reference, status FROM payments WHERE transaction_hash = $1 AND LOWER(wallet_address) = LOWER($2) LIMIT 1",
    )
    .bind::<diesel::sql_types::Text, _>(&payload.transaction_hash)
    .bind::<diesel::sql_types::Text, _>(&wallet_address)
    .get_result::<DedupRow>(&mut conn)
    .await
    .optional();

    if let Ok(Some(row)) = existing {
        info!("Transaction already submitted: {} (status={})", row.payment_reference, row.status);
        return Ok(Json(SubmitTransactionResponse {
            success: true,
            message: "Transaction already being monitored".to_string(),
            data: Some(SubmitTransactionData {
                payment_reference: row.payment_reference,
                status: row.status,
                transaction_hash: payload.transaction_hash,
            }),
        }));
    }

    let payment_reference = format!("PAY-{}", Uuid::new_v4());
    let payment_id = Uuid::new_v4();

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

    let payment_status = if remaining_amount <= 0 {
        "confirmed"
    } else {
        "pending"
    };

    let tx_hash_value = if remaining_amount <= 0 {
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
    let use_credits = credit_to_use > 0;

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
                return Err(UnifiedErrorResponse::new(
                    500,
                    "Failed to submit transaction",
                    format!("Database error: {}", e),
                ));
            }
        }
    } else {
        // No credits - insert with ON CONFLICT guard against race condition
        let result = diesel::sql_query(
            r#"
            INSERT INTO payments (
                id, payment_reference, wallet_address, amount, currency, method, status,
                plan_id, transaction_hash, network, confirmations, metadata, created_at, completed_at
            )
            VALUES ($1, $2, $3, $4, $5, 'blockchain', $6, $7, $8, $9, 0, $10, NOW(), $11)
            ON CONFLICT (transaction_hash) WHERE transaction_hash IS NOT NULL DO NOTHING
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

        match result {
            Ok(_) => { /* insert succeeded or already exists */ }
            Err(e) => {
                // If it's a unique violation on transaction_hash, it means another user 
                // (or the same one) already submitted it. We gracefully handle this 
                // by returning success for the *calling* user, but the tx_monitor 
                // will only validate the row where the on-chain from == wallet_address.
                // NOTE: If the DB has a strict UNIQUE constraint on (transaction_hash), 
                // `ON CONFLICT DO NOTHING` prevents an error.
                // We'll query to see if THIS user has a row. If they don't, 
                // the attacker successfully locked the row using the UNIQUE constraint.
                // To truly fix this at the DB level, the unique constraint must be on 
                // (transaction_hash, wallet_address), OR transaction_hash can't be unique.
                // Assuming standard unique constraint, the fallback is:
                let dup: Option<DedupRow> = diesel::sql_query(
                    "SELECT payment_reference, status FROM payments WHERE transaction_hash = $1 AND LOWER(wallet_address) = LOWER($2) LIMIT 1",
                )
                .bind::<diesel::sql_types::Text, _>(&payload.transaction_hash)
                .bind::<diesel::sql_types::Text, _>(&wallet_address)
                .get_result::<DedupRow>(&mut conn)
                .await
                .optional()
                .ok()
                .flatten();

                if let Some(row) = dup {
                    return Ok(Json(SubmitTransactionResponse {
                        success: true,
                        message: "Transaction already being monitored".to_string(),
                        data: Some(SubmitTransactionData {
                            payment_reference: row.payment_reference,
                            status: row.status,
                            transaction_hash: payload.transaction_hash,
                        }),
                    }));
                } else if format!("{}", e).contains("duplicate key") || format!("{}", e).contains("Unique violation") {
                    // Attack scenario: Attacker submitted it first. The UNIQUE constraint blocked the legitimate user.
                    // If a transaction hash already exists for a different wallet, we MUST NOT gracefully adopt it
                    // by overwriting the wallet address, as that allows an attacker to hijack a pending payment.
                    // We return a 409 Conflict indicating it's already in progress.
                    warn!("Transaction hash {} already exists for a different wallet. Preventing overwrite DoS.", payload.transaction_hash);
                    
                    return Err(UnifiedErrorResponse::new(
                        409,
                        "Transaction Conflict",
                        "This transaction is already being processed by another account.",
                    ));
                }

                error!("Failed to insert payment record: {}", e);
                return Err(UnifiedErrorResponse::new(
                    500,
                    "Failed to submit transaction",
                    format!("Database error: {}", e),
                ));
            }
        }
    }

    // Fix 1: Assign plan immediately for credit-only payments
    if payment_status == "confirmed" {
        #[derive(diesel::QueryableByName)]
        struct CreditAssignment {
            #[diesel(sql_type = diesel::sql_types::Uuid)]
            id: Uuid,
            #[diesel(sql_type = diesel::sql_types::Timestamptz)]
            expires_at: chrono::DateTime<chrono::Utc>,
            #[diesel(sql_type = diesel::sql_types::Bool)]
            is_active: bool,
        }

        // Deactivate other active subscription plans
        diesel::sql_query(
            r#"
            UPDATE wallet_plan_assignments
            SET is_active = false, updated_at = NOW()
            WHERE LOWER(wallet_address) = LOWER($1)
              AND is_active = true
              AND plan_id != $2
              AND plan_id IN (SELECT id FROM plans WHERE plan_type = 'subscription')
            "#,
        )
        .bind::<diesel::sql_types::Text, _>(&wallet_address)
        .bind::<diesel::sql_types::Uuid, _>(plan_uuid)
        .execute(&mut conn)
        .await
        .ok();

        let existing_assign: Option<CreditAssignment> = diesel::sql_query(
            "SELECT id, expires_at, is_active FROM wallet_plan_assignments WHERE LOWER(wallet_address) = LOWER($1) AND plan_id = $2 ORDER BY is_active DESC, expires_at DESC LIMIT 1"
        )
        .bind::<diesel::sql_types::Text, _>(&wallet_address)
        .bind::<diesel::sql_types::Uuid, _>(plan_uuid)
        .get_result(&mut conn)
        .await
        .optional()
        .ok()
        .flatten();

        let now = chrono::Utc::now();
        if let Some(existing) = existing_assign {
            let base = if existing.is_active && existing.expires_at > now { existing.expires_at } else { now };
            let new_expiry = base + chrono::Duration::days(30);
            diesel::sql_query(
                r#"
                UPDATE wallet_plan_assignments
                SET expires_at = $1, payment_reference = $2, updated_at = NOW(), is_active = true
                WHERE id = $3
                "#,
            )
            .bind::<diesel::sql_types::Timestamptz, _>(new_expiry)
            .bind::<diesel::sql_types::Text, _>(&payment_reference)
            .bind::<diesel::sql_types::Uuid, _>(existing.id)
            .execute(&mut conn)
            .await
            .ok();
            info!("Extended/reactivated plan {} for wallet {} via credits until {}", plan_uuid, wallet_address, new_expiry);
        } else {
            let new_expiry = now + chrono::Duration::days(30);
            diesel::sql_query(
                r#"
                INSERT INTO wallet_plan_assignments (
                    wallet_address, plan_id, assigned_at, expires_at, is_active,
                    assignment_source, assignment_reason, payment_reference,
                    auto_renew, assignment_metadata
                )
                VALUES ($1, $2, NOW(), $3, true, 'credit', 'Plan purchase via wallet credits', $4, false, '{}')
                "#,
            )
            .bind::<diesel::sql_types::Text, _>(&wallet_address)
            .bind::<diesel::sql_types::Uuid, _>(plan_uuid)
            .bind::<diesel::sql_types::Timestamptz, _>(new_expiry)
            .bind::<diesel::sql_types::Text, _>(&payment_reference)
            .execute(&mut conn)
            .await
            .ok();
            info!("Created plan assignment for wallet {} → plan {} via credits (expires: {})", wallet_address, plan_uuid, new_expiry);
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
