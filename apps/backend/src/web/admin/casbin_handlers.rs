// Admin handlers for Casbin policy management and cache control

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use crate::web::auth::AppState;

#[derive(Debug, Deserialize)]
pub struct PolicyRequest {
    pub subject: String,
    pub object: String,
    pub action: String,
}

#[derive(Debug, Deserialize)]
pub struct RoleRequest {
    pub user: String,
    pub role: String,
}

#[derive(Debug, Deserialize)]
pub struct BatchPolicyRequest {
    pub policies: Vec<PolicyRequest>,
}

#[derive(Debug, Serialize)]
pub struct PolicyResponse {
    pub success: bool,
    pub message: String,
    pub affected_count: Option<usize>,
}

#[derive(Debug, Serialize)]
pub struct CacheStatsResponse {
    pub total_entries: usize,
    pub expired_entries: usize,
    pub active_entries: usize,
    pub max_entries: usize,
    pub cache_hit_ratio: f64,
    pub default_ttl_seconds: u64,
}

/// Get all policies from Casbin
pub async fn get_all_policies_handler(
    State(app_state): State<AppState>,
) -> Result<Json<Value>, StatusCode> {
    // Verify admin permissions
    verify_admin_access(&app_state, "casbin_policies", "read").await?;
    
    match app_state.casbin_service.get_all_policies().await {
        Ok((policies, role_policies)) => {
            Ok(Json(json!({
                "status": "success",
                "data": {
                    "policies": policies,
                    "role_inheritances": role_policies,
                    "total_policies": policies.len(),
                    "total_role_inheritances": role_policies.len()
                }
            })))
        }
        Err(e) => {
            tracing::error!("Failed to get all policies: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Add a single policy
pub async fn add_policy_handler(
    State(app_state): State<AppState>,
    Json(request): Json<PolicyRequest>,
) -> Result<Json<PolicyResponse>, StatusCode> {
    // Verify admin permissions
    verify_admin_access(&app_state, "casbin_policies", "create").await?;
    
    match app_state.casbin_service.add_policy(&request.subject, &request.object, &request.action).await {
        Ok(true) => {
            tracing::info!("Policy added: {} -> {} -> {}", request.subject, request.object, request.action);
            Ok(Json(PolicyResponse {
                success: true,
                message: "Policy added successfully".to_string(),
                affected_count: Some(1),
            }))
        }
        Ok(false) => {
            Ok(Json(PolicyResponse {
                success: false,
                message: "Policy already exists".to_string(),
                affected_count: Some(0),
            }))
        }
        Err(e) => {
            tracing::error!("Failed to add policy: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Remove a single policy
pub async fn remove_policy_handler(
    State(app_state): State<AppState>,
    Json(request): Json<PolicyRequest>,
) -> Result<Json<PolicyResponse>, StatusCode> {
    // Verify admin permissions
    verify_admin_access(&app_state, "casbin_policies", "delete").await?;
    
    match app_state.casbin_service.remove_policy(&request.subject, &request.object, &request.action).await {
        Ok(true) => {
            tracing::info!("Policy removed: {} -> {} -> {}", request.subject, request.object, request.action);
            Ok(Json(PolicyResponse {
                success: true,
                message: "Policy removed successfully".to_string(),
                affected_count: Some(1),
            }))
        }
        Ok(false) => {
            Ok(Json(PolicyResponse {
                success: false,
                message: "Policy not found".to_string(),
                affected_count: Some(0),
            }))
        }
        Err(e) => {
            tracing::error!("Failed to remove policy: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Add multiple policies in batch
pub async fn add_batch_policies_handler(
    State(app_state): State<AppState>,
    Json(request): Json<BatchPolicyRequest>,
) -> Result<Json<PolicyResponse>, StatusCode> {
    // Verify admin permissions
    verify_admin_access(&app_state, "casbin_policies", "create").await?;
    
    let policies: Vec<(String, String, String)> = request.policies
        .into_iter()
        .map(|p| (p.subject, p.object, p.action))
        .collect();
    
    let count = policies.len();
    
    match app_state.casbin_service.add_policies(policies).await {
        Ok(_) => {
            tracing::info!("Batch added {} policies", count);
            Ok(Json(PolicyResponse {
                success: true,
                message: format!("Successfully processed {} policies", count),
                affected_count: Some(count),
            }))
        }
        Err(e) => {
            tracing::error!("Failed to batch add policies: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Assign role to user
pub async fn assign_role_handler(
    State(app_state): State<AppState>,
    Json(request): Json<RoleRequest>,
) -> Result<Json<PolicyResponse>, StatusCode> {
    // Verify admin permissions
    verify_admin_access(&app_state, "user_roles", "create").await?;
    
    match app_state.casbin_service.add_role_for_user(&request.user, &request.role).await {
        Ok(true) => {
            tracing::info!("Role assigned: {} -> {}", request.user, request.role);
            Ok(Json(PolicyResponse {
                success: true,
                message: "Role assigned successfully".to_string(),
                affected_count: Some(1),
            }))
        }
        Ok(false) => {
            Ok(Json(PolicyResponse {
                success: false,
                message: "Role already assigned".to_string(),
                affected_count: Some(0),
            }))
        }
        Err(e) => {
            tracing::error!("Failed to assign role: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Remove role from user
pub async fn remove_role_handler(
    State(app_state): State<AppState>,
    Json(request): Json<RoleRequest>,
) -> Result<Json<PolicyResponse>, StatusCode> {
    // Verify admin permissions
    verify_admin_access(&app_state, "user_roles", "delete").await?;
    
    match app_state.casbin_service.remove_role_for_user(&request.user, &request.role).await {
        Ok(true) => {
            tracing::info!("Role removed: {} -> {}", request.user, request.role);
            Ok(Json(PolicyResponse {
                success: true,
                message: "Role removed successfully".to_string(),
                affected_count: Some(1),
            }))
        }
        Ok(false) => {
            Ok(Json(PolicyResponse {
                success: false,
                message: "Role assignment not found".to_string(),
                affected_count: Some(0),
            }))
        }
        Err(e) => {
            tracing::error!("Failed to remove role: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Get roles for a specific user
pub async fn get_user_roles_handler(
    State(app_state): State<AppState>,
    Path(user_id): Path<String>,
) -> Result<Json<Value>, StatusCode> {
    // Verify admin permissions
    verify_admin_access(&app_state, "user_roles", "read").await?;
    
    match app_state.casbin_service.get_roles_for_user(&user_id).await {
        Ok(roles) => {
            Ok(Json(json!({
                "status": "success",
                "data": {
                    "user_id": user_id,
                    "roles": roles,
                    "role_count": roles.len()
                }
            })))
        }
        Err(e) => {
            tracing::error!("Failed to get user roles: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Get permissions for a specific user
pub async fn get_user_permissions_handler(
    State(app_state): State<AppState>,
    Path(user_id): Path<String>,
) -> Result<Json<Value>, StatusCode> {
    // Verify admin permissions
    verify_admin_access(&app_state, "user_permissions", "read").await?;
    
    match app_state.casbin_service.get_permissions_for_subject(&user_id).await {
        Ok(permissions) => {
            Ok(Json(json!({
                "status": "success", 
                "data": {
                    "user_id": user_id,
                    "permissions": permissions,
                    "permission_count": permissions.len()
                }
            })))
        }
        Err(e) => {
            tracing::error!("Failed to get user permissions: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Test policy enforcement
pub async fn test_policy_handler(
    State(app_state): State<AppState>,
    Json(request): Json<PolicyRequest>,
) -> Result<Json<Value>, StatusCode> {
    // Verify admin permissions
    verify_admin_access(&app_state, "casbin_policies", "read").await?;
    
    match app_state.casbin_service.enforce(&request.subject, &request.object, &request.action).await {
        Ok(result) => {
            Ok(Json(json!({
                "status": "success",
                "data": {
                    "subject": request.subject,
                    "object": request.object,
                    "action": request.action,
                    "allowed": result,
                    "enforcement_result": if result { "ALLOW" } else { "DENY" }
                }
            })))
        }
        Err(e) => {
            tracing::error!("Failed to test policy: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Reload policies from database
pub async fn reload_policies_handler(
    State(app_state): State<AppState>,
) -> Result<Json<PolicyResponse>, StatusCode> {
    // Verify admin permissions
    verify_admin_access(&app_state, "casbin_system", "update").await?;
    
    match app_state.casbin_service.reload_policies().await {
        Ok(_) => {
            tracing::info!("Casbin policies reloaded by admin");
            Ok(Json(PolicyResponse {
                success: true,
                message: "Policies reloaded successfully".to_string(),
                affected_count: None,
            }))
        }
        Err(e) => {
            tracing::error!("Failed to reload policies: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Get cache statistics
pub async fn get_cache_stats_handler(
    State(app_state): State<AppState>,
) -> Result<Json<CacheStatsResponse>, StatusCode> {
    // Verify admin permissions
    verify_admin_access(&app_state, "casbin_cache", "read").await?;
    
    let stats = app_state.casbin_service.cache_stats().await;
    
    // Calculate hit ratio (simplified estimation)
    let hit_ratio = if stats.active_entries > 0 {
        (stats.active_entries as f64 / stats.total_entries as f64) * 100.0
    } else {
        0.0
    };
    
    Ok(Json(CacheStatsResponse {
        total_entries: stats.total_entries,
        expired_entries: stats.expired_entries,
        active_entries: stats.active_entries,
        max_entries: stats.max_entries,
        cache_hit_ratio: hit_ratio,
        default_ttl_seconds: stats.default_ttl.as_secs(),
    }))
}

/// Clear policy cache
pub async fn clear_cache_handler(
    State(app_state): State<AppState>,
) -> Result<Json<PolicyResponse>, StatusCode> {
    // Verify admin permissions
    verify_admin_access(&app_state, "casbin_cache", "delete").await?;
    
    app_state.casbin_service.clear_cache().await;
    
    tracing::info!("Casbin policy cache cleared by admin");
    Ok(Json(PolicyResponse {
        success: true,
        message: "Policy cache cleared successfully".to_string(),
        affected_count: None,
    }))
}

/// Helper function to verify admin access with explicit user_id
async fn verify_admin_access_with_user(
    app_state: &AppState,
    user_id: &str,
    resource: &str,
    action: &str,
) -> Result<(), StatusCode> {
    // TODO: Implement proper admin permission checks
    // For development, allow access to admin_user
    if user_id == "admin_user" {
        tracing::debug!("Admin access granted for {} on {}/{} (dev mode)", user_id, resource, action);
        return Ok(());
    }
    
    match app_state.casbin_service.enforce(user_id, resource, action).await {
        Ok(true) => {
            tracing::debug!("Admin access granted for {} on {}/{}", user_id, resource, action);
            Ok(())
        }
        Ok(false) => {
            tracing::warn!("Admin access denied for {} on {}/{}", user_id, resource, action);
            Err(StatusCode::FORBIDDEN)
        }
        Err(e) => {
            tracing::error!("Failed to check admin permissions: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Legacy wrapper function for backward compatibility
/// Extracts user ID from context and calls the user-specific verification
async fn verify_admin_access(
    app_state: &AppState,
    resource: &str,
    action: &str,
) -> Result<(), StatusCode> {
    // Extract user ID from context - this should be properly implemented
    // to get the actual user ID from the authenticated request
    let user_id = extract_admin_user_from_context().await?;
    verify_admin_access_with_user(app_state, &user_id, resource, action).await
}

/// Extract admin user ID from request context
/// TODO: Implement proper extraction from authenticated session
async fn extract_admin_user_from_context() -> Result<String, StatusCode> {
    // This is a placeholder implementation
    // In a real system, this would:
    // 1. Extract the session token from the request
    // 2. Validate the session using SessionRepo
    // 3. Verify the user has admin privileges
    // 4. Return the actual user ID
    
    // For now, return a test admin user
    Ok("admin_user".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_policy_request_deserialize() {
        let json = r#"{"subject": "user1", "object": "resource1", "action": "read"}"#;
        let request: PolicyRequest = serde_json::from_str(json).unwrap();
        
        assert_eq!(request.subject, "user1");
        assert_eq!(request.object, "resource1");
        assert_eq!(request.action, "read");
    }
    
    #[test]
    fn test_batch_policy_request_deserialize() {
        let json = r#"{
            "policies": [
                {"subject": "user1", "object": "resource1", "action": "read"},
                {"subject": "user2", "object": "resource2", "action": "write"}
            ]
        }"#;
        let request: BatchPolicyRequest = serde_json::from_str(json).unwrap();
        
        assert_eq!(request.policies.len(), 2);
        assert_eq!(request.policies[0].subject, "user1");
        assert_eq!(request.policies[1].action, "write");
    }
}