// Real-time routes for WebSocket, SSE, and event management

use axum::{
    routing::{get, post},
    Router,
};
use super::super::auth::routes::AppState;
use super::{
    websocket::websocket_handler,
    sse::{sse_handler, sse_health_handler},
    handlers::{
        broadcast_notification_handler,
        simulate_payment_handler,
        simulate_stock_update_handler,
        get_connection_stats_handler,
        send_user_notification_handler,
    },
};

/// Create real-time WebSocket and SSE routes
pub fn realtime_routes() -> Router<AppState> {
    Router::new()
        // WebSocket endpoint
        .route("/ws", get(websocket_handler))
        
        // Server-Sent Events endpoint
        .route("/events", get(sse_handler))
        .route("/events/health", get(sse_health_handler))
        
        // Admin endpoints for event management
        .route("/admin/broadcast", post(broadcast_notification_handler))
        .route("/admin/simulate/payment", post(simulate_payment_handler))
        .route("/admin/simulate/stock", post(simulate_stock_update_handler))
        .route("/admin/stats", get(get_connection_stats_handler))
        .route("/admin/notify/:user_id", post(send_user_notification_handler))
}