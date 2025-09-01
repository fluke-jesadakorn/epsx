// Real-time routes for SSE and event management

use axum::{
    routing::{get, post},
    Router,
};
use super::super::auth::routes::AppState;
use crate::web::middleware::add_deprecation_headers;
use super::{
    sse::{sse_handler, sse_health_handler},
    handlers::{
        broadcast_notification_handler,
        simulate_payment_handler,
        simulate_stock_update_handler,
        get_connection_stats_handler,
        send_user_notification_handler,
    },
};

/// Create real-time SSE routes (legacy, use create_realtime_routes instead)
pub fn realtime_routes() -> Router<AppState> {
    Router::new()
        
        // Legacy Server-Sent Events endpoint (deprecated - conflicts with security events)
        .route("/events", get(sse_handler))
        .route("/events/health", get(sse_health_handler))
        
        // Admin endpoints for event management
        .route("/admin/broadcast", post(broadcast_notification_handler))
        .route("/admin/simulate/payment", post(simulate_payment_handler))
        .route("/admin/simulate/stock", post(simulate_stock_update_handler))
        .route("/admin/stats", get(get_connection_stats_handler))
        .route("/admin/notify/:user_id", post(send_user_notification_handler))
        
        // Apply deprecation headers
        .layer(axum::middleware::from_fn(add_deprecation_headers))
}

/// Create real-time routes for SSE and events with proper versioning
pub fn create_realtime_routes() -> Router<AppState> {
    // Versioned routes
    let v1_routes = Router::new()
        // Server-Sent Events for real-time notifications (proper versioning)
        .route("/api/v1/realtime/events", get(sse_handler))
        .route("/api/v1/realtime/health", get(sse_health_handler))
        // Admin real-time management
        .route("/api/v1/admin/realtime/broadcast", post(broadcast_notification_handler))
        .route("/api/v1/admin/realtime/simulate/payment", post(simulate_payment_handler))
        .route("/api/v1/admin/realtime/simulate/stock", post(simulate_stock_update_handler))
        .route("/api/v1/admin/realtime/stats", get(get_connection_stats_handler))
        .route("/api/v1/admin/realtime/notify/:user_id", post(send_user_notification_handler));

    // Legacy routes for backward compatibility
    let legacy_routes = realtime_routes();

    Router::new()
        .merge(v1_routes)
        .merge(legacy_routes)
}