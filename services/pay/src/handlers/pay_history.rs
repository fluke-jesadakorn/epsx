//! Pay-history handler (slice-3).
//!
//! Returns the combined payment history for an address —
//! all `pay_intents` + `escrows` where the address is either
//! the payer or the payee. Used by the account page
//! (`apps/frontend` wave-12 port → `apps/pay` slice-4
//! migration).
//!
//! Endpoint:
//! - `GET /api/v1/pay/history/:address?status=…&limit=…&offset=…`

use axum::{
    extract::{Path as AxPath, State},
    http::StatusCode,
    Json,
};

use crate::types::*;
use crate::AppState;

pub async fn get_pay_history(
    State(state): State<AppState>,
    AxPath(address): AxPath<String>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Result<Json<PayHistoryResponse>, StatusCode> {
    let addr = address.to_lowercase();
    let status_filter = params.get("status").cloned();
    let limit: i64 = params.get("limit").and_then(|s| s.parse().ok()).unwrap_or(50);
    let offset: i64 = params.get("offset").and_then(|s| s.parse().ok()).unwrap_or(0);

    // Intents where address is payer or payee
    let mut intent_q = "SELECT id, chain_id, payer, payee, amount, token_address, status, escrow_id, tx_hash, description, expires_at, created_at, updated_at FROM pay_intents WHERE (payer = $1 OR payee = $1)".to_string();
    if let Some(s) = &status_filter {
        intent_q.push_str(&format!(" AND status = '{}'", s.replace('\'', "''")));
    }
    intent_q.push_str(&format!(" ORDER BY created_at DESC LIMIT {} OFFSET {}", limit, offset));

    let intents: Vec<PayIntent> = sqlx::query_as::<_, PayIntent>(&intent_q)
        .bind(&addr)
        .fetch_all(&state.db)
        .await
        .map_err(|e| {
            tracing::error!("pay_history intents fetch: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // Escrows where address is payer or payee
    let mut escrow_q = "SELECT id, chain_id, payer, payee, amount, token_address, fee_amount, status, on_chain_id, tx_hash, dispute_reason, created_at, updated_at FROM escrows WHERE payer = $1 OR payee = $1".to_string();
    if let Some(s) = &status_filter {
        escrow_q.push_str(&format!(" AND status = '{}'", s.replace('\'', "''")));
    }
    escrow_q.push_str(&format!(" ORDER BY created_at DESC LIMIT {} OFFSET {}", limit, offset));

    let escrows: Vec<EscrowRecord> = sqlx::query_as::<_, EscrowRecord>(&escrow_q)
        .bind(&addr)
        .fetch_all(&state.db)
        .await
        .map_err(|e| {
            tracing::error!("pay_history escrows fetch: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let total_intents: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM pay_intents WHERE payer = $1 OR payee = $1")
        .bind(&addr)
        .fetch_one(&state.db)
        .await
        .unwrap_or(0);

    let total_escrows: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM escrows WHERE payer = $1 OR payee = $1")
        .bind(&addr)
        .fetch_one(&state.db)
        .await
        .unwrap_or(0);

    Ok(Json(PayHistoryResponse {
        address: addr,
        intents,
        escrows,
        total_intents,
        total_escrows,
    }))
}