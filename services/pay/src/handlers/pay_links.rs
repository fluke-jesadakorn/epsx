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
        "SELECT id, chain_id, payer, payee, amount, token_address, status, escrow_id, tx_hash, description, expires_at, created_at, updated_at FROM public.pay_intents WHERE id = $1"
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
        uuid::Uuid::new_v4()
            .simple()
            .to_string()
            .chars()
            .take(12)
            .collect::<String>()
    );
    let id = format!("0x{}", uuid::Uuid::new_v4().simple());
    let now = chrono::Utc::now();
    let expires_at = req
        .expires_in
        .and_then(|s| chrono::Utc::now().checked_add_signed(chrono::Duration::seconds(s)));
    let max_uses = req.max_uses.unwrap_or(1);

    sqlx::query(
        "INSERT INTO public.pay_links (id, slug, intent_id, max_uses, current_uses, expires_at, created_at)
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
        "SELECT id, slug, intent_id, max_uses, current_uses, expires_at, created_at FROM public.pay_links WHERE slug = $1"
    )
    .bind(&slug)
    .fetch_optional(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::NOT_FOUND)?;

    if !is_publicly_usable(&link, chrono::Utc::now()) {
        return Err(StatusCode::GONE);
    }

    let url = format!("/r/{}", link.slug);
    Ok(Json(PayLinkResponse { link, url }))
}

fn is_publicly_usable(link: &PayLink, now: chrono::DateTime<chrono::Utc>) -> bool {
    (link.max_uses == 0 || link.current_uses < link.max_uses)
        && link.expires_at.is_none_or(|expires_at| expires_at > now)
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
    let updated: Option<(String, i32, i32, Option<chrono::DateTime<chrono::Utc>>)> =
        sqlx::query_as(
            "UPDATE public.pay_links
            SET current_uses = current_uses + 1
          WHERE slug = $1
            AND (max_uses IS NULL OR max_uses = 0 OR current_uses < max_uses)
            AND (expires_at IS NULL OR expires_at > NOW())
        RETURNING intent_id, max_uses, current_uses, expires_at",
        )
        .bind(&slug)
        .fetch_optional(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let (intent_id, _max, _current, _exp) = updated.ok_or(StatusCode::GONE)?; // 410 — exhausted / expired

    // Fetch the resolved intent so the BFF can render the
    // checkout without a second round-trip.
    let intent: PayIntent = sqlx::query_as::<_, PayIntent>(
        "SELECT id, chain_id, payer, payee, amount, token_address, status, escrow_id, tx_hash, description, expires_at, created_at, updated_at FROM public.pay_intents WHERE id = $1"
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
        slug,
        intent.id,
        req.payer
    );

    Ok(Json(RedeemPayLinkResponse { intent, pay_url }))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn link(
        max_uses: i32,
        current_uses: i32,
        expires_at: Option<chrono::DateTime<chrono::Utc>>,
    ) -> PayLink {
        PayLink {
            id: "opaque-link-id".into(),
            slug: "epsx-public".into(),
            intent_id: "opaque-intent-id".into(),
            max_uses,
            current_uses,
            expires_at,
            created_at: chrono::Utc::now(),
        }
    }

    #[test]
    fn public_lookup_rejects_expired_and_exhausted_links() {
        let now = chrono::Utc::now();
        assert!(is_publicly_usable(
            &link(1, 0, Some(now + chrono::Duration::minutes(1))),
            now
        ));
        assert!(is_publicly_usable(&link(0, 99, None), now));
        assert!(!is_publicly_usable(&link(1, 1, None), now));
        assert!(!is_publicly_usable(
            &link(2, 0, Some(now - chrono::Duration::seconds(1))),
            now
        ));
    }

    #[test]
    fn public_projection_excludes_financial_and_identity_fields() {
        let response = PayLinkResponse {
            link: link(1, 0, None),
            url: "/r/epsx-public".into(),
        };
        let value = serde_json::to_value(response).unwrap();
        let response_keys = value
            .as_object()
            .unwrap()
            .keys()
            .cloned()
            .collect::<Vec<_>>();
        assert_eq!(response_keys, vec!["link", "url"]);
        let link = value["link"].as_object().unwrap();
        for forbidden in [
            "payer",
            "payee",
            "amount",
            "token",
            "token_address",
            "chain_id",
            "status",
            "tx_hash",
            "description",
        ] {
            assert!(
                !link.contains_key(forbidden),
                "public link leaked {forbidden}"
            );
        }
    }
}
