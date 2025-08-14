// Web layer implementation

pub mod auth;
pub mod oidc;
pub mod admin;
pub mod permission_profile;
pub mod user;
pub mod middleware;
pub mod modules;
pub mod validation;
pub mod health;
// pub mod analytics; // Temporarily disabled during auth migration
pub mod settings;
pub mod templates;
pub mod admin_assignment;
pub mod simplified_router;

use axum::{
    middleware::from_fn_with_state,
    routing::{get, post},
    Router,
    response::Json,
};
use serde_json::{json, Value};
use std::sync::Arc;

use crate::infra::AppContainer;
use auth::AppState;
use permission_profile::create_permission_profile_router;
use user::user_routes_v1;
use modules::create_modules_router;
// use analytics::create_analytics_router; // Disabled during migration
use settings::create_settings_router;
use validation::comprehensive_validation_middleware;
use auth::handlers::{
    logout_handler, refresh_handler, me_handler, validate_session_handler, 
    validate_route_access_handler, validate_bulk_routes_handler, check_permission_handler, 
    user_features_handler, navigation_handler, single_permission_handler, rotate_session_handler,
    login_handler, register_handler
};

/// Health check handler
pub async fn health_handler() -> Json<Value> {
    Json(json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now(),
        "service": "epsx-backend"
    }))
}

/// Cache handler (placeholder)
pub async fn cache_handler() -> Json<Value> {
    Json(json!({
        "status": "cache_cleared",
        "timestamp": chrono::Utc::now()
    }))
}

/// Premium rankings handler (placeholder)
pub async fn premium_rankings_handler() -> Json<Value> {
    Json(json!({
        "rankings": [],
        "last_updated": chrono::Utc::now()
    }))
}


/// Create v1 API routes
#[allow(dead_code)]
fn create_v1_routes(app_state: AppState, _container: Arc<AppContainer>) -> Router<AppState> {
    // Create public authentication routes (no auth required)
    let public_auth_routes = Router::new()
        .route("/auth/login", post(login_handler))
        .route("/auth/register", post(register_handler))
        .route_layer(from_fn_with_state(
            app_state.clone(),
            comprehensive_validation_middleware,
        ));

    // Create protected authentication routes (auth required)
    let protected_auth_routes = Router::new()
        .route("/auth/logout", post(logout_handler))
        .route("/auth/refresh", post(refresh_handler))
        .route("/auth/profile", get(me_handler))
        // Phase 1: Centralized auth API endpoints
        .route("/auth/validate-session", post(validate_session_handler))
        .route("/auth/validate-access", post(validate_route_access_handler))
        .route("/auth/validate-routes", post(validate_bulk_routes_handler))
        .route("/auth/check-permission", post(check_permission_handler))
        .route("/auth/permission", get(single_permission_handler))
        .route("/auth/features", get(user_features_handler))
        .route("/auth/navigation", get(navigation_handler))
        .route("/auth/rotate-session", post(rotate_session_handler))
        .route_layer(from_fn_with_state(
            app_state.clone(),
            comprehensive_validation_middleware,
        ));

    // Create user admin routes (auth required)
    let user_admin_routes = user_routes_v1();

    // Payment routes (auth required)
    let payment_routes = Router::new()
        .route("/payments/crypto/deposit-address", get(placeholder_crypto_deposit))
        .route("/payments/musepay/create", post(placeholder_musepay_create))
        .route("/webhooks/payments/musepay", post(placeholder_musepay_webhook));

    // System routes (auth required)
    let system_routes = Router::new()
        .route("/system/cache", post(cache_handler));

    // Admin assignment routes (for Firebase admin privileges)
    let admin_assignment_routes = Router::new()
        .route("/admin/users/:user_id/role", post(admin_assignment::assign_admin_role_handler))
        .route("/admin/users/:user_id/claims", get(admin_assignment::get_user_claims_handler));

    // Premium routes (auth + permission required)
    let premium_routes = Router::new()
        .route("/premium/rankings", get(premium_rankings_handler));

    // Permission profile routes for v1 API (auth required)
    let permission_profile_routes_v1 = Router::new()
        .nest("/permission-profiles", create_permission_profile_router());


    // Module routes for v1 API (module auth required)
    let module_routes_v1 = Router::new()
        .nest("/", create_modules_router(app_state.clone()));

    // Analytics routes temporarily disabled during auth migration
    // let analytics_routes_v1 = Router::new()
    //     .nest("/", create_analytics_router(&container.infra))
    //     .nest("/api", create_analytics_router(&container.infra)); // Legacy support

    // Settings routes for v1 API (auth required)
    let settings_routes_v1 = Router::new()
        .nest("/settings", create_settings_router());

    // Placeholder routes for frontend expectations
    let placeholder_routes = Router::new()
        .route("/notifications/subscribe", post(placeholder_notification_handler))
        .route("/notifications/unsubscribe", post(placeholder_notification_handler))
        .route("/monitoring/alerts", post(placeholder_monitoring_handler))
        .route("/monitoring/events", post(placeholder_monitoring_handler))
        .route("/stream", post(placeholder_stream_handler));

    Router::new()
        .route("/health", get(health_handler))
        .merge(public_auth_routes)
        .merge(protected_auth_routes)
        .merge(user_admin_routes)
        .merge(payment_routes)
        .merge(system_routes)
        .merge(admin_assignment_routes)
        .merge(premium_routes)
        .merge(permission_profile_routes_v1)
        .merge(module_routes_v1)
        // .merge(analytics_routes_v1) // Temporarily disabled
        .merge(settings_routes_v1)
        .merge(placeholder_routes)
        .with_state(app_state)
}


/// Placeholder handlers for payment endpoints
#[allow(dead_code)]
async fn placeholder_crypto_deposit() -> Json<Value> {
    Json(json!({
        "message": "Crypto deposit address endpoint - implementation pending",
        "address": null
    }))
}

#[allow(dead_code)]
async fn placeholder_musepay_create() -> Json<Value> {
    Json(json!({
        "message": "MusePay create payment endpoint - implementation pending",
        "payment_id": null
    }))
}

#[allow(dead_code)]
async fn placeholder_musepay_webhook() -> Json<Value> {
    Json(json!({
        "message": "MusePay webhook processed",
        "status": "received"
    }))
}

/// Placeholder handler for notification endpoints
#[allow(dead_code)]
async fn placeholder_notification_handler() -> Json<Value> {
    Json(json!({
        "message": "Notification endpoint not yet implemented",
        "status": "placeholder",
        "timestamp": chrono::Utc::now()
    }))
}

/// Placeholder handler for monitoring endpoints
#[allow(dead_code)]
async fn placeholder_monitoring_handler() -> Json<Value> {
    Json(json!({
        "message": "Monitoring endpoint not yet implemented",
        "status": "placeholder",
        "timestamp": chrono::Utc::now()
    }))
}

/// Placeholder handler for stream endpoints
#[allow(dead_code)]
async fn placeholder_stream_handler() -> Json<Value> {
    Json(json!({
        "message": "Stream endpoint not yet implemented",
        "status": "placeholder",
        "timestamp": chrono::Utc::now()
    }))
}

/// Create the main application router (simplified during auth migration)
pub async fn create_router(container: Arc<AppContainer>) -> Router {
    // Use simplified router with real JWT validation but simplified dependencies
    // This provides security without complex database dependencies during migration
    simplified_router::create_simplified_router(container).await
}

/// Create test application for integration tests
#[cfg(test)]
pub async fn create_test_app() -> Router {
    // Test router with v1 API structure only
    Router::new()
        .route("/health", get(health_handler))
        .route("/api/v1/auth/login", post(health_handler)) // Mock v1 endpoint
        .route("/api/v1/permission-profiles", get(health_handler)) // Mock v1 endpoint
        .route("/api/v1/auth/me", get(health_handler)) // Mock v1 endpoint
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn should_create_health_response() {
        let response = health_handler().await;
        let json_value = response.0;
        
        assert!(json_value.get("status").is_some());
        assert_eq!(json_value["status"], "healthy");
        assert!(json_value.get("timestamp").is_some());
        assert_eq!(json_value["service"], "epsx-backend");
    }
}