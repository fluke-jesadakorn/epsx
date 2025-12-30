// Permission Group Plan Management Handlers
// Manages plans using permission groups instead of tier-based logic

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json as JsonResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use std::collections::HashMap;
use std::str::FromStr;
use uuid::Uuid;

use crate::web::auth::AppState;

/// Helper function to derive permission group name from permissions
fn derive_group_from_permissions(permissions: &[String]) -> String {
    // Extract ranking limit from permissions
    for perm in permissions {
        if let Some(limit_str) = perm.strip_prefix("epsx:rankings:view:") {
            if limit_str == "unlimited" {
                return "Enterprise Access Group".to_string();
            }
            if let Ok(limit) = limit_str.parse::<i32>() {
                return match limit {
                    0..=3 => "Basic Access Group".to_string(),
                    4..=25 => "Standard Access Group".to_string(),
                    26..=50 => "Premium Access Group".to_string(),
                    51..=100 => "Professional Access Group".to_string(),
                    _ => "Enterprise Access Group".to_string(),
                };
            }
        }
    }
    
    // Check for wildcard permissions
    if permissions.iter().any(|p| p == "epsx:*:*" || p == "admin:*:*") {
        return "Enterprise Access Group".to_string();
    }
    
    "Basic Access Group".to_string() // Default fallback
}

/// Get permissions from group template name (mock implementation)
fn get_permissions_from_group_template(group_name: &str) -> Vec<String> {
    match group_name {
        "Basic Access Group" => vec![
            "epsx:rankings:view:3".to_string(),
            "epsx:trading:basic".to_string(),
            "epsx:portfolio:view".to_string(),
        ],
        "Standard Access Group" => vec![
            "epsx:rankings:view:25".to_string(),
            "epsx:trading:basic".to_string(),
            "epsx:trading:advanced".to_string(),
            "epsx:portfolio:view".to_string(),
            "epsx:analytics:basic".to_string(),
        ],
        "Premium Access Group" => vec![
            "epsx:rankings:view:50".to_string(),
            "epsx:trading:premium".to_string(),
            "epsx:portfolio:tools".to_string(),
            "epsx:analytics:advanced".to_string(),
        ],
        "Professional Access Group" => vec![
            "epsx:rankings:view:100".to_string(),
            "epsx:trading:premium".to_string(),
            "epsx:analytics:premium".to_string(),
            "epsx:research:reports".to_string(),
            "epsx:dashboards:custom".to_string(),
        ],
        "Enterprise Access Group" => vec![
            "epsx:rankings:view:unlimited".to_string(),
            "epsx:*:*".to_string(),
            "epsx-pay:*:*".to_string(),
            "epsx-token:*:*".to_string(),
        ],
        _ => vec!["epsx:rankings:view:3".to_string()], // Default to basic access
    }
}

/// Generate quota limits from permissions
/// Logic: -1 = unlimited, 0 = not granted, >0 = specific limit
fn generate_quota_from_permissions(permissions: &[String]) -> serde_json::Value {
    let mut api_calls = 0; // Default = not granted
    let mut rankings_limit = 0; // Default = not granted
    let mut analytics_queries = 0; // Default = not granted
    let mut export_limit = 0; // Default = not granted

    // Extract API calls limit
    for perm in permissions {
        if let Some(limit_str) = perm.strip_prefix("epsx:api:calls:") {
            if limit_str == "unlimited" {
                api_calls = -1; // -1 = unlimited
            } else if let Ok(limit) = limit_str.parse::<i32>() {
                api_calls = limit; // Specific limit
            }
        }
    }

    // Extract ranking limit
    for perm in permissions {
        if let Some(limit_str) = perm.strip_prefix("epsx:rankings:view:") {
            if limit_str == "unlimited" {
                rankings_limit = -1; // -1 = unlimited
            } else if let Ok(limit) = limit_str.parse::<i32>() {
                rankings_limit = limit; // Specific limit
            }
        }
    }

    // Extract analytics queries limit
    for perm in permissions {
        if let Some(limit_str) = perm.strip_prefix("epsx:analytics:queries:") {
            if limit_str == "unlimited" {
                analytics_queries = -1; // -1 = unlimited
            } else if let Ok(limit) = limit_str.parse::<i32>() {
                analytics_queries = limit; // Specific limit
            }
        }
    }

    // Extract export limit
    for perm in permissions {
        if let Some(limit_str) = perm.strip_prefix("epsx:export:limit:") {
            if limit_str == "unlimited" {
                export_limit = -1; // -1 = unlimited
            } else if let Ok(limit) = limit_str.parse::<i32>() {
                export_limit = limit; // Specific limit
            }
        }
    }

    // Check for premium features
    let has_trading_premium = permissions.iter().any(|p| p.contains("trading:premium"));

    serde_json::json!({
        "api_calls": api_calls,
        "rankings_limit": rankings_limit,
        "analytics_queries": analytics_queries,
        "premium_features": has_trading_premium,
        "export_limit": export_limit
    })
}

// Request/Response DTOs (reusing existing ones)

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreatePlanRequest {
    pub name: String,
    pub description: Option<String>,
    pub permission_group_name: String, // e.g., "Standard Access Group", "Enterprise Access Group"
    #[schema(value_type = String, example = "29.99")]
    pub current_price: Decimal,
    pub currency: String,
    pub target_audience: String,
    pub billing_model: String,
    pub permissions: Vec<String>, // Direct permission array
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct PermissionGroupRequest {
    pub name: String,
    pub description: Option<String>,
    pub permissions: Vec<String>,
    pub group_type: String, // Group type: "basic", "standard", "premium", etc.
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PlanResponse {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub permission_group_name: String,
    #[schema(value_type = String, example = "29.99")]
    pub current_price: Decimal,
    pub currency: String,
    pub target_audience: String,
    pub billing_model: String,
    pub group_type: String, // Group type derived from permissions
    pub is_active: bool,
    pub permissions: Vec<String>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
    pub subscriber_count: u64,
    #[schema(value_type = String, example = "1499.85")]
    pub revenue_last_30_days: Decimal,
}

#[derive(Debug, Serialize)]
pub struct PermissionGroupResponse {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub permissions: Vec<String>,
    pub group_type: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

/// Create permission template-based plan
#[utoipa::path(
    post,
    path = "/admin/plans",
    request_body = CreatePlanRequest,
    responses(
        (status = 200, description = "Plan created successfully", body = PlanResponse),
        (status = 400, description = "Invalid request data"),
        (status = 500, description = "Internal server error")
    ),
    tag = "admin-plans",
    security(("bearerAuth" = []))
)]
pub async fn create_plan_handler(
    State(app_state): State<AppState>,
    Json(request): Json<CreatePlanRequest>,
) -> Result<JsonResponse<PlanResponse>, StatusCode> {
    // Derive group type from permissions
    let group_type = derive_group_from_permissions(&request.permissions);

    // Create slug from name
    let slug = request.name.to_lowercase().replace(" ", "-");

    // Convert Decimal to BigDecimal for database
    let price_bigdecimal = bigdecimal::BigDecimal::from_str(&request.current_price.to_string())
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    // Build metadata JSON
    let mut metadata = request.metadata.unwrap_or_else(|| serde_json::json!({}));
    if let Some(obj) = metadata.as_object_mut() {
        obj.insert("target_audience".to_string(), serde_json::json!(request.target_audience));
        obj.insert("permissions".to_string(), serde_json::json!(request.permissions));
    }

    // Create new permission group in database
    let new_group = crate::infrastructure::adapters::repositories::database_types::NewPermissionGroup {
        name: request.name.clone(),
        slug,
        description: request.description.clone().unwrap_or_else(|| format!("{} subscription plan", request.name)),
        group_type: "subscription".to_string(),
        group_metadata: metadata, // permissions already inserted into metadata above
        price: Some(price_bigdecimal),
        currency: Some(request.currency.clone()),
        billing_cycle: Some("pay_per_use".to_string()),
        is_active: Some(true),
        is_promoted: Some(false),
        display_order: None,
        created_by: Some("admin".to_string()),
        rate_limit_per_minute: 0,
        rate_limit_per_hour: 0,
        rate_limit_per_day: 0,
        burst_capacity: 0,
    };

    let created_plan = match app_state.group_repo.create_group(new_group).await {
        Ok(plan) => plan,
        Err(err) => {
            tracing::error!(error = %err, "Failed to create plan in database");
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    // Convert to response format
    let permissions = created_plan.permissions().as_array()
        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect::<Vec<String>>())
        .unwrap_or_default();

    let price_decimal = created_plan.price.as_ref()
        .map(|p| Decimal::from_str(&p.to_string()).unwrap_or(Decimal::ZERO))
        .unwrap_or(Decimal::ZERO);

    let plan_response = PlanResponse {
        id: 0, // Legacy field - UUID is the real identifier
        name: created_plan.name.clone(),
        description: Some(created_plan.description),
        permission_group_name: created_plan.name,
        current_price: price_decimal,
        currency: created_plan.currency.unwrap_or_else(|| "USD".to_string()),
        target_audience: request.target_audience,
        billing_model: created_plan.billing_cycle.unwrap_or_else(|| "pay_per_use".to_string()),
        group_type,
        is_active: created_plan.is_active.unwrap_or(true),
        permissions,
        metadata: Some(created_plan.group_metadata),
        created_at: created_plan.created_at,
        updated_at: Some(created_plan.updated_at),
        subscriber_count: 0, // New plan has no subscribers yet
        revenue_last_30_days: Decimal::ZERO,
    };

    tracing::info!(
        plan_id = %created_plan.id,
        plan_name = %plan_response.name,
        permission_group = %plan_response.permission_group_name,
        group_type = %plan_response.group_type,
        "Permission group plan created in database"
    );

    Ok(JsonResponse(plan_response))
}

/// List all permission template-based plans
#[utoipa::path(
    get,
    path = "/admin/plans",
    responses(
        (status = 200, description = "Plans retrieved successfully"),
        (status = 500, description = "Internal server error")
    ),
    tag = "admin-plans",
    security(("bearerAuth" = []))
)]
pub async fn list_plans_handler(
    State(app_state): State<AppState>,
    Query(_query): Query<HashMap<String, String>>,
) -> Result<JsonResponse<serde_json::Value>, StatusCode> {
    use diesel::prelude::*;
    use diesel_async::RunQueryDsl;
    use crate::schemas::payments::subscriptions;

    // Get plans from database instead of hardcoded data
    let db_plans = match app_state.group_repo.get_subscription_plans().await {
        Ok(plans) => plans,
        Err(err) => {
            tracing::error!(error = %err, "Failed to fetch subscription plans from database");
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    // Get subscriber counts for all plans
    let mut conn = match (*app_state.db_pool).get().await {
        Ok(c) => c,
        Err(err) => {
            tracing::error!(error = %err, "Failed to get database connection for subscriber count");
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    // Query subscriber counts per plan_id
    let subscriber_counts: Vec<(Uuid, i64)> = subscriptions::table
        .filter(subscriptions::status.eq("active"))
        .group_by(subscriptions::plan_id)
        .select((subscriptions::plan_id, diesel::dsl::count_star()))
        .load(&mut conn)
        .await
        .unwrap_or_default();

    // Create a map for quick lookup
    let count_map: std::collections::HashMap<Uuid, i64> = subscriber_counts.into_iter().collect();

    // Convert database plans to admin format (with additional fields for admin UI)
    let plans: Vec<serde_json::Value> = db_plans.into_iter().map(|plan| {
        use crate::domain::subscription_management::Promotion;

        // Get subscriber count from map
        let subscriber_count = count_map.get(&plan.id).copied().unwrap_or(0);

        // Extract permissions array from JSONB
        let permissions = plan.permissions().as_array()
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect::<Vec<String>>())
            .unwrap_or_default();

        // Generate group type from permissions (for backward compatibility)
        let group_type = derive_group_from_permissions(&permissions);

        // Extract metadata values
        let target_audience = plan.group_metadata.get("target_audience")
            .and_then(|v| v.as_str())
            .unwrap_or("web_users")
            .to_string();

        // Get price as string
        let price_str = plan.price.as_ref()
            .map(|p| p.to_string())
            .unwrap_or_else(|| "0.00".to_string());

        let base_price = price_str.parse::<f64>().unwrap_or(0.0);

        // Extract and process promotion
        let promotion_data = plan.group_metadata.get("promotion");
        let (effective_price, promotion_active, promotion_status, promotion_discount) = if let Some(promo_value) = promotion_data {
            if let Ok(promo) = serde_json::from_value::<Promotion>(promo_value.clone()) {
                let effective = promo.calculate_effective_price(base_price);
                let active = promo.is_active();
                let status = promo.get_status();
                let discount = promo.get_discount_percentage(base_price);
                (effective, active, status, discount)
            } else {
                (base_price, false, crate::domain::subscription_management::PromotionStatus::Disabled, 0.0)
            }
        } else {
            (base_price, false, crate::domain::subscription_management::PromotionStatus::Disabled, 0.0)
        };

        // Map group type to plan_category
        let plan_category = match group_type.as_str() {
            "Enterprise Access Group" => "enterprise",
            "Professional Access Group" => "api",
            "Premium Access Group" => "api",
            _ => "standard"
        };

        serde_json::json!({
            "id": plan.id.to_string(),
            "name": plan.name.clone(),
            "description": plan.description,
            "permission_group_name": plan.name.clone(),
            "plan_type": group_type.clone(),
            "plan_category": plan_category,
            "current_price": price_str,
            "effective_price": effective_price,
            "promotion_active": promotion_active,
            "promotion_status": format!("{:?}", promotion_status).to_lowercase(),
            "promotion_discount": promotion_discount,
            "currency": plan.currency.unwrap_or_else(|| "USD".to_string()),
            "target_audience": target_audience,
            "billing_model": plan.billing_cycle.unwrap_or_else(|| "pay_per_use".to_string()),
            "group_type": group_type,
            "permissions": permissions,
            "features": [],
            "is_active": plan.is_active.unwrap_or(true),
            "created_at": plan.created_at.to_rfc3339(),
            "updated_at": plan.updated_at.to_rfc3339(),
            "subscriber_count": subscriber_count,
            "revenue_last_30_days": "0.00"
        })
    }).collect();

    let total_count = plans.len();

    Ok(JsonResponse(serde_json::json!({
        "success": true,
        "data": {
            "plans": plans,
            "total_count": total_count,
            "has_more": false
        },
        "message": "Permission group plans retrieved successfully"
    })))
}

/// Get permission template-based plan details
pub async fn get_plan_handler(
    State(app_state): State<AppState>,
    Path(plan_id_str): Path<String>,
) -> Result<JsonResponse<serde_json::Value>, StatusCode> {
    use crate::domain::subscription_management::Promotion;

    // Parse plan ID as UUID (new format) or handle legacy integer IDs
    let plan_uuid = match Uuid::parse_str(&plan_id_str) {
        Ok(uuid) => uuid,
        Err(_) => {
            // Handle legacy integer IDs by finding the first plan (for backward compatibility)
            // In production, you might want to maintain a mapping table or return an error
            let db_plans = match app_state.group_repo.get_subscription_plans().await {
                Ok(plans) => plans,
                Err(err) => {
                    tracing::error!(error = %err, "Failed to fetch subscription plans from database");
                    return Err(StatusCode::INTERNAL_SERVER_ERROR);
                }
            };

            if let Some(plan) = db_plans.first() {
                plan.id
            } else {
                return Err(StatusCode::NOT_FOUND);
            }
        }
    };

    // Get plan from database
    let plan = match app_state.group_repo.get_plan_by_id(plan_uuid).await {
        Ok(Some(plan)) => plan,
        Ok(None) => return Err(StatusCode::NOT_FOUND),
        Err(err) => {
            tracing::error!(error = %err, plan_id = %plan_uuid, "Failed to fetch plan from database");
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    // Extract permissions array from JSONB
    let permissions = plan.permissions().as_array()
        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect::<Vec<String>>())
        .unwrap_or_else(Vec::new);

    // Generate group type from permissions
    let group_type = derive_group_from_permissions(&permissions);

    // Extract metadata values
    let target_audience = plan.group_metadata.get("target_audience")
        .and_then(|v| v.as_str())
        .unwrap_or("web_users")
        .to_string();

    // Convert price to Decimal
    let price_decimal = plan.price.as_ref()
        .map(|p| rust_decimal::Decimal::from_str(&p.to_string()).unwrap_or(rust_decimal::Decimal::ZERO))
        .unwrap_or_else(|| rust_decimal::Decimal::ZERO);

    let base_price = price_decimal.to_string().parse::<f64>().unwrap_or(0.0);

    // Extract and process promotion
    let promotion_data = plan.group_metadata.get("promotion");
    let (effective_price, promotion_active, promotion_status, promotion_discount) = if let Some(promo_value) = promotion_data {
        if let Ok(promo) = serde_json::from_value::<Promotion>(promo_value.clone()) {
            let effective = promo.calculate_effective_price(base_price);
            let active = promo.is_active();
            let status = promo.get_status();
            let discount = promo.get_discount_percentage(base_price);
            (effective, active, status, discount)
        } else {
            (base_price, false, crate::domain::subscription_management::PromotionStatus::Disabled, 0.0)
        }
    } else {
        (base_price, false, crate::domain::subscription_management::PromotionStatus::Disabled, 0.0)
    };

    let plan_response = serde_json::json!({
        "id": 0,
        "name": plan.name.clone(),
        "description": plan.description,
        "permission_group_name": plan.name,
        "current_price": price_decimal.to_string(),
        "effective_price": effective_price,
        "promotion_active": promotion_active,
        "promotion_status": format!("{:?}", promotion_status).to_lowercase(),
        "promotion_discount": promotion_discount,
        "currency": plan.currency.unwrap_or_else(|| "USD".to_string()),
        "target_audience": target_audience,
        "billing_model": plan.billing_cycle.unwrap_or_else(|| "pay_per_use".to_string()),
        "group_type": group_type,
        "is_active": plan.is_active.unwrap_or(true),
        "permissions": permissions,
        "metadata": plan.group_metadata,
        "created_at": plan.created_at.to_rfc3339(),
        "updated_at": plan.updated_at.to_rfc3339(),
        "subscriber_count": 0,
        "revenue_last_30_days": "0.00",
    });

    Ok(JsonResponse(plan_response))
}

/// Generate API key
fn generate_api_key() -> String {
    use rand::Rng;
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    let mut rng = rand::thread_rng();
    let key: String = (0..32)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect();
    format!("epsx_{}", key)
}

/// Mock subscription creation
#[derive(Debug, Deserialize)]
pub struct CreateSubscriptionRequest {
    pub wallet_address: String,
    pub plan_id: i32,
    pub permission_group_name: String, // e.g., "Premium Access Group"
    pub access_context: String,
    pub api_key_name: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub auto_renew: bool,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
pub struct SubscriptionResponse {
    pub id: String,
    pub wallet_address: String,
    pub plan_id: i32,
    pub permission_group_name: String,
    pub permissions_granted: Vec<String>,
    pub group_type: String,
    pub access_context: String,
    pub api_key: Option<String>,
    pub api_key_name: Option<String>,
    pub status: String,
    pub expires_at: Option<DateTime<Utc>>,
    pub auto_renew: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub metadata: Option<serde_json::Value>,
    pub current_usage: serde_json::Value,
    pub quota_limits: serde_json::Value,
}

/// Update plan request
#[derive(Debug, Deserialize)]
pub struct UpdatePlanRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub current_price: Option<Decimal>,
    pub is_active: Option<bool>,
    pub permissions: Option<Vec<String>>,
    pub metadata: Option<serde_json::Value>,
}

/// Update permission template-based plan
pub async fn update_plan_handler(
    State(app_state): State<AppState>,
    Path(plan_id_str): Path<String>,
    Json(request): Json<UpdatePlanRequest>,
) -> Result<JsonResponse<PlanResponse>, StatusCode> {
    // Parse plan ID as UUID
    let plan_uuid = match Uuid::parse_str(&plan_id_str) {
        Ok(uuid) => uuid,
        Err(_) => {
            tracing::error!(plan_id = %plan_id_str, "Invalid plan ID format");
            return Err(StatusCode::BAD_REQUEST);
        }
    };

    // Get existing plan from database
    let mut plan = match app_state.group_repo.get_plan_by_id(plan_uuid).await {
        Ok(Some(plan)) => plan,
        Ok(None) => {
            tracing::error!(plan_id = %plan_uuid, "Plan not found");
            return Err(StatusCode::NOT_FOUND);
        }
        Err(err) => {
            tracing::error!(error = %err, plan_id = %plan_uuid, "Failed to fetch plan from database");
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    // Update fields if provided
    if let Some(name) = request.name {
        plan.name = name;
    }

    if let Some(description) = request.description {
        plan.description = description;
    }

    if let Some(current_price) = request.current_price {
        plan.price = Some(bigdecimal::BigDecimal::from_str(&current_price.to_string())
            .unwrap_or_else(|_| bigdecimal::BigDecimal::from(0)));
    }

    if let Some(is_active) = request.is_active {
        plan.is_active = Some(is_active);
    }

    if let Some(permissions) = request.permissions {
        // Update permissions in group_metadata
        if let Some(metadata_obj) = plan.group_metadata.as_object_mut() {
            metadata_obj.insert("permissions".to_string(), serde_json::json!(permissions));
        }
    }

    if let Some(metadata) = request.metadata {
        // Merge with existing metadata
        if let Some(existing_metadata) = plan.group_metadata.as_object_mut() {
            if let Some(new_metadata) = metadata.as_object() {
                for (key, value) in new_metadata {
                    existing_metadata.insert(key.clone(), value.clone());
                }
            }
        } else {
            plan.group_metadata = metadata;
        }
    }

    // Update plan in database
    match app_state.group_repo.update_plan(plan.clone()).await {
        Ok(_) => {
            // Extract permissions array from JSONB
            let permissions = plan.permissions().as_array()
                .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect::<Vec<String>>())
                .unwrap_or_else(Vec::new);

            // Generate group type from permissions
            let group_type = derive_group_from_permissions(&permissions);

            // Extract metadata values
            let target_audience = plan.group_metadata.get("target_audience")
                .and_then(|v| v.as_str())
                .unwrap_or("web_users")
                .to_string();

            // Convert price to Decimal
            let price_decimal = plan.price.as_ref()
                .map(|p| rust_decimal::Decimal::from_str(&p.to_string()).unwrap_or(rust_decimal::Decimal::ZERO))
                .unwrap_or_else(|| rust_decimal::Decimal::ZERO);

            let plan_response = PlanResponse {
                id: 0, // Legacy field
                name: plan.name.clone(),
                description: Some(plan.description),
                permission_group_name: plan.name,
                current_price: price_decimal,
                currency: plan.currency.unwrap_or_else(|| "USD".to_string()),
                target_audience,
                billing_model: plan.billing_cycle.unwrap_or_else(|| "pay_per_use".to_string()),
                group_type,
                is_active: plan.is_active.unwrap_or(true),
                permissions,
                metadata: Some(plan.group_metadata),
                created_at: plan.created_at,
                updated_at: Some(plan.updated_at),
                subscriber_count: 0,
                revenue_last_30_days: Decimal::ZERO,
            };

            tracing::info!(
                plan_id = %plan_uuid,
                plan_name = %plan_response.name,
                "Plan updated successfully"
            );

            Ok(JsonResponse(plan_response))
        }
        Err(err) => {
            tracing::error!(error = %err, plan_id = %plan_uuid, "Failed to update plan in database");
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Create permission template-based subscription
pub async fn create_subscription_handler(
    State(state): State<AppState>,
    Json(request): Json<CreateSubscriptionRequest>,
) -> Result<JsonResponse<SubscriptionResponse>, StatusCode> {
    use diesel::prelude::*;
    use diesel_async::RunQueryDsl;
    use crate::schemas::payments::subscriptions;
    use crate::schemas::primary::groups;
    use crate::infrastructure::models::payment::NewSubscriptionDb;

    let subscription_id = Uuid::new_v4();
    
    // Get DB connection for plan lookup
    let mut conn = (*state.db_pool).get().await.map_err(|e| {
        tracing::error!("Failed to get database connection: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    
    // Find plan UUID from permission_group_name
    let plan_uuid: Uuid = groups::table
        .filter(groups::name.eq(&request.permission_group_name))
        .select(groups::id)
        .first::<Uuid>(&mut conn)
        .await
        .unwrap_or_else(|_| {
            tracing::warn!("Could not find plan by name '{}', using placeholder UUID", request.permission_group_name);
            Uuid::nil()
        });
    
    let api_key = if request.access_context == "external" {
        Some(generate_api_key())
    } else {
        None
    };
    
    // Get permissions from group template
    let permissions_granted = get_permissions_from_group_template(&request.permission_group_name);
    let group_type = derive_group_from_permissions(&permissions_granted);
    
    // Generate quota limits from permissions
    let quota_limits = generate_quota_from_permissions(&permissions_granted);
    
    // Calculate expiry (default 1 year for admin-assigned subscriptions)
    let expires_at = request.expires_at.unwrap_or_else(|| Utc::now() + chrono::Duration::days(365));
    
    // Create database record
    let new_subscription = NewSubscriptionDb {
        wallet_address: request.wallet_address.clone(),
        plan_id: plan_uuid,
        payment_id: None, // Admin-assigned subscriptions don't require payment
        status: "active".to_string(),
        started_at: Some(Utc::now()),
        expires_at,
        cancelled_at: None,
        auto_renew: Some(request.auto_renew),
        metadata: Some(serde_json::json!({
            "permission_group_name": request.permission_group_name,
            "access_context": request.access_context,
            "api_key_name": request.api_key_name,
            "created_by": "admin",
        })),
    };
    
    // Insert into database (reusing connection from plan lookup)
    diesel::insert_into(subscriptions::table)
        .values(&new_subscription)
        .execute(&mut conn)
        .await
        .map_err(|e| {
            tracing::error!("Failed to insert subscription: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    
    let response = SubscriptionResponse {
        id: subscription_id.to_string(),
        wallet_address: request.wallet_address,
        plan_id: request.plan_id,
        permission_group_name: request.permission_group_name,
        permissions_granted,
        group_type,
        access_context: request.access_context,
        api_key,
        api_key_name: request.api_key_name,
        status: "active".to_string(),
        expires_at: Some(expires_at),
        auto_renew: request.auto_renew,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        metadata: request.metadata,
        current_usage: serde_json::json!({"api_calls": 0, "rankings_viewed": 0}),
        quota_limits,
    };
    
    tracing::info!(
        subscription_id = %response.id,
        wallet_address = %response.wallet_address,
        group = %response.permission_group_name,
        group_type = %response.group_type,
        "Permission group subscription created and persisted to database"
    );
    
    Ok(JsonResponse(response))
}

// ============================================================================
// DIRECT PAYMENT MODEL - User Access List
// ============================================================================

/// Query params for user access list
#[derive(Debug, Deserialize)]
pub struct UserAccessListQuery {
    pub page: Option<i64>,
    pub limit: Option<i64>,
    pub status: Option<String>, // "active", "expired", "expiring_soon", "no_plan"
    pub search: Option<String>,
}

/// User access data response
#[derive(Debug, Serialize)]
pub struct UserAccessData {
    pub wallet_address: String,
    pub current_plan_id: Option<i32>,
    pub plan_name: Option<String>,
    pub plan_expires_at: Option<DateTime<Utc>>,
    pub days_remaining: i64,
    pub status: String, // "active", "expiring_soon", "expired", "no_plan"
}

/// Admin handler to list users with plan access (Direct Payment model)
/// Queries wallet_group_assignments for plan access data
pub async fn admin_list_user_access_handler(
    State(app_state): State<AppState>,
    Query(query): Query<UserAccessListQuery>,
) -> Result<JsonResponse<serde_json::Value>, StatusCode> {
    use diesel_async::RunQueryDsl;
    
    let page = query.page.unwrap_or(1).max(1);
    let limit = query.limit.unwrap_or(20).min(100);
    let offset = (page - 1) * limit;
    
    let mut conn = match (*app_state.db_pool).get().await {
        Ok(c) => c,
        Err(err) => {
            tracing::error!(error = %err, "Failed to get database connection");
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };
    
    // Query wallet_group_assignments with plan info from groups table
    #[derive(diesel::QueryableByName)]
    struct UserRow {
        #[diesel(sql_type = diesel::sql_types::Text)]
        wallet_address: String,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>)]
        expires_at: Option<DateTime<Utc>>,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
        plan_name: Option<String>,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
        group_id: Option<String>,
    }
    
    // Build the query with search filter
    let search_filter = query.search.as_ref()
        .map(|s| format!("%{}%", s.to_lowercase()))
        .unwrap_or_else(|| "%".to_string());
    
    // Query wallet_group_assignments joined with groups for subscription plans
    let users: Vec<UserRow> = diesel::sql_query(
        r#"
        SELECT 
            wga.wallet_address::text,
            wga.expires_at,
            g.name as plan_name,
            g.id::text as group_id
        FROM wallet_group_assignments wga
        LEFT JOIN groups g ON wga.group_id = g.id
        WHERE wga.is_active = true
          AND g.group_type = 'subscription'
          AND LOWER(wga.wallet_address) LIKE $1
        ORDER BY wga.assigned_at DESC NULLS LAST
        LIMIT $2 OFFSET $3
        "#
    )
    .bind::<diesel::sql_types::Text, _>(&search_filter)
    .bind::<diesel::sql_types::BigInt, _>(limit)
    .bind::<diesel::sql_types::BigInt, _>(offset)
    .get_results(&mut conn)
    .await
    .map_err(|e| {
        tracing::error!(error = %e, "Failed to query user access data");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    
    // Get total count
    #[derive(diesel::QueryableByName)]
    struct CountRow {
        #[diesel(sql_type = diesel::sql_types::BigInt)]
        count: i64,
    }
    
    let total_count: i64 = diesel::sql_query(
        r#"
        SELECT COUNT(*)::bigint as count 
        FROM wallet_group_assignments wga
        LEFT JOIN groups g ON wga.group_id = g.id
        WHERE wga.is_active = true
          AND g.group_type = 'subscription'
          AND LOWER(wga.wallet_address) LIKE $1
        "#
    )
    .bind::<diesel::sql_types::Text, _>(&search_filter)
    .get_result::<CountRow>(&mut conn)
    .await
    .map(|r| r.count)
    .unwrap_or(0);
    
    let now = Utc::now();
    
    // Convert to response format with status calculation
    let users_data: Vec<UserAccessData> = users.into_iter()
        .filter_map(|user| {
            let days_remaining = user.expires_at
                .map(|exp| (exp - now).num_days())
                .unwrap_or(365); // No expiry means long-term access
            
            let status = if user.plan_name.is_none() {
                "no_plan"
            } else if days_remaining < 0 {
                "expired"
            } else if days_remaining <= 7 {
                "expiring_soon"
            } else {
                "active"
            };
            
            // Apply status filter if provided
            if let Some(ref filter_status) = query.status {
                if filter_status != status {
                    return None;
                }
            }
            
            Some(UserAccessData {
                wallet_address: user.wallet_address,
                current_plan_id: None, // Not using integer plan IDs anymore
                plan_name: user.plan_name,
                plan_expires_at: user.expires_at,
                days_remaining: days_remaining.max(0),
                status: status.to_string(),
            })
        })
        .collect();
    
    // Calculate stats
    let active_count = users_data.iter().filter(|u| u.status == "active").count();
    let expiring_count = users_data.iter().filter(|u| u.status == "expiring_soon").count();
    let expired_count = users_data.iter().filter(|u| u.status == "expired").count();
    let no_plan_count = users_data.iter().filter(|u| u.status == "no_plan").count();
    
    Ok(JsonResponse(serde_json::json!({
        "success": true,
        "data": {
            "users": users_data,
            "pagination": {
                "page": page,
                "limit": limit,
                "total": total_count,
                "total_pages": (total_count as f64 / limit as f64).ceil() as i64
            },
            "summary": {
                "total_users": total_count,
                "active": active_count,
                "expiring_soon": expiring_count,
                "expired": expired_count,
                "no_plan": no_plan_count
            }
        },
        "message": "User access data retrieved successfully"
    })))
}