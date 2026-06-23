//! Intent CRUD handlers for epsx-pay-svc.
//!
//! wave49(slice-3): extracted from main.rs as part of the
//! services/pay modularization. All endpoints here match
//! the pre-modularization behavior 1:1 — this is a pure
//! refactor (extract functions to module, no behavior change).
//!
//! Endpoints (in this module):
//! - `POST /api/v1/pay/intents`     → `create_pay_intent`
//! - `GET  /api/v1/pay/intents`     → `list_pay_intents`
//! - `GET  /api/v1/pay/intents/:id` → `get_pay_intent`
//! - `POST /api/v1/pay/intents/:id/confirm` → `confirm_pay_intent`
//! - `POST /api/v1/pay/intents/:id/cancel`  → `cancel_pay_intent`

use axum::{
    extract::{Path as AxPath, State},
    http::StatusCode,
    Json,
};
use epsx_kernel::{ChainId, Token};
use std::str::FromStr;
use tracing;

use crate::db::compute_fee;
use crate::types::*;
use crate::AppState;

// ============================================================================
// POST /api/v1/pay/intents
// ============================================================================

pub async fn create_pay_intent(
    State(state): State<AppState>,
    Json(req): Json<CreatePayIntentRequest>,
) -> Result<Json<PayIntentResponse>, StatusCode> {
    // Validate addresses
    let _payer = alloy::primitives::Address::from_str(&req.payer).map_err(|_| StatusCode::BAD_REQUEST)?;
    let _payee = alloy::primitives::Address::from_str(&req.payee).map_err(|_| StatusCode::BAD_REQUEST)?;

    // Resolve token address
    let chain_id_enum = ChainId(state.chain_id);
    let token_symbol = req.token.to_uppercase();
    let token = match token_symbol.as_str() {
        "USDT" => Token::USDT,
        "USDC" => Token::USDC,
        "BNB" => Token::BNB,
        _ => return Err(StatusCode::BAD_REQUEST),
    };
    let token_address = token.address(chain_id_enum)
        .map(|a| a.0)
        .unwrap_or_else(|| "0x0000000000000000000000000000000000000000".to_string());

    let id = format!("0x{}", uuid::Uuid::new_v4().simple());
    let now = chrono::Utc::now();
    let expires = req.expires_in.unwrap_or(3600);
    let expires_at = chrono::Utc::now()
        .checked_add_signed(chrono::Duration::seconds(expires))
        .unwrap();

    sqlx::query(
        "INSERT INTO pay_intents (id, chain_id, payer, payee, amount, token_address, status, description, expires_at, created_at, updated_at)
         VALUES ($1, $2, $3, $4, $5, $6, 'pending', $7, $8, $9, $9)"
    )
    .bind(&id)
    .bind(state.chain_id.to_string())
    .bind(&req.payer.to_lowercase())
    .bind(&req.payee.to_lowercase())
    .bind(&req.amount)
    .bind(&token_address)
    .bind(&req.description)
    .bind(expires_at)
    .bind(now)
    .execute(&state.db)
    .await
    .map_err(|e| { tracing::error!("pay intent insert: {}", e); StatusCode::INTERNAL_SERVER_ERROR })?;

    let intent = PayIntent {
        id: id.clone(),
        chain_id: state.chain_id.to_string(),
        payer: req.payer.to_lowercase(),
        payee: req.payee.to_lowercase(),
        amount: req.amount.clone(),
        token_address,
        status: "pending".to_string(),
        escrow_id: None,
        tx_hash: None,
        description: req.description,
        expires_at: Some(expires_at),
        created_at: now,
        updated_at: now,
    };

    let pay_url = format!("/pay?intent={}", id);

    Ok(Json(PayIntentResponse {
        intent,
        pay_url,
        expires_at,
    }))
}

// ============================================================================
// GET /api/v1/pay/intents
// ============================================================================

pub async fn list_pay_intents(
    State(state): State<AppState>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Result<Json<PayIntentListResponse>, StatusCode> {
    let payer = params.get("payer").cloned();
    let status = params.get("status").cloned();
    let limit: i64 = params.get("limit").and_then(|s| s.parse().ok()).unwrap_or(50);
    let offset: i64 = params.get("offset").and_then(|s| s.parse().ok()).unwrap_or(0);

    let mut q = "SELECT id, chain_id, payer, payee, amount, token_address, status, escrow_id, tx_hash, description, expires_at, created_at, updated_at FROM pay_intents WHERE 1=1".to_string();
    let mut args: Vec<String> = vec![];
    if let Some(p) = &payer {
        args.push(p.clone());
        q.push_str(&format!(" AND payer = ${}", args.len()));
    }
    if let Some(s) = &status {
        args.push(s.clone());
        q.push_str(&format!(" AND status = ${}", args.len()));
    }
    q.push_str(&format!(" ORDER BY created_at DESC LIMIT {} OFFSET {}", limit, offset));

    let mut query = sqlx::query_as::<_, PayIntent>(&q);
    for a in &args { query = query.bind(a); }
    let items: Vec<PayIntent> = query.fetch_all(&state.db).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let total: i64 = if args.is_empty() {
        sqlx::query_scalar("SELECT COUNT(*) FROM pay_intents").fetch_one(&state.db).await.unwrap_or(0)
    } else {
        let mut q2 = "SELECT COUNT(*) FROM pay_intents WHERE 1=1".to_string();
        if let Some(_p) = &payer { q2.push_str(" AND payer = $1"); }
        if let Some(_s) = &status {
            let idx = if payer.is_some() { 2 } else { 1 };
            q2.push_str(&format!(" AND status = ${}", idx));
        }
        let mut query2 = sqlx::query_scalar(&q2);
        for a in &args { query2 = query2.bind(a); }
        query2.fetch_one(&state.db).await.unwrap_or(0)
    };

    Ok(Json(PayIntentListResponse { items, total }))
}

// ============================================================================
// GET /api/v1/pay/intents/:id
// ============================================================================

pub async fn get_pay_intent(
    State(state): State<AppState>,
    AxPath(id): AxPath<String>,
) -> Result<Json<PayIntent>, StatusCode> {
    let intent: PayIntent = sqlx::query_as::<_, PayIntent>(
        "SELECT id, chain_id, payer, payee, amount, token_address, status, escrow_id, tx_hash, description, expires_at, created_at, updated_at FROM pay_intents WHERE id = $1"
    )
    .bind(&id)
    .fetch_optional(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::NOT_FOUND)?;
    Ok(Json(intent))
}

// ============================================================================
// POST /api/v1/pay/intents/:id/confirm
// ============================================================================

pub async fn confirm_pay_intent(
    State(state): State<AppState>,
    AxPath(id): AxPath<String>,
    Json(req): Json<serde_json::Value>,
) -> Result<Json<PayIntent>, StatusCode> {
    let tx_hash = req.get("tx_hash").and_then(|v| v.as_str()).unwrap_or_default().to_string();
    if tx_hash.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    // Create escrow record for this payment
    let intent: PayIntent = sqlx::query_as::<_, PayIntent>(
        "SELECT id, chain_id, payer, payee, amount, token_address, status, escrow_id, tx_hash, description, expires_at, created_at, updated_at FROM pay_intents WHERE id = $1"
    )
    .bind(&id)
    .fetch_optional(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::NOT_FOUND)?;

    if intent.status != "pending" {
        return Err(StatusCode::CONFLICT);
    }

    let escrow_id = format!("0x{}", uuid::Uuid::new_v4().simple());
    let fee = compute_fee(&intent.amount);

    sqlx::query(
        "INSERT INTO escrows (id, chain_id, payer, payee, amount, token_address, fee_amount, status, tx_hash) VALUES ($1, $2, $3, $4, $5, $6, $7, 'active', $8)"
    )
    .bind(&escrow_id)
    .bind(&intent.chain_id)
    .bind(&intent.payer)
    .bind(&intent.payee)
    .bind(&intent.amount)
    .bind(&intent.token_address)
    .bind(&fee)
    .bind(&tx_hash)
    .execute(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    sqlx::query("UPDATE pay_intents SET status = 'escrowed', escrow_id = $1, tx_hash = $2, updated_at = NOW() WHERE id = $3")
        .bind(&escrow_id)
        .bind(&tx_hash)
        .bind(&id)
        .execute(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    get_pay_intent(State(state), AxPath(id)).await
}

// ============================================================================
// POST /api/v1/pay/intents/:id/cancel
// ============================================================================

pub async fn cancel_pay_intent(
    State(state): State<AppState>,
    AxPath(id): AxPath<String>,
) -> Result<Json<PayIntent>, StatusCode> {
    sqlx::query("UPDATE pay_intents SET status = 'cancelled', updated_at = NOW() WHERE id = $1 AND status = 'pending'")
        .bind(&id)
        .execute(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    get_pay_intent(State(state), AxPath(id)).await
}