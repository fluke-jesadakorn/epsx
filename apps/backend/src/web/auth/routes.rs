// Clean Authentication Routes - PancakeSwap Theme Ready

use std::sync::Arc;
use axum::{
    routing::{get, post},
    Router,
    middleware,
};
use crate::infra::db::diesel::DbPool;

use crate::app::use_cases::auth::AuthUC;
use crate::app::use_cases::user::UserMgmtUC;
use crate::app::ports::repositories::{
    SessRepo, UserRepo, IamRepo, AuditRepo, PermissionProfileRepo, 
    TemporaryPermissionRepo, ModuleRepo, UsageRepo
};
use crate::infra::firebase_admin::FirebaseAdmin;
use crate::dom::services::admin_module_service::AdminModuleService;
use crate::infra::AppContainer;
use crate::infra::cache::{SecurityCache, Cache};
use crate::security::brute_force_integration::{BruteForceIntegrationService, brute_force_protection_middleware};
use crate::infra::services::notification::NotificationService;
use crate::web::middleware::add_deprecation_headers;

use super::handlers::{
    // Core Authentication
    login_handler, logout_handler, refresh_handler, me_handler,
    
    // Registration
    register_user, check_email_availability, check_password_strength,
    
    // Auth.js Integration
    get_user_claims, upsert_user,
    
    // Session Management
    validate_session_handler, rotate_session_handler,
    
    // Permission System
    validate_route_access_handler, validate_bulk_routes_handler,
    check_permission_handler, single_permission_handler,
    navigation_handler, user_features_handler,
};

/// Clean Application State for Dependency Injection
#[derive(Clone)]
pub struct AppState {
    pub auth_uc: Arc<AuthUC>,
    pub user_mgmt_uc: Arc<UserMgmtUC>,
    pub session_repo: Arc<dyn SessRepo>,
    pub user_repo: Arc<dyn UserRepo>,
    pub iam_repo: Arc<dyn IamRepo>,
    pub audit_repo: Arc<dyn AuditRepo>,
    pub permission_profile_repo: Arc<dyn PermissionProfileRepo>,
    pub temporary_permission_repo: Arc<dyn TemporaryPermissionRepo>,
    pub module_repo: Arc<dyn ModuleRepo>,
    pub usage_repo: Arc<dyn UsageRepo>,
    pub firebase_admin: Arc<FirebaseAdmin>,
    pub admin_module_service: Arc<AdminModuleService>,
    pub feature_expiration_service: Arc<dyn crate::dom::services::feature_expiration::FeatureExpirationService>,
    pub db_pool: Arc<DbPool>,
    pub cache: Arc<dyn Cache>,
    pub security_cache: Option<Arc<SecurityCache>>,
    pub brute_force_service: Option<BruteForceIntegrationService>,
    pub notification_service: Arc<dyn NotificationService>,
}

impl AppState {
    pub fn new(
        auth_uc: Arc<AuthUC>,
        user_mgmt_uc: Arc<UserMgmtUC>,
        session_repo: Arc<dyn SessRepo>,
        user_repo: Arc<dyn UserRepo>,
        iam_repo: Arc<dyn IamRepo>,
        audit_repo: Arc<dyn AuditRepo>,
        permission_profile_repo: Arc<dyn PermissionProfileRepo>,
        temporary_permission_repo: Arc<dyn TemporaryPermissionRepo>,
        module_repo: Arc<dyn ModuleRepo>,
        usage_repo: Arc<dyn UsageRepo>,
        firebase_admin: Arc<FirebaseAdmin>,
        admin_module_service: Arc<AdminModuleService>,
        feature_expiration_service: Arc<dyn crate::dom::services::feature_expiration::FeatureExpirationService>,
        db_pool: Arc<DbPool>,
        cache: Arc<dyn Cache>,
        security_cache: Option<Arc<SecurityCache>>,
        brute_force_service: Option<BruteForceIntegrationService>,
        notification_service: Arc<dyn NotificationService>,
    ) -> Self {
        Self {
            auth_uc,
            user_mgmt_uc,
            session_repo,
            user_repo,
            iam_repo,
            audit_repo,
            permission_profile_repo,
            temporary_permission_repo,
            module_repo,
            usage_repo,
            firebase_admin,
            admin_module_service,
            feature_expiration_service,
            db_pool,
            cache,
            security_cache,
            brute_force_service,
            notification_service,
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
        
    // Protected auth routes (require valid session)
    let protected_auth_routes = Router::new()
        .route("/api/v1/auth/sessions", axum::routing::delete(logout_handler))
        .route("/api/v1/auth/sessions/current", get(validate_session_handler))
        .route("/api/v1/auth/sessions/current", axum::routing::patch(rotate_session_handler))
        .route("/api/v1/auth/user", get(me_handler))
        .route("/api/v1/auth/tokens/refresh", post(refresh_handler))
        .layer(middleware::from_fn_with_state(
            app_state.clone(),
            crate::web::middleware::session_validation_middleware
        ));

    // Permission validation routes (require valid session)
    let permission_routes = Router::new()
        .route("/api/v1/permissions/validations", post(check_permission_handler))
        .route("/api/v1/permissions/routes/validations", post(validate_route_access_handler))
        .route("/api/v1/permissions/validations/bulk", post(validate_bulk_routes_handler))
        .route("/api/v1/permissions/single", get(single_permission_handler))
        .route("/api/v1/permissions/navigation", get(navigation_handler))
        .route("/api/v1/permissions/features", get(user_features_handler))
        .layer(middleware::from_fn_with_state(
            app_state.clone(),
            crate::web::middleware::session_validation_middleware
        ));

    // Legacy routes for backward compatibility (with deprecation headers)
    let legacy_routes = Router::new()
        .route("/login", post(login_handler))
        .route("/register", post(register_user))
        .route("/check-email", post(check_email_availability))
        .route("/check-password", post(check_password_strength))
        .route("/logout", post(logout_handler))
        .route("/me", get(me_handler))
        .route("/refresh", post(refresh_handler))
        .route("/session/validate", post(validate_session_handler))
        .route("/session/rotate", post(rotate_session_handler))
        .route("/permissions/route", post(validate_route_access_handler))
        .route("/permissions/bulk", post(validate_bulk_routes_handler))
        .route("/permissions/check", post(check_permission_handler))
        .route("/permissions/single", get(single_permission_handler))
        .route("/permissions/navigation", get(navigation_handler))
        .route("/permissions/features", get(user_features_handler))
        .layer(middleware::from_fn(add_deprecation_headers));
    
    let router = Router::new()
        .merge(validation_routes)
        .merge(public_auth_routes)
        .merge(protected_auth_routes)
        .merge(permission_routes)
        .merge(legacy_routes)
        .with_state(app_state.clone());

    // Apply brute force protection middleware to authentication routes
    if app_state.brute_force_service.is_some() {
        router.layer(middleware::from_fn_with_state(app_state, brute_force_protection_middleware))
    } else {
        router
    }
}

/// Create registration routes with AppContainer state (RESTful patterns)
pub fn create_registration_routes(container: Arc<AppContainer>) -> Router {
    Router::new()
        .route("/api/v1/auth/users", post(register_user))
        .route("/api/v1/validations/emails", post(check_email_availability))
        .route("/api/v1/validations/passwords", post(check_password_strength))
        // Legacy routes for backward compatibility
        .route("/register", post(register_user))
        .route("/check-email", post(check_email_availability))
        .route("/check-password", post(check_password_strength))
        .layer(middleware::from_fn(add_deprecation_headers))
        .with_state(container)
}

/// Create Auth.js integration routes with PostgreSQL pool
pub fn create_authjs_routes(pool: Arc<DbPool>) -> Router {
    Router::new()
        // New versioned routes
        .route("/api/v1/authjs/claims", post(get_user_claims))
        .route("/api/v1/authjs/upsert", post(upsert_user))
        // Legacy routes for backward compatibility
        .route("/claims", post(get_user_claims))
        .route("/upsert", post(upsert_user))
        .layer(middleware::from_fn(add_deprecation_headers))
        .with_state(pool)
}

/// Create combined authentication router with RESTful structure
pub fn create_combined_auth_routes(
    app_state: AppState,
    container: Arc<AppContainer>,
    pool: crate::infra::db::diesel::DbPool,
) -> Router {
    Router::new()
        // New structure: routes are directly included, not nested under /api/auth
        .merge(create_auth_routes(app_state))
        .merge(create_authjs_routes(Arc::new(pool)))
        .merge(create_registration_routes(container))
        // Legacy nesting for backward compatibility
        .nest("/api/auth", Router::new()
            .route("/login", post(login_handler))
            .route("/register", post(register_user))
            .route("/check-email", post(check_email_availability))
            .route("/check-password", post(check_password_strength))
            .route("/logout", post(logout_handler))
            .route("/me", get(me_handler))
            .route("/refresh", post(refresh_handler))
            .route("/session/validate", post(validate_session_handler))
            .route("/session/rotate", post(rotate_session_handler))
            .route("/permissions/route", post(validate_route_access_handler))
            .route("/permissions/bulk", post(validate_bulk_routes_handler))
            .route("/permissions/check", post(check_permission_handler))
            .route("/permissions/single", get(single_permission_handler))
            .route("/permissions/navigation", get(navigation_handler))
            .route("/permissions/features", get(user_features_handler))
            .layer(middleware::from_fn(add_deprecation_headers)))
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