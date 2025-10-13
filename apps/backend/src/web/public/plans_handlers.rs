use axum::{
    response::Json,
    http::StatusCode,
    extract::State,
};
use serde_json::{json, Value};
use crate::web::auth::AppState;

/// Get public pricing plans (no authentication required)
/// GET /api/public/plans
#[utoipa::path(
    get,
    path = "/api/public/plans",
    tag = "public",
    responses(
        (status = 200, description = "Successfully retrieved subscription plans"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_public_plans(State(app_state): State<AppState>) -> Result<Json<Value>, StatusCode> {
    tracing::info!("📊 Fetching public subscription plans");

    // Get plans from database instead of hardcoded data
    let db_plans = match app_state.permission_group_repo.get_subscription_plans().await {
        Ok(plans) => {
            tracing::info!("✅ Found {} subscription plans in database", plans.len());
            plans
        },
        Err(err) => {
            tracing::error!(error = %err, "❌ Failed to fetch subscription plans from database");
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    // If no plans found, return empty array (not an error)
    if db_plans.is_empty() {
        tracing::warn!("⚠️ No subscription plans found in database - returning empty array");
        return Ok(Json(json!({
            "success": true,
            "data": [],
            "message": "No subscription plans available. Please contact support or use fallback plans."
        })));
    }

    // Convert database plans to frontend format
    let plans: Vec<Value> = db_plans.into_iter().map(|plan| {
        use crate::domain::subscription_management::Promotion;

        // Extract permissions array from JSONB
        let permissions = plan.permissions.as_array()
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect::<Vec<String>>())
            .unwrap_or_default();

        // Extract features from metadata or generate from permissions
        let features = plan.group_metadata.get("features")
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
        let promotion_data = plan.group_metadata.get("promotion");
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

        json!({
            "id": plan.id.to_string(),
            "name": plan.name,
            "plan_type": plan_type,
            "current_price": price_str,
            "effective_price": effective_price,
            "promotion_active": promotion_active,
            "promotion_status": format!("{:?}", promotion_status).to_lowercase(),
            "promotion_discount": promotion_discount,
            "promotion_ends_at": promotion_ends_at,
            "currency": plan.currency.unwrap_or_else(|| "USD".to_string()),
            "billing_cycle": plan.billing_cycle.unwrap_or_else(|| "monthly".to_string()),
            "features": features,
            "permissions": permissions,
            "is_active": plan.is_active.unwrap_or(true),
            "display_order": plan.display_order.unwrap_or(0)
        })
    }).collect();

    Ok(Json(json!({
        "success": true,
        "data": plans,
        "message": "Public pricing plans retrieved successfully"
    })))
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
                        features.push(format!("{} stock rankings limit", limit));
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