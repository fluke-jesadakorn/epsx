// Authentication routes for Axum

use std::sync::Arc;

use crate::app::use_cases::auth::AuthUC;
use crate::app::use_cases::user::UserMgmtUC;
// IAM replaced with permission-based system
use crate::app::ports::repositories::{SessRepo, UserRepo, IamRepo, AuditRepo, PermissionProfileRepo, TemporaryPermissionRepo, ModuleRepo, UsageRepo};
use crate::infra::firebase_admin::FirebaseAdmin;
use crate::dom::services::admin_module_service::AdminModuleService;
// casbin_service removed - using modern JWT auth


/// Application state for dependency injection
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
    // casbin_service removed - using modern JWT auth
    pub admin_module_service: Arc<AdminModuleService>,
    pub feature_expiration_service: Arc<dyn crate::dom::services::feature_expiration::FeatureExpirationService>,
}

impl Default for AppState {
    fn default() -> Self {
        // This is a placeholder - real implementation will inject dependencies
        panic!("AppState must be properly initialized with dependencies")
    }
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
        // casbin_service removed - using modern JWT auth
        admin_module_service: Arc<AdminModuleService>,
        feature_expiration_service: Arc<dyn crate::dom::services::feature_expiration::FeatureExpirationService>,
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
            // casbin_service removed - using modern JWT auth
            admin_module_service,
            feature_expiration_service,
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn should_create_auth_routes() {
        // This test would require proper dependency injection setup
        // For now, just ensure the function exists
    }
}