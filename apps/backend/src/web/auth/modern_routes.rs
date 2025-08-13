use axum::{
    routing::{get, post},
    Router,
};
use sqlx::PgPool;

use super::modern_handlers::{get_user_claims, upsert_user};

/**
 * Modern Auth.js v5 routes
 * Provides endpoints needed for Auth.js frontend integration
 */
pub fn create_modern_auth_routes() -> Router<PgPool> {
    Router::new()
        // User claims endpoint for JWT token generation
        .route("/user-claims", post(get_user_claims))
        
        // User upsert endpoint for OAuth sign-in
        .route("/upsert-user", post(upsert_user))
        
        // Health check endpoint (public)
        .route("/health", get(auth_health_check))
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
pub fn create_auth_integration_routes() -> Router<PgPool> {
    Router::new()
        .nest("/api/v1/auth", create_modern_auth_routes())
        .nest("/api/auth", create_modern_auth_routes()) // Alternative path
}