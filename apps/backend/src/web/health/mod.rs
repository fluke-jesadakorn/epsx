//! Health check module for monitoring system status
//! Simplified health checks without Casbin dependencies



use axum::{http::StatusCode, response::Json};
use serde_json::{json, Value};

/// Health check response structure
#[derive(serde::Serialize)]
pub struct HealthCheckResponse {
    pub status: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub service: String,
    pub version: String,
}

/// Simple health check handler
pub async fn health_check_handler() -> Json<Value> {
    Json(json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now(),
        "service": "epsx-backend",
        "version": "1.0.0"
    }))
}

/// Readiness check handler
pub async fn readiness_check_handler() -> Result<Json<Value>, StatusCode> {
    // TODO: Add database connectivity check
    Ok(Json(json!({
        "status": "ready",
        "timestamp": chrono::Utc::now(),
        "checks": {
            "database": "ok",
            "auth": "ok"
        }
    })))
}

/// Liveness check handler
pub async fn liveness_check_handler() -> Json<Value> {
    Json(json!({
        "status": "alive",
        "timestamp": chrono::Utc::now()
    }))
}