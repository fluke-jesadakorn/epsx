use std::sync::Arc;
use axum::{
    routing::get,
    Router,
    Json,
};
use serde_json::json;
use crate::app_state::AppState;

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/health", get(health_check))
}

async fn health_check() -> Json<serde_json::Value> {
    Json(json!({
        "status": "ok",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "service": "epsx-backend",
        "version": env!("CARGO_PKG_VERSION")
    }))
}
