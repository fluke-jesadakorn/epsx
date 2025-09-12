// Simplified Plan Management Handlers - Mock Implementation
// Provides API endpoints for frontend integration while avoiding database schema issues

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

// Request/Response DTOs (reusing existing ones)

#[derive(Debug, Deserialize)]
pub struct CreatePlanRequest {
    pub name: String,
    pub description: Option<String>,
    pub plan_type: String,
    pub current_price: Decimal,
    pub currency: String,
    pub target_audience: String,
    pub billing_model: String,
    pub plan_category: String,
    pub features: Vec<PlanFeatureRequest>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct PlanFeatureRequest {
    pub context_name: String,
    pub feature_key: String,
    pub feature_config: serde_json::Value,
    pub resource_cost: Decimal,
    pub is_active: bool,
}

#[derive(Debug, Serialize)]
pub struct PlanResponse {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub plan_type: String,
    pub current_price: Decimal,
    pub currency: String,
    pub target_audience: String,
    pub billing_model: String,
    pub plan_category: String,
    pub is_active: bool,
    pub features: Vec<PlanFeatureResponse>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
    pub subscriber_count: u64,
    pub revenue_last_30_days: Decimal,
}

#[derive(Debug, Serialize)]
pub struct PlanFeatureResponse {
    pub id: i32,
    pub context_name: String,
    pub feature_key: String,
    pub feature_config: serde_json::Value,
    pub resource_cost: Decimal,
    pub is_active: bool,
}

/// Mock plan creation handler
pub async fn create_plan_handler(
    State(_state): State<AppState>,
    Json(request): Json<CreatePlanRequest>,
) -> Result<JsonResponse<PlanResponse>, StatusCode> {
    let plan_id = rand::random::<i32>().abs();
    
    let features: Vec<PlanFeatureResponse> = request.features.iter().enumerate()
        .map(|(index, feature)| PlanFeatureResponse {
            id: index as i32,
            context_name: feature.context_name.clone(),
            feature_key: feature.feature_key.clone(),
            feature_config: feature.feature_config.clone(),
            resource_cost: feature.resource_cost,
            is_active: feature.is_active,
        })
        .collect();
    
    let plan_response = PlanResponse {
        id: plan_id,
        name: request.name,
        description: request.description,
        plan_type: request.plan_type,
        current_price: request.current_price,
        currency: request.currency,
        target_audience: request.target_audience,
        billing_model: request.billing_model,
        plan_category: request.plan_category,
        is_active: true,
        features,
        metadata: request.metadata,
        created_at: Utc::now(),
        updated_at: Some(Utc::now()),
        subscriber_count: 0,
        revenue_last_30_days: Decimal::ZERO,
    };

    tracing::info!(
        plan_id = plan_id,
        plan_name = %plan_response.name,
        "Mock plan created"
    );

    Ok(JsonResponse(plan_response))
}

/// Mock plan list handler  
pub async fn list_plans_handler(
    State(_state): State<AppState>,
    Query(_query): Query<HashMap<String, String>>,
) -> Result<JsonResponse<serde_json::Value>, StatusCode> {
    let plans = vec![
        serde_json::json!({
            "id": 1,
            "name": "Starter Plan",
            "description": "Basic plan for beginners",
            "plan_type": "subscription",
            "current_price": "29.99",
            "currency": "USD",
            "target_audience": "web_users",
            "billing_model": "subscription",
            "plan_category": "standard",
            "is_active": true,
            "subscriber_count": 150,
            "revenue_last_30_days": "4499.85"
        }),
        serde_json::json!({
            "id": 2,
            "name": "Professional Plan", 
            "description": "Advanced features for professionals",
            "plan_type": "subscription",
            "current_price": "99.99",
            "currency": "USD",
            "target_audience": "api_developers",
            "billing_model": "subscription", 
            "plan_category": "api",
            "is_active": true,
            "subscriber_count": 75,
            "revenue_last_30_days": "7499.25"
        })
    ];

    Ok(JsonResponse(serde_json::json!({
        "success": true,
        "data": {
            "plans": plans,
            "total_count": 2,
            "has_more": false
        },
        "message": "Plans retrieved successfully"
    })))
}

/// Mock plan details handler
pub async fn get_plan_handler(
    State(_state): State<AppState>,
    Path(plan_id): Path<i32>,
) -> Result<JsonResponse<PlanResponse>, StatusCode> {
    let plan_response = PlanResponse {
        id: plan_id,
        name: "Mock Plan".to_string(),
        description: Some("This is a mock plan for testing".to_string()),
        plan_type: "subscription".to_string(),
        current_price: Decimal::new(2999, 2), // 29.99
        currency: "USD".to_string(),
        target_audience: "web_users".to_string(),
        billing_model: "subscription".to_string(),
        plan_category: "standard".to_string(),
        is_active: true,
        features: vec![
            PlanFeatureResponse {
                id: 1,
                context_name: "web_app".to_string(),
                feature_key: "api_calls".to_string(),
                feature_config: serde_json::json!({"limit": 1000}),
                resource_cost: Decimal::new(1, 3), // 0.001
                is_active: true,
            }
        ],
        metadata: Some(serde_json::json!({"tier": "basic"})),
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
    pub access_context: String,
    pub api_key: Option<String>,
    pub api_key_name: Option<String>,
    pub status: String,
    pub expires_at: Option<DateTime<Utc>>,
    pub auto_renew: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub metadata: Option<serde_json::Value>,
    pub plan_name: String,
    pub current_usage: serde_json::Value,
    pub quota_limits: serde_json::Value,
}

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
    
    let response = SubscriptionResponse {
        id: subscription_id,
        user_id: request.user_id,
        plan_id: request.plan_id,
        access_context: request.access_context,
        api_key,
        api_key_name: request.api_key_name,
        status: "active".to_string(),
        expires_at: request.expires_at,
        auto_renew: request.auto_renew,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        metadata: request.metadata,
        plan_name: "Mock Plan".to_string(),
        current_usage: serde_json::json!({"api_calls": 150}),
        quota_limits: serde_json::json!({"api_calls": 1000}),
    };
    
    Ok(JsonResponse(response))
}