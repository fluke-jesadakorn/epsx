// Security event logging API routes
// Defines the routing configuration for security endpoints

use axum::{
    routing::{get, post, put},
    Router,
};

use super::handlers::{
    create_security_event_handler,
    get_security_events_handler,
    get_security_stats_handler,
    resolve_security_event_handler,
    bulk_security_events_handler,
    get_security_system_health_handler,
    record_performance_metrics_handler,
};
use crate::web::auth::AppState;
use crate::web::middleware::add_deprecation_headers;

/// Create security routes with proper versioning and no conflicts
pub fn create_security_routes() -> Router<AppState> {
    // Versioned routes
    let v1_routes = Router::new()
        // Security event CRUD operations (avoid conflict with realtime /events)
        .route("/api/v1/security/events", post(create_security_event_handler))
        .route("/api/v1/security/events", get(get_security_events_handler))
        .route("/api/v1/security/events/:event_id/resolve", put(resolve_security_event_handler))
        .route("/api/v1/security/events/bulk", post(bulk_security_events_handler))
        
        // Statistics and monitoring
        .route("/api/v1/security/stats", get(get_security_stats_handler))
        .route("/api/v1/security/health", get(get_security_system_health_handler))
        .route("/api/v1/security/metrics", post(record_performance_metrics_handler))
        
        // Admin security endpoints
        .route("/api/v1/admin/security/events", get(get_security_events_handler))
        .route("/api/v1/admin/security/stats", get(get_security_stats_handler))
        .route("/api/v1/admin/security/health", get(get_security_system_health_handler))
        .route("/api/v1/admin/security/events/:event_id/resolve", put(resolve_security_event_handler))
        .route("/api/v1/admin/security/events/bulk", post(bulk_security_events_handler));

    // Legacy routes removed to avoid conflicts with realtime /events
    // Use versioned routes: /api/v1/security/events instead

    // Return only versioned routes to avoid conflicts
    v1_routes
}

/// Create admin security routes (deprecated - use create_security_routes instead)
pub fn create_admin_security_routes() -> Router<AppState> {
    Router::new()
        // Legacy admin security routes (deprecated)
        .route("/security/events", get(get_security_events_handler))
        .route("/security/stats", get(get_security_stats_handler))
        .route("/security/health", get(get_security_system_health_handler))
        .route("/security/events/:event_id/resolve", put(resolve_security_event_handler))
        .route("/security/events/bulk", post(bulk_security_events_handler))
        .layer(axum::middleware::from_fn(add_deprecation_headers))
}