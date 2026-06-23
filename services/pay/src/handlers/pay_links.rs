//! Pay-link handlers (slice-3).
//!
//! Shareable payment URLs. Created from an existing intent;
//! `slug` is a short URL-safe id (e.g. `epsx-abc123`) that
//! resolves to the intent. `current_uses` is incremented
//! atomically on `redeem`.
//!
//! Endpoints:
//! - `POST /api/v1/pay/links`              → `create_pay_link`
//! - `GET  /api/v1/pay/links/:slug`        → `get_pay_link`
//! - `POST /api/v1/pay/links/:slug/redeem` → `redeem_pay_link`
//!
//! Use cases:
//! - Merchant sends a link `pay.epsx.io/r/epsx-abc123` to a
//!   customer via email/SMS. Customer clicks, sees the
//!   pay-branded checkout (the bff-pay BFF resolves the slug
//!   to an intent via `GET /api/v1/pay/links/:slug`, then
//!   renders the normal `/pay?intent=…` checkout flow).
//! - "Pay me" buttons embedded in invoices, dashboards, etc.

use axum::{
    extract::{Path as AxPath, State},
    http::StatusCode,
    Json,
};

use crate::types::*;
use crate::AppState;

// ============================================================================
// POST /api/v1/pay/links
// ============================================================================

pub async fn create_pay_link(
    State(state): State<AppState>,
    Json(req): Json<CreatePayLinkRequest>,
) -> Result<Json<PayLinkResponse>, StatusCode> {
    // Verify the intent exists.
    let intent: PayIntent = sqlx::query_as::<_, PayIntent>(
        "SELECT id, chain_id, payer, payee, amount, token_address, status, escrow_id, tx_hash, description, expires_at, created_at, updated_at FROM pay_intents WHERE id = $1"
    )
    .bind(&req.intent_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::NOT_FOUND)?;

    // Build the slug — short, URL-safe, unique. We use the first
    // 12 hex chars of a UUID (collision-resistant for any
    // realistic payment volume).
    let slug = format!(
        "epsx-{}",
        uuid::Uuid::new_v4().simple().to_string().chars().take(12).collect::<String>()
    );
    let id = format!("0x{}", uuid::Uuid::new_v4().simple());
    let now = chrono::Utc::now();
    let expires_at = req.expires_in
        .and_then(|s| chrono::Utc::now().checked_add_signed(chrono::Duration::seconds(s)));
    let max_uses = req.max_uses.unwrap_or(1);

    sqlx::query(
        "INSERT INTO pay_links (id, slug, intent_id, max_uses, current_uses, expires_at, created_at)
         VALUES ($1, $2, $3, $4, 0, $5, $6)"
    )
    .bind(&id)
    .bind(&slug)
    .bind(&intent.id)
    .bind(max_uses)
    .bind(expires_at)
    .bind(now)
    .execute(&state.db)
    .await
    .map_err(|e| {
        tracing::error!("pay_link insert: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let link = PayLink {
        id: id.clone(),
        slug: slug.clone(),
        intent_id: intent.id.clone(),
        max_uses,
        current_uses: 0,
        expires_at,
        created_at: now,
    };

    let url = format!("/r/{}", slug);

    Ok(Json(PayLinkResponse { link, url }))
}

// ============================================================================
// GET /api/v1/pay/links/:slug
// ============================================================================

pub async fn get_pay_link(
    State(state): State<AppState>,
    AxPath(slug): AxPath<String>,
) -> Result<Json<PayLinkResponse>, StatusCode> {
    let link: PayLink = sqlx::query_as::<_, PayLink>(
        "SELECT id, slug, intent_id, max_uses, current_uses, expires_at, created_at FROM pay_links WHERE slug = $1"
    )
    .bind(&slug)
    .fetch_optional(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::NOT_FOUND)?;

    let url = format!("/r/{}", link.slug);
    Ok(Json(PayLinkResponse { link, url }))
}

// ============================================================================
// POST /api/v1/pay/links/:slug/redeem
// ============================================================================

pub async fn redeem_pay_link(
    State(state): State<AppState>,
    AxPath(slug): AxPath<String>,
    Json(req): Json<RedeemPayLinkRequest>,
) -> Result<Json<RedeemPayLinkResponse>, StatusCode> {
    // Atomically check + increment usage. We use UPDATE …
    // RETURNING so the (read + check + write) is one DB
    // roundtrip — no race window.
    let updated: Option<(String, i32, i32, Option<chrono::DateTime<chrono::Utc>>)> = sqlx::query_as(
        "UPDATE pay_links
            SET current_uses = current_uses + 1
          WHERE slug = $1
            AND (max_uses IS NULL OR max_uses = 0 OR current_uses < max_uses)
            AND (expires_at IS NULL OR expires_at > NOW())
        RETURNING intent_id, max_uses, current_uses, expires_at"
    )
    .bind(&slug)
    .fetch_optional(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let (intent_id, _max, _current, _exp) = updated.ok_or(StatusCode::GONE)?; // 410 — exhausted / expired

    // Fetch the resolved intent so the BFF can render the
    // checkout without a second round-trip.
    let intent: PayIntent = sqlx::query_as::<_, PayIntent>(
        "SELECT id, chain_id, payer, payee, amount, token_address, status, escrow_id, tx_hash, description, expires_at, created_at, updated_at FROM pay_intents WHERE id = $1"
    )
    .bind(&intent_id)
    .fetch_one(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // The bff-pay BFF looks at `pay_url` and `intent.id` to
    // render the checkout — both come back here so the BFF
    // doesn't need to re-derive them.
    let pay_url = format!("/pay?intent={}", intent.id);

    tracing::info!(
        "pay_link redeemed: slug={}, intent_id={}, payer={}",
        slug, intent.id, req.payer
    );

    Ok(Json(RedeemPayLinkResponse { intent, pay_url }))
}