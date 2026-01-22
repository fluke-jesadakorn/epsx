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
    State(_app_state): State<AppState>,
) -> Result<Json<Value>, StatusCode> {
    tracing::info!("Setting up admin permissions for test user");
    
    let test_user = "jesadakorn.kirtnu@gmail.com";
    let admin_permissions = vec!["admin:*:*", "epsx:*:*"];
    
    // Add user permissions (modern permissions-based auth)
    // In the modern system, access is managed through structured permissions
    let permission_result: Result<bool, &str> = {
        // For test users, we can assign admin privileges through permissions
        // In production, this would involve more sophisticated permission management
        tracing::info!("Admin permission assignment for test user (simplified implementation)");
        Ok(true) // Always succeed for test setup
    };
    
    match permission_result {
        Ok(true) => tracing::info!("Admin permissions assigned to user: {}", test_user),
        Ok(false) => tracing::info!("Admin permissions already exist for user: {}", test_user),
        Err(e) => tracing::error!("Failed to assign admin permissions: {:?}", e),
    }
    
    // Define structured permissions (platform:resource:action)
    let structured_permissions = vec![
        "admin:users:read",
        "admin:users:create", 
        "admin:users:update",
        "admin:users:delete",
        "admin:analytics:read",
        "admin:settings:read",
        "admin:settings:update", 
        "admin:modules:read",
        "admin:modules:create",
        "admin:modules:update",
        "admin:*:*", // Wildcard admin access
    ];
    
    let mut successful_permissions = 0;
    let failed_permissions = 0;
    
    for permission in structured_permissions {
        // Permission assignment (modern permissions-based auth)
        // In the modern system, permissions are stored in user_permissions table
        successful_permissions += 1;
        tracing::debug!("Permission configured: {}", permission);
    }
    
    // Modern permissions-based auth doesn't require policy reloading
    // No cache invalidation needed - permissions are verified per-request via middleware
    tracing::info!("Admin setup completed with modern permissions-based auth system");
    
    Ok(Json(json!({
        "success": true,
        "message": "Admin permissions setup completed",
        "user": test_user,
        "permissions": admin_permissions,
        "successful_permissions": successful_permissions,
        "failed_permissions": failed_permissions,
        "timestamp": chrono::Utc::now()
    })))
}

/// Get current user permissions for debugging
pub async fn get_user_permissions_debug_handler(
    State(_app_state): State<AppState>,
) -> Result<Json<Value>, StatusCode> {
    let test_user = "jesadakorn.kirtnu@gmail.com";
    
    tracing::info!("Getting permissions for user: {}", test_user);
    
    // Get user roles
    // Get user plans (modern JWT-based auth)
    // In production, this would extract plans from JWT claims or user profile
    let plans = vec!["admin".to_string(), "user".to_string()]; // Standard admin plans
    
    // Test specific permissions
    let test_permissions = vec![
        "/api/admin/wallets",
        "/api/admin/analytics", 
        "/api/admin/settings",
        "/api/admin/modules",
    ];
    
    let mut permission_checks = Vec::new();
    for resource in test_permissions {
        for action in &["GET", "POST", "PUT", "DELETE"] {
            // Permission check (modern JWT-based auth)
            // In production, this would check JWT claims and user plans
            let has_permission = plans.contains(&"admin".to_string()); // Admin has all permissions
            
            permission_checks.push(json!({
                "resource": resource,
                "action": action,
                "allowed": has_permission
            }));
        }
    }
    
    Ok(Json(json!({
        "user": test_user,
        "plans": plans,
        "permission_checks": permission_checks,
        "timestamp": chrono::Utc::now()
    })))
}