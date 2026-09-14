use crate::prelude::TlsPool;
// Health check module for monitoring system status
// Single comprehensive /health endpoint with external service status

use crate::infrastructure::cache::Cache;
use axum::{extract::State, http::StatusCode, response::Json};
use serde_json::{json, Value};
use std::sync::Arc;
use utoipa::ToSchema;

/// Health state for health endpoint
#[derive(Clone)]
pub struct HealthState {
    pub pool: Arc<TlsPool>,
    pub cache: Arc<dyn Cache>,
    pub redis: Option<Arc<crate::infrastructure::redis::RedisPool>>,
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
pub async fn health_check_handler(State(state): State<HealthState>) -> Json<Value> {
    let cache = state.cache;

    // Check PostgreSQL (All Pools)
    let db_health = crate::infrastructure::database::diesel_health_check_all()
        .await
        .unwrap_or(crate::infrastructure::database::AllPoolsHealth {
            primary: false,
            analytics: false,
            notifications: false,
            payments: false,
            healthy: false,
        });

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
    } else if db_health.primary {
        // If at least primary DB is up
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

/// Traffic must not be admitted on a merely live, degraded process.
/// Keep /health's diagnostic response compatible with existing monitors.
pub async fn readiness_handler(state: State<HealthState>) -> (StatusCode, Json<Value>) {
    let probe = async {
        let redis_connected = match &state.redis {
            Some(redis) => redis.health_check().await,
            None => false,
        };
        let mut body = health_check_handler(state).await;
        // A memory-cache fallback is not evidence that the required Redis is up.
        body.0["services"]["redis"]["status"] = json!(if redis_connected {
            "connected"
        } else {
            "disconnected"
        });
        if !redis_connected {
            body.0["status"] = json!("unavailable");
        }
        if let Some(healthy) = crate::infrastructure::services::plan_projection::healthy() {
            body.0["services"]["plan_projection"]["healthy"] = json!(healthy);
            if !healthy {
                body.0["status"] = json!("unavailable");
            }
        }
        body
    };
    match tokio::time::timeout(std::time::Duration::from_secs(5), probe).await {
        Ok(body) => (readiness_status(&body.0), body),
        Err(_) => (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({"status": "unavailable", "reason": "dependency_timeout"})),
        ),
    }
}

fn readiness_status(health: &Value) -> StatusCode {
    if health["status"] == "healthy"
        && health["services"]["database"]["healthy"] == true
        && health["services"]["redis"]["status"] == "connected"
    {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    }
}

#[cfg(test)]
mod readiness_tests {
    use super::*;

    #[test]
    fn readiness_refuses_partial_or_missing_dependency_results() {
        let mut health = json!({"status":"healthy", "services":{
            "database":{"healthy":true}, "redis":{"status":"connected"}
        }});
        assert_eq!(readiness_status(&health), StatusCode::OK);
        health["services"]["database"]["healthy"] = json!(false);
        assert_eq!(readiness_status(&health), StatusCode::SERVICE_UNAVAILABLE);
        health["services"]["database"]["healthy"] = json!(true);
        health["services"]["redis"]["status"] = json!("disconnected");
        assert_eq!(readiness_status(&health), StatusCode::SERVICE_UNAVAILABLE);
        assert_eq!(
            readiness_status(&json!({})),
            StatusCode::SERVICE_UNAVAILABLE
        );
    }
}
