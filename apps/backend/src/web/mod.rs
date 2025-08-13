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
pub mod analytics;
pub mod settings;
pub mod templates;
pub mod admin_assignment;

use axum::{
    middleware::from_fn_with_state,
    routing::{get, post},
    Router,
    response::Json,
};
use serde_json::{json, Value};
use std::sync::Arc;

use crate::infra::AppContainer;
use crate::app::use_cases::auth::AuthUC;
use crate::app::use_cases::user::UserMgmtUC;
use auth::AppState;
use admin::{create_admin_routes, create_admin_public_routes};
use permission_profile::create_permission_profile_router;
use user::user_routes_v1;
use modules::create_modules_router;
use analytics::create_analytics_router;
use settings::create_settings_router;
use validation::comprehensive_validation_middleware;
use auth::handlers::{logout_handler, refresh_handler, me_handler, validate_session_handler, validate_route_access_handler, validate_bulk_routes_handler, check_permission_handler, user_features_handler, navigation_handler, single_permission_handler, rotate_session_handler};
use auth::handlers::{login_handler as multi_login_handler, register_handler, register_handler as auto_register_handler, register_handler as password_reset_handler};

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
fn create_v1_routes(app_state: AppState, container: Arc<AppContainer>) -> Router<AppState> {
    // Create public authentication routes (no auth required)
    let public_auth_routes = Router::new()
        .route("/auth/login", post(multi_login_handler))
        .route("/auth/register", post(register_handler))
        .route("/auth/register-auto", post(auto_register_handler))
        .route("/auth/password-reset", post(password_reset_handler))
        .route_layer(from_fn_with_state(
            app_state.clone(),
            comprehensive_validation_middleware,
        ));

    // Create protected authentication routes (auth required)
    let protected_auth_routes = Router::new()
        .route("/auth/logout", post(logout_handler))
        .route("/auth/refresh", post(refresh_handler))
        .route("/auth/profile", get(me_handler))
        .route("/auth/session/clear", post(logout_handler))
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

    // Market data routes (auth required) - Moved to modules system
    // let market_data_routes = Router::new();

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



    // IAM functionality replaced with permission-based system

    // Permission profile routes for v1 API (auth required)
    let permission_profile_routes_v1 = Router::new()
        .nest("/permission-profiles", create_permission_profile_router());


    // Module routes for v1 API (module auth required)
    let module_routes_v1 = Router::new()
        .nest("/", create_modules_router(app_state.clone()));

    // Analytics routes for v1 API (temporarily public for EPS testing)
    let analytics_routes_v1 = Router::new()
        .nest("/", create_analytics_router(&container.infra));

    // Legacy analytics routes (for backward compatibility with frontend API calls)
    let legacy_analytics_routes = Router::new()
        .nest("/api", create_analytics_router(&container.infra));

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
        .route("/health", get(health_handler))  // Add health endpoint for v1 API
        .merge(public_auth_routes)
        .merge(protected_auth_routes)
        .merge(user_admin_routes)
        .merge(payment_routes)
        .merge(system_routes)
        .merge(admin_assignment_routes)
        .merge(premium_routes)
        .merge(permission_profile_routes_v1)
        // .merge(realtime_routes_v1) // Temporarily disabled during migration
        .merge(module_routes_v1)
        .merge(analytics_routes_v1)
        .merge(settings_routes_v1)
        .merge(placeholder_routes)
        .merge(legacy_analytics_routes)  // Add legacy routes support
        .with_state(app_state)
}

/// Get available trading symbols
#[allow(dead_code)]
async fn get_available_symbols() -> Json<Value> {
    Json(json!({
        "symbols": ["AAPL", "GOOGL", "MSFT", "TSLA", "AMZN", "META"],
        "total": 6,
        "last_updated": chrono::Utc::now()
    }))
}

/// Placeholder handlers for payment endpoints
async fn placeholder_crypto_deposit() -> Json<Value> {
    Json(json!({
        "message": "Crypto deposit address endpoint - implementation pending",
        "address": null
    }))
}

async fn placeholder_musepay_create() -> Json<Value> {
    Json(json!({
        "message": "MusePay create payment endpoint - implementation pending",
        "payment_id": null
    }))
}

async fn placeholder_musepay_webhook() -> Json<Value> {
    Json(json!({
        "message": "MusePay webhook processed",
        "status": "received"
    }))
}

/// Placeholder handler for notification endpoints
async fn placeholder_notification_handler() -> Json<Value> {
    Json(json!({
        "message": "Notification endpoint not yet implemented",
        "status": "placeholder",
        "timestamp": chrono::Utc::now()
    }))
}

/// Placeholder handler for monitoring endpoints
async fn placeholder_monitoring_handler() -> Json<Value> {
    Json(json!({
        "message": "Monitoring endpoint not yet implemented",
        "status": "placeholder",
        "timestamp": chrono::Utc::now()
    }))
}

/// Placeholder handler for stream endpoints
async fn placeholder_stream_handler() -> Json<Value> {
    Json(json!({
        "message": "Stream endpoint not yet implemented",
        "status": "placeholder",
        "timestamp": chrono::Utc::now()
    }))
}

/// Create the main application router
pub async fn create_router(container: Arc<AppContainer>) -> Router {
    // Create auth use case
    let auth_uc = Arc::new(AuthUC::new(
        container.user_repo.clone(),
        container.session_repo.clone(),
        container.firebase_admin.clone(),
    ));
    
    // Create user management use case
    let user_mgmt_uc = Arc::new(UserMgmtUC::new(
        container.user_repo.clone(),
        container.event_dispatcher.clone(),
        container.level_history_repo.clone(),
    ));
    
    // Modern JWT-based permission system (replaces Casbin)
    
    
    // Create admin module service for granular admin role management
    use crate::dom::services::admin_module_service::AdminModuleService;
    let admin_module_service = Arc::new(AdminModuleService::new((*container.infra.postgres_pool).clone()));
    
    // Create simple stub implementations for module and usage repos
    use crate::infra::db::stub_repos::{StubModuleRepo, StubUsageRepo};
    let module_repo: Arc<dyn crate::app::ports::repositories::ModuleRepo> = Arc::new(StubModuleRepo::new());
    let usage_repo: Arc<dyn crate::app::ports::repositories::UsageRepo> = Arc::new(StubUsageRepo::new());

    // Create app state (simplified without Casbin)
    let app_state = AppState::new(
        auth_uc.clone(),
        user_mgmt_uc.clone(),
        container.session_repo.clone(),
        container.user_repo.clone(),
        container.iam_repo.clone(),
        container.audit_repo.clone(),
        container.permission_profile_repo.clone(),
        container.temporary_permission_repo.clone(),
        module_repo,
        usage_repo,
        container.firebase_admin.clone(),
        admin_module_service.clone(),
    );
    
    // Create public routes
    let public_routes = Router::new()
        .route("/health", get(health_handler));

    // Real-time routes moved to v1 API structure - legacy routes removed

    // Create v1 API routes
    let v1_api_routes = Router::new()
        .nest("/api/v1", create_v1_routes(app_state.clone(), container.clone()));

    // Create admin API routes with separated public/protected  
    let admin_api_public_routes = Router::new()
        .nest("/api/v1/admin", create_admin_public_routes());

    let admin_api_protected_routes = Router::new()
        .nest("/api/v1/admin", create_admin_routes());

    // Create admin module routes (for module management)
    let admin_module_routes = Router::new()
        .nest("/api/v1/admin", modules::create_admin_modules_router(app_state.clone()));

    // Import modern Auth.js routes
    use crate::web::auth::create_auth_integration_routes;
    use crate::web::middleware::{modern_jwt_auth_middleware, cors_middleware, request_logging_middleware};

    // Create modern Auth.js integration routes
    let auth_integration_routes = create_auth_integration_routes()
        .with_state((*container.infra.postgres_pool).clone());

    Router::new()
        .merge(public_routes)
        .merge(auth_integration_routes) // Add Auth.js integration routes
        .merge(v1_api_routes)
        .merge(admin_api_public_routes)
        .merge(admin_api_protected_routes)
        .merge(admin_module_routes)
        .merge(oidc::routes::oidc_routes())
        // Modern 3-middleware stack (replaces 12+ legacy middlewares)
        .layer(axum::middleware::from_fn(modern_jwt_auth_middleware))
        .layer(axum::middleware::from_fn(cors_middleware))
        .layer(axum::middleware::from_fn(request_logging_middleware))
        // Add application state
        .with_state(app_state)
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