use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};

use crate::web::auth::AppState;

use crate::web::api_response::ApiResponse;

use serde::{Deserialize, Serialize};
use utoipa::ToSchema; // Ensure Serialize and Deserialize are available

#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
pub struct PublicPlanResponse {
    pub id: String,
    pub name: String,
    pub plan_type: String,
    pub current_price: String,
    pub effective_price: f64,
    pub promotion_active: bool,
    pub promotion_status: String,
    pub promotion_discount: f64,
    #[serde(default)]
    pub promotion_savings: String,
    pub promotion_ends_at: Option<String>,
    pub currency: String,
    pub billing_cycle: String,
    pub features: Vec<String>,
    pub permissions: Vec<String>,
    pub is_active: bool,
    pub tier_level: i32,
    pub plan_group: String,
    pub ranking_offset: i32,
    pub rankings_limit: i32,
    /// Exact backend-owned amount submitted to the payment verifier.
    pub checkout_price: String,
    /// Stablecoin used to settle the plan price on chain.
    pub settlement_currency: String,
    /// Access duration owned by the plan catalog. `None` is lifetime access.
    pub duration_days: Option<i64>,
}

fn hosted_checkout() -> bool {
    std::env::var("EPSX_PAY_CHECKOUT_ENABLED").as_deref() == Ok("true")
}

// Hosted sales use the same exact calculation as quotes and order creation.
fn apply_hosted_price(plan: &mut PublicPlanResponse, metadata: &serde_json::Value) -> bool {
    if !hosted_checkout() {
        return true;
    }
    let token = std::env::var("EPSX_PAY_CHECKOUT_TOKEN").unwrap_or_else(|_| "USDT".into());
    apply_token_price(plan, metadata, &token)
}
fn apply_token_price(
    plan: &mut PublicPlanResponse,
    metadata: &serde_json::Value,
    token: &str,
) -> bool {
    let Ok(pricing) = crate::domain::subscription_management::token_pricing::calculate(
        metadata,
        token,
        chrono::Utc::now(),
    ) else {
        return false;
    };
    if crate::web::payments::merchant_checkout::units(&pricing.price, 18).is_err() {
        return false;
    }
    plan.current_price = pricing.original_price;
    plan.checkout_price = pricing.price;
    plan.effective_price = plan.checkout_price.parse().unwrap_or_default();
    plan.promotion_active = pricing.promotion_active;
    plan.promotion_status = pricing.promotion_status;
    plan.promotion_discount = pricing.promotion_discount;
    plan.promotion_savings = pricing.savings;
    plan.promotion_ends_at = pricing.promotion_ends_at;
    plan.currency = token.into();
    plan.settlement_currency = token.into();
    true
}

fn ranking_access(plan_metadata: &serde_json::Value) -> (i32, i32) {
    let ranking_offset = plan_metadata
        .get("ranking_offset")
        .and_then(serde_json::Value::as_i64)
        .and_then(|value| i32::try_from(value).ok())
        .filter(|value| (0..=10_000).contains(value))
        .unwrap_or(epsx_contracts::constants::FREE_PLAN_RANKING_OFFSET);
    let rankings_limit = plan_metadata
        .get("rankings_limit")
        .and_then(serde_json::Value::as_i64)
        .and_then(|value| i32::try_from(value).ok())
        .filter(|value| *value == -1 || (1..=10_000).contains(value))
        .unwrap_or(-1);
    (ranking_offset, rankings_limit)
}

fn checkout_terms(
    plan_metadata: &serde_json::Value,
    billing_cycle: Option<&str>,
    base_price: &str,
    effective_price: f64,
    promotion_active: bool,
) -> (String, String, Option<i64>) {
    let checkout_price = if promotion_active {
        format!("{effective_price:.2}")
    } else {
        base_price.to_string()
    };
    let settlement_currency = std::env::var("PAYMENT_SETTLEMENT_CURRENCY")
        .ok()
        .map(|value| value.trim().to_ascii_uppercase())
        .filter(|value| matches!(value.as_str(), "USDT" | "USDC"))
        .unwrap_or_else(|| "USDT".to_string());
    let duration_days = plan_metadata
        .get("duration_days")
        .and_then(serde_json::Value::as_i64)
        .filter(|days| (1..=3_650).contains(days))
        .or_else(|| match billing_cycle.unwrap_or_default() {
            "daily" => Some(1),
            "weekly" => Some(7),
            "monthly" => Some(30),
            "quarterly" => Some(90),
            "yearly" | "annual" => Some(365),
            "lifetime" => None,
            _ => Some(30),
        });
    (checkout_price, settlement_currency, duration_days)
}

/// Get public pricing plans (no authentication required)
/// GET /api/public/plans
#[utoipa::path(
    get,
    path = "/api/public/plans",
    tag = "public",
    params(
        ("category" = Option<String>, Query, description = "Filter by plan category (subscription, enterprise, api)"),
        ("affiliate_code" = Option<String>, Query, description = "Affiliate code for tracking")
    ),
    responses(
        (status = 200, description = "Successfully retrieved subscription plans", body = ApiResponse<Vec<PublicPlanResponse>>),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_public_plans(
    State(app_state): State<AppState>,
    axum::extract::Query(query): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> (StatusCode, Json<ApiResponse<Vec<PublicPlanResponse>>>) {
    let category_filter = query.get("category").map(|s| s.to_lowercase());
    let group_filter = query.get("group").map(|s| s.to_lowercase());

    tracing::info!("Fetching public subscription plans");

    let category_str = category_filter.as_deref().unwrap_or("any");
    let group_str = group_filter.as_deref().unwrap_or("any");
    let cache_key = format!("cache:public_plans:cat_{}:grp_{}", category_str, group_str);

    if let Some(redis_pool) = app_state.redis_pool.as_ref().filter(|_| !hosted_checkout()) {
        let mut conn = redis_pool.get_connection();
        use redis::AsyncCommands;
        let cache_res: redis::RedisResult<Option<String>> = conn.get(&cache_key).await;
        if let Ok(Some(cached_str)) = cache_res {
            if let Ok(parsed) = serde_json::from_str::<Vec<PublicPlanResponse>>(&cached_str) {
                tracing::info!("Cache hit for public plans: {}", cache_key);
                return (StatusCode::OK, Json(ApiResponse::success(parsed)));
            }
        }
    }

    // Get plans from database instead of hardcoded data
    let db_plans = match app_state.plan_repo.get_subscription_plans().await {
        Ok(plans) => {
            tracing::info!("Found {} subscription plans in database", plans.len());
            plans
        }
        Err(err) => {
            tracing::error!(error = %err, "Failed to fetch subscription plans from database");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::error("DB_ERROR", "Failed to fetch plans")),
            );
        }
    };

    // If no plans found, return empty array (not an error)
    if db_plans.is_empty() {
        tracing::warn!("No subscription plans found in database - returning empty array");
        return (StatusCode::OK, Json(ApiResponse::success(vec![])));
    }

    // Convert database plans to frontend format
    // First filter by is_public to only show public plans
    let plans: Vec<PublicPlanResponse> = db_plans
        .into_iter()
        .filter(|plan| plan.is_public && plan.is_active.unwrap_or(true))
        .filter_map(|plan| {
            use crate::domain::subscription_management::Promotion;

            let (ranking_offset, rankings_limit) = ranking_access(&plan.plan_metadata);

            // Extract permissions array from JSONB
            let permissions = plan
                .permissions()
                .as_array()
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect::<Vec<String>>()
                })
                .unwrap_or_default();

            // Extract features from metadata or generate from permissions
            let features = plan
                .plan_metadata
                .get("features")
                .and_then(|f| f.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect::<Vec<String>>()
                })
                .unwrap_or_else(|| generate_features_from_permissions(&permissions));

            // Generate plan type from name or metadata
            let plan_type = plan
                .plan_metadata
                .get("plan_type")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .unwrap_or_else(|| {
                    plan.name
                        .to_uppercase()
                        .replace(" PLAN", "")
                        .replace(" ", "_")
                });

            // Get price as string
            let price_str = plan
                .price
                .as_ref()
                .map(|p| p.to_string())
                .unwrap_or_else(|| "0.00".to_string());

            let base_price = price_str.parse::<f64>().unwrap_or(0.0);

            // Extract and process promotion
            let promotion_data = plan.plan_metadata.get("promotion");
            let (
                effective_price,
                promotion_active,
                promotion_status,
                promotion_discount,
                promotion_ends_at,
            ) = if let Some(promo_value) = promotion_data {
                match serde_json::from_value::<Promotion>(promo_value.clone()) {
                    Ok(promo) => {
                        let effective = promo.calculate_effective_price(base_price);
                        let active = promo.is_active();
                        let status = promo.get_status();
                        let discount = promo.get_discount_percentage(base_price);
                        let ends_at = if active {
                            Some(promo.end_date.clone())
                        } else {
                            None
                        };
                        tracing::debug!(
                            "Plan {} promotion: enabled={}, active={}, effective_price={:.2}",
                            plan.id,
                            promo.enabled,
                            active,
                            effective
                        );
                        (effective, active, status, discount, ends_at)
                    }
                    Err(e) => {
                        tracing::warn!(
                            "Failed to deserialize promotion for plan {}: {}",
                            plan.id,
                            e
                        );
                        (
                            base_price,
                            false,
                            crate::domain::subscription_management::PromotionStatus::Disabled,
                            0.0,
                            None,
                        )
                    }
                }
            } else {
                (
                    base_price,
                    false,
                    crate::domain::subscription_management::PromotionStatus::Disabled,
                    0.0,
                    None,
                )
            };

            let (checkout_price, settlement_currency, duration_days) = checkout_terms(
                &plan.plan_metadata,
                plan.billing_cycle.as_deref(),
                &price_str,
                effective_price,
                promotion_active,
            );

            let metadata = plan.plan_metadata.clone();
            let mut projection = PublicPlanResponse {
                id: plan.id.to_string(),
                name: plan.name,
                plan_type,
                current_price: price_str,
                effective_price,
                promotion_active,
                promotion_status: format!("{:?}", promotion_status).to_lowercase(),
                promotion_discount,
                promotion_savings: format!("{:.2}", (base_price - effective_price).max(0.0)),
                promotion_ends_at,
                currency: plan.currency.unwrap_or_else(|| "USD".to_string()),
                billing_cycle: plan.billing_cycle.unwrap_or_else(|| "monthly".to_string()),
                features,
                permissions,
                is_active: plan.is_active.unwrap_or(true),
                tier_level: plan.tier_level,
                plan_group: plan.plan_group.clone(),
                ranking_offset,
                rankings_limit,
                checkout_price,
                settlement_currency,
                duration_days,
            };
            apply_hosted_price(&mut projection, &metadata).then_some(projection)
        })
        .filter(|p| {
            // Filter by group if requested
            if let Some(ref grp) = group_filter {
                if p.plan_group.to_lowercase() != *grp {
                    return false;
                }
            }

            // Filter by category if requested
            if let Some(ref cat) = category_filter {
                let p_type = p.plan_type.to_lowercase();
                match cat.as_str() {
                    "api" => p_type.contains("api"),
                    "enterprise" => p_type.contains("enterprise"),
                    "subscription" | "user" => {
                        !p_type.contains("api") && !p_type.contains("enterprise")
                    }
                    _ => true,
                }
            } else {
                true
            }
        })
        .collect();

    // Append constant Free Plan

    // Sort by tier_level
    let mut final_plans = plans;
    final_plans.sort_by_key(|p| p.tier_level);

    // Save to cache
    if let Some(redis_pool) = app_state.redis_pool.as_ref().filter(|_| !hosted_checkout()) {
        let mut conn = redis_pool.get_connection();
        use redis::AsyncCommands;
        if let Ok(json_str) = serde_json::to_string(&final_plans) {
            let _: redis::RedisResult<()> = conn.set_ex(&cache_key, json_str, 900).await;
        }
    }

    (StatusCode::OK, Json(ApiResponse::success(final_plans)))
}

/// Get a single public plan by ID (no authentication required)
/// GET /api/public/plans/:id
#[utoipa::path(
    get,
    path = "/api/public/plans/{id}",
    tag = "public",
    params(
        ("id" = String, Path, description = "Plan UUID")
    ),
    responses(
        (status = 200, description = "Successfully retrieved plan", body = ApiResponse<PublicPlanResponse>),
        (status = 404, description = "Plan not found"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_public_plan_by_id(
    State(app_state): State<AppState>,
    Path(plan_id): Path<String>,
) -> (StatusCode, Json<ApiResponse<PublicPlanResponse>>) {
    tracing::info!(plan_id = %plan_id, "Fetching public plan by ID");

    // Parse UUID
    let plan_uuid = match uuid::Uuid::parse_str(&plan_id) {
        Ok(id) => id,
        Err(_) => {
            tracing::warn!(plan_id = %plan_id, "Invalid plan ID format");
            return (
                StatusCode::BAD_REQUEST,
                Json(ApiResponse::error("INVALID_ID", "Invalid plan ID format")),
            );
        }
    };

    let cache_key = format!("cache:public_plan:{}", plan_uuid);

    if let Some(redis_pool) = app_state.redis_pool.as_ref().filter(|_| !hosted_checkout()) {
        let mut conn = redis_pool.get_connection();
        use redis::AsyncCommands;
        let cache_res: redis::RedisResult<Option<String>> = conn.get(&cache_key).await;
        if let Ok(Some(cached_str)) = cache_res {
            if let Ok(parsed) = serde_json::from_str::<PublicPlanResponse>(&cached_str) {
                tracing::info!("Cache hit for public plan: {}", cache_key);
                return (StatusCode::OK, Json(ApiResponse::success(parsed)));
            }
        }
    }

    // Get all plans and find the one with matching ID
    let db_plans = match app_state.plan_repo.get_subscription_plans().await {
        Ok(plans) => plans,
        Err(err) => {
            tracing::error!(error = %err, "Failed to fetch subscription plans");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::error("DB_ERROR", "Failed to fetch plans")),
            );
        }
    };

    // Find the specific plan
    let plan = match db_plans
        .into_iter()
        .find(|plan| plan.id == plan_uuid && plan.is_public && plan.is_active.unwrap_or(true))
    {
        Some(p) => p,
        None => {
            tracing::warn!(plan_id = %plan_id, "Plan not found");
            return (
                StatusCode::NOT_FOUND,
                Json(ApiResponse::error("NOT_FOUND", "Plan not found")),
            );
        }
    };

    use crate::domain::subscription_management::Promotion;

    // Extract permissions array from JSONB
    let permissions = plan
        .permissions()
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect::<Vec<String>>()
        })
        .unwrap_or_default();

    // Extract features from metadata or generate from permissions
    let features = plan
        .plan_metadata
        .get("features")
        .and_then(|f| f.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect::<Vec<String>>()
        })
        .unwrap_or_else(|| generate_features_from_permissions(&permissions));

    // Generate plan type from name
    let plan_type = plan
        .name
        .to_uppercase()
        .replace(" PLAN", "")
        .replace(" ", "_");

    // Get price as string
    let price_str = plan
        .price
        .as_ref()
        .map(|p| p.to_string())
        .unwrap_or_else(|| "0.00".to_string());

    let base_price = price_str.parse::<f64>().unwrap_or(0.0);
    let (ranking_offset, rankings_limit) = ranking_access(&plan.plan_metadata);

    // Extract and process promotion
    let promotion_data = plan.plan_metadata.get("promotion");
    let (
        effective_price,
        promotion_active,
        promotion_status,
        promotion_discount,
        promotion_ends_at,
    ) = if let Some(promo_value) = promotion_data {
        if let Ok(promo) = serde_json::from_value::<Promotion>(promo_value.clone()) {
            let effective = promo.calculate_effective_price(base_price);
            let active = promo.is_active();
            let status = promo.get_status();
            let discount = promo.get_discount_percentage(base_price);
            let ends_at = if active {
                Some(promo.end_date.clone())
            } else {
                None
            };
            (effective, active, status, discount, ends_at)
        } else {
            (
                base_price,
                false,
                crate::domain::subscription_management::PromotionStatus::Disabled,
                0.0,
                None,
            )
        }
    } else {
        (
            base_price,
            false,
            crate::domain::subscription_management::PromotionStatus::Disabled,
            0.0,
            None,
        )
    };

    let (checkout_price, settlement_currency, duration_days) = checkout_terms(
        &plan.plan_metadata,
        plan.billing_cycle.as_deref(),
        &price_str,
        effective_price,
        promotion_active,
    );

    let metadata = plan.plan_metadata.clone();
    let mut plan_data = PublicPlanResponse {
        id: plan.id.to_string(),
        name: plan.name,
        plan_type,
        current_price: price_str,
        effective_price,
        promotion_active,
        promotion_status: format!("{:?}", promotion_status).to_lowercase(),
        promotion_discount,
        promotion_savings: format!("{:.2}", (base_price - effective_price).max(0.0)),
        promotion_ends_at,
        currency: plan.currency.unwrap_or_else(|| "USD".to_string()),
        billing_cycle: plan.billing_cycle.unwrap_or_else(|| "monthly".to_string()),
        features,
        permissions,
        is_active: plan.is_active.unwrap_or(true),
        tier_level: plan.tier_level,
        plan_group: plan.plan_group.clone(),
        ranking_offset,
        rankings_limit,
        checkout_price,
        settlement_currency,
        duration_days,
    };

    if !apply_hosted_price(&mut plan_data, &metadata) {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ApiResponse::error(
                "TOKEN_PRICE_UNAVAILABLE",
                "Token price is not configured",
            )),
        );
    }

    if let Some(redis_pool) = app_state.redis_pool.as_ref().filter(|_| !hosted_checkout()) {
        let mut conn = redis_pool.get_connection();
        use redis::AsyncCommands;
        if let Ok(json_str) = serde_json::to_string(&plan_data) {
            let _: redis::RedisResult<()> = conn.set_ex(&cache_key, json_str, 900).await;
        }
    }

    tracing::info!(plan_id = %plan_id, "Plan retrieved successfully");
    (StatusCode::OK, Json(ApiResponse::success(plan_data)))
}

/// Generate user-friendly features from permission list
fn generate_features_from_permissions(permissions: &[String]) -> Vec<String> {
    let mut features = Vec::new();

    for permission in permissions {
        match permission.as_str() {
            // Basic features
            perm if perm.starts_with("epsx:analytics:view:") => {
                if let Some(limit_str) = perm.strip_prefix("epsx:analytics:view:") {
                    if limit_str == "unlimited" {
                        features.push("Unlimited stock analysis".to_string());
                    } else if let Ok(limit) = limit_str.parse::<i32>() {
                        if limit <= 5 {
                            features.push("Basic analytics view".to_string());
                        } else {
                            features.push(format!("Analytics for up to {} stocks", limit));
                        }
                    }
                }
            }
            perm if perm.starts_with("epsx:rankings:view:") => {
                if let Some(limit_str) = perm.strip_prefix("epsx:rankings:view:") {
                    if limit_str == "unlimited" {
                        features.push("Unlimited rankings access".to_string());
                    } else if let Ok(limit) = limit_str.parse::<i32>() {
                        features.push(format!("Rankings from position {}+", limit + 1));
                    }
                }
            }
            perm if perm.starts_with("epsx:rankings:offset:") => {
                if let Some(offset_str) = perm.strip_prefix("epsx:rankings:offset:") {
                    if let Ok(offset) = offset_str.parse::<i32>() {
                        if offset == 0 {
                            features.push("Full rankings access (Rank 1+)".to_string());
                        } else {
                            features.push(format!("Rankings access from position {}+", offset + 1));
                        }
                    }
                }
            }
            "epsx:analytics:export" => features.push("Export functionality".to_string()),
            "epsx:analytics:advanced" => features.push("Advanced analytics".to_string()),
            "epsx:analytics:premium" => features.push("Premium analytics".to_string()),
            "epsx:trading:basic" => features.push("Basic trading features".to_string()),
            "epsx:trading:advanced" => features.push("Advanced trading".to_string()),
            "epsx:trading:premium" => features.push("Premium trading tools".to_string()),
            "epsx:portfolio:view" => features.push("Portfolio viewing".to_string()),
            "epsx:portfolio:manage" => features.push("Portfolio management".to_string()),
            "epsx:portfolio:advanced" => features.push("Advanced portfolio tools".to_string()),
            "epsx:alerts:create" => features.push("Create alerts".to_string()),
            "epsx:alerts:manage" => features.push("Alert management".to_string()),
            "epsx:api:access" => features.push("API access".to_string()),
            perm if perm.starts_with("epsx:api:ratelimit_min:") => {
                if let Some(n) = perm.strip_prefix("epsx:api:ratelimit_min:") {
                    features.push(format!("{} requests/min", n));
                }
            }
            perm if perm.starts_with("epsx:api:ratelimit_hour:") => {
                if let Some(n) = perm.strip_prefix("epsx:api:ratelimit_hour:") {
                    features.push(format!("{} requests/hour", n));
                }
            }
            perm if perm.starts_with("epsx:api:ratelimit_day:") => {
                if let Some(n) = perm.strip_prefix("epsx:api:ratelimit_day:") {
                    features.push(format!("{} requests/day", n));
                }
            }
            perm if perm.starts_with("epsx:api:burst:") => {
                if let Some(n) = perm.strip_prefix("epsx:api:burst:") {
                    features.push(format!("Burst capacity: {}", n));
                }
            }
            perm if perm.starts_with("epsx:api:calls_limit:") => {
                if let Some(n) = perm.strip_prefix("epsx:api:calls_limit:") {
                    features.push(format!("{} API calls", n));
                }
            }
            perm if perm.starts_with("epsx:rankings:limit:") => {
                if let Some(n) = perm.strip_prefix("epsx:rankings:limit:") {
                    features.push(format!("Top {} rankings", n));
                }
            }
            "epsx:analytics:enabled" => features.push("Advanced analytics".to_string()),
            "epsx:support:premium" => features.push("Premium support".to_string()),
            "epsx:*:*" => features.push("Full platform access".to_string()),
            "epsx:enterprise:*" => features.push("Enterprise features".to_string()),
            _ => {} // Skip unknown permissions
        }
    }

    // Add default features if none found
    if features.is_empty() {
        features.push("Basic access".to_string());
    }

    features
}

#[cfg(test)]
mod hosted_price_tests {
    use super::*;
    fn sale() -> PublicPlanResponse {
        PublicPlanResponse {
            id: "plan".into(),
            name: "Monthly".into(),
            plan_type: "subscription".into(),
            current_price: "99.00".into(),
            effective_price: 9.9,
            promotion_active: true,
            promotion_status: "active".into(),
            promotion_discount: 90.0,
            promotion_savings: "89.10".into(),
            promotion_ends_at: None,
            currency: "USD".into(),
            billing_cycle: "monthly".into(),
            features: vec![],
            permissions: vec![],
            is_active: true,
            tier_level: 1,
            plan_group: "personal".into(),
            ranking_offset: 1,
            rankings_limit: 25,
            checkout_price: "9.90".into(),
            settlement_currency: "USDT".into(),
            duration_days: Some(30),
        }
    }
    #[test]
    fn catalog_sale_is_applied_once_to_base_token_price() {
        for token in ["USDT", "USDC"] {
            let mut plan = sale();
            assert!(apply_token_price(
                &mut plan,
                &serde_json::json!({"pay_prices":{token:"99.00"},"pay_use_catalog_promotion":true,"promotion":{"enabled":true,"type":"percentage","value":90}}),
                token
            ));
            assert_eq!(plan.current_price, "99.00");
            assert_eq!(plan.checkout_price, "9.90");
            assert_eq!(plan.effective_price, 9.9);
            assert!(plan.promotion_active);
            assert_eq!(plan.currency, token);
        }
    }
    #[test]
    fn independent_token_price_does_not_inherit_usd_promotion() {
        let mut plan = sale();
        assert!(apply_token_price(
            &mut plan,
            &serde_json::json!({"pay_prices":{"USDT":"7.00"}}),
            "USDT"
        ));
        assert_eq!(plan.current_price, "7.00");
        assert_eq!(plan.checkout_price, "7.00");
        assert!(!plan.promotion_active);
        assert_eq!(plan.promotion_discount, 0.0);
    }
}
