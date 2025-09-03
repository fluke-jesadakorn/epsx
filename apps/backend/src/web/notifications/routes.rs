use axum::{
    routing::{get, post, put},
    Router,
};

use super::handlers;

pub fn notification_routes() -> Router {
    Router::new()
        // User endpoints
        .route("/fcm/register", post(handlers::register_fcm_token))
        .route("/topics/subscribe", post(handlers::subscribe_to_topics))
        .route("/", get(handlers::get_user_notifications)) // Main user notifications endpoint
        .route("/user", get(handlers::get_user_notifications)) // Legacy compatibility
        .route("/unread", get(handlers::get_unread_notifications))
        .route("/preferences", get(handlers::get_preferences))
        .route("/preferences", put(handlers::update_preferences))
        .route("/track", post(handlers::track_notification))
        // Admin endpoints
        .route("/send", post(handlers::send_notification))
        .route("/broadcast", post(handlers::broadcast_to_topic))
        .route("/stats", get(handlers::get_notification_stats))
        .route("/security-alert", post(handlers::send_security_alert))
}