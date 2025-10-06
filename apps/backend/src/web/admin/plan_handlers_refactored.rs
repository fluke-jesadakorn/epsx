// REFACTORED: Plan Management Handlers (Clean Architecture + CQRS)
// This file demonstrates the proper hexagonal architecture pattern
//
// OLD PATTERN (Architecture Violation):
//   Handler → Repository (direct database access)
//
// NEW PATTERN (Clean Architecture):
//   Handler → Command/Query → CommandHandler/QueryHandler → Repository Port
//
// Benefits:
// - Web layer is thin orchestration only
// - Business logic in domain/application layers
// - Easy to test (mock command/query handlers)
// - Follows dependency inversion principle

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json as JsonResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use rust_decimal::Decimal;
use std::collections::HashMap;

use crate::web::auth::AppState;
use crate::application::subscription_management::{
    // Commands
    CreatePlanCommand,
    CreatePlanResponse as AppCreatePlanResponse,
    UpdatePlanCommand,
    UpdatePlanResponse as AppUpdatePlanResponse,
    DeletePlanCommand,
    // Queries
    GetPlanQuery,
    GetPlanResponse as AppGetPlanResponse,
    ListPlansQuery,
    ListPlansResponse as AppListPlansResponse,
};
use crate::application::shared::{CommandHandler, QueryHandler};

// ============================================================================
// REQUEST/RESPONSE DTOs (Web Layer)
// These are separate from application DTOs to allow independent evolution
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreatePlanRequest {
    pub name: String,
    pub description: String,
    pub permission_group_id: String,
    pub price: f64,
    pub currency: String,
    pub billing_cycle: String,
    pub target_audience: String,
    pub api_calls_limit: Option<i32>,
    pub rankings_limit: Option<i32>,
    pub analytics_enabled: bool,
    pub premium_support: bool,
    pub is_active: Option<bool>,
    pub is_promoted: Option<bool>,
    pub display_order: Option<i32>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct UpdatePlanRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub price: Option<f64>,
    pub currency: Option<String>,
    pub billing_cycle: Option<String>,
    pub target_audience: Option<String>,
    pub is_active: Option<bool>,
    pub is_promoted: Option<bool>,
    pub display_order: Option<i32>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
pub struct PlanResponse {
    pub id: i32,
    pub name: String,
    pub description: String,
    pub permission_group_id: String,
    pub price: f64,
    pub currency: String,
    pub billing_cycle: String,
    pub target_audience: String,
    pub is_active: bool,
    pub is_promoted: bool,
    pub features: serde_json::Value,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize)]
pub struct PlansListResponse {
    pub plans: Vec<PlanSummary>,
    pub total: i64,
    pub page: u32,
    pub limit: u32,
}

#[derive(Debug, Serialize)]
pub struct PlanSummary {
    pub id: i32,
    pub name: String,
    pub description: String,
    pub price: f64,
    pub currency: String,
    pub billing_cycle: String,
    pub is_active: bool,
    pub is_promoted: bool,
}

// ============================================================================
// CLEAN HANDLERS (Following Hexagonal Architecture)
// Pattern: Request DTO → Command/Query → Handler → Response DTO
// ============================================================================

/// Create a new plan
///
/// Clean Architecture Flow:
/// 1. Extract & validate request DTO
/// 2. Map to application command
/// 3. Execute command handler (domain logic)
/// 4. Map result to response DTO
/// 5. Return HTTP response
pub async fn create_plan(
    State(app_state): State<AppState>,
    Json(req): Json<CreatePlanRequest>,
) -> Result<JsonResponse<PlanResponse>, StatusCode> {
    // 1. Validate request (web layer responsibility)
    if req.name.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    // 2. Map web DTO to application command
    let command = CreatePlanCommand {
        name: req.name,
        description: req.description,
        permission_group_id: req.permission_group_id,
        price: req.price,
        currency: req.currency,
        billing_cycle: req.billing_cycle,
        target_audience: req.target_audience,
        api_calls_limit: req.api_calls_limit,
        rankings_limit: req.rankings_limit,
        analytics_enabled: req.analytics_enabled,
        premium_support: req.premium_support,
        is_active: req.is_active,
        is_promoted: req.is_promoted,
        display_order: req.display_order,
        metadata: req.metadata,
    };

    // 3. Execute command through handler (application layer)
    let handler = app_state.create_plan_handler();
    let result = handler.handle(command).await
        .map_err(|err| {
            tracing::error!("Failed to create plan: {}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // 4. Map application response to web response DTO
    let response = PlanResponse {
        id: result.plan_id,
        name: result.name,
        description: String::new(), // Will be populated by query
        permission_group_id: String::new(),
        price: 0.0,
        currency: String::new(),
        billing_cycle: String::new(),
        target_audience: String::new(),
        is_active: true,
        is_promoted: false,
        features: serde_json::json!({}),
        created_at: result.created_at,
    };

    // 5. Return HTTP response
    Ok(JsonResponse(response))
}

/// List all plans
///
/// Clean Architecture Flow:
/// 1. Extract query parameters
/// 2. Map to application query
/// 3. Execute query handler (read model)
/// 4. Map result to response DTO
pub async fn list_plans(
    State(app_state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<JsonResponse<PlansListResponse>, StatusCode> {
    // 1. Extract & parse query parameters
    let is_active = params.get("is_active")
        .and_then(|v| v.parse::<bool>().ok());
    let is_promoted = params.get("is_promoted")
        .and_then(|v| v.parse::<bool>().ok());
    let page = params.get("page")
        .and_then(|v| v.parse::<u32>().ok());
    let limit = params.get("limit")
        .and_then(|v| v.parse::<u32>().ok());

    // 2. Create application query
    let query = ListPlansQuery {
        is_active,
        is_promoted,
        page,
        limit,
    };

    // 3. Execute query through handler
    let handler = app_state.list_plans_handler();
    let result = handler.handle(query).await
        .map_err(|err| {
            tracing::error!("Failed to list plans: {}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // 4. Map application response to web response
    let response = PlansListResponse {
        plans: result.plans.into_iter().map(|p| PlanSummary {
            id: p.id,
            name: p.name,
            description: p.description,
            price: p.price,
            currency: p.currency,
            billing_cycle: p.billing_cycle,
            is_active: p.is_active,
            is_promoted: p.is_promoted,
        }).collect(),
        total: result.total,
        page: result.page,
        limit: result.limit,
    };

    Ok(JsonResponse(response))
}

/// Get plan by ID
///
/// Clean Architecture Flow:
/// 1. Extract & validate path parameter
/// 2. Create application query
/// 3. Execute query handler
/// 4. Map result to response DTO
pub async fn get_plan(
    State(app_state): State<AppState>,
    Path(plan_id): Path<i32>,
) -> Result<JsonResponse<PlanResponse>, StatusCode> {
    // 1. Validate input
    if plan_id <= 0 {
        return Err(StatusCode::BAD_REQUEST);
    }

    // 2. Create application query
    let query = GetPlanQuery { plan_id };

    // 3. Execute query through handler
    let handler = app_state.get_plan_handler();
    let result = handler.handle(query).await
        .map_err(|err| {
            tracing::error!("Failed to get plan {}: {}", plan_id, err);
            StatusCode::NOT_FOUND
        })?;

    // 4. Map to web response DTO
    let response = PlanResponse {
        id: result.id,
        name: result.name,
        description: result.description,
        permission_group_id: result.permission_group_id,
        price: result.price,
        currency: result.currency,
        billing_cycle: result.billing_cycle,
        target_audience: result.target_audience,
        is_active: result.is_active,
        is_promoted: result.is_promoted,
        features: result.features,
        created_at: result.created_at,
    };

    Ok(JsonResponse(response))
}

/// Update plan
///
/// Clean Architecture Flow:
/// 1. Validate request and path parameter
/// 2. Map to update command
/// 3. Execute command handler
/// 4. Return updated plan
pub async fn update_plan(
    State(app_state): State<AppState>,
    Path(plan_id): Path<i32>,
    Json(req): Json<UpdatePlanRequest>,
) -> Result<JsonResponse<serde_json::Value>, StatusCode> {
    // 1. Validate
    if plan_id <= 0 {
        return Err(StatusCode::BAD_REQUEST);
    }

    // 2. Map to application command
    let command = UpdatePlanCommand {
        plan_id,
        name: req.name,
        description: req.description,
        price: req.price,
        currency: req.currency,
        billing_cycle: req.billing_cycle,
        target_audience: req.target_audience,
        is_active: req.is_active,
        is_promoted: req.is_promoted,
        display_order: req.display_order,
        metadata: req.metadata,
    };

    // 3. Execute command
    let handler = app_state.update_plan_handler();
    let result = handler.handle(command).await
        .map_err(|err| {
            tracing::error!("Failed to update plan {}: {}", plan_id, err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // 4. Return success response
    Ok(JsonResponse(serde_json::json!({
        "success": true,
        "data": {
            "plan_id": result.plan_id,
            "updated_at": result.updated_at,
        },
        "message": "Plan updated successfully"
    })))
}

/// Delete plan
///
/// Clean Architecture Flow:
/// 1. Validate path parameter
/// 2. Create delete command
/// 3. Execute command handler
/// 4. Return success response
pub async fn delete_plan(
    State(app_state): State<AppState>,
    Path(plan_id): Path<i32>,
) -> Result<JsonResponse<serde_json::Value>, StatusCode> {
    // 1. Validate
    if plan_id <= 0 {
        return Err(StatusCode::BAD_REQUEST);
    }

    // 2. Create command
    let command = DeletePlanCommand { plan_id };

    // 3. Execute command
    let handler = app_state.delete_plan_handler();
    let _result = handler.handle(command).await
        .map_err(|err| {
            tracing::error!("Failed to delete plan {}: {}", plan_id, err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // 4. Return success
    Ok(JsonResponse(serde_json::json!({
        "success": true,
        "message": "Plan deleted successfully"
    })))
}

// ============================================================================
// HELPER: AppState Extension (Dependency Injection)
// ============================================================================

impl AppState {
    /// Get CreatePlanCommandHandler (injected dependency)
    pub fn create_plan_handler(&self) -> impl CommandHandler<CreatePlanCommand> {
        // TODO: Return actual handler from DI container
        // For now, this is a placeholder showing the pattern
        todo!("Inject CreatePlanCommandHandler from container")
    }

    pub fn update_plan_handler(&self) -> impl CommandHandler<UpdatePlanCommand> {
        todo!("Inject UpdatePlanCommandHandler from container")
    }

    pub fn delete_plan_handler(&self) -> impl CommandHandler<DeletePlanCommand> {
        todo!("Inject DeletePlanCommandHandler from container")
    }

    pub fn get_plan_handler(&self) -> impl QueryHandler<GetPlanQuery> {
        todo!("Inject GetPlanQueryHandler from container")
    }

    pub fn list_plans_handler(&self) -> impl QueryHandler<ListPlansQuery> {
        todo!("Inject ListPlansQueryHandler from container")
    }
}

// ============================================================================
// KEY IMPROVEMENTS OVER OLD HANDLERS:
// ============================================================================
//
// 1. ✅ NO DIRECT DATABASE ACCESS
//    - Old: `app_state.permission_group_repo.get_subscription_plans()`
//    - New: `handler.handle(query)` → repository called by application layer
//
// 2. ✅ CLEAR LAYER SEPARATION
//    - Web: HTTP concerns, DTO mapping
//    - Application: Use case orchestration
//    - Domain: Business logic
//    - Infrastructure: Database access
//
// 3. ✅ DEPENDENCY INVERSION
//    - Web depends on application (commands/queries)
//    - Application depends on domain (ports)
//    - Infrastructure implements domain ports
//
// 4. ✅ TESTABILITY
//    - Can mock command/query handlers
//    - No database required for handler tests
//    - Business logic testable in isolation
//
// 5. ✅ MAINTAINABILITY
//    - Thin handlers (10-20 lines each)
//    - Business logic in domain services
//    - Easy to find and change code
//
// 6. ✅ FOLLOWS SOLID PRINCIPLES
//    - Single Responsibility: Handler only handles HTTP
//    - Open/Closed: Extend via new commands/queries
//    - Liskov Substitution: Handler interfaces
//    - Interface Segregation: Specific command/query types
//    - Dependency Inversion: Depend on abstractions (handlers)
