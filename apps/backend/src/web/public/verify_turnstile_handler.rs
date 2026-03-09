use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    Json,
};
use serde::Deserialize;
use serde_json::{json, Value};
use tracing::{error, info, warn};

use crate::web::auth::AppState;

#[derive(Debug, Deserialize)]
pub struct VerifyTurnstileRequest {
    pub token: String,
}

/// POST /api/public/verify-turnstile
/// No auth required. Validates a Turnstile token and returns 200 on success.
pub async fn verify_turnstile_handler(
    _state: State<AppState>,
    headers: HeaderMap,
    Json(body): Json<VerifyTurnstileRequest>,
) -> Result<Json<Value>, StatusCode> {
    let remote_ip = headers
        .get("cf-connecting-ip")
        .or_else(|| headers.get("x-forwarded-for"))
        .and_then(|h| h.to_str().ok())
        .map(|s| s.split(',').next().unwrap_or(s).trim());

    match crate::infrastructure::security::verify_turnstile_token(&body.token, remote_ip).await {
        Ok(result) if result.success => {
            info!("Turnstile verification passed");
            Ok(Json(json!({ "success": true })))
        }
        Ok(result) => {
            warn!(error_codes = ?result.error_codes, "Turnstile verification failed");
            Ok(Json(json!({
                "success": false,
                "error": "captcha_failed",
                "message": "Human verification failed. Please try again."
            })))
        }
        Err(e) => {
            error!("Turnstile verification error: {}", e);
            Err(StatusCode::SERVICE_UNAVAILABLE)
        }
    }
}
