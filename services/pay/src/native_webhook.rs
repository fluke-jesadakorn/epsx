//! HMAC-authenticated hints only. No webhook payload can finalize money.
use crate::AppState;
use alloy::primitives::B256;
use axum::{
    body::Bytes,
    extract::State,
    http::{HeaderMap, StatusCode},
    Json,
};
use hmac::{Hmac, Mac};
use serde::Deserialize;
use serde_json::{json, Value};
use sha2::Sha256;
use std::str::FromStr;
#[derive(Deserialize)]
struct Hint {
    event_id: String,
    tx_hash: String,
}
pub async fn hint(
    State(s): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<(StatusCode, Json<Value>), StatusCode> {
    let secret = std::env::var("EPSX_PAY_WEBHOOK_SECRET")
        .ok()
        .filter(|s| s.len() >= 32)
        .ok_or(StatusCode::NOT_FOUND)?;
    let signature = headers
        .get("x-pay-webhook-signature")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| hex::decode(s.strip_prefix("sha256=").unwrap_or(s)).ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;
    let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes())
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    mac.update(&body);
    mac.verify_slice(&signature)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;
    let hint: Hint = serde_json::from_slice(&body).map_err(|_| StatusCode::BAD_REQUEST)?;
    if hint.event_id.is_empty() || hint.event_id.len() > 128 {
        return Err(StatusCode::BAD_REQUEST);
    }
    let hash = B256::from_str(&hint.tx_hash).map_err(|_| StatusCode::BAD_REQUEST)?;
    sqlx::query(
        "INSERT INTO pay_v1_chain_hints(event_id,tx_hash) VALUES($1,$2) ON CONFLICT DO NOTHING",
    )
    .bind(hint.event_id)
    .bind(format!("{hash:#x}"))
    .execute(&s.db)
    .await
    .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    Ok((
        StatusCode::ACCEPTED,
        Json(json!({"received":true,"status":"pending_verification"})),
    ))
}
