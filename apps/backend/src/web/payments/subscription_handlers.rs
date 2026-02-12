//! Subscription Management API Handlers
//!
//! Direct Payment model: users pay to activate/extend their plan duration
//! No auto-renewal - users must pay again when their plan expires
//! Uses wallet_plan_assignments table for all plan access data

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
    web::middleware::{UnifiedErrorResponse, OpenIDUserContext},
};

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

/// Extract ranking offset from plan metadata
/// Returns: 0 for top ranks access, higher values for lower tier access
fn extract_ranking_offset(metadata: &serde_json::Value) -> i32 {
    // Check for ranking_offset in metadata root or features
    if let Some(offset) = metadata.get("ranking_offset").and_then(|v| v.as_i64()) {
        return offset as i32;
    }
    if let Some(features) = metadata.get("features").and_then(|f| f.as_object()) {
        if let Some(offset) = features.get("ranking_offset").and_then(|v| v.as_i64()) {
            return offset as i32;
        }
    }
    // Check permissions for offset pattern
    if let Some(permissions) = metadata.get("permissions").and_then(|p| p.as_array()) {
        for perm in permissions {
            if let Some(perm_str) = perm.as_str() {
                if let Some(offset_str) = perm_str.strip_prefix("epsx:rankings:offset:") {
                    if let Ok(offset) = offset_str.parse::<i32>() {
                        return offset;
                    }
                }
                // Wildcard = full access (offset 0)
                if perm_str == "epsx:*:*" || perm_str == "epsx:rankings:*" {
                    return 0;
                }
            }
        }
    }
    // Default: free tier sees ranks 101+ (offset 100)
    crate::core::constants::FREE_PLAN_RANKING_OFFSET
}

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
    pub plan_name: Option<String>,
    pub plan_expires_at: Option<DateTime<Utc>>,
    pub days_remaining: i64,
    pub status: String, // "active", "expiring_soon", "expired", "no_plan"
    pub ranking_offset: i32, // Starting rank position (0 = top ranks, 100 = ranks 101+)
    pub can_upgrade: bool,
    pub tier_level: i32, // Plan tier level for upgrade/downgrade logic
    pub all_plans: Vec<PlanSummary>,
}

/// Summary of an active subscription
#[derive(Debug, Clone, Serialize)]
pub struct PlanSummary {
    pub plan_name: String,
    pub expires_at: Option<DateTime<Utc>>,
    pub tier_level: i32,
    pub is_effective: bool,
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
/// Get user's plan access status
pub async fn get_user_plans_handler(
    State(app_state): State<crate::web::auth::AppState>,
    Extension(user_context): Extension<OpenIDUserContext>,
) -> Result<Json<UserPlansResponse>, Json<UnifiedErrorResponse>> {
    info!("Getting plan access for user: {}", user_context.wallet_address);

    let mut conn = app_state.db_pool
        .get()
        .await
        .map_err(|e| UnifiedErrorResponse::json(500, "Database connection failed", e.to_string()))?;

    let now = Utc::now();

    // 1. Primary Check: Look for active subscription in wallet_plan_assignments
    // This is the source of truth for all new payments/plans
    #[derive(diesel::QueryableByName)]
    struct ActiveSubscriptionRow {
        #[diesel(sql_type = diesel::sql_types::Text)]
        name: String,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>)]
        expires_at: Option<DateTime<Utc>>,
        #[diesel(sql_type = diesel::sql_types::Jsonb)]
        plan_metadata: serde_json::Value,
        #[diesel(sql_type = diesel::sql_types::Integer)]
        tier_level: i32,
        #[diesel(sql_type = diesel::sql_types::Integer)]
        grace_period_hours: i32,
    }

    // Re-run safely with load (includes grace period window)
    let active_subs: Vec<ActiveSubscriptionRow> = diesel::sql_query(
        r#"
        SELECT g.name, wga.expires_at, g.plan_metadata, g.tier_level, g.grace_period_hours
        FROM wallet_plan_assignments wga
        JOIN plans g ON g.id = wga.plan_id
        WHERE LOWER(wga.wallet_address) = LOWER($1)
          AND wga.is_active = true
          AND (wga.expires_at IS NULL
               OR wga.expires_at > NOW()
               OR (wga.expires_at + (g.grace_period_hours || ' hours')::INTERVAL) > NOW())
          AND (g.plan_type = 'subscription' OR g.plan_type = 'enterprise' OR g.plan_type = 'api-developer')
        ORDER BY g.tier_level DESC, wga.assigned_at DESC
        "#
    )
    .bind::<diesel::sql_types::Text, _>(&user_context.wallet_address)
    .load(&mut conn)
    .await
    .unwrap_or_default();


    if let Some(sub) = active_subs.as_slice().first() {
        // Effective plan is the one with highest tier_level (first in list due to sort)
        let days_remaining = sub.expires_at
            .map(|exp| if exp > now { (exp - now).num_days() } else { 0 })
            .unwrap_or(3650); // Effectively unlimited if NULL (permanent)

        // Determine status: active, expiring_soon, or grace_period
        let is_past_expiry = sub.expires_at.map(|exp| exp <= now).unwrap_or(false);
        let status = if is_past_expiry && sub.grace_period_hours > 0 {
            "grace_period"
        } else if days_remaining <= 7 && sub.expires_at.is_some() {
            "expiring_soon"
        } else {
            "active"
        };
        
        // Extract ranking offset from metadata
        let ranking_offset = extract_ranking_offset(&sub.plan_metadata);
        
        // Check if user can upgrade (ranking_offset > 0 means not full access)
        let can_upgrade = ranking_offset > 0;

        // Build summary list
        let all_plans = active_subs.iter().map(|s| PlanSummary {
            plan_name: s.name.clone(),
            expires_at: s.expires_at,
            tier_level: s.tier_level,
            is_effective: s.tier_level == sub.tier_level, // Simple check, assuming unique tiers or first is best
        }).collect();

        return Ok(Json(UserPlansResponse {
            success: true,
            message: "User plan access retrieved successfully".to_string(),
            data: Some(PlanAccessData {
                wallet_address: user_context.wallet_address.clone(),
                plan_name: Some(sub.name.clone()),
                plan_expires_at: sub.expires_at,
                days_remaining: days_remaining.max(0),
                status: status.to_string(),
                ranking_offset,
                can_upgrade,
                tier_level: sub.tier_level,
                all_plans,
            }),
        }));
    }

    // No active plan assignment found
    let data = PlanAccessData {
        wallet_address: user_context.wallet_address.clone(),
        plan_name: None,
        plan_expires_at: None,
        days_remaining: 0,
        status: "no_plan".to_string(),
        ranking_offset: crate::core::constants::FREE_PLAN_RANKING_OFFSET,
        can_upgrade: true,
        tier_level: 0,
        all_plans: vec![],
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

/// Cancel plan (deactivate assignment, keep expiry so user can use until it expires)
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
        .map_err(|e| UnifiedErrorResponse::json(500, "Database connection failed", e.to_string()))?;

    // Deactivate wallet_plan_assignments for this wallet and plan
    let rows_affected = diesel::sql_query(
        r#"
        UPDATE wallet_plan_assignments
        SET is_active = false, updated_at = NOW()
        WHERE LOWER(wallet_address) = LOWER($1)
          AND plan_id = $2
          AND is_active = true
        "#
    )
    .bind::<diesel::sql_types::Text, _>(&user_context.wallet_address)
    .bind::<diesel::sql_types::Uuid, _>(plan_id)
    .execute(&mut conn)
    .await
    .map_err(|e| UnifiedErrorResponse::json(500, "Failed to cancel plan", e.to_string()))?;

    if rows_affected == 0 {
        return Ok(Json(CancelPlanResponse {
            success: false,
            message: "No active plan found for this wallet".to_string(),
        }));
    }

    Ok(Json(CancelPlanResponse {
        success: true,
        message: "Plan cancelled successfully.".to_string(),
    }))
}

// ============================================================================
// UPGRADE PREVIEW (With Credit Calculation)
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
    pub credit_from_current_plan: String,  // Pro-rata credit from remaining time
    pub amount_to_pay: String,             // new_price - credit (what user pays)
    pub new_duration_days: i64,
    pub new_expiry_date: DateTime<Utc>,
    pub is_upgrade_allowed: bool,          // false if attempting downgrade
}

/// Current plan info
#[derive(Debug, Serialize)]
pub struct CurrentPlanInfo {
    pub id: Option<i32>,
    pub name: String,
    pub price: String,                     // Original price paid
    pub expires_at: Option<DateTime<Utc>>,
    pub started_at: Option<DateTime<Utc>>, // When plan was activated
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
        .map_err(|e| UnifiedErrorResponse::json(500, "Database connection failed", e.to_string()))?;

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
    .map_err(|e| UnifiedErrorResponse::json(500, "Failed to fetch plan", e.to_string()))?;

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

    let now = Utc::now();
    let standard_duration_days: i64 = 30;

    // Get current plan info with assignment details
    #[derive(diesel::QueryableByName)]
    #[allow(dead_code)]
    struct AssignmentRow {
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        plan_id: uuid::Uuid,
        #[diesel(sql_type = diesel::sql_types::Timestamptz)]
        assigned_at: DateTime<Utc>,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>)]
        expires_at: Option<DateTime<Utc>>,
    }

    // Get active plan assignment for this wallet
    let assignment: Option<AssignmentRow> = diesel::sql_query(
        r#"
        SELECT plan_id, assigned_at, expires_at
        FROM wallet_plan_assignments
        WHERE LOWER(wallet_address) = LOWER($1)
          AND is_active = true
          AND (expires_at IS NULL OR expires_at > NOW())
        ORDER BY assigned_at DESC
        LIMIT 1
        "#
    )
    .bind::<diesel::sql_types::Text, _>(&user_context.wallet_address)
    .get_result(&mut conn)
    .await
    .optional()
    .ok()
    .flatten();

    // Get current plan details if user has an assignment
    let (current_plan_info, current_plan_price) = if let Some(ref a) = assignment {
        #[derive(diesel::QueryableByName)]
        #[allow(dead_code)]
        struct PlanRow {
            #[diesel(sql_type = diesel::sql_types::Uuid)]
            id: uuid::Uuid,
            #[diesel(sql_type = diesel::sql_types::Text)]
            name: String,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Numeric>)]
            price: Option<bigdecimal::BigDecimal>,
        }

        let plan: Option<PlanRow> = diesel::sql_query(
            "SELECT id, name, price FROM plans WHERE id = $1"
        )
        .bind::<diesel::sql_types::Uuid, _>(a.plan_id)
        .get_result(&mut conn)
        .await
        .optional()
        .ok()
        .flatten();

        if let Some(g) = plan {
            let price: rust_decimal::Decimal = g.price.as_ref()
                .and_then(|bd| rust_decimal::Decimal::from_str(&bd.to_string()).ok())
                .unwrap_or(rust_decimal::Decimal::ZERO);

            let days_remaining = a.expires_at
                .map(|exp| if exp > now { (exp - now).num_days() } else { 0 })
                .unwrap_or(0);

            (Some(CurrentPlanInfo {
                id: None, // Plans use UUID, not i32
                name: g.name,
                price: price.to_string(),
                expires_at: a.expires_at,
                started_at: Some(a.assigned_at),
                days_remaining,
            }), price)
        } else {
            (None, rust_decimal::Decimal::ZERO)
        }
    } else {
        (None, rust_decimal::Decimal::ZERO)
    };

    // Calculate credit using upgrade_service
    use super::upgrade_service::{calculate_upgrade_credit, is_upgrade_allowed, calculate_amount_to_pay};

    let credit = if let Some(ref a) = assignment {
        if let Some(expires_at) = a.expires_at {
            calculate_upgrade_credit(current_plan_price, a.assigned_at, expires_at)
        } else {
            rust_decimal::Decimal::ZERO
        }
    } else {
        rust_decimal::Decimal::ZERO
    };

    let is_upgrade = current_plan_info.is_none() || is_upgrade_allowed(current_plan_price, new_plan_price);
    let amount_to_pay = calculate_amount_to_pay(new_plan_price, credit);

    // Calculate new expiry
    let new_expiry = now + chrono::Duration::days(standard_duration_days);

    // Build response
    let response_data = UpgradePreviewData {
        current_plan: current_plan_info,
        new_plan: NewPlanInfo {
            id: new_plan.id,
            name: new_plan.name.clone(),
            price: new_plan_price.to_string(),
        },
        credit_from_current_plan: credit.to_string(),
        amount_to_pay: amount_to_pay.to_string(),
        new_duration_days: standard_duration_days,
        new_expiry_date: new_expiry,
        is_upgrade_allowed: is_upgrade,
    };

    let message = if !is_upgrade {
        // Downgrade / Sidegrade logic
        format!("Plan Switch: {} (Activates after your current plan expires)", new_plan.name)
    } else if credit > rust_decimal::Decimal::ZERO {
        format!("Upgrade with ${} credit from your current plan. Amount to pay: ${}", credit, amount_to_pay)
    } else {
        format!("New subscription - {} days access", standard_duration_days)
    };

    Ok(Json(UpgradePreviewResponse {
        success: true, // ALWAYS ALLOW switch (upgrade or downgrade)
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
        "message": "Use /api/plans/expiry-status instead"
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