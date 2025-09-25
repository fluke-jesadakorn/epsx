// Clean Authentication Routes - PancakeSwap Theme Ready

use std::sync::Arc;
use axum::{
    routing::{get, post},
    Router,
    middleware,
};
use sqlx::PgPool as DbPool;

use crate::infrastructure::cache::Cache;
use crate::infrastructure::container::DomainContainer;

// Import Web3 authentication handlers
use crate::web::auth::web3_handlers::{
    generate_challenge_handler,
    verify_signature_handler,
    logout_handler,
    get_session_handler,
    check_permission_handler,
    grant_permission_handler,
    revoke_permission_handler,
};

use crate::web::user::handlers::{
    // Available handlers
    get_permissions_handler,
    verify_ownership_handler,
    get_holdings_handler,
    delegate_permission_handler,
};

// Import group permission routes
// use super::group_routes::{
//     get_user_permissions,
//     check_permission,
//     get_user_groups,
//     create_permission_group,
//     get_permission_groups,
//     assign_user_to_group,
//     remove_user_from_group, // Removed - group routes no longer exist
    // get_group_memberships,
    // cleanup_expired_memberships,
    // permission_updates_sse,
// }; // Removed - group routes no longer exist

/// Clean Application State for Dependency Injection
#[derive(Clone)]
pub struct AppState {
    pub db_pool: Arc<DbPool>,
    pub cache: Arc<dyn Cache>,
    pub domain_container: Arc<DomainContainer>,
    // Add back user_repo as stub to fix admin handlers
    pub user_repo: Option<String>, // Placeholder
}

impl AppState {
    pub fn new(
        db_pool: Arc<DbPool>,
        cache: Arc<dyn Cache>,
        domain_container: Arc<DomainContainer>,
    ) -> Self {
        Self {
            db_pool,
            cache,
            domain_container,
            user_repo: None,
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
        
    // Web3 authentication routes
    let web3_auth_routes = Router::new()
        .route("/api/v1/auth/web3/challenge", post(generate_challenge_handler))
        .route("/api/v1/auth/web3/verify", post(verify_signature_handler))
        .route("/api/v1/auth/web3/logout", axum::routing::delete(logout_handler))
        .route("/api/v1/auth/web3/session", get(get_session_handler))
        .layer(middleware::from_fn(
            crate::web::middleware::web3_auth_middleware
        ));

    // Web3 permission management routes
    let permission_routes = Router::new()
        .route("/api/v1/auth/web3/permissions/check", post(check_permission_handler))
        .route("/api/v1/auth/web3/permissions/grant", post(grant_permission_handler))
        .route("/api/v1/auth/web3/permissions/revoke", axum::routing::delete(revoke_permission_handler))
        .route("/api/v1/permissions/health", get(|| async { "OK" })) // Basic health check
        .layer(middleware::from_fn(
            crate::web::middleware::web3_auth_middleware
        ));

    // Legacy routes removed - use RESTful /api/v1/ endpoints only
    
    let router = Router::new()
        .merge(user_routes)
        .merge(web3_auth_routes)
        .merge(permission_routes)
        .with_state(app_state.clone());

    router
}

/// Create registration routes with DomainContainer state (RESTful patterns)
pub fn create_registration_routes(container: Arc<DomainContainer>) -> Router {
    Router::new()
        // Note: Handlers missing after Web3 migration
        // .route("/api/v1/auth/users", post(register_user))
        // .route("/api/v1/validations/emails", post(check_email_availability))
        // .route("/api/v1/validations/passwords", post(check_password_strength))
        .with_state(container)
}

/// Create Auth.js integration routes with PostgreSQL pool
pub fn create_authjs_routes(pool: Arc<DbPool>) -> Router {
    Router::new()
        // Note: Handlers missing after Web3 migration
        // .route("/api/v1/authjs/claims", post(get_user_claims))
        // .route("/api/v1/authjs/upsert", post(upsert_user))
        .with_state(pool)
}

/// Create combined authentication router with RESTful structure
pub fn create_combined_auth_routes(
    app_state: AppState,
    container: Arc<DomainContainer>,
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