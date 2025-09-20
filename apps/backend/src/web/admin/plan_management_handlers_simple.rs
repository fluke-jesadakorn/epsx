// Permission Template Plan Management Handlers
// Manages plans using permission templates instead of tier-based logic

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

use crate::web::auth::AppState;
use crate::auth::permissions::derive_tier_from_ranking_limit;

/// Helper function to derive display tier from permissions for UI compatibility
fn derive_display_tier_from_permissions(permissions: &[String]) -> String {
    // Extract ranking limit from permissions
    for perm in permissions {
        if let Some(limit_str) = perm.strip_prefix("epsx:rankings:view:") {
            if limit_str == "unlimited" {
                return "ENTERPRISE".to_string();
            }
            if let Ok(limit) = limit_str.parse::<i32>() {
                return derive_tier_from_ranking_limit(limit);
            }
        }
    }
    
    // Check for wildcard permissions
    if permissions.iter().any(|p| p == "epsx:*:*" || p == "admin:*:*") {
        return "ENTERPRISE".to_string();
    }
    
    "FREE".to_string() // Default fallback
}

/// Get permissions from template name (mock implementation)
fn get_permissions_from_template(template_name: &str) -> Vec<String> {
    match template_name {
        "Free Template" => vec![
            "epsx:rankings:view:3".to_string(),
            "epsx:trading:basic".to_string(),
            "epsx:portfolio:view".to_string(),
        ],
        "Bronze Template" => vec![
            "epsx:rankings:view:5".to_string(),
            "epsx:trading:basic".to_string(),
            "epsx:portfolio:view".to_string(),
            "epsx:portfolio:history".to_string(),
        ],
        "Silver Template" => vec![
            "epsx:rankings:view:25".to_string(),
            "epsx:trading:basic".to_string(),
            "epsx:trading:advanced".to_string(),
            "epsx:portfolio:view".to_string(),
            "epsx:analytics:basic".to_string(),
        ],
        "Gold Template" => vec![
            "epsx:rankings:view:50".to_string(),
            "epsx:trading:premium".to_string(),
            "epsx:portfolio:tools".to_string(),
            "epsx:analytics:advanced".to_string(),
        ],
        "Platinum Template" => vec![
            "epsx:rankings:view:100".to_string(),
            "epsx:trading:premium".to_string(),
            "epsx:analytics:premium".to_string(),
            "epsx:research:reports".to_string(),
            "epsx:dashboards:custom".to_string(),
        ],
        "Enterprise Template" => vec![
            "epsx:rankings:view:unlimited".to_string(),
            "epsx:*:*".to_string(),
            "epsx-pay:*:*".to_string(),
            "epsx-token:*:*".to_string(),
        ],
        _ => vec!["epsx:rankings:view:3".to_string()], // Default to free
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
    pub permission_template_name: String, // e.g., "Bronze Template", "Enterprise Template"
    pub current_price: Decimal,
    pub currency: String,
    pub target_audience: String,
    pub billing_model: String,
    pub permissions: Vec<String>, // Direct permission array
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct PermissionTemplateRequest {
    pub name: String,
    pub description: Option<String>,
    pub permissions: Vec<String>,
    pub display_tier: String, // For UI compatibility: "BRONZE", "SILVER", etc.
}

#[derive(Debug, Serialize)]
pub struct PlanResponse {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub permission_template_name: String,
    pub current_price: Decimal,
    pub currency: String,
    pub target_audience: String,
    pub billing_model: String,
    pub display_tier: String, // Derived from permissions for UI
    pub is_active: bool,
    pub permissions: Vec<String>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
    pub subscriber_count: u64,
    pub revenue_last_30_days: Decimal,
}

#[derive(Debug, Serialize)]
pub struct PermissionTemplateResponse {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub permissions: Vec<String>,
    pub display_tier: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

/// Create permission template-based plan
pub async fn create_plan_handler(
    State(_state): State<AppState>,
    Json(request): Json<CreatePlanRequest>,
) -> Result<JsonResponse<PlanResponse>, StatusCode> {
    let plan_id = rand::random::<i32>().abs();
    
    // Derive display tier from permissions for UI compatibility
    let display_tier = derive_display_tier_from_permissions(&request.permissions);
    
    let plan_response = PlanResponse {
        id: plan_id,
        name: request.name,
        description: request.description,
        permission_template_name: request.permission_template_name,
        current_price: request.current_price,
        currency: request.currency,
        target_audience: request.target_audience,
        billing_model: request.billing_model,
        display_tier,
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
        permission_template = %plan_response.permission_template_name,
        display_tier = %plan_response.display_tier,
        "Permission template plan created"
    );

    Ok(JsonResponse(plan_response))
}

/// List permission template-based plans
pub async fn list_plans_handler(
    State(_state): State<AppState>,
    Query(_query): Query<HashMap<String, String>>,
) -> Result<JsonResponse<serde_json::Value>, StatusCode> {
    let plans = vec![
        serde_json::json!({
            "id": 1,
            "name": "Bronze Plan",
            "description": "Enhanced access with basic features",
            "permission_template_name": "Bronze Template",
            "current_price": "9.99",
            "currency": "USD",
            "target_audience": "web_users",
            "billing_model": "subscription",
            "display_tier": "BRONZE",
            "permissions": ["epsx:rankings:view:5", "epsx:trading:basic", "epsx:portfolio:view"],
            "is_active": true,
            "subscriber_count": 150,
            "revenue_last_30_days": "1499.85"
        }),
        serde_json::json!({
            "id": 2,
            "name": "Gold Plan", 
            "description": "VIP access with premium features",
            "permission_template_name": "Gold Template",
            "current_price": "49.99",
            "currency": "USD",
            "target_audience": "power_users",
            "billing_model": "subscription", 
            "display_tier": "GOLD",
            "permissions": ["epsx:rankings:view:50", "epsx:trading:premium", "epsx:analytics:advanced"],
            "is_active": true,
            "subscriber_count": 75,
            "revenue_last_30_days": "3749.25"
        }),
        serde_json::json!({
            "id": 3,
            "name": "Enterprise Plan", 
            "description": "Unlimited access for enterprise customers",
            "permission_template_name": "Enterprise Template",
            "current_price": "199.99",
            "currency": "USD",
            "target_audience": "enterprise",
            "billing_model": "subscription", 
            "display_tier": "ENTERPRISE",
            "permissions": ["epsx:*:*", "epsx-pay:*:*", "epsx-token:*:*"],
            "is_active": true,
            "subscriber_count": 25,
            "revenue_last_30_days": "4999.75"
        })
    ];

    Ok(JsonResponse(serde_json::json!({
        "success": true,
        "data": {
            "plans": plans,
            "total_count": 3,
            "has_more": false
        },
        "message": "Permission template plans retrieved successfully"
    })))
}

/// Get permission template-based plan details
pub async fn get_plan_handler(
    State(_state): State<AppState>,
    Path(plan_id): Path<i32>,
) -> Result<JsonResponse<PlanResponse>, StatusCode> {
    // Mock plan based on ID
    let (name, template_name, permissions, price, tier) = match plan_id {
        1 => (
            "Bronze Plan".to_string(),
            "Bronze Template".to_string(),
            vec!["epsx:rankings:view:5".to_string(), "epsx:trading:basic".to_string()],
            Decimal::new(999, 2), // 9.99
            "BRONZE".to_string()
        ),
        2 => (
            "Gold Plan".to_string(),
            "Gold Template".to_string(),
            vec!["epsx:rankings:view:50".to_string(), "epsx:trading:premium".to_string()],
            Decimal::new(4999, 2), // 49.99
            "GOLD".to_string()
        ),
        _ => (
            "Enterprise Plan".to_string(),
            "Enterprise Template".to_string(),
            vec!["epsx:*:*".to_string(), "epsx-pay:*:*".to_string()],
            Decimal::new(19999, 2), // 199.99
            "ENTERPRISE".to_string()
        )
    };
    
    let plan_response = PlanResponse {
        id: plan_id,
        name,
        description: Some("Permission template-based plan".to_string()),
        permission_template_name: template_name,
        current_price: price,
        currency: "USD".to_string(),
        target_audience: "web_users".to_string(),
        billing_model: "subscription".to_string(),
        display_tier: tier,
        is_active: true,
        permissions,
        metadata: Some(serde_json::json!({"permission_based": true})),
        created_at: Utc::now(),
        updated_at: Some(Utc::now()),
        subscriber_count: 25,
        revenue_last_30_days: Decimal::new(74975, 2), // 749.75
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
    pub user_id: String,
    pub plan_id: i32,
    pub permission_template_name: String, // e.g., "Gold Template"
    pub access_context: String,
    pub api_key_name: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub auto_renew: bool,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
pub struct SubscriptionResponse {
    pub id: String,
    pub user_id: String,
    pub plan_id: i32,
    pub permission_template_name: String,
    pub permissions_granted: Vec<String>,
    pub display_tier: String,
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
    
    // Get permissions from template (mock implementation)
    let permissions_granted = get_permissions_from_template(&request.permission_template_name);
    let display_tier = derive_display_tier_from_permissions(&permissions_granted);
    
    // Generate quota limits from permissions
    let quota_limits = generate_quota_from_permissions(&permissions_granted);
    
    let response = SubscriptionResponse {
        id: subscription_id,
        user_id: request.user_id,
        plan_id: request.plan_id,
        permission_template_name: request.permission_template_name,
        permissions_granted,
        display_tier,
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
        user_id = %response.user_id,
        template = %response.permission_template_name,
        tier = %response.display_tier,
        "Permission template subscription created"
    );
    
    Ok(JsonResponse(response))
}