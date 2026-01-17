use axum::{
    extract::{State, Path},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use serde_json::Value;
use crate::application::admin::commands::assign_admin_group::{
    AssignAdminGroupCommand, AssignAdminGroupHandler
};


use crate::web::auth::AppState;
// Removed unused imports


#[derive(Debug, Serialize, Deserialize)]
pub struct AdminAssignmentRequest {
    pub group: String,
    pub custom_claims: Option<HashMap<String, Value>>,
}

#[derive(Debug, Serialize)]
pub struct AdminAssignmentResponse {
    pub success: bool,
    pub message: String,
    pub wallet_address: String,
    pub assigned_group: String,
    pub custom_claims: HashMap<String, Value>,
}

// Helper functions removed as they are now in infrastructure adapter


/// Assign admin group to a Firebase user
pub async fn assign_admin_group_handler(
    State(app_state): State<AppState>,
    Path(wallet_address): Path<String>,
    Json(request): Json<AdminAssignmentRequest>,
) -> Result<Json<AdminAssignmentResponse>, StatusCode> {
    tracing::info!("Admin assignment request for user {} with group {}", wallet_address, request.group);
    
    let identity_provider = app_state
        .identity_provider
        .clone()
        .ok_or_else(|| {
            tracing::error!("IdentityProvider not configured");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let command = AssignAdminGroupCommand {
        wallet_address,
        group_name: request.group,
        custom_claims: request.custom_claims,
    };
    
    let handler = AssignAdminGroupHandler::new(identity_provider);
    
    match handler.handle(command).await {
        Ok(response) => {
            // Convert application response to web response if needed (structs are identical/compatible)
            Ok(Json(AdminAssignmentResponse {
                success: response.success,
                message: response.message,
                wallet_address: response.wallet_address,
                assigned_group: response.assigned_group,
                custom_claims: response.custom_claims,
            }))
        }
        Err(e) => {
            tracing::error!("Failed to assign admin group: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}


/// Get user's current custom claims
pub async fn get_user_claims_handler(
    State(app_state): State<AppState>,
    Path(wallet_address): Path<String>,
) -> Result<Json<HashMap<String, Value>>, StatusCode> {
    let identity_provider = app_state
        .identity_provider
        .clone()
        .ok_or_else(|| {
            tracing::error!("IdentityProvider not configured");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    
    match identity_provider.get_user_claims(&wallet_address).await {
        Ok(claims) => Ok(Json(claims)),
        Err(e) => {
            tracing::error!("Failed to get user claims for {}: {}", wallet_address, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}