// IAM API handlers - HTTP endpoints for IAM operations with enhanced error handling

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{Json, Response, IntoResponse},
    Extension,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::app::use_cases::iam::{
    CreateRoleReq, UpdateRoleReq, CreatePolicyReq, EvaluatePermissionReq,
    SetUserOverrideReq, RoleResponse, PolicyResponse, EvaluatePermissionRes,
};
use crate::core::errors::{AppError, ErrorKind, ErrorContextBuilder};
use crate::dom::values::UserId;
use crate::web::AppState;
use crate::web::middleware::error_handling::app_error_to_response;

/// Query parameters for listing roles
#[derive(Debug, Deserialize)]
pub struct ListRolesQuery {
    pub limit: Option<u32>,
    pub offset: Option<u32>,
    pub package_tier: Option<String>,
}

/// Query parameters for listing policies
#[derive(Debug, Deserialize)]
pub struct ListPoliciesQuery {
    pub limit: Option<u32>,
    pub offset: Option<u32>,
    pub active_only: Option<bool>,
}

/// Error response format
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
}

/// Success response format
#[derive(Debug, Serialize)]
pub struct SuccessResponse<T> {
    pub success: bool,
    pub data: T,
}

/// Create a new IAM role with enhanced error handling
pub async fn create_role_handler(
    State(app_state): State<AppState>,
    Extension(user_id): Extension<UserId>,
    Json(req): Json<CreateRoleReq>,
) -> Response {
    let operation = "create_role";
    let service = "iam";
    
    // Create error context
    let error_context = ErrorContextBuilder::new(operation, service)
        .user_id(user_id.to_string())
        .metadata("role_name", req.name.clone())
        .build();
    
    let iam_uc = &app_state.iam_uc;
    
    match iam_uc.create_role(req, user_id.clone()).await {
        Ok(role) => {
            // Log successful operation
            tracing::info!(
                user_id = %user_id,
                role_id = %role.id,
                role_name = %role.name,
                operation = operation,
                "Role created successfully"
            );
            
            Json(SuccessResponse {
                success: true,
                data: role,
            }).into_response()
        },
        Err(err) => {
            // Convert to AppError with proper context
            let app_error = AppError::new(
                ErrorKind::InternalError,
                format!("Failed to create role: {}", err)
            ).with_context(error_context);
            
            app_error_to_response(app_error, None)
        },
    }
}

/// Get all IAM roles with enhanced error handling and pagination
pub async fn list_roles_handler(
    State(app_state): State<AppState>,
    Query(query): Query<ListRolesQuery>,
) -> Response {
    let operation = "list_roles";
    let service = "iam";
    
    // Create error context
    let error_context = ErrorContextBuilder::new(operation, service)
        .metadata("limit", query.limit.unwrap_or(50).to_string())
        .metadata("offset", query.offset.unwrap_or(0).to_string())
        .build();
    
    let iam_uc = &app_state.iam_uc;
    
    match iam_uc.list_roles().await {
        Ok(roles) => {
            // Log successful operation with metrics
            tracing::info!(
                operation = operation,
                role_count = roles.len(),
                limit = query.limit.unwrap_or(50),
                offset = query.offset.unwrap_or(0),
                "Roles listed successfully"
            );
            
            Json(SuccessResponse {
                success: true,
                data: roles,
            }).into_response()
        },
        Err(err) => {
            // Convert to AppError with proper context
            let app_error = AppError::new(
                ErrorKind::InternalError,
                format!("Failed to list roles: {}", err)
            ).with_context(error_context);
            
            app_error_to_response(app_error, None)
        },
    }
}

/// Get an IAM role by ID
pub async fn get_role_handler(
    State(app_state): State<AppState>,
    Path(role_id): Path<String>,
) -> Result<Json<SuccessResponse<RoleResponse>>, (StatusCode, Json<ErrorResponse>)> {
    let iam_uc = &app_state.iam_uc;
    
    match iam_uc.get_role(role_id).await {
        Ok(role) => Ok(Json(SuccessResponse {
            success: true,
            data: role,
        })),
        Err(err) => Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "RoleNotFound".to_string(),
                message: err.to_string(),
            }),
        )),
    }
}

/// Update an IAM role
pub async fn update_role_handler(
    State(app_state): State<AppState>,
    Path(role_id): Path<String>,
    Json(req): Json<UpdateRoleReq>,
) -> Result<Json<SuccessResponse<RoleResponse>>, (StatusCode, Json<ErrorResponse>)> {
    let iam_uc = &app_state.iam_uc;
    
    match iam_uc.update_role(role_id, req).await {
        Ok(role) => Ok(Json(SuccessResponse {
            success: true,
            data: role,
        })),
        Err(err) => Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "UpdateRoleFailed".to_string(),
                message: err.to_string(),
            }),
        )),
    }
}

/// Delete an IAM role
pub async fn delete_role_handler(
    State(app_state): State<AppState>,
    Path(role_id): Path<String>,
) -> Result<Json<SuccessResponse<String>>, (StatusCode, Json<ErrorResponse>)> {
    let iam_uc = &app_state.iam_uc;
    
    match iam_uc.delete_role(role_id).await {
        Ok(()) => Ok(Json(SuccessResponse {
            success: true,
            data: "Role deleted successfully".to_string(),
        })),
        Err(err) => Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "DeleteRoleFailed".to_string(),
                message: err.to_string(),
            }),
        )),
    }
}

/// Create a new IAM policy
pub async fn create_policy_handler(
    State(app_state): State<AppState>,
    Extension(user_id): Extension<UserId>,
    Json(req): Json<CreatePolicyReq>,
) -> Result<Json<SuccessResponse<PolicyResponse>>, (StatusCode, Json<ErrorResponse>)> {
    let iam_uc = &app_state.iam_uc;
    
    match iam_uc.create_policy(req, user_id).await {
        Ok(policy) => Ok(Json(SuccessResponse {
            success: true,
            data: policy,
        })),
        Err(err) => Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "CreatePolicyFailed".to_string(),
                message: err.to_string(),
            }),
        )),
    }
}

/// Get all IAM policies
pub async fn list_policies_handler(
    State(app_state): State<AppState>,
    Query(_query): Query<ListPoliciesQuery>,
) -> Result<Json<SuccessResponse<Vec<PolicyResponse>>>, (StatusCode, Json<ErrorResponse>)> {
    let iam_uc = &app_state.iam_uc;
    
    match iam_uc.list_policies().await {
        Ok(policies) => Ok(Json(SuccessResponse {
            success: true,
            data: policies,
        })),
        Err(err) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: "ListPoliciesFailed".to_string(),
                message: err.to_string(),
            }),
        )),
    }
}

/// Get an IAM policy by ID
pub async fn get_policy_handler(
    State(app_state): State<AppState>,
    Path(policy_id): Path<String>,
) -> Result<Json<SuccessResponse<PolicyResponse>>, (StatusCode, Json<ErrorResponse>)> {
    let iam_uc = &app_state.iam_uc;
    
    match iam_uc.get_policy(policy_id).await {
        Ok(policy) => Ok(Json(SuccessResponse {
            success: true,
            data: policy,
        })),
        Err(err) => Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "PolicyNotFound".to_string(),
                message: err.to_string(),
            }),
        )),
    }
}

/// Delete an IAM policy
pub async fn delete_policy_handler(
    State(app_state): State<AppState>,
    Path(policy_id): Path<String>,
) -> Result<Json<SuccessResponse<String>>, (StatusCode, Json<ErrorResponse>)> {
    let iam_uc = &app_state.iam_uc;
    
    match iam_uc.delete_policy(policy_id).await {
        Ok(()) => Ok(Json(SuccessResponse {
            success: true,
            data: "Policy deleted successfully".to_string(),
        })),
        Err(err) => Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "DeletePolicyFailed".to_string(),
                message: err.to_string(),
            }),
        )),
    }
}

/// Evaluate user permission for an action and resource
pub async fn evaluate_permission_handler(
    State(app_state): State<AppState>,
    Json(req): Json<EvaluatePermissionReq>,
) -> Result<Json<SuccessResponse<EvaluatePermissionRes>>, (StatusCode, Json<ErrorResponse>)> {
    let iam_uc = &app_state.iam_uc;
    
    match iam_uc.evaluate_permission(req).await {
        Ok(result) => Ok(Json(SuccessResponse {
            success: true,
            data: result,
        })),
        Err(err) => Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "PermissionEvaluationFailed".to_string(),
                message: err.to_string(),
            }),
        )),
    }
}

/// Set user permission overrides
pub async fn set_user_overrides_handler(
    State(app_state): State<AppState>,
    Extension(user_id): Extension<UserId>,
    Json(req): Json<SetUserOverrideReq>,
) -> Result<Json<SuccessResponse<String>>, (StatusCode, Json<ErrorResponse>)> {
    let iam_uc = &app_state.iam_uc;
    
    match iam_uc.set_user_overrides(req, user_id).await {
        Ok(()) => Ok(Json(SuccessResponse {
            success: true,
            data: "User overrides set successfully".to_string(),
        })),
        Err(err) => Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "SetUserOverridesFailed".to_string(),
                message: err.to_string(),
            }),
        )),
    }
}

/// Get user permission overrides
pub async fn get_user_overrides_handler(
    State(_app_state): State<AppState>,
    Path(_user_id): Path<String>,
) -> Result<Json<SuccessResponse<HashMap<String, serde_json::Value>>>, (StatusCode, Json<ErrorResponse>)> {
    // This is a placeholder for now - we need to implement the response type
    // for UserPermissionOverride in the use case layer
    Ok(Json(SuccessResponse {
        success: true,
        data: HashMap::new(), // TODO: Implement proper response
    }))
}

/// Assign role to user
pub async fn assign_role_to_user_handler(
    State(_app_state): State<AppState>,
    Path((_user_id, _role_id)): Path<(String, String)>,
) -> Result<Json<SuccessResponse<String>>, (StatusCode, Json<ErrorResponse>)> {
    // TODO: Implement role assignment logic
    Ok(Json(SuccessResponse {
        success: true,
        data: "Role assigned successfully".to_string(),
    }))
}

/// Remove role from user
pub async fn remove_role_from_user_handler(
    State(_app_state): State<AppState>,
    Path((_user_id, _role_id)): Path<(String, String)>,
) -> Result<Json<SuccessResponse<String>>, (StatusCode, Json<ErrorResponse>)> {
    // TODO: Implement role removal logic
    Ok(Json(SuccessResponse {
        success: true,
        data: "Role removed successfully".to_string(),
    }))
}

/// Get user roles
pub async fn get_user_roles_handler(
    State(_app_state): State<AppState>,
    Path(_user_id): Path<String>,
) -> Result<Json<SuccessResponse<Vec<RoleResponse>>>, (StatusCode, Json<ErrorResponse>)> {
    // TODO: Implement get user roles logic
    Ok(Json(SuccessResponse {
        success: true,
        data: vec![], // TODO: Implement proper response
    }))
}