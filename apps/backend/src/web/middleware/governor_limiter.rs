use axum::extract::{Request, State};
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::Response;

use crate::web::auth::AppState;
use tracing::warn;

// Re-export the 3 pure rate-limiter factories that moved into the
// `epsx-web-middleware` shared crate during the wave 10 prep pass.
// `unified_router.rs` still does
// `crate::web::middleware::governor_limiter::auth_rate_limiter()` so
// we re-export from here to keep that path working.
pub use epsx_web_middleware::governor_limiters::{
    auth_rate_limiter, chat_rate_limiter, email_rate_limiter,
};

pub async fn threat_aware_middleware(
    State(_app_state): State<AppState>,
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Extract IP address from request headers
    let ip = req.headers().get("cf-connecting-ip")
        .or_else(|| req.headers().get("x-forwarded-for"))
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_string())
        .unwrap_or_else(|| "unknown".to_string());

    let threat_service = crate::infrastructure::security::get_threat_detection_service();

    // Check if IP is explicitly blocked by our Threat Detection Service
    if let Some(summary) = threat_service.get_security_summary(&ip) {
        if summary.is_blocked {
            warn!("Blocked request from IP {} due to Critical Threat Level", ip);
            return Err(StatusCode::TOO_MANY_REQUESTS);
        }
    }

    Ok(next.run(req).await)
}
