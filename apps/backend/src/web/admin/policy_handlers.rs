// Policy handlers - DISABLED during legacy cleanup
// These handlers require database tables that don't exist yet
use axum::{
    extract::{Path, Query, State},
    response::Json,
    http::StatusCode,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::warn;

use crate::auth::{
    PolicyCondition, PolicyAction, PolicyEvaluationResult
};
use crate::auth::policy_engine::PolicyType;
use crate::infrastructure::container::AuthenticatedUser;
use crate::web::auth::AppState;

/// Request to create a dynamic policy
#[derive(Debug, Deserialize)]
pub struct CreatePolicyRequest {
    pub name: String,
    pub description: Option<String>,
    pub policy_type: PolicyType,
    pub target_actions: Vec<String>,
    pub conditions: PolicyCondition,
    pub actions: PolicyAction,
    pub priority: Option<i32>,
    pub effective_from: Option<DateTime<Utc>>,
    pub effective_until: Option<DateTime<Utc>>,
}

/// Request to evaluate a policy
#[derive(Debug, Deserialize)]
pub struct PolicyEvaluationRequest {
    pub user_id: String,
    pub user_email: String,
    pub action: String,
    pub resource: Option<HashMap<String, serde_json::Value>>,
    pub simulate_context: Option<SimulatedContext>,
}

/// Simulated context for testing policies
#[derive(Debug, Deserialize)]
pub struct SimulatedContext {
    pub time_of_day: Option<u32>,
    pub day_of_week: Option<u32>,
    pub device_trust_score: Option<f64>,
    pub location_country: Option<String>,
}

/// Response with policy data
#[derive(Debug, Serialize)]
pub struct PolicyResponse {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub policy_type: PolicyType,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: String,
}

/// List policies response
#[derive(Debug, Serialize)]
pub struct ListPoliciesResponse {
    pub policies: Vec<PolicyResponse>,
    pub total_count: u64,
    pub page: u32,
    pub limit: u32,
}

/// Query parameters for listing policies
#[derive(Debug, Deserialize)]
pub struct ListPoliciesQuery {
    pub page: Option<u32>,
    pub limit: Option<u32>,
    pub policy_type: Option<PolicyType>,
    pub active_only: Option<bool>,
}

/// Policy statistics response
#[derive(Debug, Serialize)]
pub struct PolicyStatsResponse {
    pub total_policies: u64,
    pub active_policies: u64,
    pub type_distribution: HashMap<String, u64>,
    pub evaluations_last_24h: u64,
    pub avg_evaluation_time_ms: f64,
    pub decision_distribution: HashMap<String, u64>,
}

/// Delete policy response
#[derive(Debug, Serialize)]
pub struct DeletePolicyResponse {
    pub success: bool,
    pub message: String,
}

/// Toggle policy response
#[derive(Debug, Serialize)]
pub struct TogglePolicyResponse {
    pub id: String,
    pub is_active: bool,
    pub message: String,
}

// DISABLED HANDLERS - Placeholder implementations

/// Create a new dynamic policy
pub async fn create_policy_handler(
    _user: AuthenticatedUser,
    State(_state): State<AppState>,
    Json(_request): Json<CreatePolicyRequest>,
) -> Result<Json<PolicyResponse>, StatusCode> {
    warn!("Policy handlers are disabled during legacy cleanup");
    Err(StatusCode::SERVICE_UNAVAILABLE)
}

/// List policies with pagination and filtering
pub async fn list_policies_handler(
    _user: AuthenticatedUser,
    State(_state): State<AppState>,
    Query(_query): Query<ListPoliciesQuery>,
) -> Result<Json<ListPoliciesResponse>, StatusCode> {
    warn!("Policy handlers are disabled during legacy cleanup");
    Ok(Json(ListPoliciesResponse {
        policies: vec![],
        total_count: 0,
        page: 1,
        limit: 10,
    }))
}

/// Get policy by ID
pub async fn get_policy_handler(
    _user: AuthenticatedUser,
    State(_state): State<AppState>,
    Path(_policy_id): Path<String>,
) -> Result<Json<PolicyResponse>, StatusCode> {
    warn!("Policy handlers are disabled during legacy cleanup");
    Err(StatusCode::SERVICE_UNAVAILABLE)
}

/// Update an existing policy
pub async fn update_policy_handler(
    _user: AuthenticatedUser,
    State(_state): State<AppState>,
    Path(_policy_id): Path<String>,
    Json(_request): Json<CreatePolicyRequest>,
) -> Result<Json<PolicyResponse>, StatusCode> {
    warn!("Policy handlers are disabled during legacy cleanup");
    Err(StatusCode::SERVICE_UNAVAILABLE)
}

/// Delete a policy
pub async fn delete_policy_handler(
    _user: AuthenticatedUser,
    State(_state): State<AppState>,
    Path(_policy_id): Path<String>,
) -> Result<Json<DeletePolicyResponse>, StatusCode> {
    warn!("Policy handlers are disabled during legacy cleanup");
    Ok(Json(DeletePolicyResponse {
        success: false,
        message: "Policy handlers are disabled during legacy cleanup".to_string(),
    }))
}

/// Evaluate a policy for a user/action combination
pub async fn evaluate_policy_handler(
    _user: AuthenticatedUser,
    State(_state): State<AppState>,
    Json(_request): Json<PolicyEvaluationRequest>,
) -> Result<Json<PolicyEvaluationResult>, StatusCode> {
    warn!("Policy handlers are disabled during legacy cleanup");
    Err(StatusCode::SERVICE_UNAVAILABLE)
}

/// Get policy statistics
pub async fn get_policy_stats_handler(
    _user: AuthenticatedUser,
    State(_state): State<AppState>,
) -> Result<Json<PolicyStatsResponse>, StatusCode> {
    warn!("Policy handlers are disabled during legacy cleanup");
    Ok(Json(PolicyStatsResponse {
        total_policies: 0,
        active_policies: 0,
        type_distribution: HashMap::new(),
        evaluations_last_24h: 0,
        avg_evaluation_time_ms: 0.0,
        decision_distribution: HashMap::new(),
    }))
}

/// Toggle policy active/inactive status
pub async fn toggle_policy_handler(
    _user: AuthenticatedUser,
    State(_state): State<AppState>,
    Path(_policy_id): Path<String>,
) -> Result<Json<TogglePolicyResponse>, StatusCode> {
    warn!("Policy handlers are disabled during legacy cleanup");
    Err(StatusCode::SERVICE_UNAVAILABLE)
}