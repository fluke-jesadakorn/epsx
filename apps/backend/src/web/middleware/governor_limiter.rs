use axum::body::Body;
use axum::extract::{Request, State};
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::Response;
use tower_governor::{
    governor::GovernorConfigBuilder,
    key_extractor::SmartIpKeyExtractor,
    GovernorLayer,
};

use crate::web::auth::AppState;
use tracing::warn;

// Rate limiting configurations using tower-governor v0.8 API
// Returns GovernorLayer directly so it can be used with .layer()
pub fn auth_rate_limiter() -> GovernorLayer<SmartIpKeyExtractor, governor::middleware::StateInformationMiddleware, Body> {
    // 5 requests per minute for auth endpoints
    let config = GovernorConfigBuilder::default()
        .per_second(60)
        .burst_size(5)
        .key_extractor(SmartIpKeyExtractor)
        .use_headers()
        .finish()
        .unwrap();

    GovernorLayer::new(config)
}

pub fn chat_rate_limiter() -> GovernorLayer<SmartIpKeyExtractor, governor::middleware::StateInformationMiddleware, Body> {
    // 10 requests per minute for chat endpoints
    let config = GovernorConfigBuilder::default()
        .per_second(60)
        .burst_size(10)
        .key_extractor(SmartIpKeyExtractor)
        .use_headers()
        .finish()
        .unwrap();

    GovernorLayer::new(config)
}

pub fn email_rate_limiter() -> GovernorLayer<SmartIpKeyExtractor, governor::middleware::StateInformationMiddleware, Body> {
    // 3 requests per minute for email operations
    let config = GovernorConfigBuilder::default()
        .per_second(60)
        .burst_size(3)
        .key_extractor(SmartIpKeyExtractor)
        .use_headers()
        .finish()
        .unwrap();

    GovernorLayer::new(config)
}

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
