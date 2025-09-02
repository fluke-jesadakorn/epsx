// ============================================================================
// FCM API ENDPOINTS MODULE
// ============================================================================
// API endpoints for Firebase Cloud Messaging operations

pub mod token_endpoints;
pub mod push_endpoints;  
pub mod admin_endpoints;
pub mod admin_system_endpoints;
pub mod admin_template_endpoints;
pub mod test_endpoints;

// Re-export all endpoint handlers
pub use token_endpoints::*;
pub use push_endpoints::*;
pub use admin_endpoints::*;
pub use admin_system_endpoints::*;
pub use admin_template_endpoints::*;
pub use test_endpoints::*;

use axum::{
    routing::{get, post},
    Router,
};

use crate::infra::container::AppContainer;

// Temporary user struct for FCM endpoints
#[derive(Debug, Clone)]
pub struct AuthenticatedUser {
    pub user_id: String,
    pub display_name: Option<String>,
}

impl AuthenticatedUser {
    pub fn has_admin_access(&self) -> bool {
        // TODO: Implement proper admin check
        true
    }
}

/// Configure FCM routes
pub fn fcm_routes() -> Router<AppContainer> {
    Router::new()
        // FCM Token Management Routes
        .route("/tokens/register", post(register_fcm_token))
        .route("/tokens/my", get(get_my_fcm_tokens))
        .route("/tokens/:token_id/deactivate", post(deactivate_fcm_token))
        
        // Push Notification Routes (User)
        .route("/notifications/test", post(send_test_notification))
        
        // Admin FCM Routes
        .route("/admin/tokens", get(admin_get_all_tokens))
        .route("/admin/tokens/:user_id", get(admin_get_user_tokens))
        .route("/admin/tokens/:token_id/deactivate", post(admin_deactivate_token))
        .route("/admin/push/user/:user_id", post(admin_send_to_user))
        .route("/admin/push/broadcast", post(admin_send_broadcast))
        .route("/admin/push/platform/:platform", post(admin_send_to_platform))
        .route("/admin/fcm/stats", get(admin_get_fcm_stats))
        
        // Admin System Message Routes
        .route("/admin/system-messages", get(get_system_messages))
        .route("/admin/system-messages", post(create_system_message))
        .route("/admin/system-messages/:message_id", post(update_system_message))
        .route("/admin/system-messages/:message_id", axum::routing::delete(delete_system_message))
        .route("/admin/system-messages/test", post(test_system_message))
        .route("/admin/system-messages/bulk", post(send_bulk_system_message))
        .route("/admin/system-messages/stats", get(get_system_notification_stats))
        
        // Admin Template Routes
        .route("/admin/templates", get(get_templates))
        .route("/admin/templates", post(create_template))
        .route("/admin/templates/:template_id", get(get_template))
        .route("/admin/templates/:template_id", axum::routing::put(update_template))
        .route("/admin/templates/:template_id", axum::routing::delete(delete_template))
        .route("/admin/templates/render", post(render_template))
        .route("/admin/templates/stats", get(get_template_stats))
        
        // Test FCM Routes (no auth required)
        .route("/test/health", get(test_fcm_health))
        .route("/test/register-token", post(test_register_token))
        .route("/test/send-notification", post(test_send_notification))
        .route("/test/user/:user_id/tokens", get(test_get_user_tokens))
}