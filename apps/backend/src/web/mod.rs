// Web layer implementation

pub mod auth;
pub mod admin;
pub mod iam;
pub mod permission_profile;
pub mod user;
pub mod realtime;
pub mod middleware;
pub mod modules;
pub mod validation;
pub mod market_data;

use axum::{
    middleware::from_fn_with_state,
    routing::{get, post},
    Router,
    response::Json,
};
use serde_json::{json, Value};
use tower_http::cors::CorsLayer;
use std::sync::Arc;

use crate::infra::AppContainer;
use crate::app::use_cases::auth::AuthUC;
use crate::app::use_cases::user::UserMgmtUC;
use crate::app::use_cases::iam::IamUC;
use auth::AppState;
use admin::{create_admin_routes, create_admin_public_routes};
use iam::create_iam_router;
use permission_profile::create_permission_profile_router;
use user::user_routes_v1;
use realtime::realtime_routes;
use modules::create_modules_router;
use middleware::{
    auth_middleware, 
    permission_middleware::permission_middleware,
    error_handling::error_handling_middleware
};
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

/// Create CORS layer with environment-based allowed origins
fn create_cors_layer() -> CorsLayer {
    let frontend_url = std::env::var("FRONTEND_URL")
        .unwrap_or_else(|_| "http://localhost:3000".to_string());
    let admin_frontend_url = std::env::var("ADMIN_FRONTEND_URL")
        .unwrap_or_else(|_| "http://localhost:3001".to_string());
    
    // Production URLs (fallback to development if not set)
    let production_frontend_url = std::env::var("PRODUCTION_FRONTEND_URL")
        .unwrap_or_else(|_| "https://epsx.com".to_string());
    let production_admin_url = std::env::var("PRODUCTION_ADMIN_URL")
        .unwrap_or_else(|_| "https://admin.epsx.com".to_string());
    
    let allowed_origins = vec![
        frontend_url.parse().expect("Invalid FRONTEND_URL"),
        admin_frontend_url.parse().expect("Invalid ADMIN_FRONTEND_URL"),
        production_frontend_url.parse().expect("Invalid PRODUCTION_FRONTEND_URL"),
        production_admin_url.parse().expect("Invalid PRODUCTION_ADMIN_URL"),
    ];
    
    tracing::info!("CORS allowed origins: {:?}", allowed_origins);
    
    CorsLayer::new()
        .allow_origin(allowed_origins)
        .allow_methods([
            axum::http::Method::GET,
            axum::http::Method::POST,
            axum::http::Method::PUT,
            axum::http::Method::DELETE,
            axum::http::Method::OPTIONS,
        ])
        .allow_headers([
            axum::http::header::AUTHORIZATION,
            axum::http::header::CONTENT_TYPE,
            axum::http::header::ACCEPT,
            axum::http::header::ORIGIN,
            axum::http::header::USER_AGENT,
        ])
        .allow_credentials(true)
        .max_age(std::time::Duration::from_secs(3600))
}

/// Create v1 API routes
fn create_v1_routes(app_state: AppState, _container: Arc<AppContainer>) -> Router<AppState> {
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
            auth_middleware,
        ))
        .route_layer(from_fn_with_state(
            app_state.clone(),
            comprehensive_validation_middleware,
        ));

    // Create user admin routes (auth required)
    let user_admin_routes = user_routes_v1()
        .route_layer(from_fn_with_state(
            app_state.clone(),
            auth_middleware,
        ));

    // Market data routes (auth required) - Moved to modules system
    // let market_data_routes = Router::new();

    // Payment routes (auth required)
    let payment_routes = Router::new()
        .route("/payments/crypto/deposit-address", get(placeholder_crypto_deposit))
        .route("/payments/musepay/create", post(placeholder_musepay_create))
        .route("/webhooks/payments/musepay", post(placeholder_musepay_webhook))
        .route_layer(from_fn_with_state(
            app_state.clone(),
            auth_middleware,
        ));

    // System routes (auth required)
    let system_routes = Router::new()
        .route("/system/cache", post(cache_handler))
        .route_layer(from_fn_with_state(
            app_state.clone(),
            auth_middleware,
        ));

    // Premium routes (auth + permission required)
    let premium_routes = Router::new()
        .route("/premium/rankings", get(premium_rankings_handler))
        .route_layer(from_fn_with_state(
            app_state.clone(),
            permission_middleware,
        ))
        .route_layer(from_fn_with_state(
            app_state.clone(),
            auth_middleware,
        ));


    // Admin routes for v1 API (auth required)
    let admin_routes_v1 = Router::new()
        .nest("/admin", create_admin_routes())
        .route_layer(from_fn_with_state(
            app_state.clone(),
            auth_middleware,
        ));

    // IAM routes for v1 API (auth required)
    let iam_routes_v1 = Router::new()
        .nest("/iam", create_iam_router())
        .route_layer(from_fn_with_state(
            app_state.clone(),
            auth_middleware,
        ));

    // Permission profile routes for v1 API (auth required)
    let permission_profile_routes_v1 = Router::new()
        .nest("/permission-profiles", create_permission_profile_router())
        .route_layer(from_fn_with_state(
            app_state.clone(),
            auth_middleware,
        ));

    // Real-time routes for v1 API (auth required)
    let realtime_routes_v1 = Router::new()
        .nest("/realtime", realtime_routes())
        .route_layer(from_fn_with_state(
            app_state.clone(),
            auth_middleware,
        ));

    // Module routes for v1 API (module auth required)
    let module_routes_v1 = Router::new()
        .nest("/", create_modules_router(app_state.clone()));

    Router::new()
        .merge(public_auth_routes)
        .merge(protected_auth_routes)
        .merge(user_admin_routes)
        .merge(payment_routes)
        .merge(system_routes)
        .merge(premium_routes)
        .merge(admin_routes_v1)
        .merge(iam_routes_v1)
        .merge(permission_profile_routes_v1)
        .merge(realtime_routes_v1)
        .merge(module_routes_v1)
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

/// Create the main application router
pub fn create_router(container: Arc<AppContainer>) -> Router {
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
    
    // Create IAM use case
    let iam_uc = Arc::new(IamUC::new(
        container.user_repo.clone(),
        container.iam_repo.clone(),
        Arc::new(tokio::sync::Mutex::new(
            crate::dom::services::policy_engine::PolicyEngine::new()
        )),
    ));
    
    // Create temporary stub implementations for module and usage repos  
    // TODO: Replace with proper implementations
    use crate::app::ports::repositories::{ModuleRepo, UsageRepo};
    
    struct StubModuleRepo;
    impl StubModuleRepo {
        fn new() -> Self { Self }
    }
    #[async_trait::async_trait]
    impl ModuleRepo for StubModuleRepo {
        async fn create_sub_module(&self, _module: &crate::dom::entities::SubModule) -> Result<(), crate::dom::error::DomainError> { Ok(()) }
        async fn update_sub_module(&self, _module: &crate::dom::entities::SubModule) -> Result<(), crate::dom::error::DomainError> { Ok(()) }
        async fn delete_sub_module(&self, _module_id: &uuid::Uuid) -> Result<(), crate::dom::error::DomainError> { Ok(()) }
        async fn get_sub_module(&self, _module_id: &uuid::Uuid) -> Result<Option<crate::dom::entities::SubModule>, crate::dom::error::DomainError> { Ok(None) }
        async fn get_sub_module_by_name(&self, _name: &str) -> Result<Option<crate::dom::entities::SubModule>, crate::dom::error::DomainError> { Ok(None) }
        async fn list_active_modules(&self) -> Result<Vec<crate::dom::entities::SubModule>, crate::dom::error::DomainError> { Ok(vec![]) }
        async fn create_assignment(&self, _assignment: &crate::dom::entities::UserSubModuleAssignment) -> Result<(), crate::dom::error::DomainError> { Ok(()) }
        async fn update_assignment(&self, _assignment: &crate::dom::entities::UserSubModuleAssignment) -> Result<(), crate::dom::error::DomainError> { Ok(()) }
        async fn delete_assignment(&self, _assignment_id: &uuid::Uuid) -> Result<(), crate::dom::error::DomainError> { Ok(()) }
        async fn get_assignment(&self, _assignment_id: &uuid::Uuid) -> Result<Option<crate::dom::entities::UserSubModuleAssignment>, crate::dom::error::DomainError> { Ok(None) }
        async fn get_user_module_assignments(&self, _user_id: &crate::dom::values::UserId) -> Result<Vec<crate::web::middleware::module_auth_middleware::UserModuleAccess>, crate::dom::error::DomainError> { Ok(vec![]) }
        async fn has_user_module_access(&self, _user_id: &crate::dom::values::UserId, _module_name: &str) -> Result<bool, crate::dom::error::DomainError> { Ok(false) }
        async fn get_user_access_level(&self, _user_id: &crate::dom::values::UserId, _module_name: &str) -> Result<Option<String>, crate::dom::error::DomainError> { Ok(None) }
        async fn create_api_key(&self, _api_key: &crate::dom::entities::ApiKey) -> Result<(), crate::dom::error::DomainError> { Ok(()) }
        async fn update_api_key(&self, _api_key: &crate::dom::entities::ApiKey) -> Result<(), crate::dom::error::DomainError> { Ok(()) }
        async fn delete_api_key(&self, _key_id: &uuid::Uuid) -> Result<(), crate::dom::error::DomainError> { Ok(()) }
        async fn get_api_key(&self, _key_id: &uuid::Uuid) -> Result<Option<crate::dom::entities::ApiKey>, crate::dom::error::DomainError> { Ok(None) }
        async fn get_api_key_by_hash(&self, _key_hash: &str) -> Result<Option<crate::dom::entities::ApiKey>, crate::dom::error::DomainError> { Ok(None) }
        async fn get_api_key_access(&self, _key_hash: &str) -> Result<Option<crate::web::middleware::module_auth_middleware::ApiKeyAccess>, crate::dom::error::DomainError> { Ok(None) }
        async fn log_usage(&self, _usage_log: &crate::dom::entities::ModuleUsageLog) -> Result<(), crate::dom::error::DomainError> { Ok(()) }
        async fn get_current_usage(&self, _user_id: &crate::dom::values::UserId, _module_name: &str, _quota_type: &str) -> Result<i32, crate::dom::error::DomainError> { Ok(0) }
        async fn get_quota_limits(&self, _user_id: &crate::dom::values::UserId, _module_name: &str) -> Result<std::collections::HashMap<String, i32>, crate::dom::error::DomainError> { Ok(std::collections::HashMap::new()) }
        async fn check_quota_availability(&self, _user_id: &crate::dom::values::UserId, _module_name: &str, _quota_type: &str, _amount: i32) -> Result<bool, crate::dom::error::DomainError> { Ok(true) }
    }
    
    struct StubUsageRepo;
    impl StubUsageRepo {
        fn new() -> Self { Self }
    }
    #[async_trait::async_trait]
    impl UsageRepo for StubUsageRepo {
        async fn log_usage(&self, _usage: crate::dom::entities::ModuleUsageLog) -> Result<(), crate::dom::error::DomainError> { Ok(()) }
        async fn get_usage_stats(&self, _user_id: &crate::dom::values::UserId, _module_name: &str) -> Result<std::collections::HashMap<String, i32>, crate::dom::error::DomainError> { Ok(std::collections::HashMap::new()) }
        async fn get_current_usage(&self, _user_id: &crate::dom::values::UserId, _module_name: &str, _quota_type: &str) -> Result<i32, crate::dom::error::DomainError> { Ok(0) }
    }
    
    let stub_module_repo: Arc<dyn ModuleRepo> = Arc::new(StubModuleRepo::new());
    let stub_usage_repo: Arc<dyn UsageRepo> = Arc::new(StubUsageRepo::new());

    // Create app state
    let app_state = AppState::new(
        auth_uc.clone(),
        user_mgmt_uc.clone(),
        iam_uc.clone(),
        container.session_repo.clone(),
        container.user_repo.clone(),
        container.iam_repo.clone(),
        container.audit_repo.clone(),
        container.permission_profile_repo.clone(),
        stub_module_repo,
        stub_usage_repo,
        container.firebase_admin.clone(),
    );
    
    // Create public routes
    let public_routes = Router::new()
        .route("/health", get(health_handler));
        // .route("/auth/me-public", get(me_handler_public)); // Removed: no longer needed with bearer tokens

    // Real-time routes moved to v1 API structure - legacy routes removed

    // Create v1 API routes
    let v1_api_routes = Router::new()
        .nest("/api/v1", create_v1_routes(app_state.clone(), container.clone()));

    // Create admin API routes with separated public/protected  
    let admin_api_public_routes = Router::new()
        .nest("/api/admin", create_admin_public_routes());

    let admin_api_protected_routes = Router::new()
        .nest("/api/admin", create_admin_routes())
        .route_layer(from_fn_with_state(
            app_state.clone(),
            auth_middleware,
        ));

    // Create admin module routes (for module management)
    let admin_module_routes = Router::new()
        .nest("/api/admin", modules::create_admin_modules_router(app_state.clone()));

    Router::new()
        .merge(public_routes)
        .merge(v1_api_routes)
        .merge(admin_api_public_routes)
        .merge(admin_api_protected_routes)
        .merge(admin_module_routes)
        // Add error handling middleware (first layer to catch all errors)
        .layer(from_fn_with_state(
            app_state.clone(),
            error_handling_middleware,
        ))
        // Add CORS middleware with environment-based origins
        .layer(create_cors_layer())
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