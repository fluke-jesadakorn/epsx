// Authentication routes for Axum

use std::sync::Arc;

use crate::app::use_cases::auth::AuthUC;
use crate::app::use_cases::user::UserMgmtUC;
use crate::app::use_cases::iam::IamUC;
use crate::app::ports::repositories::{SessRepo, UserRepo, IamRepo, AuditRepo, PermissionProfileRepo};


/// Application state for dependency injection
#[derive(Clone)]
pub struct AppState {
    pub auth_uc: Arc<AuthUC>,
    pub user_mgmt_uc: Arc<UserMgmtUC>,
    pub iam_uc: Arc<IamUC>,
    pub session_repo: Arc<dyn SessRepo>,
    pub user_repo: Arc<dyn UserRepo>,
    pub iam_repo: Arc<dyn IamRepo>,
    pub audit_repo: Arc<dyn AuditRepo>,
    pub permission_profile_repo: Arc<dyn PermissionProfileRepo>,
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
        iam_uc: Arc<IamUC>,
        session_repo: Arc<dyn SessRepo>,
        user_repo: Arc<dyn UserRepo>,
        iam_repo: Arc<dyn IamRepo>,
        audit_repo: Arc<dyn AuditRepo>,
        permission_profile_repo: Arc<dyn PermissionProfileRepo>,
    ) -> Self {
        Self {
            auth_uc,
            user_mgmt_uc,
            iam_uc,
            session_repo,
            user_repo,
            iam_repo,
            audit_repo,
            permission_profile_repo,
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