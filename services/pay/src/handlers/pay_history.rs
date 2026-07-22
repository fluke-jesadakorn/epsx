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
    extract::{Extension, Path as AxPath, State},
    http::StatusCode,
    Json,
};
use epsx_pay_svc::canonical_owner;
use epsx_service_auth::VerifiedPrincipal;

use crate::types::*;
use crate::AppState;

pub async fn get_pay_history(
    State(state): State<AppState>,
    Extension(principal): Extension<VerifiedPrincipal>,
    AxPath(address): AxPath<String>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Result<Json<PayHistoryResponse>, StatusCode> {
    let addr = canonical_owner(&principal, Some(&address))?;
    let status_filter = params.get("status").cloned();
    let limit = params
        .get("limit")
        .and_then(|value| value.parse::<i64>().ok())
        .unwrap_or(50)
        .clamp(1, 100);
    let offset = params
        .get("offset")
        .and_then(|value| value.parse::<i64>().ok())
        .unwrap_or(0)
        .max(0);

    // Intents where address is payer or payee
    let intents: Vec<PayIntent> = if let Some(status) = status_filter.as_deref() {
        sqlx::query_as::<_, PayIntent>(
            "SELECT id, chain_id, payer, payee, amount, token_address, status, escrow_id, tx_hash, description, expires_at, created_at, updated_at
             FROM public.pay_intents
             WHERE (payer = $1 OR payee = $1) AND status = $2
             ORDER BY created_at DESC LIMIT $3 OFFSET $4",
        )
        .bind(&addr)
        .bind(status)
        .bind(limit)
        .bind(offset)
        .fetch_all(&state.db)
        .await
    } else {
        sqlx::query_as::<_, PayIntent>(
            "SELECT id, chain_id, payer, payee, amount, token_address, status, escrow_id, tx_hash, description, expires_at, created_at, updated_at
             FROM public.pay_intents
             WHERE payer = $1 OR payee = $1
             ORDER BY created_at DESC LIMIT $2 OFFSET $3",
        )
        .bind(&addr)
        .bind(limit)
        .bind(offset)
        .fetch_all(&state.db)
        .await
    }
    .map_err(|e| {
        tracing::error!("pay_history intents fetch: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Escrows where address is payer or payee
    let escrows: Vec<EscrowRecord> = if let Some(status) = status_filter.as_deref() {
        sqlx::query_as::<_, EscrowRecord>(
            "SELECT id, chain_id, payer, payee, amount, token_address, fee_amount, status, on_chain_id, tx_hash, dispute_reason, created_at, updated_at
             FROM public.escrows
             WHERE (payer = $1 OR payee = $1) AND status = $2
             ORDER BY created_at DESC LIMIT $3 OFFSET $4",
        )
        .bind(&addr)
        .bind(status)
        .bind(limit)
        .bind(offset)
        .fetch_all(&state.db)
        .await
    } else {
        sqlx::query_as::<_, EscrowRecord>(
            "SELECT id, chain_id, payer, payee, amount, token_address, fee_amount, status, on_chain_id, tx_hash, dispute_reason, created_at, updated_at
             FROM public.escrows
             WHERE payer = $1 OR payee = $1
             ORDER BY created_at DESC LIMIT $2 OFFSET $3",
        )
        .bind(&addr)
        .bind(limit)
        .bind(offset)
        .fetch_all(&state.db)
        .await
    }
    .map_err(|e| {
        tracing::error!("pay_history escrows fetch: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let total_intents: i64 = if let Some(status) = status_filter.as_deref() {
        sqlx::query_scalar(
            "SELECT COUNT(*) FROM public.pay_intents
             WHERE (payer = $1 OR payee = $1) AND status = $2",
        )
        .bind(&addr)
        .bind(status)
        .fetch_one(&state.db)
        .await
    } else {
        sqlx::query_scalar("SELECT COUNT(*) FROM public.pay_intents WHERE payer = $1 OR payee = $1")
            .bind(&addr)
            .fetch_one(&state.db)
            .await
    }
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let total_escrows: i64 = if let Some(status) = status_filter.as_deref() {
        sqlx::query_scalar(
            "SELECT COUNT(*) FROM public.escrows
             WHERE (payer = $1 OR payee = $1) AND status = $2",
        )
        .bind(&addr)
        .bind(status)
        .fetch_one(&state.db)
        .await
    } else {
        sqlx::query_scalar("SELECT COUNT(*) FROM public.escrows WHERE payer = $1 OR payee = $1")
            .bind(&addr)
            .fetch_one(&state.db)
            .await
    }
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(PayHistoryResponse {
        address: addr,
        intents,
        escrows,
        total_intents,
        total_escrows,
    }))
}
