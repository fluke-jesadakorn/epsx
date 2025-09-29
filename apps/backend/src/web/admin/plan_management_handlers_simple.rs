// Permission Group Plan Management Handlers
// Manages plans using permission groups instead of tier-based logic

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json as JsonResponse,
    Json,
};
use serde::{Deserialize, Serialize};
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
fn generate_quota_from_permissions(permissions: &[String]) -> serde_json::Value {
    let mut api_calls = 100; // Default
    let mut rankings_limit = 3; // Default
    
    // Extract ranking limit
    for perm in permissions {
        if let Some(limit_str) = perm.strip_prefix("epsx:rankings:view:") {
            if limit_str == "unlimited" {
                rankings_limit = -1;
                api_calls = -1; // Unlimited API calls for enterprise
                break;
            }
            if let Ok(limit) = limit_str.parse::<i32>() {
                rankings_limit = limit;
                // Scale API calls based on ranking tier
                api_calls = match limit {
                    0..=5 => 100,
                    6..=25 => 500,
                    26..=50 => 2000,
                    51..=100 => 5000,
                    _ => 10000,
                };
            }
        }
    }
    
    // Check for premium features
    let has_analytics = permissions.iter().any(|p| p.contains("analytics"));
    let has_trading_premium = permissions.iter().any(|p| p.contains("trading:premium"));
    
    serde_json::json!({
        "api_calls": api_calls,
        "rankings_limit": rankings_limit,
        "analytics_queries": if has_analytics { api_calls / 10 } else { 0 },
        "premium_features": has_trading_premium,
        "export_limit": if rankings_limit > 25 { 50 } else { 10 }
    })
}

// Request/Response DTOs (reusing existing ones)

#[derive(Debug, Deserialize)]
pub struct CreatePlanRequest {
    pub name: String,
    pub description: Option<String>,
    pub permission_group_name: String, // e.g., "Standard Access Group", "Enterprise Access Group"
    pub current_price: Decimal,
    pub currency: String,
    pub target_audience: String,
    pub billing_model: String,
    pub permissions: Vec<String>, // Direct permission array
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct PermissionGroupRequest {
    pub name: String,
    pub description: Option<String>,
    pub permissions: Vec<String>,
    pub group_type: String, // Group type: "basic", "standard", "premium", etc.
}

#[derive(Debug, Serialize)]
pub struct PlanResponse {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub permission_group_name: String,
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
pub async fn create_plan_handler(
    State(_state): State<AppState>,
    Json(request): Json<CreatePlanRequest>,
) -> Result<JsonResponse<PlanResponse>, StatusCode> {
    let plan_id = rand::random::<i32>().abs();
    
    // Derive group type from permissions
    let group_type = derive_group_from_permissions(&request.permissions);
    
    let plan_response = PlanResponse {
        id: plan_id,
        name: request.name,
        description: request.description,
        permission_group_name: request.permission_group_name,
        current_price: request.current_price,
        currency: request.currency,
        target_audience: request.target_audience,
        billing_model: request.billing_model,
        group_type,
        is_active: true,
        permissions: request.permissions,
        metadata: request.metadata,
        created_at: Utc::now(),
        updated_at: Some(Utc::now()),
        subscriber_count: 0,
        revenue_last_30_days: Decimal::ZERO,
    };

    tracing::info!(
        plan_id = plan_id,
        plan_name = %plan_response.name,
        permission_group = %plan_response.permission_group_name,
        group_type = %plan_response.group_type,
        "Permission group plan created"
    );

    Ok(JsonResponse(plan_response))
}

/// List permission template-based plans
pub async fn list_plans_handler(
    State(app_state): State<AppState>,
    Query(_query): Query<HashMap<String, String>>,
) -> Result<JsonResponse<serde_json::Value>, StatusCode> {
    // Get plans from database instead of hardcoded data
    let db_plans = match app_state.permission_group_repo.get_subscription_plans().await {
        Ok(plans) => plans,
        Err(err) => {
            tracing::error!(error = %err, "Failed to fetch subscription plans from database");
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    // Convert database plans to admin format (with additional fields for admin UI)
    let plans: Vec<serde_json::Value> = db_plans.into_iter().map(|plan| {
        // Extract permissions array from JSONB
        let permissions = plan.permissions.as_array()
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect::<Vec<String>>())
            .unwrap_or_else(Vec::new);
        
        // Generate group type from permissions (for backward compatibility)
        let group_type = derive_group_from_permissions(&permissions);
        
        // Extract metadata values
        let target_audience = plan.group_metadata.get("target_audience")
            .and_then(|v| v.as_str())
            .unwrap_or("web_users")
            .to_string();
        
        // Mock subscriber data (could be calculated from database in the future)
        let price_str = plan.price.as_ref()
            .map(|p| p.to_string())
            .unwrap_or_else(|| "0.00".to_string());
        let (subscriber_count, revenue_last_30_days) = match price_str.as_str() {
            "0.00" => (500, "0.00"),
            price if price.parse::<f64>().unwrap_or(0.0) < 20.0 => (150, "1499.85"),
            price if price.parse::<f64>().unwrap_or(0.0) < 60.0 => (75, "3749.25"),
            _ => (25, "4999.75"),
        };

        serde_json::json!({
            "id": plan.id,
            "name": plan.name.clone(),
            "description": plan.description,
            "permission_group_name": plan.name, // Use plan name as group name
            "current_price": price_str,
            "currency": plan.currency.unwrap_or_else(|| "USD".to_string()),
            "target_audience": target_audience,
            "billing_model": plan.billing_cycle.unwrap_or_else(|| "monthly".to_string()),
            "group_type": group_type,
            "permissions": permissions,
            "is_active": plan.is_active.unwrap_or(true),
            "subscriber_count": subscriber_count,
            "revenue_last_30_days": revenue_last_30_days
        })
    }).collect();

    Ok(JsonResponse(serde_json::json!({
        "success": true,
        "data": {
            "plans": plans,
            "total_count": 3,
            "has_more": false
        },
        "message": "Permission group plans retrieved successfully"
    })))
}

/// Get permission template-based plan details
pub async fn get_plan_handler(
    State(app_state): State<AppState>,
    Path(plan_id_str): Path<String>,
) -> Result<JsonResponse<PlanResponse>, StatusCode> {
    // Parse plan ID as UUID (new format) or handle legacy integer IDs
    let plan_uuid = match Uuid::parse_str(&plan_id_str) {
        Ok(uuid) => uuid,
        Err(_) => {
            // Handle legacy integer IDs by finding the first plan (for backward compatibility)
            // In production, you might want to maintain a mapping table or return an error
            let db_plans = match app_state.permission_group_repo.get_subscription_plans().await {
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
    let plan = match app_state.permission_group_repo.get_plan_by_id(plan_uuid).await {
        Ok(Some(plan)) => plan,
        Ok(None) => return Err(StatusCode::NOT_FOUND),
        Err(err) => {
            tracing::error!(error = %err, plan_id = %plan_uuid, "Failed to fetch plan from database");
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };
    
    // Extract permissions array from JSONB
    let permissions = plan.permissions.as_array()
        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect::<Vec<String>>())
        .unwrap_or_else(Vec::new);
    
    // Generate group type from permissions
    let group_type = derive_group_from_permissions(&permissions);
    
    // Extract metadata values
    let target_audience = plan.group_metadata.get("target_audience")
        .and_then(|v| v.as_str())
        .unwrap_or("web_users")
        .to_string();
    
    // Mock subscriber data (could be calculated from database in the future)
    let price_decimal = plan.price.as_ref()
        .map(|p| rust_decimal::Decimal::from_str(&p.to_string()).unwrap_or_else(|_| rust_decimal::Decimal::ZERO))
        .unwrap_or_else(|| rust_decimal::Decimal::ZERO);
    
    let (subscriber_count, revenue_last_30_days) = match price_decimal.to_string().as_str() {
        "0" | "0.00" => (500, Decimal::ZERO),
        price if price.parse::<f64>().unwrap_or(0.0) < 20.0 => (150, Decimal::new(149985, 2)), // 1499.85
        price if price.parse::<f64>().unwrap_or(0.0) < 60.0 => (75, Decimal::new(374925, 2)),  // 3749.25
        _ => (25, Decimal::new(499975, 2)), // 4999.75
    };
    
    let plan_response = PlanResponse {
        id: 0, // Legacy field - could be removed or mapped differently
        name: plan.name.clone(),
        description: Some(plan.description),
        permission_group_name: plan.name, // Use plan name as group name
        current_price: price_decimal,
        currency: plan.currency.unwrap_or_else(|| "USD".to_string()),
        target_audience,
        billing_model: plan.billing_cycle.unwrap_or_else(|| "monthly".to_string()),
        group_type,
        is_active: plan.is_active.unwrap_or(true),
        permissions,
        metadata: Some(plan.group_metadata),
        created_at: plan.created_at,
        updated_at: Some(plan.updated_at),
        subscriber_count,
        revenue_last_30_days,
    };

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

/// Create permission template-based subscription
pub async fn create_subscription_handler(
    State(_state): State<AppState>,
    Json(request): Json<CreateSubscriptionRequest>,
) -> Result<JsonResponse<SubscriptionResponse>, StatusCode> {
    let subscription_id = uuid::Uuid::new_v4().to_string();
    
    let api_key = if request.access_context == "external" {
        Some(generate_api_key())
    } else {
        None
    };
    
    // Get permissions from group template (mock implementation)
    let permissions_granted = get_permissions_from_group_template(&request.permission_group_name);
    let group_type = derive_group_from_permissions(&permissions_granted);
    
    // Generate quota limits from permissions
    let quota_limits = generate_quota_from_permissions(&permissions_granted);
    
    let response = SubscriptionResponse {
        id: subscription_id,
        wallet_address: request.wallet_address,
        plan_id: request.plan_id,
        permission_group_name: request.permission_group_name,
        permissions_granted,
        group_type,
        access_context: request.access_context,
        api_key,
        api_key_name: request.api_key_name,
        status: "active".to_string(),
        expires_at: request.expires_at,
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
        "Permission group subscription created"
    );
    
    Ok(JsonResponse(response))
}