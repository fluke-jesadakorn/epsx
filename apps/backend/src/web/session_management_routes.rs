use axum::{
    routing::{get, post, delete},
    Router,
};
use std::sync::Arc;

use crate::web::session_management_handlers::SessionManagementHandlers;

/// Create session management routes
pub fn create_session_management_routes() -> Router<Arc<SessionManagementHandlers>> {
    Router::new()
        // Token management endpoints
        .route("/api/v1/auth/token/refresh", post(SessionManagementHandlers::refresh_token))
        .route("/api/v1/auth/token/revoke", post(SessionManagementHandlers::revoke_token))
        
        // Session management endpoints
        .route("/api/v1/sessions/users/:user_id", get(SessionManagementHandlers::get_user_sessions))
        .route("/api/v1/sessions/users/:user_id/:session_id", delete(SessionManagementHandlers::revoke_session))
        .route("/api/v1/sessions/users/:user_id/revoke-all", post(SessionManagementHandlers::revoke_all_user_sessions))
        
        // Security analysis endpoints
        .route("/api/v1/security/analyze/:user_id", post(SessionManagementHandlers::analyze_session_security))
        .route("/api/v1/security/events/:user_id", get(SessionManagementHandlers::get_security_events))
        
        // Cleanup management endpoints
        .route("/api/v1/admin/cleanup/health", get(SessionManagementHandlers::get_cleanup_health))
        .route("/api/v1/admin/cleanup/run", post(SessionManagementHandlers::run_manual_cleanup))
}

/// Create OAuth 2.0 compatible token endpoints  
pub fn create_oauth_token_routes() -> Router<Arc<SessionManagementHandlers>> {
    Router::new()
        // OAuth 2.0 standard endpoints
        .route("/oauth/token", post(SessionManagementHandlers::refresh_token))
        .route("/oauth/revoke", post(SessionManagementHandlers::revoke_token))
}

/// Create session management API routes with proper nesting
pub fn create_comprehensive_session_routes() -> Router<Arc<SessionManagementHandlers>> {
    Router::new()
        .merge(create_session_management_routes())
        .merge(create_oauth_token_routes())
}