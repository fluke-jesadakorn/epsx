//! Escrow handlers for epsx-pay-svc.
//!
//! wave49(slice-3): extracted from main.rs as part of the
//! services/pay modularization. All endpoints here match
//! the pre-modularization behavior 1:1 — pure refactor.
//!
//! Endpoints:
//! - `GET  /api/v1/pay/escrows`                  → `list_escrows`
//! - `GET  /api/v1/pay/escrows/:id`              → `get_escrow`
//! - `POST /api/v1/pay/escrows/:id/release`      → `release_escrow`
//! - `POST /api/v1/pay/escrows/:id/refund`       → `refund_escrow`
//! - `POST /api/v1/pay/escrows/:id/dispute`      → `dispute_escrow`
//! - `POST /api/v1/pay/escrows/:id/resolve`      → `resolve_dispute`
//! - `POST /api/v1/pay/escrows/:id/confirm-deposit` → `confirm_escrow_deposit`

use axum::{
    extract::{Extension, Path as AxPath, State},
    http::StatusCode,
    Json,
};
use epsx_pay_svc::canonical_owner;
use epsx_service_auth::VerifiedPrincipal;

use crate::types::*;
use crate::AppState;

// ============================================================================
// GET /api/v1/pay/escrows
// ============================================================================

pub async fn list_escrows(
    State(state): State<AppState>,
    Extension(principal): Extension<VerifiedPrincipal>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Result<Json<EscrowListResponse>, StatusCode> {
    let owner = canonical_owner(&principal, None)?;
    let status = params.get("status").cloned();
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

    let items: Vec<EscrowRecord> = if let Some(s) = &status {
        sqlx::query_as::<_, EscrowRecord>(
            "SELECT id, chain_id, payer, payee, amount, token_address, fee_amount, status, on_chain_id, tx_hash, dispute_reason, created_at, updated_at
             FROM escrows
             WHERE (payer = $1 OR payee = $1) AND status = $2
             ORDER BY created_at DESC LIMIT $3 OFFSET $4"
        )
        .bind(&owner)
        .bind(s)
        .bind(limit)
        .bind(offset)
        .fetch_all(&state.db)
        .await
    } else {
        sqlx::query_as::<_, EscrowRecord>(
            "SELECT id, chain_id, payer, payee, amount, token_address, fee_amount, status, on_chain_id, tx_hash, dispute_reason, created_at, updated_at
             FROM escrows
             WHERE payer = $1 OR payee = $1
             ORDER BY created_at DESC LIMIT $2 OFFSET $3"
        )
        .bind(&owner)
        .bind(limit)
        .bind(offset)
        .fetch_all(&state.db)
        .await
    }
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let total: i64 = if let Some(status) = status.as_deref() {
        sqlx::query_scalar(
            "SELECT COUNT(*) FROM escrows
             WHERE (payer = $1 OR payee = $1) AND status = $2",
        )
        .bind(&owner)
        .bind(status)
        .fetch_one(&state.db)
        .await
    } else {
        sqlx::query_scalar("SELECT COUNT(*) FROM escrows WHERE payer = $1 OR payee = $1")
            .bind(&owner)
            .fetch_one(&state.db)
            .await
    }
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(EscrowListResponse { items, total }))
}

// ============================================================================
// GET /api/v1/pay/escrows/:id
// ============================================================================

pub async fn get_escrow(
    State(state): State<AppState>,
    Extension(principal): Extension<VerifiedPrincipal>,
    AxPath(id): AxPath<String>,
) -> Result<Json<EscrowRecord>, StatusCode> {
    let owner = canonical_owner(&principal, None)?;
    let escrow: EscrowRecord = sqlx::query_as::<_, EscrowRecord>(
        "SELECT id, chain_id, payer, payee, amount, token_address, fee_amount, status, on_chain_id, tx_hash, dispute_reason, created_at, updated_at
         FROM escrows WHERE id = $1 AND (payer = $2 OR payee = $2)"
    )
    .bind(&id)
    .bind(&owner)
    .fetch_optional(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::NOT_FOUND)?;
    Ok(Json(escrow))
}

// ============================================================================
// POST /api/v1/pay/escrows/:id/release
// ============================================================================

pub async fn release_escrow(
    State(state): State<AppState>,
    AxPath(id): AxPath<String>,
    Json(_req): Json<ReleaseEscrowRequest>,
) -> Result<Json<EscrowRecord>, StatusCode> {
    sqlx::query("UPDATE escrows SET status = 'released', updated_at = NOW() WHERE id = $1 AND status = 'active'")
        .bind(&id)
        .execute(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    get_escrow_unscoped(&state, &id).await
}

// ============================================================================
// POST /api/v1/pay/escrows/:id/refund
// ============================================================================

pub async fn refund_escrow(
    State(state): State<AppState>,
    AxPath(id): AxPath<String>,
    Json(req): Json<RefundEscrowRequest>,
) -> Result<Json<EscrowRecord>, StatusCode> {
    sqlx::query("UPDATE escrows SET status = 'refunded', dispute_reason = $1, updated_at = NOW() WHERE id = $2 AND status IN ('active', 'disputed')")
        .bind(&req.reason)
        .bind(&id)
        .execute(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    get_escrow_unscoped(&state, &id).await
}

// ============================================================================
// POST /api/v1/pay/escrows/:id/dispute
// ============================================================================

pub async fn dispute_escrow(
    State(state): State<AppState>,
    AxPath(id): AxPath<String>,
    Json(req): Json<DisputeEscrowRequest>,
) -> Result<Json<EscrowRecord>, StatusCode> {
    sqlx::query("UPDATE escrows SET status = 'disputed', dispute_reason = $1, updated_at = NOW() WHERE id = $2 AND status = 'active'")
        .bind(&req.reason)
        .bind(&id)
        .execute(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    get_escrow_unscoped(&state, &id).await
}

// ============================================================================
// POST /api/v1/pay/escrows/:id/resolve
// ============================================================================

pub async fn resolve_dispute(
    State(state): State<AppState>,
    AxPath(id): AxPath<String>,
    Json(req): Json<ResolveDisputeRequest>,
) -> Result<Json<EscrowRecord>, StatusCode> {
    let new_status = if req.to_payee { "released" } else { "refunded" };
    sqlx::query(
        "UPDATE escrows SET status = $1, updated_at = NOW() WHERE id = $2 AND status = 'disputed'",
    )
    .bind(new_status)
    .bind(&id)
    .execute(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    get_escrow_unscoped(&state, &id).await
}

// ============================================================================
// POST /api/v1/pay/escrows/:id/confirm-deposit
// ============================================================================

pub async fn confirm_escrow_deposit(
    State(state): State<AppState>,
    AxPath(id): AxPath<String>,
    Json(req): Json<serde_json::Value>,
) -> Result<Json<EscrowRecord>, StatusCode> {
    let on_chain_id = req
        .get("on_chain_id")
        .and_then(|v| v.as_str())
        .unwrap_or_default();
    let tx_hash = req
        .get("tx_hash")
        .and_then(|v| v.as_str())
        .unwrap_or_default();

    sqlx::query(
        "UPDATE escrows SET on_chain_id = $1, tx_hash = $2, updated_at = NOW() WHERE id = $3",
    )
    .bind(on_chain_id)
    .bind(tx_hash)
    .bind(&id)
    .execute(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    get_escrow_unscoped(&state, &id).await
}

async fn get_escrow_unscoped(state: &AppState, id: &str) -> Result<Json<EscrowRecord>, StatusCode> {
    let escrow: EscrowRecord = sqlx::query_as::<_, EscrowRecord>(
        "SELECT id, chain_id, payer, payee, amount, token_address, fee_amount, status, on_chain_id, tx_hash, dispute_reason, created_at, updated_at
         FROM escrows WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::NOT_FOUND)?;
    Ok(Json(escrow))
}
