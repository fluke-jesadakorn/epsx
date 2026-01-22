use axum::{
    response::Json,
    http::StatusCode,
    extract::{State, Path},
};

use crate::web::auth::AppState;

use crate::web::api_response::ApiResponse;

use utoipa::ToSchema;
use serde::Serialize; // Ensure Serialize is available

#[derive(Debug, Serialize, ToSchema, Clone)]
pub struct PublicPlanResponse {
    pub id: String,
    pub name: String,
    pub plan_type: String,
    pub current_price: String,
    pub effective_price: f64,
    pub promotion_active: bool,
    pub promotion_status: String,
    pub promotion_discount: f64,
    pub promotion_ends_at: Option<String>,
    pub currency: String,
    pub billing_cycle: String,
    pub features: Vec<String>,
    pub permissions: Vec<String>,
    pub is_active: bool,
    pub display_order: i32,
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

    tracing::info!("📊 Fetching public subscription plans");

    // Get plans from database instead of hardcoded data
    let db_plans = match app_state.plan_repo.get_subscription_plans().await {
        Ok(plans) => {
            tracing::info!("✅ Found {} subscription plans in database", plans.len());
            plans
        },
        Err(err) => {
            tracing::error!(error = %err, "❌ Failed to fetch subscription plans from database");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::error("DB_ERROR", "Failed to fetch plans"))
            );
        }
    };

    // If no plans found, return empty array (not an error)
    if db_plans.is_empty() {
        tracing::warn!("⚠️ No subscription plans found in database - returning empty array");
        return (
            StatusCode::OK,
            Json(ApiResponse::success(vec![]))
        );
    }

    // Convert database plans to frontend format
    let plans: Vec<PublicPlanResponse> = db_plans.into_iter().map(|plan| {
        use crate::domain::subscription_management::Promotion;

        // Extract permissions array from JSONB
        let permissions = plan.permissions().as_array()
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect::<Vec<String>>())
            .unwrap_or_default();

        // Extract features from metadata or generate from permissions
        let features = plan.plan_metadata.get("features")
            .and_then(|f| f.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect::<Vec<String>>())
            .unwrap_or_else(|| generate_features_from_permissions(&permissions));

        // Generate plan type from name or metadata
        let plan_type = plan.plan_metadata.get("plan_type")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| {
                plan.name.to_uppercase()
                    .replace(" PLAN", "")
                    .replace(" ", "_")
            });

        // Get price as string
        let price_str = plan.price.as_ref()
            .map(|p| p.to_string())
            .unwrap_or_else(|| "0.00".to_string());

        let base_price = price_str.parse::<f64>().unwrap_or(0.0);

        // Extract and process promotion
        let promotion_data = plan.plan_metadata.get("promotion");
        let (effective_price, promotion_active, promotion_status, promotion_discount, promotion_ends_at) = if let Some(promo_value) = promotion_data {
            if let Ok(promo) = serde_json::from_value::<Promotion>(promo_value.clone()) {
                let effective = promo.calculate_effective_price(base_price);
                let active = promo.is_active();
                let status = promo.get_status();
                let discount = promo.get_discount_percentage(base_price);
                let ends_at = if active { Some(promo.end_date.clone()) } else { None };
                (effective, active, status, discount, ends_at)
            } else {
                (base_price, false, crate::domain::subscription_management::PromotionStatus::Disabled, 0.0, None)
            }
        } else {
            (base_price, false, crate::domain::subscription_management::PromotionStatus::Disabled, 0.0, None)
        };

        PublicPlanResponse {
            id: plan.id.to_string(),
            name: plan.name,
            plan_type,
            current_price: price_str,
            effective_price,
            promotion_active,
            promotion_status: format!("{:?}", promotion_status).to_lowercase(),
            promotion_discount,
            promotion_ends_at,
            currency: plan.currency.unwrap_or_else(|| "USD".to_string()),
            billing_cycle: plan.billing_cycle.unwrap_or_else(|| "monthly".to_string()),
            features,
            permissions,
            is_active: plan.is_active.unwrap_or(true),
            display_order: plan.display_order.unwrap_or(0),
        }
    }).filter(|p| {
        // Filter out free plans
        let is_free = p.effective_price == 0.0 || p.name.to_lowercase() == "free plan";
        if is_free { return false; }
        
        // Filter by category if requested
        if let Some(ref cat) = category_filter {
            // Map plan_type to category-like string or check exact match
            // "starter", "pro" -> subscription
            // "enterprise" -> enterprise
            // "api", "api_developer" -> api
            let p_type = p.plan_type.to_lowercase();
            match cat.as_str() {
                "api" => p_type.contains("api"),
                "enterprise" => p_type.contains("enterprise"),
                "subscription" | "user" => !p_type.contains("api") && !p_type.contains("enterprise"),
                _ => true,
            }
        } else {
            true
        }
    }).collect();

    (
        StatusCode::OK,
        Json(ApiResponse::success(plans))
    )
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
    tracing::info!(plan_id = %plan_id, "📊 Fetching public plan by ID");

    // Parse UUID
    let plan_uuid = match uuid::Uuid::parse_str(&plan_id) {
        Ok(id) => id,
        Err(_) => {
            tracing::warn!(plan_id = %plan_id, "⚠️ Invalid plan ID format");
            return (
                StatusCode::BAD_REQUEST,
                Json(ApiResponse::error("INVALID_ID", "Invalid plan ID format"))
            );
        }
    };

    // Get all plans and find the one with matching ID
    let db_plans = match app_state.plan_repo.get_subscription_plans().await {
        Ok(plans) => plans,
        Err(err) => {
            tracing::error!(error = %err, "❌ Failed to fetch subscription plans");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::error("DB_ERROR", "Failed to fetch plans"))
            );
        }
    };

    // Find the specific plan
    let plan = match db_plans.into_iter().find(|p| p.id == plan_uuid) {
        Some(p) => p,
        None => {
            tracing::warn!(plan_id = %plan_id, "⚠️ Plan not found");
            return (
                StatusCode::NOT_FOUND,
                Json(ApiResponse::error("NOT_FOUND", "Plan not found"))
            );
        }
    };

    use crate::domain::subscription_management::Promotion;

    // Extract permissions array from JSONB
    let permissions = plan.permissions().as_array()
        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect::<Vec<String>>())
        .unwrap_or_default();

    // Extract features from metadata or generate from permissions
    let features = plan.plan_metadata.get("features")
        .and_then(|f| f.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect::<Vec<String>>())
        .unwrap_or_else(|| generate_features_from_permissions(&permissions));

    // Generate plan type from name
    let plan_type = plan.name.to_uppercase()
        .replace(" PLAN", "")
        .replace(" ", "_");

    // Get price as string
    let price_str = plan.price.as_ref()
        .map(|p| p.to_string())
        .unwrap_or_else(|| "0.00".to_string());

    let base_price = price_str.parse::<f64>().unwrap_or(0.0);

    // Extract and process promotion
    let promotion_data = plan.plan_metadata.get("promotion");
    let (effective_price, promotion_active, promotion_status, promotion_discount, promotion_ends_at) = if let Some(promo_value) = promotion_data {
        if let Ok(promo) = serde_json::from_value::<Promotion>(promo_value.clone()) {
            let effective = promo.calculate_effective_price(base_price);
            let active = promo.is_active();
            let status = promo.get_status();
            let discount = promo.get_discount_percentage(base_price);
            let ends_at = if active { Some(promo.end_date.clone()) } else { None };
            (effective, active, status, discount, ends_at)
        } else {
            (base_price, false, crate::domain::subscription_management::PromotionStatus::Disabled, 0.0, None)
        }
    } else {
        (base_price, false, crate::domain::subscription_management::PromotionStatus::Disabled, 0.0, None)
    };

    let plan_data = PublicPlanResponse {
        id: plan.id.to_string(),
        name: plan.name,
        plan_type,
        current_price: price_str,
        effective_price,
        promotion_active,
        promotion_status: format!("{:?}", promotion_status).to_lowercase(),
        promotion_discount,
        promotion_ends_at,
        currency: plan.currency.unwrap_or_else(|| "USD".to_string()),
        billing_cycle: plan.billing_cycle.unwrap_or_else(|| "monthly".to_string()),
        features,
        permissions,
        is_active: plan.is_active.unwrap_or(true),
        display_order: plan.display_order.unwrap_or(0),
    };

    tracing::info!(plan_id = %plan_id, "✅ Plan retrieved successfully");
    (
        StatusCode::OK,
        Json(ApiResponse::success(plan_data))
    )
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
            },
            perm if perm.starts_with("epsx:rankings:view:") => {
                if let Some(limit_str) = perm.strip_prefix("epsx:rankings:view:") {
                    if limit_str == "unlimited" {
                        features.push("Unlimited rankings access".to_string());
                    } else if let Ok(limit) = limit_str.parse::<i32>() {
                        features.push(format!("Rankings from position {}+", limit + 1));
                    }
                }
            },
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
            },
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