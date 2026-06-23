//! Admin handlers (slice-3).
//!
//! Force-cancel / force-release / force-refund for ops + admin
//! tooling. Today these endpoints trust the caller — there's
//! no auth check at the service layer because the monolith
//! backend already filters by admin role before calling.
//! Slice-3.5+ will add JWT verification via `epsx_identity`
//! when the Identity service exposes admin tokens.
//!
//! Endpoints:
//! - `GET  /api/v1/admin/pay/intents`                   → `admin_list_pay_intents`
//! - `POST /api/v1/admin/pay/intents/:id/force-cancel`  → `admin_force_cancel_pay_intent`
//! - `POST /api/v1/admin/pay/escrows/:id/force-release` → `admin_force_release_escrow`
//! - `POST /api/v1/admin/pay/escrows/:id/force-refund`  → `admin_force_refund_escrow`

use axum::{
    extract::{Path as AxPath, State},
    http::StatusCode,
    Json,
};

use crate::types::*;
use crate::AppState;

// ============================================================================
// GET /api/v1/admin/pay/intents
// ============================================================================

pub async fn admin_list_pay_intents(
    State(state): State<AppState>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Result<Json<PayIntentListResponse>, StatusCode> {
    // Reuse the public list endpoint shape — admin gets the
    // same paginated/filtered list. Auth is checked upstream
    // in the monolith (apps/backend) before this is reached
    // via the proxy; in dev/staging the monolith trusts the
    // session cookie, in prod the API gateway enforces
    // `pay:admin` scope.
    crate::handlers::intents::list_pay_intents(
        State(state),
        axum::extract::Query(params),
    ).await
}

// ============================================================================
// POST /api/v1/admin/pay/intents/:id/force-cancel
// ============================================================================

pub async fn admin_force_cancel_pay_intent(
    State(state): State<AppState>,
    AxPath(id): AxPath<String>,
) -> Result<Json<PayIntent>, StatusCode> {
    // Force-cancel ignores the `status = 'pending'` guard
    // that the public cancel uses. Admins can cancel any
    // intent regardless of its current state — useful for
    // cleaning up stuck or fraudulent intents.
    sqlx::query("UPDATE pay_intents SET status = 'cancelled', updated_at = NOW() WHERE id = $1")
        .bind(&id)
        .execute(&state.db)
        .await
        .map_err(|e| {
            tracing::error!("admin force_cancel: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let intent: PayIntent = sqlx::query_as::<_, PayIntent>(
        "SELECT id, chain_id, payer, payee, amount, token_address, status, escrow_id, tx_hash, description, expires_at, created_at, updated_at FROM pay_intents WHERE id = $1"
    )
    .bind(&id)
    .fetch_one(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    tracing::warn!(
        "admin force-cancel: intent_id={}, prev_status=any",
        id
    );

    Ok(Json(intent))
}

// ============================================================================
// POST /api/v1/admin/pay/escrows/:id/force-release
// ============================================================================

pub async fn admin_force_release_escrow(
    State(state): State<AppState>,
    AxPath(id): AxPath<String>,
) -> Result<Json<EscrowRecord>, StatusCode> {
    sqlx::query("UPDATE escrows SET status = 'released', updated_at = NOW() WHERE id = $1")
        .bind(&id)
        .execute(&state.db)
        .await
        .map_err(|e| {
            tracing::error!("admin force_release: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let escrow: EscrowRecord = sqlx::query_as::<_, EscrowRecord>(
        "SELECT id, chain_id, payer, payee, amount, token_address, fee_amount, status, on_chain_id, tx_hash, dispute_reason, created_at, updated_at FROM escrows WHERE id = $1"
    )
    .bind(&id)
    .fetch_one(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    tracing::warn!("admin force-release: escrow_id={}", id);

    Ok(Json(escrow))
}

// ============================================================================
// POST /api/v1/admin/pay/escrows/:id/force-refund
// ============================================================================

pub async fn admin_force_refund_escrow(
    State(state): State<AppState>,
    AxPath(id): AxPath<String>,
) -> Result<Json<EscrowRecord>, StatusCode> {
    sqlx::query("UPDATE escrows SET status = 'refunded', updated_at = NOW() WHERE id = $1")
        .bind(&id)
        .execute(&state.db)
        .await
        .map_err(|e| {
            tracing::error!("admin force_refund: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let escrow: EscrowRecord = sqlx::query_as::<_, EscrowRecord>(
        "SELECT id, chain_id, payer, payee, amount, token_address, fee_amount, status, on_chain_id, tx_hash, dispute_reason, created_at, updated_at FROM escrows WHERE id = $1"
    )
    .bind(&id)
    .fetch_one(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    tracing::warn!("admin force-refund: escrow_id={}", id);

    Ok(Json(escrow))
}