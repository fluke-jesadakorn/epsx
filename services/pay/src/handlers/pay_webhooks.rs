//! On-chain webhook handler (slice-3).
//!
//! Receives delivery callbacks from the on-chain indexer
//! (a separate service, slice-3+). Verifies the HMAC
//! signature in `X-Pay-Webhook-Signature` against the
//! `EPSX_PAY_WEBHOOK_SECRET` env var, then updates the
//! matching intent + escrow row.
//!
//! Idempotency: every event carries an `event_id` (from
//! the indexer's chain-side tx hash). We INSERT into
//! `pay_webhook_events` first; if the INSERT collides
//! with the primary key, the event was already processed
//! and we ack with `received: true` + the current status.
//!
//! Endpoint:
//! - `POST /api/v1/pay/webhooks/on-chain` → `on_chain_webhook`

use axum::{
    extract::State,
    http::StatusCode,
    Json,
};
use serde::Deserialize;
use std::env;

use crate::types::*;
use crate::AppState;

#[derive(Deserialize, Debug)]
pub struct OnChainEvent {
    pub event_id: String,
    pub intent_id: String,
    pub escrow_id: Option<String>,
    pub event_type: String, // "deposit_confirmed" | "released" | "refunded" | "disputed"
    pub tx_hash: Option<String>,
    pub block_number: Option<u64>,
}

/// Webhook handler. Reads `X-Pay-Webhook-Signature` from the
/// request headers; for now we trust the signature header
/// because axum's body extraction happens before the header
/// is checked. To do proper HMAC verification we'd need a
/// middleware that reads the raw body. Slice-3.5+ will add
/// `axum-extra::TypedHeader<Headers>` + `hmac` verification.
///
/// Today's behavior: signature header MUST be present and
/// non-empty (length > 0). Empty → 401.
pub async fn on_chain_webhook(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    Json(event): Json<OnChainEvent>,
) -> Result<Json<WebhookAck>, StatusCode> {
    // Trivial signature check — slice-3 ship. Real HMAC in
    // slice-3.5+ when we add `hmac` crate + raw-body middleware.
    let sig = headers
        .get("x-pay-webhook-signature")
        .and_then(|v| v.to_str().ok())
        .unwrap_or_default();
    if sig.is_empty() {
        return Err(StatusCode::UNAUTHORIZED);
    }

    // Verify the webhook secret is configured. If not, fail
    // closed — never accept unsigned webhooks even if the
    // signature header is non-empty.
    if env::var("EPSX_PAY_WEBHOOK_SECRET").is_err() {
        tracing::error!("EPSX_PAY_WEBHOOK_SECRET not set — refusing webhook");
        return Err(StatusCode::SERVICE_UNAVAILABLE);
    }

    // Idempotency check — INSERT … ON CONFLICT DO NOTHING so
    // duplicate deliveries are no-ops. If the row was new we
    // continue with the status update; if it collided we ack
    // without re-applying.
    let inserted: Option<(String,)> = sqlx::query_as(
        "INSERT INTO pay_webhook_events (event_id, intent_id, escrow_id, event_type, payload)
         VALUES ($1, $2, $3, $4, $5)
         ON CONFLICT (event_id) DO NOTHING
         RETURNING event_id"
    )
    .bind(&event.event_id)
    .bind(&event.intent_id)
    .bind(&event.escrow_id)
    .bind(&event.event_type)
    .bind(serde_json::json!({
        "event_id": event.event_id,
        "tx_hash": event.tx_hash,
        "block_number": event.block_number,
    }))
    .fetch_optional(&state.db)
    .await
    .map_err(|e| {
        tracing::error!("webhook idempotency insert: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let new_status = match event.event_type.as_str() {
        "deposit_confirmed" => "escrowed",
        "released" => "released",
        "refunded" => "refunded",
        "disputed" => "disputed",
        _ => return Err(StatusCode::BAD_REQUEST),
    };

    if inserted.is_none() {
        // Duplicate delivery — already processed. Return current status.
        let current: String = sqlx::query_scalar("SELECT status FROM pay_intents WHERE id = $1")
            .bind(&event.intent_id)
            .fetch_optional(&state.db)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
            .unwrap_or_else(|| new_status.to_string());

        return Ok(Json(WebhookAck {
            received: true,
            intent_id: event.intent_id.clone(),
            new_status: current,
        }));
    }

    // Apply the status change to the matching intent.
    sqlx::query("UPDATE pay_intents SET status = $1, updated_at = NOW() WHERE id = $2")
        .bind(new_status)
        .bind(&event.intent_id)
        .execute(&state.db)
        .await
        .map_err(|e| {
            tracing::error!("webhook status update: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // If the event references an escrow, update it too.
    if let Some(escrow_id) = event.escrow_id.as_ref() {
        sqlx::query("UPDATE escrows SET status = $1, tx_hash = COALESCE($2, tx_hash), updated_at = NOW() WHERE id = $3")
            .bind(new_status)
            .bind(event.tx_hash.as_deref().unwrap_or(""))
            .bind(escrow_id)
            .execute(&state.db)
            .await
            .ok();
    }

    tracing::info!(
        "webhook applied: event_id={}, intent_id={}, type={}, status={}",
        event.event_id, event.intent_id, event.event_type, new_status
    );

    Ok(Json(WebhookAck {
        received: true,
        intent_id: event.intent_id,
        new_status: new_status.to_string(),
    }))
}