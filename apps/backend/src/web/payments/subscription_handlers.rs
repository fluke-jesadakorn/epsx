//! Subscription Management API Handlers
//!
//! Pay-to-extend model: users pay to activate/extend their plan duration
//! No auto-renewal - users must pay again when their plan expires

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
    pub data: Option<Vec<PlanData>>,
}

/// Get plan details response
#[derive(Debug, Serialize)]
pub struct PlanDetailsResponse {
    pub success: bool,
    pub message: String,
    pub data: Option<PlanData>,
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

/// Plan expiry status response
#[derive(Debug, Serialize)]
pub struct PlanExpiryResponse {
    pub success: bool,
    pub message: String,
    pub data: Option<PlanExpiryData>,
}

/// Plan data (simplified for pay-to-extend model)
#[derive(Debug, Serialize)]
pub struct PlanData {
    pub id: Uuid,
    pub plan_name: String,
    pub status: String, // "active", "expired", "cancelled"
    pub started_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub days_remaining: Option<i64>,
    pub wallet_address: String,
}

/// Plan expiry status data
#[derive(Debug, Serialize)]
pub struct PlanExpiryData {
    pub plan_id: String,
    pub plan_name: String,
    pub expires_at: Option<DateTime<Utc>>,
    pub days_remaining: i64,
    pub is_expired: bool,
    pub status: String, // "active", "expiring_soon", "expired", "no_plan"
}

// ============================================================================
// HANDLERS
// ============================================================================

/// Get user's active plans
pub async fn get_user_plans_handler(
    State(app_state): State<crate::web::auth::AppState>,
    Extension(user_context): Extension<OpenIDUserContext>,
) -> Result<Json<UserPlansResponse>, Json<UnifiedErrorResponse>> {
    info!("Getting plans for user: {}", user_context.wallet_address);

    // Query active_subscriptions for this wallet
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
    struct SubRow {
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        id: Uuid,
        #[diesel(sql_type = diesel::sql_types::Text)]
        tier: String,
        #[diesel(sql_type = diesel::sql_types::Text)]
        status: String,
        #[diesel(sql_type = diesel::sql_types::Timestamptz)]
        start_date: DateTime<Utc>,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>)]
        next_payment_date: Option<DateTime<Utc>>,
    }

    let plans: Vec<SubRow> = diesel::sql_query(
        r#"
        SELECT id, tier, status, start_date, next_payment_date
        FROM active_subscriptions
        WHERE LOWER(wallet_address) = LOWER($1)
        ORDER BY created_at DESC
        "#
    )
    .bind::<diesel::sql_types::Text, _>(&user_context.wallet_address)
    .get_results(&mut conn)
    .await
    .map_err(|e| Json(UnifiedErrorResponse {
        success: false,
        error: ErrorDetails {
            code: 500,
            message: "Failed to fetch plans".to_string(),
            reason: e.to_string(),
        },
    }))?;

    let now = Utc::now();
    let plan_data: Vec<PlanData> = plans.into_iter().map(|p| {
        let days_remaining = p.next_payment_date.map(|exp| (exp - now).num_days());
        PlanData {
            id: p.id,
            plan_name: p.tier.clone(),
            status: p.status,
            started_at: p.start_date,
            expires_at: p.next_payment_date,
            days_remaining,
            wallet_address: user_context.wallet_address.clone(),
        }
    }).collect();

    Ok(Json(UserPlansResponse {
        success: true,
        message: "User plans retrieved successfully".to_string(),
        data: Some(plan_data),
    }))
}

/// Get plan expiry status (for renewal prompts)
pub async fn get_plan_expiry_status_handler(
    State(app_state): State<crate::web::auth::AppState>,
    Extension(user_context): Extension<OpenIDUserContext>,
) -> Result<Json<PlanExpiryResponse>, Json<UnifiedErrorResponse>> {
    info!("Checking plan expiry for user: {}", user_context.wallet_address);

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
    struct SubRow {
        #[diesel(sql_type = diesel::sql_types::Text)]
        tier: String,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>)]
        next_payment_date: Option<DateTime<Utc>>,
    }

    let plan: Option<SubRow> = diesel::sql_query(
        r#"
        SELECT tier, next_payment_date
        FROM active_subscriptions
        WHERE LOWER(wallet_address) = LOWER($1) AND status = 'active'
        ORDER BY created_at DESC
        LIMIT 1
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
            message: "Failed to fetch plan".to_string(),
            reason: e.to_string(),
        },
    }))?;

    let data = match plan {
        Some(p) => {
            let now = Utc::now();
            let days_remaining = p.next_payment_date.map(|exp| (exp - now).num_days()).unwrap_or(0);
            let is_expired = days_remaining < 0;
            let status = if is_expired {
                "expired"
            } else if days_remaining <= 7 {
                "expiring_soon"
            } else {
                "active"
            };

            PlanExpiryData {
                plan_id: "current".to_string(),
                plan_name: p.tier,
                expires_at: p.next_payment_date,
                days_remaining,
                is_expired,
                status: status.to_string(),
            }
        },
        None => PlanExpiryData {
            plan_id: "".to_string(),
            plan_name: "No active plan".to_string(),
            expires_at: None,
            days_remaining: 0,
            is_expired: true,
            status: "no_plan".to_string(),
        },
    };

    Ok(Json(PlanExpiryResponse {
        success: true,
        message: "Plan expiry status retrieved".to_string(),
        data: Some(data),
    }))
}

/// Cancel plan (user-initiated, plan continues until expiry)
pub async fn cancel_plan_handler(
    State(app_state): State<crate::web::auth::AppState>,
    Extension(user_context): Extension<OpenIDUserContext>,
    Path(plan_id): Path<Uuid>,
    Json(_payload): Json<CancelPlanRequest>,
) -> Result<Json<CancelPlanResponse>, Json<UnifiedErrorResponse>> {
    info!("Cancelling plan {} for user {}", plan_id, user_context.wallet_address);

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

    // Update status to cancelled (user can still use until expiry)
    let rows_affected = diesel::sql_query(
        r#"
        UPDATE active_subscriptions
        SET status = 'cancelled', updated_at = $1
        WHERE id = $2 AND LOWER(wallet_address) = LOWER($3)
        "#
    )
    .bind::<diesel::sql_types::Timestamptz, _>(Utc::now())
    .bind::<diesel::sql_types::Uuid, _>(plan_id)
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
            message: "Plan not found or not owned by this wallet".to_string(),
        }));
    }

    Ok(Json(CancelPlanResponse {
        success: true,
        message: "Plan cancelled. You can continue using it until expiry.".to_string(),
    }))
}

// ============================================================================
// UPGRADE PREVIEW
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
    pub upgrade_details: UpgradeDetails,
    pub is_upgrade: bool,
    pub is_downgrade: bool,
    pub payment_required: String,
}

/// Current plan info
#[derive(Debug, Serialize)]
pub struct CurrentPlanInfo {
    pub name: String,
    pub price: String,
    pub expires_at: Option<DateTime<Utc>>,
    pub days_remaining: i64,
}

/// New plan info
#[derive(Debug, Serialize)]
pub struct NewPlanInfo {
    pub id: i32,
    pub name: String,
    pub price: String,
    pub standard_duration_days: i64,
}

/// Upgrade calculation details
#[derive(Debug, Serialize)]
pub struct UpgradeDetails {
    pub remaining_credit: String,
    pub bonus_days: i64,
    pub total_duration_days: i64,
    pub new_expiry_date: DateTime<Utc>,
}

/// Query params for upgrade preview
#[derive(Debug, Deserialize)]
pub struct UpgradePreviewQuery {
    pub new_plan_id: i32,
}

/// Get upgrade preview - shows credit calculation before payment
pub async fn get_upgrade_preview_handler(
    State(app_state): State<crate::web::auth::AppState>,
    Extension(user_context): Extension<OpenIDUserContext>,
    axum::extract::Query(query): axum::extract::Query<UpgradePreviewQuery>,
) -> Result<Json<UpgradePreviewResponse>, Json<UnifiedErrorResponse>> {
    use rust_decimal::Decimal;
    use std::str::FromStr;
    use crate::domain::subscription_management::domain_services::UpgradeCalculator;
    
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
            message: "Failed to fetch new plan".to_string(),
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

    let new_plan_price: Decimal = new_plan.price.as_ref()
        .and_then(|bd| Decimal::from_str(&bd.to_string()).ok())
        .unwrap_or(Decimal::ZERO);

    // Get current subscription
    #[derive(diesel::QueryableByName)]
    struct SubRow {
        #[diesel(sql_type = diesel::sql_types::Text)]
        tier: String,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>)]
        next_payment_date: Option<DateTime<Utc>>,
    }

    let current_sub: Option<SubRow> = diesel::sql_query(
        r#"
        SELECT tier, next_payment_date
        FROM active_subscriptions
        WHERE LOWER(wallet_address) = LOWER($1) AND status = 'active'
        ORDER BY created_at DESC
        LIMIT 1
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
            message: "Failed to fetch current subscription".to_string(),
            reason: e.to_string(),
        },
    }))?;

    let now = Utc::now();
    let standard_duration_days: i64 = 30;

    let (current_plan_info, old_plan_price, days_remaining) = if let Some(ref sub) = current_sub {
        // Get old plan price
        let old_plan: Option<PlanRow> = diesel::sql_query(
            "SELECT id, name, price FROM pricing_plans WHERE name = $1 LIMIT 1"
        )
        .bind::<diesel::sql_types::Text, _>(&sub.tier)
        .get_result(&mut conn)
        .await
        .optional()
        .map_err(|e| Json(UnifiedErrorResponse {
            success: false,
            error: ErrorDetails {
                code: 500,
                message: "Failed to fetch current plan details".to_string(),
                reason: e.to_string(),
            },
        }))?;

        let old_price: Decimal = old_plan.as_ref()
            .and_then(|p| p.price.as_ref())
            .and_then(|bd| Decimal::from_str(&bd.to_string()).ok())
            .unwrap_or(Decimal::ZERO);

        let days = sub.next_payment_date
            .map(|exp| if exp > now { (exp - now).num_days() } else { 0 })
            .unwrap_or(0);

        (
            Some(CurrentPlanInfo {
                name: sub.tier.clone(),
                price: old_price.to_string(),
                expires_at: sub.next_payment_date,
                days_remaining: days,
            }),
            old_price,
            days,
        )
    } else {
        (None, Decimal::ZERO, 0)
    };

    // Calculate upgrade details
    let calc = UpgradeCalculator::calculate(old_plan_price, new_plan_price, days_remaining);

    let new_expiry = now + chrono::Duration::days(calc.total_new_duration_days);

    let upgrade_details = UpgradeDetails {
        remaining_credit: calc.remaining_credit.to_string(),
        bonus_days: calc.bonus_days,
        total_duration_days: calc.total_new_duration_days,
        new_expiry_date: new_expiry,
    };

    let is_same_plan = current_sub.as_ref()
        .map(|s| s.tier == new_plan.name)
        .unwrap_or(false);

    let response_data = UpgradePreviewData {
        current_plan: current_plan_info,
        new_plan: NewPlanInfo {
            id: new_plan.id,
            name: new_plan.name,
            price: new_plan_price.to_string(),
            standard_duration_days,
        },
        upgrade_details,
        is_upgrade: calc.is_valid_upgrade,
        is_downgrade: UpgradeCalculator::is_downgrade(old_plan_price, new_plan_price) && !is_same_plan,
        payment_required: new_plan_price.to_string(),
    };

    let message = if calc.is_valid_upgrade {
        format!("Upgrade gives you {} bonus days from remaining credit!", calc.bonus_days)
    } else if is_same_plan {
        "Extending your current plan by 30 days".to_string()
    } else if current_sub.is_none() {
        "New subscription - 30 days access".to_string()
    } else {
        "Downgrade not allowed. Payment will extend your current plan.".to_string()
    };

    Ok(Json(UpgradePreviewResponse {
        success: true,
        message,
        data: Some(response_data),
    }))
}

// ============================================================================
// LEGACY HANDLERS (for backward compatibility, redirect to new endpoints)
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
    Path(subscription_id): Path<Uuid>,
) -> Result<Json<PlanDetailsResponse>, Json<UnifiedErrorResponse>> {
    info!("Legacy: Getting subscription details for {}", subscription_id);
    
    Ok(Json(PlanDetailsResponse {
        success: false,
        message: "Use /api/v1/plans/expiry-status instead".to_string(),
        data: None,
    }))
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

/// Legacy: Check subscription status (maps to get_plan_expiry_status_handler)
pub async fn check_subscription_status_handler(
    state: State<crate::web::auth::AppState>,
    ext: Extension<OpenIDUserContext>,
    Path(_plan_id): Path<Uuid>,
) -> Result<Json<PlanExpiryResponse>, Json<UnifiedErrorResponse>> {
    get_plan_expiry_status_handler(state, ext).await
}