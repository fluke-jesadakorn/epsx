//! Subscription Management API Handlers
//!
//! Direct Payment model: users pay to activate/extend their plan duration
//! No auto-renewal - users must pay again when their plan expires
//! Uses wallet_users.plan_expires_at instead of active_subscriptions table

use axum::{
    extract::{State, Path, Extension},
    response::Json,
};
use diesel::OptionalExtension;
use diesel_async::RunQueryDsl;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use tracing::info;

use crate::{
    prelude::*,
    web::middleware::{UnifiedErrorResponse, ErrorDetails, OpenIDUserContext},
};

// ============================================================================
// REQUEST/RESPONSE TYPES
// ============================================================================

/// Get user plans response
#[derive(Debug, Serialize)]
pub struct UserPlansResponse {
    pub success: bool,
    pub message: String,
    pub data: Option<PlanAccessData>,
}

/// Plan access data (simplified for Direct Payment model)
#[derive(Debug, Clone, Serialize)]
pub struct PlanAccessData {
    pub wallet_address: String,
    pub current_plan_id: Option<i32>,
    pub plan_name: Option<String>,
    pub plan_expires_at: Option<DateTime<Utc>>,
    pub days_remaining: i64,
    pub status: String, // "active", "expiring_soon", "expired", "no_plan"
}

/// Plan expiry status response
#[derive(Debug, Serialize)]
pub struct PlanExpiryResponse {
    pub success: bool,
    pub message: String,
    pub data: Option<PlanAccessData>,
}

/// Cancel plan request
#[derive(Debug, Deserialize)]
pub struct CancelPlanRequest {
    pub reason: Option<String>,
}

/// Cancel plan response
#[derive(Debug, Serialize)]
pub struct CancelPlanResponse {
    pub success: bool,
    pub message: String,
}

// ============================================================================
// HANDLERS
// ============================================================================

/// Get user's plan access status
pub async fn get_user_plans_handler(
    State(app_state): State<crate::web::auth::AppState>,
    Extension(user_context): Extension<OpenIDUserContext>,
) -> Result<Json<UserPlansResponse>, Json<UnifiedErrorResponse>> {
    info!("Getting plan access for user: {}", user_context.wallet_address);

    let mut conn = app_state.db_pool
        .get()
        .await
        .map_err(|e| Json(UnifiedErrorResponse {
            success: false,
            error: ErrorDetails {
                code: 500,
                message: "Database connection failed".to_string(),
                reason: e.to_string(),
            },
        }))?;

    #[derive(diesel::QueryableByName)]
    struct UserRow {
        #[diesel(sql_type = diesel::sql_types::Text)]
        wallet_address: String,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>)]
        plan_expires_at: Option<DateTime<Utc>>,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Integer>)]
        current_plan_id: Option<i32>,
    }

    let user: Option<UserRow> = diesel::sql_query(
        r#"
        SELECT wallet_address, plan_expires_at, current_plan_id
        FROM wallet_users
        WHERE LOWER(wallet_address) = LOWER($1)
        "#
    )
    .bind::<diesel::sql_types::Text, _>(&user_context.wallet_address)
    .get_result(&mut conn)
    .await
    .optional()
    .map_err(|e| Json(UnifiedErrorResponse {
        success: false,
        error: ErrorDetails {
            code: 500,
            message: "Failed to fetch user".to_string(),
            reason: e.to_string(),
        },
    }))?;

    let now = Utc::now();
    
    let data = match user {
        Some(u) => {
            let days_remaining = u.plan_expires_at
                .map(|exp| (exp - now).num_days())
                .unwrap_or(0);
            
            let is_expired = days_remaining < 0;
            let status = if u.current_plan_id.is_none() {
                "no_plan"
            } else if is_expired {
                "expired"
            } else if days_remaining <= 7 {
                "expiring_soon"
            } else {
                "active"
            };

            // Get plan name if we have a plan ID
            let plan_name = if let Some(plan_id) = u.current_plan_id {
                #[derive(diesel::QueryableByName)]
                struct PlanNameRow {
                    #[diesel(sql_type = diesel::sql_types::Text)]
                    name: String,
                }
                
                diesel::sql_query("SELECT name FROM pricing_plans WHERE id = $1")
                    .bind::<diesel::sql_types::Integer, _>(plan_id)
                    .get_result::<PlanNameRow>(&mut conn)
                    .await
                    .ok()
                    .map(|p| p.name)
            } else {
                None
            };

            PlanAccessData {
                wallet_address: u.wallet_address,
                current_plan_id: u.current_plan_id,
                plan_name,
                plan_expires_at: u.plan_expires_at,
                days_remaining: days_remaining.max(0),
                status: status.to_string(),
            }
        },
        None => PlanAccessData {
            wallet_address: user_context.wallet_address.clone(),
            current_plan_id: None,
            plan_name: None,
            plan_expires_at: None,
            days_remaining: 0,
            status: "no_plan".to_string(),
        },
    };

    Ok(Json(UserPlansResponse {
        success: true,
        message: "User plan access retrieved successfully".to_string(),
        data: Some(data),
    }))
}

/// Get plan expiry status (for renewal prompts)
pub async fn get_plan_expiry_status_handler(
    State(app_state): State<crate::web::auth::AppState>,
    Extension(user_context): Extension<OpenIDUserContext>,
) -> Result<Json<PlanExpiryResponse>, Json<UnifiedErrorResponse>> {
    info!("Checking plan expiry for user: {}", user_context.wallet_address);

    // Reuse get_user_plans_handler logic
    let result = get_user_plans_handler(State(app_state), Extension(user_context)).await?;
    
    Ok(Json(PlanExpiryResponse {
        success: result.success,
        message: "Plan expiry status retrieved".to_string(),
        data: result.data.clone(),
    }))
}

/// Cancel plan (clear the current_plan_id, keep expiry so user can use until it expires)
pub async fn cancel_plan_handler(
    State(app_state): State<crate::web::auth::AppState>,
    Extension(user_context): Extension<OpenIDUserContext>,
    Path(_plan_id): Path<Uuid>,
    Json(_payload): Json<CancelPlanRequest>,
) -> Result<Json<CancelPlanResponse>, Json<UnifiedErrorResponse>> {
    info!("Cancelling plan for user {}", user_context.wallet_address);

    let mut conn = app_state.db_pool
        .get()
        .await
        .map_err(|e| Json(UnifiedErrorResponse {
            success: false,
            error: ErrorDetails {
                code: 500,
                message: "Database connection failed".to_string(),
                reason: e.to_string(),
            },
        }))?;

    // Mark the plan as cancelled but don't change expiry (user can still use until expiry)
    // In Direct Payment model, we just clear the current_plan_id
    let rows_affected = diesel::sql_query(
        r#"
        UPDATE wallet_users
        SET current_plan_id = NULL, updated_at = $1
        WHERE LOWER(wallet_address) = LOWER($2) AND current_plan_id IS NOT NULL
        "#
    )
    .bind::<diesel::sql_types::Timestamptz, _>(Utc::now())
    .bind::<diesel::sql_types::Text, _>(&user_context.wallet_address)
    .execute(&mut conn)
    .await
    .map_err(|e| Json(UnifiedErrorResponse {
        success: false,
        error: ErrorDetails {
            code: 500,
            message: "Failed to cancel plan".to_string(),
            reason: e.to_string(),
        },
    }))?;

    if rows_affected == 0 {
        return Ok(Json(CancelPlanResponse {
            success: false,
            message: "No active plan found for this wallet".to_string(),
        }));
    }

    Ok(Json(CancelPlanResponse {
        success: true,
        message: "Plan cancelled. You can continue using it until expiry.".to_string(),
    }))
}

// ============================================================================
// UPGRADE PREVIEW (Simplified - no complex credit calculation)
// ============================================================================

/// Upgrade preview response
#[derive(Debug, Serialize)]
pub struct UpgradePreviewResponse {
    pub success: bool,
    pub message: String,
    pub data: Option<UpgradePreviewData>,
}

/// Upgrade preview data
#[derive(Debug, Serialize)]
pub struct UpgradePreviewData {
    pub current_plan: Option<CurrentPlanInfo>,
    pub new_plan: NewPlanInfo,
    pub payment_required: String,
    pub new_duration_days: i64,
    pub new_expiry_date: DateTime<Utc>,
}

/// Current plan info
#[derive(Debug, Serialize)]
pub struct CurrentPlanInfo {
    pub id: Option<i32>,
    pub name: String,
    pub expires_at: Option<DateTime<Utc>>,
    pub days_remaining: i64,
}

/// New plan info
#[derive(Debug, Serialize)]
pub struct NewPlanInfo {
    pub id: i32,
    pub name: String,
    pub price: String,
}

/// Query params for upgrade preview
#[derive(Debug, Deserialize)]
pub struct UpgradePreviewQuery {
    pub new_plan_id: i32,
}

/// Get upgrade preview - shows what happens when user pays
pub async fn get_upgrade_preview_handler(
    State(app_state): State<crate::web::auth::AppState>,
    Extension(user_context): Extension<OpenIDUserContext>,
    axum::extract::Query(query): axum::extract::Query<UpgradePreviewQuery>,
) -> Result<Json<UpgradePreviewResponse>, Json<UnifiedErrorResponse>> {
    use std::str::FromStr;
    
    info!("Getting upgrade preview for user: {}, new_plan_id: {}", 
          user_context.wallet_address, query.new_plan_id);

    let mut conn = app_state.db_pool
        .get()
        .await
        .map_err(|e| Json(UnifiedErrorResponse {
            success: false,
            error: ErrorDetails {
                code: 500,
                message: "Database connection failed".to_string(),
                reason: e.to_string(),
            },
        }))?;

    // Get new plan details
    #[derive(diesel::QueryableByName)]
    struct PlanRow {
        #[diesel(sql_type = diesel::sql_types::Integer)]
        id: i32,
        #[diesel(sql_type = diesel::sql_types::Text)]
        name: String,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Numeric>)]
        price: Option<bigdecimal::BigDecimal>,
    }

    let new_plan: Option<PlanRow> = diesel::sql_query(
        "SELECT id, name, price FROM pricing_plans WHERE id = $1"
    )
    .bind::<diesel::sql_types::Integer, _>(query.new_plan_id)
    .get_result(&mut conn)
    .await
    .optional()
    .map_err(|e| Json(UnifiedErrorResponse {
        success: false,
        error: ErrorDetails {
            code: 500,
            message: "Failed to fetch plan".to_string(),
            reason: e.to_string(),
        },
    }))?;

    let new_plan = match new_plan {
        Some(p) => p,
        None => {
            return Ok(Json(UpgradePreviewResponse {
                success: false,
                message: "Plan not found".to_string(),
                data: None,
            }));
        }
    };

    let new_plan_price: rust_decimal::Decimal = new_plan.price.as_ref()
        .and_then(|bd| rust_decimal::Decimal::from_str(&bd.to_string()).ok())
        .unwrap_or(rust_decimal::Decimal::ZERO);

    // Get current user plan info
    #[derive(diesel::QueryableByName)]
    struct UserRow {
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>)]
        plan_expires_at: Option<DateTime<Utc>>,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Integer>)]
        current_plan_id: Option<i32>,
    }

    let user: Option<UserRow> = diesel::sql_query(
        "SELECT plan_expires_at, current_plan_id FROM wallet_users WHERE LOWER(wallet_address) = LOWER($1)"
    )
    .bind::<diesel::sql_types::Text, _>(&user_context.wallet_address)
    .get_result(&mut conn)
    .await
    .optional()
    .map_err(|e| Json(UnifiedErrorResponse {
        success: false,
        error: ErrorDetails {
            code: 500,
            message: "Failed to fetch user".to_string(),
            reason: e.to_string(),
        },
    }))?;

    let now = Utc::now();
    let standard_duration_days: i64 = 30;

    let current_plan_info = if let Some(ref u) = user {
        if let Some(plan_id) = u.current_plan_id {
            let current_plan: Option<PlanRow> = diesel::sql_query(
                "SELECT id, name, price FROM pricing_plans WHERE id = $1"
            )
            .bind::<diesel::sql_types::Integer, _>(plan_id)
            .get_result(&mut conn)
            .await
            .optional()
            .ok()
            .flatten();

            let days_remaining = u.plan_expires_at
                .map(|exp| if exp > now { (exp - now).num_days() } else { 0 })
                .unwrap_or(0);

            current_plan.map(|p| CurrentPlanInfo {
                id: Some(p.id),
                name: p.name,
                expires_at: u.plan_expires_at,
                days_remaining,
            })
        } else {
            None
        }
    } else {
        None
    };

    // Simple calculation: new expiry = now + 30 days (or extend from current if same plan)
    let new_expiry = if let Some(ref u) = user {
        if u.current_plan_id == Some(query.new_plan_id) {
            // Same plan - extend from current expiry
            let base = u.plan_expires_at.filter(|&exp| exp > now).unwrap_or(now);
            base + chrono::Duration::days(standard_duration_days)
        } else {
            // Different plan - start fresh
            now + chrono::Duration::days(standard_duration_days)
        }
    } else {
        now + chrono::Duration::days(standard_duration_days)
    };

    let response_data = UpgradePreviewData {
        current_plan: current_plan_info,
        new_plan: NewPlanInfo {
            id: new_plan.id,
            name: new_plan.name,
            price: new_plan_price.to_string(),
        },
        payment_required: new_plan_price.to_string(),
        new_duration_days: standard_duration_days,
        new_expiry_date: new_expiry,
    };

    let message = if user.as_ref().and_then(|u| u.current_plan_id) == Some(query.new_plan_id) {
        format!("Extending your current plan by {} days", standard_duration_days)
    } else if user.as_ref().and_then(|u| u.current_plan_id).is_some() {
        format!("Switching to new plan - {} days access", standard_duration_days)
    } else {
        format!("New subscription - {} days access", standard_duration_days)
    };

    Ok(Json(UpgradePreviewResponse {
        success: true,
        message,
        data: Some(response_data),
    }))
}

// ============================================================================
// LEGACY HANDLERS (for backward compatibility)
// ============================================================================

/// Legacy: Get user subscriptions (maps to get_user_plans_handler)
pub async fn get_user_subscriptions_handler(
    state: State<crate::web::auth::AppState>,
    ext: Extension<OpenIDUserContext>,
) -> Result<Json<UserPlansResponse>, Json<UnifiedErrorResponse>> {
    get_user_plans_handler(state, ext).await
}

/// Legacy: Get subscription details
pub async fn get_subscription_details_handler(
    State(_app_state): State<crate::web::auth::AppState>,
    Path(_subscription_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, Json<UnifiedErrorResponse>> {
    Ok(Json(serde_json::json!({
        "success": false,
        "message": "Use /api/v1/plans/expiry-status instead"
    })))
}

/// Legacy: Cancel subscription (maps to cancel_plan_handler)
pub async fn cancel_subscription_handler(
    state: State<crate::web::auth::AppState>,
    ext: Extension<OpenIDUserContext>,
    path: Path<Uuid>,
    payload: Json<CancelPlanRequest>,
) -> Result<Json<CancelPlanResponse>, Json<UnifiedErrorResponse>> {
    cancel_plan_handler(state, ext, path, payload).await
}

/// Legacy: Renew subscription - REMOVED (users pay via smart contract)
pub async fn renew_subscription_handler(
    State(_app_state): State<crate::web::auth::AppState>,
    Path(_subscription_id): Path<Uuid>,
    Json(_payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, Json<UnifiedErrorResponse>> {
    Ok(Json(serde_json::json!({
        "success": false,
        "message": "Auto-renewal removed. Please make a new payment to extend your plan.",
        "payment_url": "/pricing"
    })))
}

/// Legacy: Check subscription status
pub async fn check_subscription_status_handler(
    state: State<crate::web::auth::AppState>,
    ext: Extension<OpenIDUserContext>,
    Path(_plan_id): Path<Uuid>,
) -> Result<Json<PlanExpiryResponse>, Json<UnifiedErrorResponse>> {
    get_plan_expiry_status_handler(state, ext).await
}