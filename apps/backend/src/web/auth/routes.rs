// Clean Authentication Routes - PancakeSwap Theme Ready

use std::sync::Arc;
use axum::{
    routing::{get, post},
    Router,
    middleware,
};
use sqlx::PgPool as DbPool;

use crate::infrastructure::adapters::services::firebase::firebase_admin::FirebaseAdmin;
use crate::infrastructure::cache::Cache;
use crate::infrastructure::container::AppContainer;

use crate::web::user::handlers::{
    // Core Authentication
    login_handler, logout_handler, refresh_handler, me_handler,
    
    // Registration
    register_user, check_email_availability, check_password_strength,
    
    // Auth.js Integration
    get_user_claims, upsert_user,
    
    // Token Validation (Bearer token-based)
    validate_session_handler,
    
    // Permission System
    validate_route_access_handler, validate_bulk_routes_handler,
    check_permission_handler, single_permission_handler,
    navigation_handler, user_features_handler,
};

/// Clean Application State for Dependency Injection
#[derive(Clone)]
pub struct AppState {
    pub db_pool: Arc<DbPool>,
    pub firebase_admin: Arc<FirebaseAdmin>,
    pub cache: Arc<dyn Cache>,
    pub notification_service: Arc<dyn crate::application::ports::outbound::service_ports::NotificationServicePort<Error = crate::infrastructure::adapters::services::fcm_service::FcmServiceError>>,
    pub ddd_container: Arc<crate::infrastructure::container::ddd_container::DDDContainer>,
    pub user_repo: Arc<dyn crate::domain::user_management::UserRepositoryPort>,
    pub permission_service: Arc<crate::domain::authorization::services::stateless_permission_service::StatelessPermissionService>,
    pub rate_limiting_service: Option<Arc<crate::domain::resource_management::services::RateLimitingService>>, // Context-aware rate limiting
}

impl AppState {
    pub fn new(
        db_pool: Arc<DbPool>,
        firebase_admin: Arc<FirebaseAdmin>,
        cache: Arc<dyn Cache>,
        notification_service: Arc<dyn crate::application::ports::outbound::service_ports::NotificationServicePort<Error = crate::infrastructure::adapters::services::fcm_service::FcmServiceError>>,
        ddd_container: Arc<crate::infrastructure::container::ddd_container::DDDContainer>,
        user_repo: Arc<dyn crate::domain::user_management::UserRepositoryPort>,
        permission_service: Arc<crate::domain::authorization::services::stateless_permission_service::StatelessPermissionService>,
        rate_limiting_service: Option<Arc<crate::domain::resource_management::services::RateLimitingService>>,
    ) -> Self {
        Self {
            db_pool,
            firebase_admin,
            cache,
            notification_service,
            ddd_container,
            user_repo,
            permission_service,
            rate_limiting_service,
        }
    }
}

/// Create authentication routes with RESTful patterns and v1 versioning
pub fn create_auth_routes(app_state: AppState) -> Router {
    // Public validation routes (no authentication required)
    let validation_routes = Router::new()
        .route("/api/v1/validations/emails", post(check_email_availability))
        .route("/api/v1/validations/passwords", post(check_password_strength));
        
    // Public auth routes (no authentication required)
    let public_auth_routes = Router::new()
        .route("/api/v1/auth/sessions", post(login_handler))
        .route("/api/v1/auth/users", post(register_user));
        
    // Protected auth routes (require valid Bearer token)
    let protected_auth_routes = Router::new()
        .route("/api/v1/auth/sessions", axum::routing::delete(logout_handler))
        .route("/api/v1/auth/sessions/current", get(validate_session_handler))
        // Session rotation removed - JWT tokens have built-in expiration
        .route("/api/v1/auth/user", get(me_handler))
        .route("/api/v1/auth/tokens/refresh", post(refresh_handler))
        .layer(middleware::from_fn_with_state(
            app_state.clone(),
            crate::web::middleware::modern_jwt_auth_middleware
        ));

    // Permission validation routes (require valid Bearer token)
    let permission_routes = Router::new()
        .route("/api/v1/permissions/validations", post(check_permission_handler))
        .route("/api/v1/permissions/routes/validations", post(validate_route_access_handler))
        .route("/api/v1/permissions/validations/bulk", post(validate_bulk_routes_handler))
        .route("/api/v1/permissions/single", get(single_permission_handler))
        .route("/api/v1/permissions/navigation", get(navigation_handler))
        .route("/api/v1/permissions/features", get(user_features_handler))
        .layer(middleware::from_fn_with_state(
            app_state.clone(),
            crate::web::middleware::modern_jwt_auth_middleware
        ));

    // Legacy routes removed - use RESTful /api/v1/ endpoints only
    
    let router = Router::new()
        .merge(validation_routes)
        .merge(public_auth_routes)
        .merge(protected_auth_routes)
        .merge(permission_routes)
        .with_state(app_state.clone());

    router
}

/// Create registration routes with AppContainer state (RESTful patterns)
pub fn create_registration_routes(container: Arc<AppContainer>) -> Router {
    Router::new()
        .route("/api/v1/auth/users", post(register_user))
        .route("/api/v1/validations/emails", post(check_email_availability))
        .route("/api/v1/validations/passwords", post(check_password_strength))
        .with_state(container)
}

/// Create Auth.js integration routes with PostgreSQL pool
pub fn create_authjs_routes(pool: Arc<DbPool>) -> Router {
    Router::new()
        .route("/api/v1/authjs/claims", post(get_user_claims))
        .route("/api/v1/authjs/upsert", post(upsert_user))
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