//! Health check module for monitoring system status
//! Single comprehensive /health endpoint with external service status

use axum::{response::Json, extract::State};
use serde_json::{json, Value};
use utoipa::ToSchema;
use std::sync::Arc;
use sqlx::PgPool;
use crate::infrastructure::cache::Cache;

/// Health state for health endpoint
#[derive(Clone)]
pub struct HealthState {
    pub pool: Arc<PgPool>,
    pub cache: Arc<dyn Cache>,
}

/// Health check response structure
#[derive(serde::Serialize, ToSchema)]
pub struct HealthCheckResponse {
    pub status: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub service: String,
    pub version: String,
    pub services: ServiceStatuses,
}

#[derive(serde::Serialize, ToSchema)]
pub struct ServiceStatuses {
    pub postgres: ServiceStatus,
    pub redis: ServiceStatus,
}

#[derive(serde::Serialize, ToSchema)]
pub struct ServiceStatus {
    pub status: String,
    pub latency_ms: Option<u64>,
}

/// Comprehensive health check endpoint with external service status
#[utoipa::path(
    get,
    path = "/health",
    responses(
        (status = 200, description = "Service health status with external services", body = Value)
    ),
    tag = "health"
)]
pub async fn health_check_handler(
    State(state): State<HealthState>,
) -> Json<Value> {
    let pool = state.pool;
    let cache = state.cache;
    // Check PostgreSQL
    let postgres_start = std::time::Instant::now();
    let postgres_status = match sqlx::query("SELECT 1").fetch_one(pool.as_ref()).await {
        Ok(_) => ServiceStatus {
            status: "connected".to_string(),
            latency_ms: Some(postgres_start.elapsed().as_millis() as u64),
        },
        Err(_) => ServiceStatus {
            status: "disconnected".to_string(),
            latency_ms: None,
        },
    };

    // Check Redis
    let redis_start = std::time::Instant::now();
    let redis_status = match cache.get("health_check") {
        Some(_) => ServiceStatus {
            status: "connected".to_string(),
            latency_ms: Some(redis_start.elapsed().as_millis() as u64),
        },
        None => ServiceStatus {
            status: "disconnected".to_string(),
            latency_ms: None,
        },
    };

    // Overall status
    let overall_status = if postgres_status.status == "connected" && redis_status.status == "connected" {
        "healthy"
    } else if postgres_status.status == "connected" || redis_status.status == "connected" {
        "degraded"
    } else {
        "unhealthy"
    };

    Json(json!({
        "status": overall_status,
        "timestamp": chrono::Utc::now(),
        "service": "epsx-backend",
        "version": "1.0.0",
        "services": {
            "postgres": postgres_status,
            "redis": redis_status,
        }
    }))
}