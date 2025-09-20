use axum::{
    routing::{get, post},
    Router,
};
use sqlx::PgPool as DbPool;
use std::sync::Arc;

// Note: get_user_claims and upsert_user handlers missing after Web3 migration

/**
 * Modern Auth.js v5 routes
 * Provides endpoints needed for Auth.js frontend integration
 */
pub fn create_modern_auth_routes() -> Router<Arc<DbPool>> {
    Router::new()
        // Note: Handlers commented out - need Web3 implementation
        // .route("/user-claims", post(get_user_claims))
        // .route("/upsert-user", post(upsert_user))
        
        // Health check endpoint (public)
        .route("/auth/health", get(auth_health_check))
}

/**
 * Simple health check for auth service
 */
async fn auth_health_check() -> &'static str {
    "Auth service is healthy"
}

/**
 * Create Auth.js integration routes for main router
 */
pub fn create_auth_integration_routes() -> Router<Arc<DbPool>> {
    Router::new()
        .nest("/api/v1/auth", create_modern_auth_routes())
        .nest("/api/auth", create_modern_auth_routes()) // Alternative path
}