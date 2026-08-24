// BIG-BANG: migrated to sqlx (real).

use std::str::FromStr;

use axum::{
    extract::{Extension, State},
    http::StatusCode,
    Json,
};
use bigdecimal::BigDecimal;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::sync::Arc;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::prelude::*;
use crate::web::middleware::{OpenIDUserContext, UnifiedErrorResponse};
use epsx_contracts::state::AppState;

#[derive(Debug, Deserialize)]
pub struct SubmitTransactionRequest {
    pub plan_id: String,
    #[serde(default)]
    pub transaction_hash: String,
    #[serde(default)]
    pub expected_amount: serde_json::Value,
    #[serde(default = "default_currency")]
    pub currency: String,
    #[serde(default)]
    pub network: Option<String>,
}

fn default_currency() -> String {
    "USD".to_string()
}

#[derive(Debug, Serialize)]
pub struct SubmitTransactionData {
    pub payment_reference: String,
    pub status: String,
    pub transaction_hash: String,
}

#[derive(Debug, Serialize)]
pub struct SubmitTransactionResponse {
    pub success: bool,
    pub message: String,
    pub data: Option<SubmitTransactionData>,
}

/// Submit a transaction hash for backend monitoring.
/// Credit deduction + payment record insert are atomic (single DB transaction).
#[axum::debug_handler]
pub async fn submit_transaction_handler(
    State(app_state): State<AppState>,
    Extension(user_context): Extension<OpenIDUserContext>,
    Json(payload): Json<SubmitTransactionRequest>,
) -> Result<Json<SubmitTransactionResponse>, UnifiedErrorResponse> {
    let wallet_address = user_context.wallet_address.clone();

    debug!("api/payments/submit HIT by wallet: {}", wallet_address);

    // H5: Rate limit — max 10 payment submissions per wallet per minute
    {
        use crate::web::middleware::rate_limiter::{ClientId, RateLimitConfig, UnifiedRateLimiter};
        let limiter = UnifiedRateLimiter::new(app_state.cache.clone());
        let config = RateLimitConfig {
            requests_per_minute: Some(10),
            requests_per_hour: Some(60),
            requests_per_day: Some(200),
        };
        let client = ClientId::User(wallet_address.clone().into());
        match limiter
            .check_client_rate_limit(&client, "/api/payments/submit", "POST", &config)
            .await
        {
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
    let plan_uuid = Uuid::parse_str(&payload.plan_id).map_err(|_| {
        UnifiedErrorResponse::new(400, "Invalid plan ID", "Plan ID must be a valid UUID")
    })?;

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
    let payment_repo = app_state.payment_repo.as_ref().ok_or_else(|| {
        error!(
            "PaymentRepositoryPort not wired in AppState — wave 11 track A scaffolding incomplete"
        );
        UnifiedErrorResponse::new(500, "Internal error", "Payment service is not initialized")
    })?;
    let wallet_address_vo =
        crate::domain::wallet_management::value_objects::WalletAddress::new(wallet_address.clone())
            .map_err(|e| {
                UnifiedErrorResponse::new(
                    400,
                    "Invalid wallet",
                    format!("Wallet address is invalid: {}", e),
                )
            })?;
    let plan_info = payment_repo
        .validate_submit_tx(plan_uuid, &wallet_address_vo)
        .await
        .map_err(|e| {
            error!("Failed to validate plan via PaymentRepositoryPort: {}", e);
            UnifiedErrorResponse::new(
                500,
                "Database error",
                format!("Failed to verify plan: {}", e),
            )
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

    // C3: Validate amount matches plan price (allow 5% tolerance for rounding & promotion edge cases)
    let base_price = BigDecimal::from_str(&plan_info.plan_price).map_err(|_| {
        UnifiedErrorResponse::new(500, "Database error", "Plan price format invalid")
    })?;
    let effective_price_decimal = BigDecimal::from_str(&plan_info.effective_price).ok();
    let effective_price = plan_info
        .plan_metadata
        .get("promotion")
        .and_then(|promo_val| {
            serde_json::from_value::<crate::domain::subscription_management::promotion::Promotion>(
                promo_val.clone(),
            )
            .ok()
        })
        .map(|promo| {
            let bp = base_price.to_string().parse::<f64>().unwrap_or(0.0);
            let ep = promo.calculate_effective_price(bp);
            BigDecimal::from_str(&format!("{:.2}", ep)).unwrap_or_else(|_| base_price.clone())
        })
        .or(effective_price_decimal);

    let price_to_validate = effective_price.as_ref().unwrap_or(&base_price);
    let price_diff = (&payment_amount - price_to_validate).abs();
    let tolerance = price_to_validate
        * BigDecimal::from_str("0.05").unwrap_or_else(|_| BigDecimal::from(0));
    if price_diff > tolerance && *price_to_validate > 0 {
        let base_diff = (&payment_amount - &base_price).abs();
        let base_tolerance = &base_price
            * BigDecimal::from_str("0.05").unwrap_or_else(|_| BigDecimal::from(0));
        if base_diff > base_tolerance && base_price > 0 {
            error!(
                "Amount mismatch: submitted={}, plan_price={}, effective_price={:?}, plan_id={}, tolerance={}%",
                payment_amount, plan_info.plan_price, effective_price, plan_uuid, 5
            );
            return Err(UnifiedErrorResponse::new(
                400,
                "Amount mismatch",
                "Payment amount does not match plan price. Please refresh and try again.",
            ));
        }
    }

    // Get payments database connection
    let payments_pool = app_state.db_pool.clone();

    // Atomic dedup check (idempotent retry)
    #[derive(sqlx::FromRow)]
    struct DedupRow {
        payment_reference: String,
        status: String,
    }

    let existing: Option<DedupRow> = sqlx::query_as(
        "SELECT payment_reference, status FROM payments WHERE transaction_hash = $1 AND LOWER(wallet_address) = LOWER($2) LIMIT 1",
    )
    .bind(&payload.transaction_hash)
    .bind(&wallet_address)
    .fetch_optional(payments_pool.as_ref())
    .await
    .map_err(|e| {
        error!("Failed to check for existing transaction: {}", e);
        UnifiedErrorResponse::new(500, "Database error", "Failed to query for duplicate")
    })?;

    if let Some(row) = existing {
        info!(
            "Transaction already submitted: {} (status={})",
            row.payment_reference, row.status
        );
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
    #[derive(sqlx::FromRow)]
    struct BalanceRow {
        bal: BigDecimal,
    }

    let wallet_credit_balance: BigDecimal = sqlx::query_as::<_, BalanceRow>(
        "SELECT COALESCE((SELECT balance FROM wallet_credits WHERE wallet_address = $1), 0)::numeric as bal",
    )
    .bind(&wallet_address)
    .fetch_optional(payments_pool.as_ref())
    .await
    .ok()
    .flatten()
    .map(|r| r.bal)
    .unwrap_or_else(|| BigDecimal::from(0));

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
        Some(Utc::now())
    } else {
        None
    };

    let metadata = serde_json::json!({
        "credit_used": credit_to_use.to_string(),
        "original_amount": payment_amount.to_string(),
        "blockchain_amount": remaining_amount.to_string(),
    });

    // Atomic transaction: credit deduction + payment insert
    let use_credits = credit_to_use > 0;

    if use_credits {
        let result = sqlx::query(
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
        .bind(&wallet_address)
        .bind(-credit_to_use.clone()) // negative amount for credit deduction
        .bind(payment_id)
        .bind(format!("Payment for plan {}", payload.plan_id))
        .bind(serde_json::json!({
            "payment_reference": payment_reference,
            "plan_id": payload.plan_id,
            "tx_hash": payload.transaction_hash,
        }))
        .bind(payment_id)
        .bind(&payment_reference)
        .bind(&payment_amount)
        .bind(&payload.currency)
        .bind(payment_status)
        .bind(plan_uuid)
        .bind(tx_hash_value.as_ref())
        .bind(&network)
        .bind(&metadata)
        .bind(completed_at)
        .execute(payments_pool.as_ref())
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
        let result = sqlx::query(
            r#"
            INSERT INTO payments (
                id, payment_reference, wallet_address, amount, currency, method, status,
                plan_id, transaction_hash, network, confirmations, metadata, created_at, completed_at
            )
            VALUES ($1, $2, $3, $4, $5, 'blockchain', $6, $7, $8, $9, 0, $10, NOW(), $11)
            ON CONFLICT (transaction_hash) WHERE transaction_hash IS NOT NULL DO NOTHING
            "#,
        )
        .bind(payment_id)
        .bind(&payment_reference)
        .bind(&wallet_address)
        .bind(&payment_amount)
        .bind(&payload.currency)
        .bind(payment_status)
        .bind(plan_uuid)
        .bind(tx_hash_value.as_ref())
        .bind(&network)
        .bind(&metadata)
        .bind(completed_at)
        .execute(payments_pool.as_ref())
        .await;

        match result {
            Ok(_) => { /* insert succeeded or already exists */ }
            Err(e) => {
                let dup: Option<DedupRow> = sqlx::query_as(
                    "SELECT payment_reference, status FROM payments WHERE transaction_hash = $1 AND LOWER(wallet_address) = LOWER($2) LIMIT 1",
                )
                .bind(&payload.transaction_hash)
                .bind(&wallet_address)
                .fetch_optional(payments_pool.as_ref())
                .await
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
                } else if format!("{}", e).contains("duplicate key")
                    || format!("{}", e).contains("Unique violation")
                {
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
        let primary_pool = app_state.db_pool.clone();
        let primary_conn = primary_pool.clone();

        #[derive(sqlx::FromRow)]
        struct CreditPlanTerms {
            plan_metadata: serde_json::Value,
            billing_cycle: Option<String>,
        }

        let plan_terms: CreditPlanTerms = sqlx::query_as(
            "SELECT plan_metadata, billing_cycle FROM plans WHERE id = $1 AND is_active = true",
        )
        .bind(plan_uuid)
        .fetch_one(primary_conn.as_ref())
        .await
        .map_err(|e| {
            error!("Unable to load plan terms for credit assignment: {}", e);
            UnifiedErrorResponse::new(500, "Database error", "Cannot assign purchased plan")
        })?;

        #[derive(sqlx::FromRow)]
        struct CreditAssignment {
            id: Uuid,
            expires_at: Option<chrono::DateTime<chrono::Utc>>,
            is_active: bool,
        }

        // Deactivate other active subscription plans
        sqlx::query(
            r#"
            UPDATE wallet_plan_assignments
            SET is_active = false, updated_at = NOW()
            WHERE LOWER(wallet_address) = LOWER($1)
              AND is_active = true
              AND plan_id != $2
              AND plan_id IN (SELECT id FROM plans WHERE plan_type = 'subscription')
            "#,
        )
        .bind(&wallet_address)
        .bind(plan_uuid)
        .execute(primary_conn.as_ref())
        .await
        .map_err(|e| {
            error!("Unable to deactivate previous plan assignment: {}", e);
            UnifiedErrorResponse::new(500, "Database error", "Cannot assign purchased plan")
        })?;

        let existing_assign: Option<CreditAssignment> = sqlx::query_as(
            "SELECT id, expires_at, is_active FROM wallet_plan_assignments WHERE LOWER(wallet_address) = LOWER($1) AND plan_id = $2 ORDER BY is_active DESC, expires_at DESC LIMIT 1",
        )
        .bind(&wallet_address)
        .bind(plan_uuid)
        .fetch_optional(primary_conn.as_ref())
        .await
        .map_err(|e| {
            error!("Unable to inspect existing plan assignment: {}", e);
            UnifiedErrorResponse::new(500, "Database error", "Cannot assign purchased plan")
        })?;

        let now = chrono::Utc::now();
        if let Some(existing) = existing_assign {
            let new_expiry = plan_assignment_expiry(
                now,
                &plan_terms.plan_metadata,
                plan_terms.billing_cycle.as_deref().unwrap_or_default(),
                existing.is_active.then_some(existing.expires_at).flatten(),
            );
            sqlx::query(
                r#"
                UPDATE wallet_plan_assignments
                SET expires_at = $1, payment_reference = $2, updated_at = NOW(), is_active = true
                WHERE id = $3
                "#,
            )
            .bind(new_expiry)
            .bind(&payment_reference)
            .bind(existing.id)
            .execute(primary_conn.as_ref())
            .await
            .map_err(|e| {
                error!("Unable to extend purchased plan assignment: {}", e);
                UnifiedErrorResponse::new(500, "Database error", "Cannot assign purchased plan")
            })?;
            info!(
                "Extended/reactivated plan {} for wallet {} via credits until {:?}",
                plan_uuid, wallet_address, new_expiry
            );
        } else {
            let new_expiry = plan_assignment_expiry(
                now,
                &plan_terms.plan_metadata,
                plan_terms.billing_cycle.as_deref().unwrap_or_default(),
                None,
            );
            sqlx::query(
                r#"
                INSERT INTO wallet_plan_assignments (
                    wallet_address, plan_id, assigned_at, expires_at, is_active,
                    assignment_source, assignment_reason, payment_reference,
                    auto_renew, assignment_metadata
                )
                VALUES ($1, $2, NOW(), $3, true, 'credit', 'Plan purchase via wallet credits', $4, false, '{}')
                "#,
            )
            .bind(&wallet_address)
            .bind(plan_uuid)
            .bind(new_expiry)
            .bind(&payment_reference)
            .execute(primary_conn.as_ref())
            .await
            .map_err(|e| {
                error!("Unable to create purchased plan assignment: {}", e);
                UnifiedErrorResponse::new(500, "Database error", "Cannot assign purchased plan")
            })?;
            info!(
                "Created plan assignment for wallet {} → plan {} via credits (expires: {:?})",
                wallet_address, plan_uuid, new_expiry
            );
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
        let notif_state = app_state.clone();
        tokio::spawn(async move {
            // Wave 10 / R3: route through the NotificationPort.
            use epsx_contracts::notification_port::SendNotificationRequest;
            if let Some(port) = notif_state.notification_port.as_ref() {
                let _ = port
                    .send_with_event_id_retry(
                        &format!("payment.confirmed:{notif_ref}"),
                        SendNotificationRequest {
                            recipient_wallet_address: notif_wallet.clone(),
                            notification_type: "payment".to_string(),
                            priority: "normal".to_string(),
                            title: "Payment Confirmed".to_string(),
                            message: "Your payment has been confirmed".to_string(),
                            data: Some(serde_json::json!({ "payment_reference": notif_ref })),
                            action_url: None,
                            expires_at: None,
                        },
                    )
                    .await;
            } else {
                tracing::warn!(
                    "notification_port not wired in AppState; payment-confirmed \
                     notification for wallet={} dropped",
                    notif_wallet
                );
            }
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

// Re-export the plan_assignment_expiry helper from the upgrade service module
use super::upgrade_service::plan_assignment_expiry;
