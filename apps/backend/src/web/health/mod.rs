use crate::prelude::TlsPool;
// Health check module for monitoring system status
// Single comprehensive /health endpoint with external service status

use axum::{response::Json, extract::State};
use serde_json::{json, Value};
use utoipa::ToSchema;
use std::sync::Arc;
// use diesel::prelude::*;
use crate::infrastructure::cache::Cache;

/// Health state for health endpoint
#[derive(Clone)]
pub struct HealthState {
    pub pool: Arc<&'static TlsPool>,
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
    pub database: crate::infrastructure::database::AllPoolsHealth,
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
    let cache = state.cache;

    // Check PostgreSQL (All Pools)
    let db_health = crate::infrastructure::database::diesel_health_check_all().await;

    // Check Redis
    let redis_start = std::time::Instant::now();
    let redis_status = match cache.health_check() {
        Ok(_) => ServiceStatus {
            status: "connected".to_string(),
            latency_ms: Some(redis_start.elapsed().as_millis() as u64),
        },
        Err(e) => {
            tracing::warn!("Redis health check failed: {}", e);
            ServiceStatus {
                status: "disconnected".to_string(),
                latency_ms: None,
            }
        }
    };

    // Overall status
    let overall_status = if db_health.healthy && redis_status.status == "connected" {
        "healthy"
    } else if db_health.primary { // If at least primary DB is up
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
            "database": db_health,
            "redis": redis_status,
        }
    }))
}