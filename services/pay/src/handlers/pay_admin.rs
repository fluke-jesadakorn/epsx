//! Admin handlers (slice-3).
//!
//! Force-cancel / force-release / force-refund for ops + admin
//! tooling. The direct service middleware now verifies the admin
//! audience and canonical payment permission before Axum can
//! extract the principal or invoke these handlers.
//!
//! Endpoints:
//! - `GET  /api/v1/admin/pay/intents`                   → `admin_list_pay_intents`
//! - `POST /api/v1/admin/pay/intents/:id/force-cancel`  → `admin_force_cancel_pay_intent`
//! - `POST /api/v1/admin/pay/escrows/:id/force-release` → `admin_force_release_escrow`
//! - `POST /api/v1/admin/pay/escrows/:id/force-refund`  → `admin_force_refund_escrow`

use axum::{
    extract::{Extension, Path as AxPath, State},
    http::StatusCode,
    Json,
};
use epsx_service_auth::VerifiedPrincipal;

use crate::types::*;
use crate::AppState;

// ============================================================================
// GET /api/v1/admin/pay/intents
// ============================================================================

pub async fn admin_list_pay_intents(
    State(state): State<AppState>,
    Extension(_principal): Extension<VerifiedPrincipal>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Result<Json<PayIntentListResponse>, StatusCode> {
    let payer = params.get("payer").cloned();
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

    let (items, total): (Vec<PayIntent>, i64) = match (payer.as_deref(), status.as_deref()) {
        (Some(payer), Some(status)) => {
            let items = sqlx::query_as::<_, PayIntent>(
                "SELECT id, chain_id, payer, payee, amount, token_address, status, escrow_id, tx_hash, description, expires_at, created_at, updated_at
                 FROM public.pay_intents WHERE payer = $1 AND status = $2
                 ORDER BY created_at DESC LIMIT $3 OFFSET $4",
            )
            .bind(payer.to_ascii_lowercase())
            .bind(status)
            .bind(limit)
            .bind(offset)
            .fetch_all(&state.db)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            let total = sqlx::query_scalar(
                "SELECT COUNT(*) FROM public.pay_intents WHERE payer = $1 AND status = $2",
            )
            .bind(payer.to_ascii_lowercase())
            .bind(status)
            .fetch_one(&state.db)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            (items, total)
        }
        (Some(payer), None) => {
            let payer = payer.to_ascii_lowercase();
            let items = sqlx::query_as::<_, PayIntent>(
                "SELECT id, chain_id, payer, payee, amount, token_address, status, escrow_id, tx_hash, description, expires_at, created_at, updated_at
                 FROM public.pay_intents WHERE payer = $1
                 ORDER BY created_at DESC LIMIT $2 OFFSET $3",
            )
            .bind(&payer)
            .bind(limit)
            .bind(offset)
            .fetch_all(&state.db)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            let total =
                sqlx::query_scalar("SELECT COUNT(*) FROM public.pay_intents WHERE payer = $1")
                    .bind(&payer)
                    .fetch_one(&state.db)
                    .await
                    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            (items, total)
        }
        (None, Some(status)) => {
            let items = sqlx::query_as::<_, PayIntent>(
                "SELECT id, chain_id, payer, payee, amount, token_address, status, escrow_id, tx_hash, description, expires_at, created_at, updated_at
                 FROM public.pay_intents WHERE status = $1
                 ORDER BY created_at DESC LIMIT $2 OFFSET $3",
            )
            .bind(status)
            .bind(limit)
            .bind(offset)
            .fetch_all(&state.db)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            let total =
                sqlx::query_scalar("SELECT COUNT(*) FROM public.pay_intents WHERE status = $1")
                    .bind(status)
                    .fetch_one(&state.db)
                    .await
                    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            (items, total)
        }
        (None, None) => {
            let items = sqlx::query_as::<_, PayIntent>(
                "SELECT id, chain_id, payer, payee, amount, token_address, status, escrow_id, tx_hash, description, expires_at, created_at, updated_at
                 FROM public.pay_intents ORDER BY created_at DESC LIMIT $1 OFFSET $2",
            )
            .bind(limit)
            .bind(offset)
            .fetch_all(&state.db)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            let total = sqlx::query_scalar("SELECT COUNT(*) FROM public.pay_intents")
                .fetch_one(&state.db)
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            (items, total)
        }
    };

    Ok(Json(PayIntentListResponse { items, total }))
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
    sqlx::query(
        "UPDATE public.pay_intents SET status = 'cancelled', updated_at = NOW() WHERE id = $1",
    )
    .bind(&id)
    .execute(&state.db)
    .await
    .map_err(|e| {
        tracing::error!("admin force_cancel: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let intent: PayIntent = sqlx::query_as::<_, PayIntent>(
        "SELECT id, chain_id, payer, payee, amount, token_address, status, escrow_id, tx_hash, description, expires_at, created_at, updated_at FROM public.pay_intents WHERE id = $1"
    )
    .bind(&id)
    .fetch_one(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    tracing::warn!("admin force-cancel: intent_id={}, prev_status=any", id);

    Ok(Json(intent))
}

// ============================================================================
// POST /api/v1/admin/pay/escrows/:id/force-release
// ============================================================================

pub async fn admin_force_release_escrow(
    State(state): State<AppState>,
    AxPath(id): AxPath<String>,
) -> Result<Json<EscrowRecord>, StatusCode> {
    sqlx::query("UPDATE public.escrows SET status = 'released', updated_at = NOW() WHERE id = $1")
        .bind(&id)
        .execute(&state.db)
        .await
        .map_err(|e| {
            tracing::error!("admin force_release: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let escrow: EscrowRecord = sqlx::query_as::<_, EscrowRecord>(
        "SELECT id, chain_id, payer, payee, amount, token_address, fee_amount, status, on_chain_id, tx_hash, dispute_reason, created_at, updated_at FROM public.escrows WHERE id = $1"
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
    sqlx::query("UPDATE public.escrows SET status = 'refunded', updated_at = NOW() WHERE id = $1")
        .bind(&id)
        .execute(&state.db)
        .await
        .map_err(|e| {
            tracing::error!("admin force_refund: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let escrow: EscrowRecord = sqlx::query_as::<_, EscrowRecord>(
        "SELECT id, chain_id, payer, payee, amount, token_address, fee_amount, status, on_chain_id, tx_hash, dispute_reason, created_at, updated_at FROM public.escrows WHERE id = $1"
    )
    .bind(&id)
    .fetch_one(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    tracing::warn!("admin force-refund: escrow_id={}", id);

    Ok(Json(escrow))
}
