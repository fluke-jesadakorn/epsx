// External API routes for developer access
// Context: API key authenticated developers, plan-based access, billable usage tracking

use axum::{
    routing::{get, post},
    Router,
    middleware as axum_middleware,
    Extension,
    response::Json,
};
use serde_json::{json, Value};
use std::sync::Arc;
use crate::{
    infrastructure::AppContainer,
    web::middleware::{
        contextual_middleware::external_middleware_stack,
    },
};

pub struct ExternalRoutes;

impl ExternalRoutes {
    /// Create external API routes with API key authentication and plan-based access control
    pub async fn create_routes(
        container: Arc<AppContainer>,
    ) -> Result<Router, Box<dyn std::error::Error + Send + Sync>> {
        
        // Create external-specific services
        let app_state = container.create_app_state().await?;
        
        // API v1 routes - core data endpoints (billable)
        let api_v1_routes = Router::new()
            .route("/plans", get(get_available_plans))
            .route("/plans/:plan_id", get(get_plan_details))
            .route("/analytics/rankings", get(get_analytics_rankings_api))
            .route("/analytics/countries", get(get_available_countries_api))
            .route("/analytics/sectors", get(get_sectors_by_country_api));

        // Developer management routes
        let developer_routes = Router::new()
            .route("/profile", get(get_developer_profile))
            .route("/usage", get(get_api_usage_stats))
            .route("/quota", get(get_api_quota_status));

        // Webhook routes for integrations
        let webhook_routes = Router::new()
            .route("/register", post(register_webhook_endpoint))
            .route("/test", post(test_webhook_delivery));

        // Combine all external routes
        let external_router = Router::new()
            .nest("/v1", api_v1_routes)
            .nest("/developer", developer_routes)
            .nest("/webhooks", webhook_routes)
            .route("/health", get(external_health_check))
            // Add external-specific middleware stack
            .layer(axum_middleware::from_fn_with_state(
                app_state.clone(),
                external_middleware_stack
            ))
            .layer(Extension(container.clone()))
            .with_state(app_state);

        Ok(external_router)
    }
}


// External API handlers (billable endpoints)

async fn get_available_plans() -> Json<Value> {
    Json(json!({
        "success": true,
        "data": [],
        "message": "Available API plans",
        "billable": true
    }))
}

async fn get_plan_details() -> Json<Value> {
    Json(json!({
        "success": true,
        "data": {},
        "message": "Plan details",
        "billable": true
    }))
}

async fn get_analytics_rankings_api() -> Json<Value> {
    // This would call the same analytics service but track as billable usage
    Json(json!({
        "success": true,
        "data": [],
        "message": "Analytics rankings - billable API access",
        "billable": true
    }))
}

async fn get_available_countries_api() -> Json<Value> {
    Json(json!({
        "success": true,
        "data": [],
        "message": "Available countries for analytics",
        "billable": true
    }))
}

async fn get_sectors_by_country_api() -> Json<Value> {
    Json(json!({
        "success": true,
        "data": [],
        "message": "Sectors by country",
        "billable": true
    }))
}

async fn get_developer_profile() -> Json<Value> {
    Json(json!({
        "success": true,
        "data": {
            "developer_id": "dev_123",
            "plan": "api_starter",
            "status": "active"
        },
        "billable": false
    }))
}

async fn get_api_usage_stats() -> Json<Value> {
    Json(json!({
        "success": true,
        "data": {
            "calls_today": 150,
            "calls_this_month": 4500,
            "quota_remaining": 5500
        },
        "billable": false
    }))
}

async fn get_api_quota_status() -> Json<Value> {
    Json(json!({
        "success": true,
        "data": {
            "plan": "api_starter",
            "quota_limit": 10000,
            "quota_used": 4500,
            "quota_remaining": 5500,
            "reset_date": "2024-01-01T00:00:00Z"
        },
        "billable": false
    }))
}

async fn register_webhook_endpoint() -> Json<Value> {
    Json(json!({
        "success": true,
        "data": {
            "webhook_id": "wh_123",
            "url": "https://example.com/webhook",
            "status": "active"
        },
        "billable": false
    }))
}

async fn test_webhook_delivery() -> Json<Value> {
    Json(json!({
        "success": true,
        "data": {
            "test_id": "test_123",
            "status": "delivered",
            "response_code": 200
        },
        "billable": true
    }))
}

async fn external_health_check() -> Json<Value> {
    Json(json!({
        "status": "healthy",
        "service": "epsx-external-api",
        "context": "external",
        "timestamp": chrono::Utc::now(),
        "billable": false
    }))
}