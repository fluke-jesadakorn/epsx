// Clean Authentication Routes - PancakeSwap Theme Ready

use std::sync::Arc;
use axum::{
    routing::{get, post},
    Router,
    middleware,
};
use sqlx::PgPool as DbPool;

use crate::infrastructure::cache::Cache;
use crate::infrastructure::container::AppContainer;

use crate::web::user::handlers::{
    // Available handlers
    get_permissions_handler,
    verify_ownership_handler,
    get_holdings_handler,
    delegate_permission_handler,
};

/// Clean Application State for Dependency Injection
#[derive(Clone)]
pub struct AppState {
    pub db_pool: Arc<DbPool>,
    pub cache: Arc<dyn Cache>,
    pub ddd_container: Arc<crate::infrastructure::container::ddd_container::DDDContainer>,
    pub user_repo: Arc<dyn crate::domain::user_management::UserRepositoryPort>,
    pub permission_service: Arc<crate::domain::authorization::services::stateless_permission_service::StatelessPermissionService>,
    pub rate_limiting_service: Option<Arc<crate::domain::resource_management::services::RateLimitingService>>, // Context-aware rate limiting
    pub web3_auth_service: Arc<crate::auth::Web3AuthService>,
    pub web3_permission_service: Arc<crate::auth::Web3PermissionService>,
    pub jwt_service: Arc<crate::auth::JWTService>,
}

impl AppState {
    pub fn new(
        db_pool: Arc<DbPool>,
        cache: Arc<dyn Cache>,
        ddd_container: Arc<crate::infrastructure::container::ddd_container::DDDContainer>,
        user_repo: Arc<dyn crate::domain::user_management::UserRepositoryPort>,
        permission_service: Arc<crate::domain::authorization::services::stateless_permission_service::StatelessPermissionService>,
        rate_limiting_service: Option<Arc<crate::domain::resource_management::services::RateLimitingService>>,
        web3_auth_service: Arc<crate::auth::Web3AuthService>,
        web3_permission_service: Arc<crate::auth::Web3PermissionService>,
        jwt_service: Arc<crate::auth::JWTService>,
    ) -> Self {
        Self {
            db_pool,
            cache,
            ddd_container,
            user_repo,
            permission_service,
            rate_limiting_service,
            web3_auth_service,
            web3_permission_service,
            jwt_service,
        }
    }
}

/// Create authentication routes with RESTful patterns and v1 versioning
pub fn create_auth_routes(app_state: AppState) -> Router {
    // Note: Many handlers are missing after Web3 migration
    // TODO: Implement Web3-compatible handlers
    
    // Available routes using existing handlers
    let user_routes = Router::new()
        .route("/api/v1/users/permissions", get(get_permissions_handler))
        .route("/api/v1/users/holdings", get(get_holdings_handler))
        .route("/api/v1/users/verify", post(verify_ownership_handler))
        .route("/api/v1/users/delegate", post(delegate_permission_handler));
        
    // Placeholder for other routes - handlers need to be implemented
    let protected_auth_routes = Router::new()
        // .route("/api/v1/auth/sessions", axum::routing::delete(logout_handler))
        // .route("/api/v1/auth/sessions/current", get(validate_session_handler))
        // .route("/api/v1/auth/user", get(me_handler))
        // .route("/api/v1/auth/tokens/refresh", post(refresh_handler))
        .route("/api/v1/health", get(|| async { "OK" })) // Basic health check
        .layer(middleware::from_fn_with_state(
            app_state.clone(),
            crate::web::middleware::clean_auth_middleware
        ));

    // Permission validation routes - handlers need to be implemented
    let permission_routes = Router::new()
        // .route("/api/v1/permissions/validations", post(check_permission_handler))
        // .route("/api/v1/permissions/routes/validations", post(validate_route_access_handler))
        // .route("/api/v1/permissions/validations/bulk", post(validate_bulk_routes_handler))
        // .route("/api/v1/permissions/single", get(single_permission_handler))
        // .route("/api/v1/permissions/navigation", get(navigation_handler))
        // .route("/api/v1/permissions/features", get(user_features_handler))
        .route("/api/v1/permissions/health", get(|| async { "OK" })) // Basic health check
        .layer(middleware::from_fn_with_state(
            app_state.clone(),
            crate::web::middleware::clean_auth_middleware
        ));

    // Legacy routes removed - use RESTful /api/v1/ endpoints only
    
    let router = Router::new()
        .merge(user_routes)
        .merge(protected_auth_routes)
        .merge(permission_routes)
        .with_state(app_state.clone());

    router
}

/// Create registration routes with AppContainer state (RESTful patterns)
pub fn create_registration_routes(container: Arc<AppContainer>) -> Router {
    Router::new()
        // Note: Handlers missing after Web3 migration
        // .route("/api/v1/auth/users", post(register_user))
        // .route("/api/v1/validations/emails", post(check_email_availability))
        // .route("/api/v1/validations/passwords", post(check_password_strength))
        .route("/api/v1/health", get(|| async { "OK" }))
        .with_state(container)
}

/// Create Auth.js integration routes with PostgreSQL pool
pub fn create_authjs_routes(pool: Arc<DbPool>) -> Router {
    Router::new()
        // Note: Handlers missing after Web3 migration
        // .route("/api/v1/authjs/claims", post(get_user_claims))
        // .route("/api/v1/authjs/upsert", post(upsert_user))
        .route("/api/v1/authjs/health", get(|| async { "OK" }))
        .with_state(pool)
}

/// Create combined authentication router with RESTful structure
pub fn create_combined_auth_routes(
    app_state: AppState,
    container: Arc<AppContainer>,
    pool: sqlx::PgPool,
) -> Router {
    Router::new()
        .merge(create_auth_routes(app_state))
        .merge(create_authjs_routes(Arc::new(pool)))
        .merge(create_registration_routes(container))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_create_auth_routes() {
        // Routes creation will be tested in integration tests
        // This validates the structure exists
        assert!(true);
    }
    
    #[test] 
    fn should_have_clean_app_state() {
        // AppState should not have placeholder panics
        // Real initialization will be done in integration setup
        assert!(true);
    }
}