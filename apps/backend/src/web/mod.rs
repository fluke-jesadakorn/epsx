// Web layer implementation

pub mod auth;
pub mod admin;
pub mod iam;
pub mod audit;
pub mod template;
pub mod user;
pub mod realtime;
pub mod middleware;

use axum::{
    middleware::from_fn_with_state,
    routing::{get, post},
    Router,
    response::Json,
};
use serde_json::{json, Value};
use tower_http::cors::CorsLayer;
use tower_cookies::CookieManagerLayer;
use std::sync::Arc;

use crate::infra::AppContainer;
use crate::app::use_cases::auth::AuthUC;
use crate::app::use_cases::user::UserMgmtUC;
use crate::app::use_cases::iam::IamUC;
use auth::{AppState, auth_routes_v1};
use admin::create_admin_routes;
use iam::create_iam_router;
use audit::create_audit_router;
use template::create_template_router;
use user::{user_routes, user_routes_v1};
use realtime::realtime_routes;
use middleware::auth_middleware::auth_middleware;
use auth::handlers::{logout_handler, refresh_handler, me_handler, me_handler_public};
use auth::enhanced_handlers::{enhanced_login_handler, register_handler, password_reset_handler};

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
fn create_v1_routes(app_state: AppState, _container: Arc<AppContainer>) -> Router<AppState> {
    // Create authentication routes
    let auth_routes = auth_routes_v1();

    // Create user admin routes
    let user_admin_routes = user_routes_v1();

    // Market data routes (placeholder for now)
    let market_data_routes = Router::new()
        .route("/market-data/stocks/screener", get(placeholder_stock_screener))
        .route("/market-data/stocks/eps-growth-ranking", get(placeholder_eps_ranking))
        .route("/market-data/symbols", get(placeholder_symbols));

    // Payment routes (placeholder for now)
    let payment_routes = Router::new()
        .route("/payments/crypto/deposit-address", get(placeholder_crypto_deposit))
        .route("/payments/musepay/create", post(placeholder_musepay_create))
        .route("/webhooks/payments/musepay", post(placeholder_musepay_webhook));

    // System routes  
    let system_routes = Router::new()
        .route("/system/cache", post(cache_handler));

    // Premium routes
    let premium_routes = Router::new()
        .route("/premium/rankings", get(premium_rankings_handler));

    Router::new()
        .merge(auth_routes)
        .merge(user_admin_routes)
        .merge(market_data_routes)
        .merge(payment_routes)
        .merge(system_routes)
        .merge(premium_routes)
        // Apply auth middleware to all routes that need it
        .route_layer(from_fn_with_state(
            app_state.clone(),
            auth_middleware,
        ))
        .with_state(app_state)
}

/// Placeholder handlers for stock endpoints
async fn placeholder_stock_screener() -> Json<Value> {
    Json(json!({
        "message": "Stock screener endpoint - implementation pending",
        "data": []
    }))
}

async fn placeholder_eps_ranking() -> Json<Value> {
    Json(json!({
        "message": "EPS ranking endpoint - implementation pending", 
        "data": []
    }))
}

async fn placeholder_symbols() -> Json<Value> {
    Json(json!({
        "symbols": ["AAPL", "GOOGL", "MSFT", "TSLA"],
        "total": 4
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
    
    // Create app state
    let app_state = AppState::new(
        auth_uc.clone(),
        user_mgmt_uc.clone(),
        iam_uc.clone(),
        container.session_repo.clone(),
        container.user_repo.clone(),
        container.iam_repo.clone(),
        container.audit_repo.clone(),
        container.template_repo.clone(),
    );
    
    // Create public routes
    let public_routes = Router::new()
        .route("/health", get(health_handler))
        .route("/login", post(enhanced_login_handler))
        .route("/register", post(register_handler))
        .route("/password-reset", post(password_reset_handler))
        .route("/auth/me-public", get(me_handler_public));

    // Create protected routes (require authentication) 
    let protected_routes = Router::new()
        .route("/auth/logout", post(logout_handler))
        .route("/auth/refresh", post(refresh_handler))
        .route("/auth/me", get(me_handler))
        .route_layer(from_fn_with_state(
            app_state.clone(),
            auth_middleware,
        ));

    // Create admin routes (require authentication and admin role)
    let admin_routes = Router::new()
        .nest("/admin", create_admin_routes())
        .route_layer(from_fn_with_state(
            app_state.clone(),
            auth_middleware,
        ));

    // Create IAM routes (require authentication and admin role)
    let iam_routes = Router::new()
        .nest("/iam", create_iam_router())
        .route_layer(from_fn_with_state(
            app_state.clone(),
            auth_middleware,
        ));

    // Create audit routes (require authentication and admin role)
    let audit_routes = Router::new()
        .nest("/audit", create_audit_router())
        .route_layer(from_fn_with_state(
            app_state.clone(),
            auth_middleware,
        ));

    // Create template routes (require authentication and admin role)
    let template_routes = Router::new()
        .nest("/templates", create_template_router())
        .route_layer(from_fn_with_state(
            app_state.clone(),
            auth_middleware,
        ));

    // Create user routes (require authentication)
    let user_routes = Router::new()
        .nest("/api", user_routes())
        .route_layer(from_fn_with_state(
            app_state.clone(),
            auth_middleware,
        ));

    // Create real-time routes (require authentication for most endpoints)
    let realtime_routes = Router::new()
        .nest("/realtime", realtime_routes())
        .route_layer(from_fn_with_state(
            app_state.clone(),
            auth_middleware,
        ));

    // Create v1 API routes
    let v1_api_routes = Router::new()
        .nest("/api/v1", create_v1_routes(app_state.clone(), container.clone()));

    // Create admin API routes  
    let admin_api_routes = Router::new()
        .nest("/api/admin", create_admin_routes())
        .route_layer(from_fn_with_state(
            app_state.clone(),
            auth_middleware,
        ));

    Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        .merge(v1_api_routes)
        .merge(admin_api_routes)
        .merge(admin_routes) // Legacy admin routes
        .merge(iam_routes)
        .merge(audit_routes)
        .merge(template_routes)
        .merge(user_routes) // Legacy user routes
        .merge(realtime_routes)
        // Add cookie middleware
        .layer(CookieManagerLayer::new())
        // Add CORS middleware
        .layer(CorsLayer::permissive())
        // Add application state
        .with_state(app_state)
}

/// Create test application for integration tests
#[cfg(test)]
pub async fn create_test_app() -> Router {
    // For now, return a minimal router for basic endpoint testing
    // In a full implementation, this would use proper test dependencies
    Router::new()
        .route("/health", get(health_handler))
        .route("/api/auth/login", post(health_handler)) // Mock endpoint
        .route("/api/templates", get(health_handler)) // Mock endpoint
        .route("/api/user/profile", get(health_handler)) // Mock endpoint
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