// Notification API routes for real-time notifications

use axum::{
    middleware,
    routing::{delete, get, post, put},
    Router,
};

use super::handlers::*;
use crate::web::{
    auth::routes::AppState,
    middleware::{modern_auth::modern_jwt_auth_middleware},
};

/// Create notification routes for user endpoints
pub fn create_notification_routes() -> Router<AppState> {
    Router::new()
        // User notification endpoints
        .route("/api/v1/notifications", get(list_notifications_handler))
        .route("/api/v1/notifications/:id", get(get_notification_handler))
        .route("/api/v1/notifications/read/:id", post(mark_notification_read_handler))
        .route("/api/v1/notifications/read-all", post(mark_all_notifications_read_handler))
        .route("/api/v1/notifications/:id", delete(delete_notification_handler))
        .route("/api/v1/notifications/unread-count", get(get_unread_count_handler))
        
        // Device token management
        .route("/api/v1/notifications/device-token", post(register_device_token_handler))
        
        // Notification preferences
        .route("/api/v1/notifications/preferences", get(get_preferences_handler))
        .route("/api/v1/notifications/preferences", put(update_preferences_handler))
        
        // Apply middleware
        .layer(middleware::from_fn(modern_jwt_auth_middleware))
}

/// Create admin notification routes
pub fn create_admin_notification_routes() -> Router<AppState> {
    Router::new()
        // Admin notification management
        .route("/api/v1/admin/notifications", post(create_admin_notification_handler))
        .route("/api/v1/admin/notifications/broadcast", post(create_admin_notification_handler))
        
        // Apply admin-specific middleware
        .layer(middleware::from_fn(modern_jwt_auth_middleware))
}

// Legacy routes removed - using standardized /api/v1/notifications endpoints

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::StatusCode;
    use axum::ServiceExt;
    use axum::body::Body;
    use axum::http::{Request, Method};

    #[tokio::test]
    async fn test_notification_routes_creation() {
        let routes = create_notification_routes();
        
        // Test that routes can be created without panicking
        let _service = axum::ServiceExt::into_make_service(routes);
    }

    #[tokio::test] 
    async fn test_admin_notification_routes_creation() {
        let routes = create_admin_notification_routes();
        
        // Test that routes can be created without panicking  
        let _service = axum::ServiceExt::into_make_service(routes);
    }
}