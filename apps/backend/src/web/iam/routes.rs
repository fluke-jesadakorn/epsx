// IAM API routes configuration

use axum::{
    routing::{get, post, put, delete},
    Router,
};

use crate::web::AppState;
use super::handlers::*;

/// Create IAM router with all endpoints
pub fn create_iam_router() -> Router<AppState> {
    Router::new()
        // Role management routes
        .route("/roles", post(create_role_handler))
        .route("/roles", get(list_roles_handler))
        .route("/roles/:role_id", get(get_role_handler))
        .route("/roles/:role_id", put(update_role_handler))
        .route("/roles/:role_id", delete(delete_role_handler))
        
        // Policy management routes
        .route("/policies", post(create_policy_handler))
        .route("/policies", get(list_policies_handler))
        .route("/policies/:policy_id", get(get_policy_handler))
        .route("/policies/:policy_id", delete(delete_policy_handler))
        
        // Permission evaluation routes
        .route("/evaluate", post(evaluate_permission_handler))
        
        // User permission override routes
        .route("/users/:user_id/overrides", post(set_user_overrides_handler))
        .route("/users/:user_id/overrides", get(get_user_overrides_handler))
        
        // User-role assignment routes
        .route("/users/:user_id/roles/:role_id", post(assign_role_to_user_handler))
        .route("/users/:user_id/roles/:role_id", delete(remove_role_from_user_handler))
        .route("/users/:user_id/roles", get(get_user_roles_handler))
}