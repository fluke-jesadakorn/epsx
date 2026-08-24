//! Subscription Management API Handlers
//!
//! Direct Payment model: users pay to activate/extend their plan duration
//! No auto-renewal - users must pay again when their plan expires
//! Uses wallet_plan_assignments table for all plan access data
//!
//! BIG-BANG: migrated to sqlx (real). All diesel DSL replaced with raw SQL.

use axum::{
    extract::{Extension, Path, State},
    response::Json,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tracing::info;
use uuid::Uuid;

use crate::{
    prelude::*,
    web::middleware::{OpenIDUserContext, UnifiedErrorResponse},
};

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

/// Extract ranking offset from plan metadata or permission string
fn extract_ranking_offset(metadata: &serde_json::Value, offset_permission: Option<&str>) -> i32 {
    if let Some(offset) = metadata.get("ranking_offset").and_then(|v| v.as_i64()) {
        return offset as i32;
    }
    if let Some(perm) = offset_permission {
        if let Some(offset_str) = perm.strip_prefix("epsx:rankings:offset:") {
            if let Ok(offset) = offset_str.parse::<i32>() {
                return offset;
            }
        }
    }
    if let Some(features) = metadata.get("features").and_then(|f| f.as_object()) {
        if let Some(offset) = features.get("ranking_offset").and_then(|v| v.as_i64()) {
            return offset as i32;
        }
    }
    epsx_contracts::constants::FREE_PLAN_RANKING_OFFSET
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
    pub plan_id: Option<String>,
    pub plan_expires_at: Option<DateTime<Utc>>,
    pub days_remaining: i64,
    pub status: String,
    pub ranking_offset: i32,
    pub can_upgrade: bool,
    pub tier_level: i32,
    pub all_plans: Vec<PlanSummary>,
    pub proration_credit: Option<String>,
    pub current_plan_price: Option<String>,
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
pub async fn get_user_plans_handler(
    State(app_state): State<crate::web::auth::AppState>,
    Extension(user_context): Extension<OpenIDUserContext>,
) -> Result<Json<UserPlansResponse>, Json<UnifiedErrorResponse>> {
    info!(
        "Getting plan access for user: {}",
        user_context.wallet_address
    );

    let now = Utc::now();

    #[derive(sqlx::FromRow)]
    struct ActiveSubscriptionRow {
        plan_id: Uuid,
        name: String,
        expires_at: Option<DateTime<Utc>>,
        plan_metadata: serde_json::Value,
        tier_level: i32,
        grace_period_hours: i32,
        offset_permission: Option<String>,
        assigned_at: DateTime<Utc>,
        price: Option<bigdecimal::BigDecimal>,
        billing_cycle: Option<String>,
    }

    let active_subs: Vec<ActiveSubscriptionRow> = sqlx::query_as(
        r#"
        SELECT wga.plan_id, g.name, wga.expires_at, g.plan_metadata, g.tier_level, g.grace_period_hours,
               wga.assigned_at, g.price, g.billing_cycle,
               (SELECT p.permission_string FROM plan_permissions pp
                 JOIN permissions p ON pp.permission_id = p.id
                 WHERE pp.plan_id = g.id AND p.permission_string LIKE 'epsx:rankings:offset:%'
                 LIMIT 1) as offset_permission
        FROM wallet_plan_assignments wga
        JOIN plans g ON g.id = wga.plan_id
        WHERE LOWER(wga.wallet_address) = LOWER($1)
          AND wga.is_active = true
          AND (wga.expires_at IS NULL
               OR wga.expires_at > NOW()
               OR (wga.expires_at + (g.grace_period_hours || ' hours')::INTERVAL) > NOW())
          AND (g.plan_type = 'subscription' OR g.plan_type = 'enterprise' OR g.plan_type = 'api-developer' OR g.plan_type = 'manual' OR g.plan_type = 'system')
        ORDER BY wga.assigned_at DESC
        "#,
    )
    .bind(&user_context.wallet_address)
    .fetch_all(app_state.db_pool.as_ref())
    .await
    .unwrap_or_default();

    if !active_subs.is_empty() {
        let mut subs_with_offset: Vec<(i32, usize)> = active_subs
            .iter()
            .enumerate()
            .map(|(i, s)| {
                (
                    extract_ranking_offset(&s.plan_metadata, s.offset_permission.as_deref()),
                    i,
                )
            })
            .collect();
        subs_with_offset.sort_by(|(offset_a, idx_a), (offset_b, idx_b)| {
            offset_a.cmp(offset_b).then(
                active_subs[*idx_b]
                    .tier_level
                    .cmp(&active_subs[*idx_a].tier_level),
            )
        });

        let &(ranking_offset, best_idx) = subs_with_offset.as_slice().first().expect("non-empty");
        let sub = &active_subs[best_idx];

        let proration_credit = {
            use super::upgrade_service::{billing_period_days, calculate_upgrade_credit};
            use std::str::FromStr;
            if let (Some(exp), Some(price_bd)) = (sub.expires_at, sub.price.as_ref()) {
                let price =
                    rust_decimal::Decimal::from_str(&price_bd.to_string()).unwrap_or_default();
                if price > rust_decimal::Decimal::ZERO && exp > now {
                    let period = billing_period_days(sub.billing_cycle.as_deref());
                    Some(calculate_upgrade_credit(price, exp, period).to_string())
                } else {
                    None
                }
            } else {
                None
            }
        };

        let current_plan_price = sub.price.as_ref().map(|p| p.to_string());

        let days_remaining = sub
            .expires_at
            .map(|exp| if exp > now { (exp - now).num_days() } else { 0 })
            .unwrap_or(3650);

        let is_past_expiry = sub.expires_at.map(|exp| exp <= now).unwrap_or(false);
        let status = if is_past_expiry && sub.grace_period_hours > 0 {
            "grace_period"
        } else if days_remaining <= 7 && sub.expires_at.is_some() {
            "expiring_soon"
        } else {
            "active"
        };

        let can_upgrade = ranking_offset > 1;

        let all_plans = active_subs
            .iter()
            .enumerate()
            .map(|(i, s)| PlanSummary {
                plan_name: s.name.clone(),
                expires_at: s.expires_at,
                tier_level: s.tier_level,
                is_effective: i == best_idx,
            })
            .collect();

        return Ok(Json(UserPlansResponse {
            success: true,
            message: "User plan access retrieved successfully".to_string(),
            data: Some(PlanAccessData {
                wallet_address: user_context.wallet_address.clone(),
                plan_name: Some(sub.name.clone()),
                plan_id: Some(sub.plan_id.to_string()),
                plan_expires_at: sub.expires_at,
                days_remaining: days_remaining.max(0),
                status: status.to_string(),
                ranking_offset,
                can_upgrade,
                tier_level: sub.tier_level,
                all_plans,
                proration_credit,
                current_plan_price,
            }),
        }));
    }

    let data = PlanAccessData {
        wallet_address: user_context.wallet_address.clone(),
        plan_name: None,
        plan_id: None,
        plan_expires_at: None,
        days_remaining: 0,
        status: "no_plan".to_string(),
        ranking_offset: epsx_contracts::constants::FREE_PLAN_RANKING_OFFSET,
        can_upgrade: true,
        tier_level: 0,
        all_plans: vec![],
        proration_credit: None,
        current_plan_price: None,
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
    info!(
        "Checking plan expiry for user: {}",
        user_context.wallet_address
    );

    let result = get_user_plans_handler(State(app_state), Extension(user_context)).await?;

    Ok(Json(PlanExpiryResponse {
        success: result.success,
        message: "Plan expiry status retrieved".to_string(),
        data: result.data.clone(),
    }))
}

/// Cancel plan
pub async fn cancel_plan_handler(
    State(app_state): State<crate::web::auth::AppState>,
    Extension(user_context): Extension<OpenIDUserContext>,
    Path(plan_id): Path<Uuid>,
    Json(_payload): Json<CancelPlanRequest>,
) -> Result<Json<CancelPlanResponse>, Json<UnifiedErrorResponse>> {
    info!(
        "Cancelling plan {} for user {}",
        plan_id, user_context.wallet_address
    );

    let result = sqlx::query(
        r#"
        UPDATE wallet_plan_assignments
        SET is_active = false, updated_at = NOW()
        WHERE LOWER(wallet_address) = LOWER($1)
          AND plan_id = $2
          AND is_active = true
        "#,
    )
    .bind(&user_context.wallet_address)
    .bind(plan_id)
    .execute(app_state.db_pool.as_ref())
    .await
    .map_err(|e| UnifiedErrorResponse::json(500, "Failed to cancel plan", e.to_string()))?;

    if result.rows_affected() == 0 {
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
    pub credit_from_current_plan: String,
    pub wallet_credit_balance: String,
    pub total_credits_available: String,
    pub amount_to_pay: String,
    pub new_duration_days: i64,
    pub new_expiry_date: DateTime<Utc>,
    pub is_upgrade_allowed: bool,
}

/// Current plan info
#[derive(Debug, Serialize)]
pub struct CurrentPlanInfo {
    pub id: Option<i32>,
    pub name: String,
    pub price: String,
    pub expires_at: Option<DateTime<Utc>>,
    pub started_at: Option<DateTime<Utc>>,
    pub days_remaining: i64,
}

/// New plan info
#[derive(Debug, Serialize)]
pub struct NewPlanInfo {
    pub id: String,
    pub name: String,
    pub price: String,
}

/// Query params for upgrade preview
#[derive(Debug, Deserialize)]
pub struct UpgradePreviewQuery {
    pub new_plan_id: String,
}

/// Get upgrade preview
pub async fn get_upgrade_preview_handler(
    State(app_state): State<crate::web::auth::AppState>,
    Extension(user_context): Extension<OpenIDUserContext>,
    axum::extract::Query(query): axum::extract::Query<UpgradePreviewQuery>,
) -> Result<Json<UpgradePreviewResponse>, Json<UnifiedErrorResponse>> {
    use std::str::FromStr;

    info!(
        "Getting upgrade preview for user: {}, new_plan_id: {}",
        user_context.wallet_address, query.new_plan_id
    );

    let new_plan_uuid = Uuid::parse_str(&query.new_plan_id).map_err(|_| {
        UnifiedErrorResponse::json(400, "Invalid plan ID", "Plan ID must be a valid UUID")
    })?;

    #[derive(sqlx::FromRow)]
    struct PlanRow {
        id: Uuid,
        name: String,
        price: Option<bigdecimal::BigDecimal>,
        tier_level: i32,
        billing_cycle: Option<String>,
        plan_metadata: serde_json::Value,
    }

    let new_plan: Option<PlanRow> = sqlx::query_as(
        "SELECT id, name, price, tier_level, billing_cycle, COALESCE(plan_metadata, '{}'::jsonb) as plan_metadata FROM plans WHERE id = $1",
    )
    .bind(new_plan_uuid)
    .fetch_optional(app_state.db_pool.as_ref())
    .await
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

    let new_plan_base_price: rust_decimal::Decimal = new_plan
        .price
        .as_ref()
        .and_then(|bd| rust_decimal::Decimal::from_str(&bd.to_string()).ok())
        .unwrap_or(rust_decimal::Decimal::ZERO);

    let new_plan_price = new_plan
        .plan_metadata
        .get("promotion")
        .and_then(|promo_val| {
            serde_json::from_value::<crate::domain::subscription_management::promotion::Promotion>(
                promo_val.clone(),
            )
            .ok()
        })
        .map(|promo| {
            let bp = new_plan_base_price
                .to_string()
                .parse::<f64>()
                .unwrap_or(0.0);
            let ep = promo.calculate_effective_price(bp);
            rust_decimal::Decimal::from_str(&format!("{:.2}", ep)).unwrap_or(new_plan_base_price)
        })
        .unwrap_or(new_plan_base_price);

    let now = Utc::now();
    let standard_duration_days: i64 = 30;

    #[derive(sqlx::FromRow)]
    struct AssignmentRow {
        plan_id: uuid::Uuid,
        assigned_at: DateTime<Utc>,
        expires_at: Option<DateTime<Utc>>,
    }

    let assignment: Option<AssignmentRow> = sqlx::query_as(
        r#"
        SELECT plan_id, assigned_at, expires_at
        FROM wallet_plan_assignments
        WHERE LOWER(wallet_address) = LOWER($1)
          AND is_active = true
          AND (expires_at IS NULL OR expires_at > NOW())
        ORDER BY assigned_at DESC
        LIMIT 1
        "#,
    )
    .bind(&user_context.wallet_address)
    .fetch_optional(app_state.db_pool.as_ref())
    .await
    .ok()
    .flatten();

    let (current_plan_info, current_plan_price, current_billing_period) = if let Some(ref a) =
        assignment
    {
        #[derive(sqlx::FromRow)]
        struct CurPlanRow {
            id: uuid::Uuid,
            name: String,
            price: Option<bigdecimal::BigDecimal>,
            billing_cycle: Option<String>,
        }

        let plan: Option<CurPlanRow> =
            sqlx::query_as("SELECT id, name, price, billing_cycle FROM plans WHERE id = $1")
                .bind(a.plan_id)
                .fetch_optional(app_state.db_pool.as_ref())
                .await
                .ok()
                .flatten();

        if let Some(g) = plan {
            let price: rust_decimal::Decimal = g
                .price
                .as_ref()
                .and_then(|bd| rust_decimal::Decimal::from_str(&bd.to_string()).ok())
                .unwrap_or(rust_decimal::Decimal::ZERO);

            let period = super::upgrade_service::billing_period_days(g.billing_cycle.as_deref());

            let days_remaining = a
                .expires_at
                .map(|exp| if exp > now { (exp - now).num_days() } else { 0 })
                .unwrap_or(0);

            (
                Some(CurrentPlanInfo {
                    id: None,
                    name: g.name,
                    price: price.to_string(),
                    expires_at: a.expires_at,
                    started_at: Some(a.assigned_at),
                    days_remaining,
                }),
                price,
                period,
            )
        } else {
            (None, rust_decimal::Decimal::ZERO, 30i64)
        }
    } else {
        (None, rust_decimal::Decimal::ZERO, 30i64)
    };

    use super::upgrade_service::{
        billing_period_days, calculate_upgrade_credit, calculate_upgrade_days, is_upgrade_allowed,
    };

    let is_extension = assignment
        .as_ref()
        .is_some_and(|a| a.plan_id == new_plan_uuid);

    let proration_credit = if is_extension {
        rust_decimal::Decimal::ZERO
    } else if let Some(ref a) = assignment {
        if let Some(expires_at) = a.expires_at {
            calculate_upgrade_credit(current_plan_price, expires_at, current_billing_period)
        } else {
            rust_decimal::Decimal::ZERO
        }
    } else {
        rust_decimal::Decimal::ZERO
    };

    let (wallet_credit_balance, total_credits, amount_to_pay, new_expiry, new_duration_days) =
        if is_extension {
            let remaining = assignment
                .as_ref()
                .and_then(|a| a.expires_at)
                .map(|exp| if exp > now { (exp - now).num_days() } else { 0 })
                .unwrap_or(0);
            let ext_expiry = assignment
                .as_ref()
                .and_then(|a| a.expires_at)
                .map(|exp| {
                    if exp > now {
                        exp + chrono::Duration::days(standard_duration_days)
                    } else {
                        now + chrono::Duration::days(standard_duration_days)
                    }
                })
                .unwrap_or(now + chrono::Duration::days(standard_duration_days));
            (
                rust_decimal::Decimal::ZERO,
                rust_decimal::Decimal::ZERO,
                new_plan_price,
                ext_expiry,
                remaining + standard_duration_days,
            )
        } else if assignment.is_some() && current_plan_price > rust_decimal::Decimal::ZERO {
            let days_remaining = assignment
                .as_ref()
                .and_then(|a| a.expires_at)
                .map(|exp| if exp > now { (exp - now).num_days() } else { 0 })
                .unwrap_or(0);
            let new_period = billing_period_days(new_plan.billing_cycle.as_deref());
            let converted = calculate_upgrade_days(
                current_plan_price,
                current_billing_period,
                days_remaining,
                new_plan_price,
                new_period,
            )
            .max(1);
            (
                rust_decimal::Decimal::ZERO,
                proration_credit,
                rust_decimal::Decimal::ZERO,
                now + chrono::Duration::days(converted),
                converted,
            )
        } else {
            let exp = now + chrono::Duration::days(standard_duration_days);
            (
                rust_decimal::Decimal::ZERO,
                rust_decimal::Decimal::ZERO,
                new_plan_price,
                exp,
                standard_duration_days,
            )
        };

    let is_upgrade = current_plan_info.is_none()
        || is_extension
        || is_upgrade_allowed(current_plan_price, new_plan_base_price);

    let response_data = UpgradePreviewData {
        current_plan: current_plan_info,
        new_plan: NewPlanInfo {
            id: new_plan.id.to_string(),
            name: new_plan.name.clone(),
            price: new_plan_price.to_string(),
        },
        credit_from_current_plan: proration_credit.to_string(),
        wallet_credit_balance: wallet_credit_balance.to_string(),
        total_credits_available: total_credits.to_string(),
        amount_to_pay: amount_to_pay.to_string(),
        new_duration_days,
        new_expiry_date: new_expiry,
        is_upgrade_allowed: is_upgrade,
    };

    let message = if is_extension {
        format!(
            "Extension — 30 days added to your current plan. Amount to pay: ${}",
            amount_to_pay
        )
    } else if !is_upgrade {
        "Downgrade not available. You can only upgrade to a higher-tier plan.".to_string()
    } else if amount_to_pay == rust_decimal::Decimal::ZERO && assignment.is_some() {
        format!(
            "Free upgrade — your remaining time converts to {} days on {}",
            new_duration_days, new_plan.name
        )
    } else {
        format!("New subscription - {} days access", new_duration_days)
    };

    Ok(Json(UpgradePreviewResponse {
        success: true,
        message,
        data: Some(response_data),
    }))
}

// ============================================================================
// PLAN SWITCH
// ============================================================================

/// Plan switch request
#[derive(Debug, Deserialize)]
pub struct PlanSwitchRequest {
    pub new_plan_id: String,
}

/// Plan switch response
#[derive(Debug, Serialize)]
pub struct PlanSwitchResponse {
    pub success: bool,
    pub message: String,
    pub data: Option<PlanSwitchData>,
}

/// Plan switch result data
#[derive(Debug, Serialize)]
pub struct PlanSwitchData {
    pub proration_credit: String,
    pub new_wallet_balance: String,
    pub new_plan_name: String,
    pub new_plan_expires_at: Option<String>,
    pub switch_type: String,
}

/// Execute plan switch
pub async fn execute_plan_switch_handler(
    State(app_state): State<crate::web::auth::AppState>,
    Extension(user_context): Extension<OpenIDUserContext>,
    Json(payload): Json<PlanSwitchRequest>,
) -> Result<Json<PlanSwitchResponse>, Json<UnifiedErrorResponse>> {
    use std::str::FromStr;

    let wallet = &user_context.wallet_address;
    info!("Plan switch requested by {} to plan {}", wallet, payload.new_plan_id);

    let new_plan_uuid = Uuid::parse_str(&payload.new_plan_id).map_err(|_| {
        UnifiedErrorResponse::json(400, "Invalid plan ID", "Plan ID must be a valid UUID")
    })?;

    #[derive(sqlx::FromRow)]
    struct AssignmentRow {
        plan_id: Uuid,
        assigned_at: DateTime<Utc>,
        expires_at: Option<DateTime<Utc>>,
    }

    let assignment: Option<AssignmentRow> = sqlx::query_as(
        r#"
        SELECT plan_id, assigned_at, expires_at
        FROM wallet_plan_assignments
        WHERE LOWER(wallet_address) = LOWER($1)
          AND is_active = true
          AND (expires_at IS NULL OR expires_at > NOW())
        ORDER BY assigned_at DESC
        LIMIT 1
        "#,
    )
    .bind(wallet)
    .fetch_optional(app_state.db_pool.as_ref())
    .await
    .map_err(|e| UnifiedErrorResponse::json(500, "Failed to fetch plan", e.to_string()))?;

    let assignment = match assignment {
        Some(a) => a,
        None => {
            return Err(UnifiedErrorResponse::json(
                400,
                "No active plan",
                "You don't have an active plan to switch from",
            ))
        }
    };

    let expires_at = match assignment.expires_at {
        Some(exp) => exp,
        None => {
            return Err(UnifiedErrorResponse::json(
                400,
                "Cannot switch",
                "Permanent plans cannot be switched",
            ))
        }
    };

    if assignment.plan_id == new_plan_uuid {
        return Err(UnifiedErrorResponse::json(
            400,
            "Already on this plan",
            "You are already on this plan",
        ));
    }

    #[derive(sqlx::FromRow)]
    struct SwitchPlanRow {
        id: Uuid,
        name: String,
        price: Option<bigdecimal::BigDecimal>,
        is_active: bool,
        billing_cycle: Option<String>,
    }

    let current_plan: SwitchPlanRow = sqlx::query_as(
        "SELECT id, name, price, is_active, billing_cycle FROM plans WHERE id = $1",
    )
    .bind(assignment.plan_id)
    .fetch_one(app_state.db_pool.as_ref())
    .await
    .map_err(|e| UnifiedErrorResponse::json(500, "Failed to fetch current plan", e.to_string()))?;

    let new_plan: SwitchPlanRow = sqlx::query_as(
        "SELECT id, name, price, is_active, billing_cycle FROM plans WHERE id = $1",
    )
    .bind(new_plan_uuid)
    .fetch_optional(app_state.db_pool.as_ref())
    .await
    .map_err(|e| UnifiedErrorResponse::json(500, "Failed to fetch new plan", e.to_string()))?
    .ok_or_else(|| {
        UnifiedErrorResponse::json(400, "Plan not found", "The selected plan does not exist")
    })?;

    if !new_plan.is_active {
        return Err(UnifiedErrorResponse::json(
            400,
            "Plan inactive",
            "The selected plan is not available",
        ));
    }

    let current_price: rust_decimal::Decimal = current_plan
        .price
        .as_ref()
        .and_then(|bd| rust_decimal::Decimal::from_str(&bd.to_string()).ok())
        .unwrap_or(rust_decimal::Decimal::ZERO);

    let new_price: rust_decimal::Decimal = new_plan
        .price
        .as_ref()
        .and_then(|bd| rust_decimal::Decimal::from_str(&bd.to_string()).ok())
        .unwrap_or(rust_decimal::Decimal::ZERO);

    use super::upgrade_service::{
        billing_period_days, calculate_upgrade_credit, calculate_upgrade_days,
    };

    let cur_period = billing_period_days(current_plan.billing_cycle.as_deref());
    let new_period = billing_period_days(new_plan.billing_cycle.as_deref());
    let days_remaining = (expires_at - Utc::now()).num_days().max(0);

    let converted_days = calculate_upgrade_days(
        current_price,
        cur_period,
        days_remaining,
        new_price,
        new_period,
    )
    .max(1);

    let new_expires_at = Utc::now() + chrono::Duration::days(converted_days);

    let proration_credit = calculate_upgrade_credit(current_price, expires_at, cur_period);

    let is_downgrade = new_price < current_price;

    if is_downgrade {
        return Err(UnifiedErrorResponse::json(
            400,
            "Downgrade not allowed",
            "You can only upgrade to a higher-tier plan.",
        ));
    }

    // Deactivate old plan
    sqlx::query(
        r#"
        UPDATE wallet_plan_assignments
        SET is_active = false, updated_at = NOW()
        WHERE LOWER(wallet_address) = LOWER($1) AND is_active = true
        "#,
    )
    .bind(wallet)
    .execute(app_state.db_pool.as_ref())
    .await
    .map_err(|e| UnifiedErrorResponse::json(500, "Failed to deactivate old plan", e.to_string()))?;

    // Create new plan assignment with converted days
    sqlx::query(
        r#"
        INSERT INTO wallet_plan_assignments (wallet_address, plan_id, assigned_at, expires_at, is_active, assignment_source, assignment_reason)
        VALUES ($1, $2, NOW(), $3, true, 'upgrade', 'Plan upgrade day conversion')
        ON CONFLICT (wallet_address, plan_id) DO UPDATE
        SET expires_at = $3, is_active = true, assigned_at = NOW(), updated_at = NOW(),
            assignment_source = 'upgrade', assignment_reason = 'Plan upgrade day conversion'
        "#,
    )
    .bind(wallet)
    .bind(new_plan_uuid)
    .bind(new_expires_at)
    .execute(app_state.db_pool.as_ref())
    .await
    .map_err(|e| UnifiedErrorResponse::json(500, "Failed to create new plan assignment", e.to_string()))?;

    info!(
        "Upgrade day conversion: {} switched from {} ({} days left) to {} ({} converted days)",
        wallet, current_plan.name, days_remaining, new_plan.name, converted_days
    );

    Ok(Json(PlanSwitchResponse {
        success: true,
        message: format!(
            "Upgrade complete! {} days on {} (converted from {} remaining days).",
            converted_days, new_plan.name, days_remaining
        ),
        data: Some(PlanSwitchData {
            proration_credit: proration_credit.to_string(),
            new_wallet_balance: "0".to_string(),
            new_plan_name: new_plan.name,
            new_plan_expires_at: Some(new_expires_at.to_rfc3339()),
            switch_type: "upgrade_day_conversion".to_string(),
        }),
    }))
}

// ============================================================================
// LEGACY HANDLERS (for backward compatibility)
// ============================================================================

/// Legacy: Get user subscriptions
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

/// Legacy: Cancel subscription
pub async fn cancel_subscription_handler(
    state: State<crate::web::auth::AppState>,
    ext: Extension<OpenIDUserContext>,
    path: Path<Uuid>,
    payload: Json<CancelPlanRequest>,
) -> Result<Json<CancelPlanResponse>, Json<UnifiedErrorResponse>> {
    cancel_plan_handler(state, ext, path, payload).await
}

/// Legacy: Renew subscription - REMOVED
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