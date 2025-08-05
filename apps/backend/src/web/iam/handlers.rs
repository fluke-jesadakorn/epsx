// IAM API handlers with Casbin integration

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use crate::web::auth::AppState;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Serialize)]
pub struct PolicyResponse {
    subject: String,
    object: String,
    action: String,
}

#[derive(Deserialize)]
pub struct AddPolicyRequest {
    subject: String,
    object: String,
    action: String,
}

// IAM handlers with Casbin integration

pub async fn assign_role_handler(
    State(app_state): State<AppState>,
    Path((user_id, role)): Path<(String, String)>,
) -> Result<StatusCode, StatusCode> {
    match app_state.casbin_service.add_role_for_user(&user_id, &role).await {
        Ok(true) => {
            tracing::info!("Successfully assigned role {} to user {}", role, user_id);
            Ok(StatusCode::OK)
        }
        Ok(false) => {
            tracing::warn!("Role {} already assigned to user {}", role, user_id);
            Ok(StatusCode::OK)
        }
        Err(e) => {
            tracing::error!("Failed to assign role {} to user {}: {}", role, user_id, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn revoke_role_handler(
    State(app_state): State<AppState>,
    Path((user_id, role)): Path<(String, String)>,
) -> Result<StatusCode, StatusCode> {
    match app_state.casbin_service.remove_role_for_user(&user_id, &role).await {
        Ok(true) => {
            tracing::info!("Successfully revoked role {} from user {}", role, user_id);
            Ok(StatusCode::OK)
        }
        Ok(false) => {
            tracing::warn!("Role {} was not assigned to user {}", role, user_id);
            Ok(StatusCode::NOT_FOUND)
        }
        Err(e) => {
            tracing::error!("Failed to revoke role {} from user {}: {}", role, user_id, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn add_policy_handler(
    State(app_state): State<AppState>,
    Json(request): Json<AddPolicyRequest>,
) -> Result<StatusCode, StatusCode> {
    match app_state.casbin_service.add_policy(&request.subject, &request.object, &request.action).await {
        Ok(true) => {
            tracing::info!("Successfully added policy: {} -> {} -> {}", request.subject, request.object, request.action);
            Ok(StatusCode::CREATED)
        }
        Ok(false) => {
            tracing::warn!("Policy already exists: {} -> {} -> {}", request.subject, request.object, request.action);
            Ok(StatusCode::OK)
        }
        Err(e) => {
            tracing::error!("Failed to add policy: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn remove_policy_handler(
    State(app_state): State<AppState>,
    Json(request): Json<AddPolicyRequest>,
) -> Result<StatusCode, StatusCode> {
    match app_state.casbin_service.remove_policy(&request.subject, &request.object, &request.action).await {
        Ok(true) => {
            tracing::info!("Successfully removed policy: {} -> {} -> {}", request.subject, request.object, request.action);
            Ok(StatusCode::OK)
        }
        Ok(false) => {
            tracing::warn!("Policy not found: {} -> {} -> {}", request.subject, request.object, request.action);
            Ok(StatusCode::NOT_FOUND)
        }
        Err(e) => {
            tracing::error!("Failed to remove policy: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn check_permission_handler(
    State(app_state): State<AppState>,
    Path((user_id, resource, action)): Path<(String, String, String)>,
) -> Result<Json<bool>, StatusCode> {
    match app_state.casbin_service.enforce(&user_id, &resource, &action).await {
        Ok(allowed) => {
            tracing::info!("Permission check for user {} on {} with action {}: {}", user_id, resource, action, allowed);
            Ok(Json(allowed))
        }
        Err(e) => {
            tracing::error!("Failed to check permission for user {}: {}", user_id, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn list_policies_handler(
    State(app_state): State<AppState>,
) -> Result<Json<Vec<PolicyResponse>>, StatusCode> {
    match app_state.casbin_service.get_all_policies().await {
        Ok((policies, _role_policies)) => {
            let policy_responses: Vec<PolicyResponse> = policies
                .into_iter()
                .filter_map(|policy| {
                    if policy.len() >= 3 {
                        Some(PolicyResponse {
                            subject: policy[0].clone(),
                            object: policy[1].clone(),
                            action: policy[2].clone(),
                        })
                    } else {
                        None
                    }
                })
                .collect();
            tracing::info!("Retrieved {} policies", policy_responses.len());
            Ok(Json(policy_responses))
        }
        Err(e) => {
            tracing::error!("Failed to list policies: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// Additional handlers for IAM routes

pub async fn create_role_handler(
    State(_app_state): State<AppState>,
    Json(_request): Json<Value>,
) -> Result<Json<Value>, StatusCode> {
    tracing::info!("IAM create role handler called during migration");
    Ok(Json(json!({"message": "Create role - implementation pending during migration"})))
}

pub async fn list_roles_handler(
    State(_app_state): State<AppState>,
) -> Result<Json<Value>, StatusCode> {
    tracing::info!("IAM list roles handler called during migration");
    Ok(Json(json!({"roles": [], "message": "List roles - implementation pending during migration"})))
}

pub async fn get_role_handler(
    State(_app_state): State<AppState>,
    Path(_role_id): Path<String>,
) -> Result<Json<Value>, StatusCode> {
    tracing::info!("IAM get role handler called during migration");
    Ok(Json(json!({"message": "Get role - implementation pending during migration"})))
}

pub async fn update_role_handler(
    State(_app_state): State<AppState>,
    Path(_role_id): Path<String>,
    Json(_request): Json<Value>,
) -> Result<Json<Value>, StatusCode> {
    tracing::info!("IAM update role handler called during migration");
    Ok(Json(json!({"message": "Update role - implementation pending during migration"})))
}

pub async fn delete_role_handler(
    State(_app_state): State<AppState>,
    Path(_role_id): Path<String>,
) -> Result<StatusCode, StatusCode> {
    tracing::info!("IAM delete role handler called during migration");
    Ok(StatusCode::OK)
}

pub async fn create_policy_handler(
    State(_app_state): State<AppState>,
    Json(_request): Json<Value>,
) -> Result<Json<Value>, StatusCode> {
    tracing::info!("IAM create policy handler called during migration");
    Ok(Json(json!({"message": "Create policy - implementation pending during migration"})))
}

pub async fn get_policy_handler(
    State(_app_state): State<AppState>,
    Path(_policy_id): Path<String>,
) -> Result<Json<Value>, StatusCode> {
    tracing::info!("IAM get policy handler called during migration");
    Ok(Json(json!({"message": "Get policy - implementation pending during migration"})))
}

pub async fn delete_policy_handler(
    State(_app_state): State<AppState>,
    Path(_policy_id): Path<String>,
) -> Result<StatusCode, StatusCode> {
    tracing::info!("IAM delete policy handler called during migration");
    Ok(StatusCode::OK)
}

#[derive(Deserialize)]
pub struct EvaluatePermissionRequest {
    user_id: String,
    resource: String,
    action: String,
}

pub async fn evaluate_permission_handler(
    State(app_state): State<AppState>,
    Json(request): Json<EvaluatePermissionRequest>,
) -> Result<Json<Value>, StatusCode> {
    match app_state.casbin_service.enforce(&request.user_id, &request.resource, &request.action).await {
        Ok(allowed) => {
            let response = json!({
                "allowed": allowed,
                "user_id": request.user_id,
                "resource": request.resource,
                "action": request.action
            });
            tracing::info!("Permission evaluation for user {} on {}/{}: {}", request.user_id, request.resource, request.action, allowed);
            Ok(Json(response))
        }
        Err(e) => {
            tracing::error!("Failed to evaluate permission: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn set_user_overrides_handler(
    State(_app_state): State<AppState>,
    Path(_user_id): Path<String>,
    Json(_request): Json<Value>,
) -> Result<Json<Value>, StatusCode> {
    tracing::info!("IAM set user overrides handler called during migration");
    Ok(Json(json!({"message": "Set user overrides - implementation pending during migration"})))
}

pub async fn get_user_overrides_handler(
    State(_app_state): State<AppState>,
    Path(_user_id): Path<String>,
) -> Result<Json<Value>, StatusCode> {
    tracing::info!("IAM get user overrides handler called during migration");
    Ok(Json(json!({"overrides": [], "message": "Get user overrides - implementation pending during migration"})))
}

pub async fn assign_role_to_user_handler(
    State(app_state): State<AppState>,
    Path((user_id, role_id)): Path<(String, String)>,
) -> Result<StatusCode, StatusCode> {
    match app_state.casbin_service.add_role_for_user(&user_id, &role_id).await {
        Ok(true) => {
            tracing::info!("Successfully assigned role {} to user {}", role_id, user_id);
            Ok(StatusCode::OK)
        }
        Ok(false) => {
            tracing::warn!("Role {} already assigned to user {}", role_id, user_id);
            Ok(StatusCode::OK)
        }
        Err(e) => {
            tracing::error!("Failed to assign role {} to user {}: {}", role_id, user_id, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn revoke_role_from_user_handler(
    State(app_state): State<AppState>,
    Path((user_id, role_id)): Path<(String, String)>,
) -> Result<StatusCode, StatusCode> {
    match app_state.casbin_service.remove_role_for_user(&user_id, &role_id).await {
        Ok(true) => {
            tracing::info!("Successfully revoked role {} from user {}", role_id, user_id);
            Ok(StatusCode::OK)
        }
        Ok(false) => {
            tracing::warn!("Role {} was not assigned to user {}", role_id, user_id);
            Ok(StatusCode::NOT_FOUND)
        }
        Err(e) => {
            tracing::error!("Failed to revoke role {} from user {}: {}", role_id, user_id, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn list_user_roles_handler(
    State(app_state): State<AppState>,
    Path(user_id): Path<String>,
) -> Result<Json<Value>, StatusCode> {
    match app_state.casbin_service.get_roles_for_user(&user_id).await {
        Ok(roles) => {
            tracing::info!("Retrieved {} roles for user {}", roles.len(), user_id);
            Ok(Json(json!({"roles": roles, "user_id": user_id})))
        }
        Err(e) => {
            tracing::error!("Failed to get roles for user {}: {}", user_id, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}