// Web layer implementation

pub mod auth;
pub mod admin;
pub mod iam;
pub mod audit;
pub mod template;
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
use auth::AppState;
use admin::create_admin_routes;
use iam::create_iam_router;
use audit::create_audit_router;
use template::create_template_router;
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

/// Create the main application router
pub fn create_router(container: Arc<AppContainer>) -> Router {
    // Create auth use case
    let auth_uc = Arc::new(AuthUC::new(
        container.user_repo.clone(),
        container.session_repo.clone(),
        container.firebase_auth_svc.clone(),
    ));
    
    // Create user management use case
    let user_mgmt_uc = Arc::new(UserMgmtUC::new(
        container.user_repo.clone(),
        container.event_dispatcher.clone(),
        container.level_history_repo.clone(),
        container.firebase_auth_svc.clone(),
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

    Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        .merge(admin_routes)
        .merge(iam_routes)
        .merge(audit_routes)
        .merge(template_routes)
        // Add cookie middleware
        .layer(CookieManagerLayer::new())
        // Add CORS middleware
        .layer(CorsLayer::permissive())
        // Add application state
        .with_state(app_state)
}

#[cfg(test)]
mod tests {
    #[test]
    fn should_create_health_response() {
        // This would need async test setup
        // For now, just ensure the function exists
    }
}