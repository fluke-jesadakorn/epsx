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

/// Create security routes for internal API use (backend to frontend communication)
pub fn create_security_routes() -> Router<AppState> {
    Router::new()
        // Security event CRUD operations
        .route("/events", post(create_security_event_handler))
        .route("/events", get(get_security_events_handler))
        .route("/events/:event_id/resolve", put(resolve_security_event_handler))
        .route("/events/bulk", post(bulk_security_events_handler))
        
        // Statistics and monitoring
        .route("/stats", get(get_security_stats_handler))
        .route("/security/health", get(get_security_system_health_handler))
        .route("/metrics", post(record_performance_metrics_handler))
}

/// Create admin security routes (for admin dashboard)
pub fn create_admin_security_routes() -> Router<AppState> {
    Router::new()
        // Security monitoring for admins
        .route("/security/events", get(get_security_events_handler))
        .route("/security/stats", get(get_security_stats_handler))
        .route("/security/health", get(get_security_system_health_handler))
        .route("/security/events/:event_id/resolve", put(resolve_security_event_handler))
        .route("/security/events/bulk", post(bulk_security_events_handler))
}