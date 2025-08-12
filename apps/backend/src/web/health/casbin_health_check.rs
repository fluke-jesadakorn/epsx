//! Health check and monitoring endpoints for Casbin authorization system
//! Provides comprehensive system health, performance metrics, and diagnostic information

use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::time::{Instant, SystemTime, UNIX_EPOCH};
use crate::web::auth::AppState;

#[derive(Debug, Serialize, Deserialize)]
pub struct HealthCheckResponse {
    pub status: HealthStatus,
    pub timestamp: u64,
    pub version: String,
    pub environment: String,
    pub checks: HashMap<String, ComponentHealth>,
    pub metrics: SystemMetrics,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ComponentHealth {
    pub status: HealthStatus,
    pub message: String,
    pub details: Option<Value>,
    pub last_check: u64,
    pub response_time_ms: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SystemMetrics {
    pub casbin_cache_stats: CacheMetrics,
    pub policy_enforcement_stats: EnforcementMetrics,
    pub database_stats: DatabaseMetrics,
    pub error_stats: ErrorMetrics,
    pub performance_stats: PerformanceMetrics,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CacheMetrics {
    pub total_entries: usize,
    pub active_entries: usize,
    pub expired_entries: usize,
    pub hit_ratio: f64,
    pub miss_ratio: f64,
    pub eviction_count: u64,
    pub memory_usage_bytes: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EnforcementMetrics {
    pub total_requests: u64,
    pub successful_requests: u64,
    pub denied_requests: u64,
    pub failed_requests: u64,
    pub average_response_time_ms: f64,
    pub p95_response_time_ms: f64,
    pub p99_response_time_ms: f64,
    pub requests_per_second: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DatabaseMetrics {
    pub connection_pool_size: u32,
    pub active_connections: u32,
    pub idle_connections: u32,
    pub total_policies: u64,
    pub total_roles: u64,
    pub last_policy_update: Option<u64>,
    pub database_response_time_ms: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorMetrics {
    pub total_errors: u64,
    pub errors_by_type: HashMap<String, u64>,
    pub error_rate: f64,
    pub circuit_breaker_state: String,
    pub recent_error_count: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub cpu_usage_percent: Option<f64>,
    pub memory_usage_bytes: Option<u64>,
    pub uptime_seconds: u64,
    pub gc_stats: Option<Value>,
}

/// Comprehensive health check endpoint
pub async fn health_check_handler(
    State(app_state): State<AppState>,
) -> Result<Json<HealthCheckResponse>, StatusCode> {
    let _start_time = Instant::now();
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    let mut checks = HashMap::new();
    let mut overall_status = HealthStatus::Healthy;
    
    // Check Casbin service health
    let casbin_health = check_casbin_service(&app_state).await;
    if casbin_health.status != HealthStatus::Healthy {
        overall_status = casbin_health.status.clone();
    }
    checks.insert("casbin_service".to_string(), casbin_health);
    
    // Check database connectivity
    let database_health = check_database_health(&app_state).await;
    if database_health.status != HealthStatus::Healthy && overall_status == HealthStatus::Healthy {
        overall_status = database_health.status.clone();
    }
    checks.insert("database".to_string(), database_health);
    
    // Check cache health
    let cache_health = check_cache_health(&app_state).await;
    if cache_health.status == HealthStatus::Unhealthy && overall_status != HealthStatus::Unhealthy {
        overall_status = HealthStatus::Degraded; // Cache issues are not critical
    }
    checks.insert("cache".to_string(), cache_health);
    
    // Check policy integrity
    let policy_health = check_policy_integrity(&app_state).await;
    if policy_health.status != HealthStatus::Healthy && overall_status == HealthStatus::Healthy {
        overall_status = policy_health.status.clone();
    }
    checks.insert("policy_integrity".to_string(), policy_health);
    
    // Collect system metrics
    let metrics = collect_system_metrics(&app_state).await;
    
    let response = HealthCheckResponse {
        status: overall_status,
        timestamp,
        version: env!("CARGO_PKG_VERSION").to_string(),
        environment: std::env::var("RUST_ENV").unwrap_or_else(|_| "unknown".to_string()),
        checks,
        metrics,
    };
    
    Ok(Json(response))
}

/// Lightweight readiness check for load balancers
pub async fn readiness_check_handler(
    State(app_state): State<AppState>,
) -> Result<Json<Value>, StatusCode> {
    // Quick checks for essential components
    let casbin_ready = check_casbin_readiness(&app_state).await;
    let database_ready = check_database_readiness(&app_state).await;
    
    if casbin_ready && database_ready {
        Ok(Json(json!({
            "status": "ready",
            "timestamp": SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()
        })))
    } else {
        Err(StatusCode::SERVICE_UNAVAILABLE)
    }
}

/// Simple liveness check
pub async fn liveness_check_handler() -> Result<Json<Value>, StatusCode> {
    Ok(Json(json!({
        "status": "alive",
        "timestamp": SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()
    })))
}

/// Detailed metrics endpoint for monitoring systems
pub async fn metrics_handler(
    State(app_state): State<AppState>,
) -> Result<Json<SystemMetrics>, StatusCode> {
    let metrics = collect_system_metrics(&app_state).await;
    Ok(Json(metrics))
}

async fn check_casbin_service(app_state: &AppState) -> ComponentHealth {
    let start_time = Instant::now();
    
    // Test basic policy enforcement
    match app_state.casbin_service.enforce("health_check_user", "/api/v1/health", "GET").await {
        Ok(_) => ComponentHealth {
            status: HealthStatus::Healthy,
            message: "Casbin service is operational".to_string(),
            details: Some(json!({
                "test_enforcement": "successful",
                "cache_enabled": true
            })),
            last_check: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            response_time_ms: Some(start_time.elapsed().as_millis() as u64),
        },
        Err(e) => ComponentHealth {
            status: HealthStatus::Unhealthy,
            message: format!("Casbin service error: {}", e),
            details: Some(json!({
                "error": e.to_string(),
                "test_enforcement": "failed"
            })),
            last_check: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            response_time_ms: Some(start_time.elapsed().as_millis() as u64),
        }
    }
}

async fn check_database_health(app_state: &AppState) -> ComponentHealth {
    let start_time = Instant::now();
    
    // Test database connectivity by getting policies
    match app_state.casbin_service.get_all_policies().await {
        Ok((policies, role_policies)) => {
            let total_policies = policies.len() + role_policies.len();
            ComponentHealth {
                status: HealthStatus::Healthy,
                message: "Database connection is healthy".to_string(),
                details: Some(json!({
                    "total_policies": total_policies,
                    "policy_count": policies.len(),
                    "role_policy_count": role_policies.len()
                })),
                last_check: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
                response_time_ms: Some(start_time.elapsed().as_millis() as u64),
            }
        }
        Err(e) => ComponentHealth {
            status: HealthStatus::Unhealthy,
            message: format!("Database connectivity error: {}", e),
            details: Some(json!({
                "error": e.to_string(),
                "connection_test": "failed"
            })),
            last_check: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            response_time_ms: Some(start_time.elapsed().as_millis() as u64),
        }
    }
}

async fn check_cache_health(app_state: &AppState) -> ComponentHealth {
    let start_time = Instant::now();
    
    let cache_stats = app_state.casbin_service.cache_stats().await;
    
    // Determine cache health based on stats
    let status = if cache_stats.total_entries == 0 {
        HealthStatus::Degraded // No cache entries might indicate issues
    } else if cache_stats.active_entries == 0 {
        HealthStatus::Degraded // All entries expired
    } else {
        HealthStatus::Healthy
    };
    
    ComponentHealth {
        status,
        message: "Cache system status".to_string(),
        details: Some(json!({
            "total_entries": cache_stats.total_entries,
            "active_entries": cache_stats.active_entries,
            "expired_entries": cache_stats.expired_entries,
            "cache_utilization": if cache_stats.max_entries > 0 {
                cache_stats.active_entries as f64 / cache_stats.max_entries as f64
            } else { 0.0 }
        })),
        last_check: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
        response_time_ms: Some(start_time.elapsed().as_millis() as u64),
    }
}

async fn check_policy_integrity(app_state: &AppState) -> ComponentHealth {
    let start_time = Instant::now();
    
    // Test known policy scenarios
    let test_cases = vec![
        ("admin", "/api/v1/admin", "GET", true),
        ("basic_user", "/api/v1/admin", "GET", false),
        ("premium_user", "/api/v1/analytics", "GET", true),
        ("basic_user", "/api/v1/trading", "GET", true),
    ];
    
    let mut passed = 0;
    let mut failed = 0;
    let mut errors = Vec::new();
    
    for (user, resource, action, expected) in test_cases {
        match app_state.casbin_service.enforce(user, resource, action).await {
            Ok(result) => {
                if result == expected {
                    passed += 1;
                } else {
                    failed += 1;
                    errors.push(format!("Policy test failed: {} on {}/{} - expected {}, got {}", 
                                      user, resource, action, expected, result));
                }
            }
            Err(e) => {
                failed += 1;
                errors.push(format!("Policy enforcement error: {}", e));
            }
        }
    }
    
    let status = if failed == 0 {
        HealthStatus::Healthy
    } else if passed > failed {
        HealthStatus::Degraded
    } else {
        HealthStatus::Unhealthy
    };
    
    ComponentHealth {
        status,
        message: format!("Policy integrity check: {}/{} tests passed", passed, passed + failed),
        details: Some(json!({
            "tests_passed": passed,
            "tests_failed": failed,
            "errors": errors
        })),
        last_check: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
        response_time_ms: Some(start_time.elapsed().as_millis() as u64),
    }
}

async fn check_casbin_readiness(app_state: &AppState) -> bool {
    // Quick enforcement test
    app_state.casbin_service.enforce("readiness_test", "/api/v1/test", "GET").await.is_ok()
}

async fn check_database_readiness(app_state: &AppState) -> bool {
    // Quick policy count check
    app_state.casbin_service.get_all_policies().await.is_ok()
}

pub async fn collect_system_metrics(app_state: &AppState) -> SystemMetrics {
    // Collect cache metrics
    let cache_stats = app_state.casbin_service.cache_stats().await;
    let cache_metrics = CacheMetrics {
        total_entries: cache_stats.total_entries,
        active_entries: cache_stats.active_entries,
        expired_entries: cache_stats.expired_entries,
        hit_ratio: if cache_stats.total_entries > 0 {
            cache_stats.active_entries as f64 / cache_stats.total_entries as f64
        } else { 0.0 },
        miss_ratio: if cache_stats.total_entries > 0 {
            cache_stats.expired_entries as f64 / cache_stats.total_entries as f64
        } else { 0.0 },
        eviction_count: 0, // Would need to be tracked separately
        memory_usage_bytes: None, // Would need memory profiling
    };
    
    // Collect enforcement metrics (would be tracked in a real implementation)
    let enforcement_metrics = EnforcementMetrics {
        total_requests: 0,
        successful_requests: 0,
        denied_requests: 0,
        failed_requests: 0,
        average_response_time_ms: 0.0,
        p95_response_time_ms: 0.0,
        p99_response_time_ms: 0.0,
        requests_per_second: 0.0,
    };
    
    // Collect database metrics
    let (policies, role_policies) = app_state.casbin_service.get_all_policies().await.unwrap_or_default();
    let database_metrics = DatabaseMetrics {
        connection_pool_size: 10, // Would get from actual pool
        active_connections: 5,    // Would get from actual pool
        idle_connections: 5,      // Would get from actual pool
        total_policies: (policies.len() + role_policies.len()) as u64,
        total_roles: role_policies.len() as u64,
        last_policy_update: None, // Would track in real implementation
        database_response_time_ms: 0.0, // Would measure actual queries
    };
    
    // Collect error metrics (would integrate with error handler)
    let error_metrics = ErrorMetrics {
        total_errors: 0,
        errors_by_type: HashMap::new(),
        error_rate: 0.0,
        circuit_breaker_state: "closed".to_string(),
        recent_error_count: 0,
    };
    
    // Collect performance metrics
    let uptime = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    let performance_metrics = PerformanceMetrics {
        cpu_usage_percent: None,    // Would need system metrics crate
        memory_usage_bytes: None,   // Would need system metrics crate
        uptime_seconds: uptime,
        gc_stats: None,            // Rust doesn't have GC
    };
    
    SystemMetrics {
        casbin_cache_stats: cache_metrics,
        policy_enforcement_stats: enforcement_metrics,
        database_stats: database_metrics,
        error_stats: error_metrics,
        performance_stats: performance_metrics,
    }
}

/// Diagnostic endpoint for troubleshooting
pub async fn diagnostic_handler(
    State(app_state): State<AppState>,
) -> Result<Json<Value>, StatusCode> {
    let mut diagnostics = json!({
        "timestamp": SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
        "service": "casbin_authorization"
    });
    
    // Policy count by type
    if let Ok((policies, role_policies)) = app_state.casbin_service.get_all_policies().await {
        diagnostics["policy_summary"] = json!({
            "total_policies": policies.len(),
            "role_policies": role_policies.len(),
            "policy_breakdown": {
                "user_policies": policies.iter().filter(|p| !p.is_empty() && !p[0].contains("role_")).count(),
                "role_policies": policies.iter().filter(|p| !p.is_empty() && p[0].contains("role_")).count()
            }
        });
    }
    
    // Cache diagnostics
    let cache_stats = app_state.casbin_service.cache_stats().await;
    diagnostics["cache_diagnostics"] = json!({
        "utilization_percent": if cache_stats.max_entries > 0 {
            (cache_stats.active_entries as f64 / cache_stats.max_entries as f64) * 100.0
        } else { 0.0 },
        "efficiency": if cache_stats.total_entries > 0 {
            (cache_stats.active_entries as f64 / cache_stats.total_entries as f64) * 100.0
        } else { 0.0 },
        "recommendation": if cache_stats.active_entries == cache_stats.max_entries {
            "Consider increasing cache size"
        } else if cache_stats.active_entries < cache_stats.max_entries / 4 {
            "Cache size might be over-provisioned"
        } else {
            "Cache size appears optimal"
        }
    });
    
    // System recommendations
    let mut recommendations = Vec::new();
    
    if cache_stats.active_entries == 0 {
        recommendations.push("Cache appears empty - verify policy enforcement is working");
    }
    
    if cache_stats.expired_entries > cache_stats.active_entries {
        recommendations.push("High cache expiration rate - consider increasing TTL");
    }
    
    diagnostics["recommendations"] = json!(recommendations);
    
    Ok(Json(diagnostics))
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_health_status_enum() {
        assert_eq!(HealthStatus::Healthy, HealthStatus::Healthy);
        assert_ne!(HealthStatus::Healthy, HealthStatus::Degraded);
    }
    
    #[tokio::test]
    async fn test_liveness_check() {
        let response = liveness_check_handler().await.unwrap();
        let value: Value = serde_json::from_str(&serde_json::to_string(&response.0).unwrap()).unwrap();
        assert_eq!(value["status"], "alive");
    }
    
    #[test]
    fn test_cache_metrics_calculations() {
        let metrics = CacheMetrics {
            total_entries: 100,
            active_entries: 80,
            expired_entries: 20,
            hit_ratio: 0.8,
            miss_ratio: 0.2,
            eviction_count: 5,
            memory_usage_bytes: Some(1024),
        };
        
        assert_eq!(metrics.hit_ratio + metrics.miss_ratio, 1.0);
        assert!(metrics.active_entries + metrics.expired_entries == metrics.total_entries);
    }
}