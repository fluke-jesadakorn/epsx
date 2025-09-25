use axum::{
    response::Json,
    http::StatusCode,
};
use serde_json::{json, Value};

/// Get public pricing plans (no authentication required)
/// GET /api/v1/public/plans
pub async fn get_public_plans() -> Result<Json<Value>, StatusCode> {
    // Static plans data that matches what the frontend expects
    let plans = vec![
        json!({
            "id": "1",
            "name": "Free Plan",
            "plan_type": "FREE",
            "current_price": "0.00",
            "currency": "USD",
            "billing_cycle": "monthly",
            "features": [
                "Basic analytics view",
                "5 stock limit",
                "Email notifications"
            ],
            "permissions": [
                "epsx:analytics:view"
            ],
            "is_active": true,
            "display_order": 1
        }),
        json!({
            "id": "2", 
            "name": "Pro Plan",
            "plan_type": "PRO",
            "current_price": "29.99",
            "currency": "USD",
            "billing_cycle": "monthly",
            "features": [
                "Advanced analytics",
                "50 stock limit", 
                "Real-time data",
                "Export functionality",
                "Priority support"
            ],
            "permissions": [
                "epsx:analytics:view",
                "epsx:analytics:export",
                "epsx:analytics:advanced",
                "epsx:rankings:view:50"
            ],
            "is_active": true,
            "display_order": 2
        }),
        json!({
            "id": "3",
            "name": "Premium Plan", 
            "plan_type": "PREMIUM",
            "current_price": "79.99",
            "currency": "USD",
            "billing_cycle": "monthly",
            "features": [
                "Premium analytics",
                "Unlimited stocks",
                "Real-time data",
                "Advanced exports",
                "API access",
                "24/7 support"
            ],
            "permissions": [
                "epsx:analytics:view",
                "epsx:analytics:export",
                "epsx:analytics:advanced", 
                "epsx:analytics:premium",
                "epsx:rankings:view:unlimited",
                "epsx:api:access"
            ],
            "is_active": true,
            "display_order": 3
        }),
        json!({
            "id": "4",
            "name": "Enterprise Plan",
            "plan_type": "ENTERPRISE", 
            "current_price": "199.99",
            "currency": "USD",
            "billing_cycle": "monthly",
            "features": [
                "Full platform access",
                "Unlimited everything",
                "Custom integrations",
                "Dedicated support",
                "White-label options"
            ],
            "permissions": [
                "epsx:*:*"
            ],
            "is_active": true,
            "display_order": 4
        })
    ];

    Ok(Json(json!({
        "success": true,
        "data": plans,
        "message": "Public pricing plans retrieved successfully"
    })))
}