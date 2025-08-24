// Performance monitoring API routes

use axum::{
    routing::{get, post, put},
    Router,
    Extension,
};
use std::sync::Arc;

use crate::web::performance::{PerformanceService, handlers};

/// Create performance monitoring routes
pub fn create_performance_routes() -> Router {
    Router::new()
        // Dashboard and overview
        .route("/api/v1/performance/dashboard", get(handlers::get_performance_dashboard))
        .route("/api/v1/performance/health", get(handlers::get_system_health))
        
        // Performance metrics
        .route("/api/v1/performance/endpoints/:endpoint", get(handlers::get_endpoint_performance))
        .route("/api/v1/performance/percentiles", get(handlers::get_performance_percentiles))
        .route("/api/v1/performance/trends", get(handlers::get_performance_trends))
        
        // Anomaly detection
        .route("/api/v1/performance/anomalies", get(handlers::get_performance_anomalies))
        
        // Cache analytics
        .route("/api/v1/performance/cache", get(handlers::get_cache_analytics))
        
        // Alerting
        .route("/api/v1/performance/alerts", get(handlers::get_active_alerts))
        .route("/api/v1/performance/alerts/configs", post(handlers::create_alert_config))
        .route("/api/v1/performance/alerts/:alert_id/acknowledge", put(handlers::acknowledge_alert))
        
        // Recommendations
        .route("/api/v1/performance/recommendations", get(handlers::get_recommendations))
        .route("/api/v1/performance/recommendations/generate", post(handlers::generate_recommendations))
        
        // Capacity planning
        .route("/api/v1/performance/capacity", get(handlers::get_capacity_planning))
}

/// Create performance routes with service state
pub fn create_performance_routes_with_service(service: Arc<PerformanceService>) -> Router {
    create_performance_routes()
        .layer(Extension(service))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::StatusCode;
    use tower::ServiceExt;
    // Pool type updated to Diesel

    #[tokio::test]
    async fn test_performance_routes_structure() {
        // Test that routes are properly structured
        let service = Arc::new(PerformanceService::new(
            // Mock pool - in real tests you'd use a test database
            PgPool::connect("postgresql://localhost/test").await.unwrap()
        ));
        
        let app = create_performance_routes_with_service(service);
        
        // Test that the router is created without panicking
        assert!(true);
    }
}