// Admin setup handlers for initial system configuration

use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
};
use serde_json::{json, Value};
use crate::web::auth::AppState;

/// Setup admin permissions for the test user
/// This handler is for development/testing purposes
pub async fn setup_admin_permissions_handler(
    State(app_state): State<AppState>,
) -> Result<Json<Value>, StatusCode> {
    tracing::info!("Setting up admin permissions for test user");
    
    let test_user = "jesadakorn.kirtnu@gmail.com";
    let admin_role = "admin";
    
    // Add user role assignment
    let role_result = app_state
        .casbin_service
        .add_role_for_user(test_user, admin_role)
        .await;
    
    match role_result {
        Ok(true) => tracing::info!("Admin role assigned to user: {}", test_user),
        Ok(false) => tracing::info!("Admin role already exists for user: {}", test_user),
        Err(e) => tracing::error!("Failed to assign admin role: {:?}", e),
    }
    
    // Define admin permissions
    let admin_permissions = vec![
        (admin_role, "/api/v1/admin/users", "GET"),
        (admin_role, "/api/v1/admin/users", "POST"),
        (admin_role, "/api/v1/admin/users", "PUT"),
        (admin_role, "/api/v1/admin/users", "DELETE"),
        (admin_role, "/api/v1/admin/analytics", "GET"),
        (admin_role, "/api/v1/admin/settings", "GET"),
        (admin_role, "/api/v1/admin/settings", "PUT"),
        (admin_role, "/api/v1/admin/modules", "GET"),
        (admin_role, "/api/v1/admin/modules", "POST"),
        (admin_role, "/api/v1/admin/modules", "PUT"),
        (admin_role, "/api/v1/admin", "*"), // Wildcard admin access
    ];
    
    let mut successful_permissions = 0;
    let mut failed_permissions = 0;
    
    for (subject, object, action) in admin_permissions {
        match app_state
            .casbin_service
            .add_policy(subject, object, action)
            .await
        {
            Ok(true) => {
                successful_permissions += 1;
                tracing::info!("Permission added: {} -> {} {}", subject, object, action);
            },
            Ok(false) => {
                // Permission already exists
                successful_permissions += 1;
                tracing::info!("Permission already exists: {} -> {} {}", subject, object, action);
            },
            Err(e) => {
                failed_permissions += 1;
                tracing::error!("Failed to add permission {} -> {} {}: {:?}", subject, object, action, e);
            }
        }
    }
    
    // Reload policies to ensure they're active
    if let Err(e) = app_state.casbin_service.reload_policies().await {
        tracing::error!("Failed to reload Casbin policies: {:?}", e);
    }
    
    Ok(Json(json!({
        "success": true,
        "message": "Admin permissions setup completed",
        "user": test_user,
        "role": admin_role,
        "successful_permissions": successful_permissions,
        "failed_permissions": failed_permissions,
        "timestamp": chrono::Utc::now()
    })))
}

/// Get current user permissions for debugging
pub async fn get_user_permissions_debug_handler(
    State(app_state): State<AppState>,
) -> Result<Json<Value>, StatusCode> {
    let test_user = "jesadakorn.kirtnu@gmail.com";
    
    tracing::info!("Getting permissions for user: {}", test_user);
    
    // Get user roles
    let roles_result = app_state
        .casbin_service
        .get_roles_for_user(test_user)
        .await;
    
    let roles = match roles_result {
        Ok(roles) => roles,
        Err(e) => {
            tracing::error!("Failed to get roles for user {}: {:?}", test_user, e);
            vec![]
        }
    };
    
    // Test specific permissions
    let test_permissions = vec![
        "/api/v1/admin/users",
        "/api/v1/admin/analytics", 
        "/api/v1/admin/settings",
        "/api/v1/admin/modules",
    ];
    
    let mut permission_checks = Vec::new();
    for resource in test_permissions {
        for action in &["GET", "POST", "PUT", "DELETE"] {
            let has_permission = app_state
                .casbin_service
                .enforce(test_user, resource, action)
                .await
                .unwrap_or(false);
            
            permission_checks.push(json!({
                "resource": resource,
                "action": action,
                "allowed": has_permission
            }));
        }
    }
    
    Ok(Json(json!({
        "user": test_user,
        "roles": roles,
        "permission_checks": permission_checks,
        "timestamp": chrono::Utc::now()
    })))
}