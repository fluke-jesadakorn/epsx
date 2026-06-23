//! On-chain webhook handler (slice-3 + slice-5 HMAC).
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
    body::Bytes,
    extract::State,
    http::StatusCode,
    Json,
};
use hmac::{Hmac, Mac};
use serde::Deserialize;
use sha2::Sha256;
use std::env;
use subtle::ConstantTimeEq;

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

/// HMAC-SHA256 verifier. Reads the raw request body, computes
/// HMAC-SHA256(EPSX_PAY_WEBHOOK_SECRET, body), and compares it
/// against the hex digest in `X-Pay-Webhook-Signature` using a
/// constant-time equality check (defends against timing attacks).
///
/// Returns `Ok(())` on signature match, `Err(StatusCode)` on any
/// failure:
/// - 503 when `EPSX_PAY_WEBHOOK_SECRET` is not configured
///   (fail-closed: never accept webhooks when secret is missing)
/// - 401 when the signature header is missing, malformed (not
///   hex), or doesn't match the computed digest
fn verify_webhook_signature(
    headers: &axum::http::HeaderMap,
    body: &[u8],
) -> Result<(), StatusCode> {
    // Fail-closed if the secret isn't configured.
    let secret = env::var("EPSX_PAY_WEBHOOK_SECRET")
        .map_err(|_| {
            tracing::error!("EPSX_PAY_WEBHOOK_SECRET not set — refusing webhook");
            StatusCode::SERVICE_UNAVAILABLE
        })?;

    // Read the signature header.
    let sig_hex = headers
        .get("x-pay-webhook-signature")
        .and_then(|v| v.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;

    // Decode the hex signature.
    let provided = hex::decode(sig_hex)
        .map_err(|_| {
            tracing::warn!("webhook signature is not valid hex");
            StatusCode::UNAUTHORIZED
        })?;

    // Compute HMAC-SHA256(secret, body).
    let mut mac = <Hmac<Sha256> as Mac>::new_from_slice(secret.as_bytes())
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    mac.update(body);
    let expected = mac.finalize().into_bytes();

    // Constant-time compare.
    if provided.ct_eq(&expected).unwrap_u8() == 1 {
        Ok(())
    } else {
        tracing::warn!("webhook signature mismatch");
        Err(StatusCode::UNAUTHORIZED)
    }
}

/// Webhook handler. Reads the raw body for HMAC verification,
/// then parses it as JSON. Applies the status change to the
/// matching intent + escrow.
pub async fn on_chain_webhook(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    body: Bytes,
) -> Result<Json<WebhookAck>, StatusCode> {
    // 1. Verify signature first (before parsing JSON) so we
    //    don't waste cycles on unsigned payloads.
    verify_webhook_signature(&headers, &body)?;

    // 2. Parse the now-trusted body as JSON.
    let event: OnChainEvent = serde_json::from_slice(&body)
        .map_err(|e| {
            tracing::warn!("webhook body is not valid JSON: {}", e);
            StatusCode::BAD_REQUEST
        })?;

    // 3. Idempotency check — INSERT … ON CONFLICT DO NOTHING so
    //    duplicate deliveries are no-ops. If the row was new we
    //    continue with the status update; if it collided we ack
    //    without re-applying.
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