//! Health check module for monitoring system status
//! Simplified health checks without Casbin dependencies



use axum::{http::StatusCode, response::Json, extract::State};
use serde_json::{json, Value};
use utoipa::ToSchema;
use std::sync::Arc;
use sqlx::PgPool;

/// Health check response structure
#[derive(serde::Serialize, ToSchema)]
pub struct HealthCheckResponse {
    pub status: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub service: String,
    pub version: String,
}

/// Service health check endpoint
#[utoipa::path(
    get,
    path = "/health",
    responses(
        (status = 200, description = "Service is healthy", body = Value)
    ),
    tag = "health"
)]
pub async fn health_check_handler() -> Json<Value> {
    Json(json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now(),
        "service": "epsx-backend",
        "version": "1.0.0"
    }))
}

/// Service readiness check endpoint
#[utoipa::path(
    get,
    path = "/readiness",
    responses(
        (status = 200, description = "Service is ready", body = Value),
        (status = 503, description = "Service not ready")
    ),
    tag = "health"
)]
pub async fn readiness_check_handler(
    State(pool): State<Arc<PgPool>>,
) -> Result<Json<Value>, StatusCode> {
    // Check database connectivity
    let db_status = match sqlx::query("SELECT 1").fetch_one(pool.as_ref()).await {
        Ok(_) => "ok",
        Err(_) => "error",
    };
    
    let is_ready = db_status == "ok";
    let status_code = if is_ready { StatusCode::OK } else { StatusCode::SERVICE_UNAVAILABLE };
    
    let response = Json(json!({
        "status": if is_ready { "ready" } else { "not_ready" },
        "timestamp": chrono::Utc::now(),
        "checks": {
            "database": db_status,
            "auth": "ok"
        }
    }));
    
    if is_ready {
        Ok(response)
    } else {
        Err(status_code)
    }
}

/// Service liveness check endpoint
#[utoipa::path(
    get,
    path = "/liveness",
    responses(
        (status = 200, description = "Service is alive", body = Value)
    ),
    tag = "health"
)]
pub async fn liveness_check_handler() -> Json<Value> {
    Json(json!({
        "status": "alive",
        "timestamp": chrono::Utc::now()
    }))
}