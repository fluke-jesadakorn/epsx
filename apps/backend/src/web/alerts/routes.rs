// Alert routes - Stubbed for Diesel migration
// TODO: Implement with Diesel

use axum::{routing::get, Router};
use tracing::warn;

pub fn create_alert_routes() -> Router {
    warn!("Alert routes stubbed - implement with Diesel");
    
    Router::new()
        .route("/alerts", get(|| async { "stubbed" }))
        .route("/alerts/health", get(|| async { "ok" }))
}

pub fn stub_function() {
    warn!("Alert routes stubbed - implement with Diesel");
}