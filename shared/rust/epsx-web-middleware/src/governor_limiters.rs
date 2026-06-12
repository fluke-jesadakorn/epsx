//! Pure (no backend-state) rate-limiter factories built on
//! `tower-governor`. Moved from
//! `apps/backend/src/web/middleware/governor_limiter.rs` in the
//! wave 10 prep pass — the 3 `*_rate_limiter()` factory functions
//! here don't touch any backend-internal types, only the governor
//! config primitives.
//!
//! The `threat_aware_middleware` function (which uses
//! `crate::infrastructure::security::get_threat_detection_service`)
//! stays in the backend because it reaches into the threat
//! detection runtime state. A future wave can split the threat
//! service into a port/trait that the shared crate depends on,
//! and at that point `threat_aware_middleware` can move here too.

use axum::body::Body;
use tower_governor::{
    governor::GovernorConfigBuilder,
    key_extractor::SmartIpKeyExtractor,
    GovernorLayer,
};

/// 5 requests per minute for auth endpoints (e.g. login, SIWE challenge).
pub fn auth_rate_limiter() -> GovernorLayer<SmartIpKeyExtractor, governor::middleware::StateInformationMiddleware, Body> {
    let config = GovernorConfigBuilder::default()
        .per_second(60)
        .burst_size(5)
        .key_extractor(SmartIpKeyExtractor)
        .use_headers()
        .finish()
        .unwrap();

    GovernorLayer::new(config)
}

/// 10 requests per minute for chat endpoints.
pub fn chat_rate_limiter() -> GovernorLayer<SmartIpKeyExtractor, governor::middleware::StateInformationMiddleware, Body> {
    let config = GovernorConfigBuilder::default()
        .per_second(60)
        .burst_size(10)
        .key_extractor(SmartIpKeyExtractor)
        .use_headers()
        .finish()
        .unwrap();

    GovernorLayer::new(config)
}

/// 3 requests per minute for email operations.
pub fn email_rate_limiter() -> GovernorLayer<SmartIpKeyExtractor, governor::middleware::StateInformationMiddleware, Body> {
    let config = GovernorConfigBuilder::default()
        .per_second(60)
        .burst_size(3)
        .key_extractor(SmartIpKeyExtractor)
        .use_headers()
        .finish()
        .unwrap();

    GovernorLayer::new(config)
}
