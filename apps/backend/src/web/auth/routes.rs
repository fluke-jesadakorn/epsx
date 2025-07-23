// Authentication routes for Axum

use axum::{
    routing::{get, post},
    Router,
};
use std::sync::Arc;

use crate::app::use_cases::auth::AuthUC;
use crate::app::use_cases::user::UserMgmtUC;
use crate::app::use_cases::iam::IamUC;
use crate::app::ports::repositories::{SessRepo, UserRepo, IamRepo, AuditRepo, TemplateRepo};
use super::handlers::{login_handler, logout_handler, refresh_handler, me_handler};
use super::enhanced_handlers::{enhanced_login_handler, register_handler, password_reset_handler};

/// Create authentication routes
pub fn auth_routes() -> Router<AppState> {
    Router::new()
        // Public routes (no authentication required)
        .route("/login", post(login_handler))
        .route("/enhanced-login", post(enhanced_login_handler))
        .route("/register", post(register_handler))
        .route("/password-reset", post(password_reset_handler))
        // Protected routes (authentication required)
        .route("/logout", post(logout_handler))
        .route("/refresh", post(refresh_handler))
        .route("/me", get(me_handler))
}

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
    pub template_repo: Arc<dyn TemplateRepo>,
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
        template_repo: Arc<dyn TemplateRepo>,
    ) -> Self {
        Self {
            auth_uc,
            user_mgmt_uc,
            iam_uc,
            session_repo,
            user_repo,
            iam_repo,
            audit_repo,
            template_repo,
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