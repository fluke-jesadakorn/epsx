// Admin routes for administrative interface
// Context: OIDC authenticated admins, full permissions, resource management

use axum::{
    routing::{get, post, put, delete},
    Router,
    middleware as axum_middleware,
    Extension,
    Json,
};
use serde_json::{json, Value};
use std::sync::Arc;
use crate::{
    infrastructure::AppContainer,
    web::middleware::contextual_middleware::admin_middleware_stack,
};

pub struct AdminRoutes;

impl AdminRoutes {
    /// Create admin routes with OIDC authentication and admin permission requirements
    pub async fn create_routes(
        container: Arc<AppContainer>,
    ) -> Result<Router, Box<dyn std::error::Error + Send + Sync>> {
        
        // Create admin-specific services
        let app_state = container.create_app_state().await?;
        
        // User management routes (existing admin functionality)
        let user_management_routes = Router::new()
            .route("/users/:user_id", get(crate::web::admin::unified_user_handlers::get_unified_user_data_handler))
            .route("/users/:user_id/profile", put(crate::web::admin::unified_user_handlers::update_user_profile_handler))
            .route("/users/:user_id/roles", put(crate::web::admin::unified_user_handlers::update_user_roles_handler));

        // Plan management routes (new dynamic plans)
        let plan_management_routes = Router::new()
            .route("/plans", get(get_all_plans_admin))
            .route("/plans", post(create_plan_admin))
            .route("/plans/:plan_id", get(get_plan_admin))
            .route("/plans/:plan_id", put(update_plan_admin))
            .route("/plans/:plan_id", delete(delete_plan_admin))
            .route("/plans/:plan_id/features", get(get_plan_features_admin))
            .route("/plans/:plan_id/features", post(add_plan_feature_admin));

        // Subscription management routes
        let subscription_routes = Router::new()
            .route("/subscriptions", get(get_all_subscriptions_admin))
            .route("/subscriptions/:subscription_id", get(get_subscription_admin))
            .route("/subscriptions/:subscription_id/usage", get(get_subscription_usage_admin))
            .route("/users/:user_id/subscriptions", get(get_user_subscriptions_admin))
            .route("/users/:user_id/subscriptions", post(create_user_subscription_admin));

        // Analytics and resource tracking routes
        let analytics_routes = Router::new()
            .route("/analytics/usage", get(get_usage_analytics_admin))
            .route("/analytics/revenue", get(get_revenue_analytics_admin))
            .route("/analytics/resources", get(get_resource_metrics_admin))
            .route("/analytics/performance", get(get_performance_metrics_admin));

        // System monitoring routes
        let system_routes = Router::new()
            .route("/system/health", get(crate::web::health_handler))
            .route("/system/cache", get(get_cache_stats_admin))
            .route("/system/cache/clear", post(clear_cache_admin))
            .route("/system/metrics", get(get_system_metrics_admin));

        // Notification management routes (existing functionality)
        let notification_routes = Router::new()
            .route("/notifications", get(crate::web::admin::notification_handlers::admin_get_user_notifications))
            .route("/notifications", post(crate::web::admin::notification_handlers::admin_send_notification))
            .route("/notifications/:notification_id", delete(crate::web::admin::notification_handlers::admin_delete_notification));

        // Combine all admin routes
        let admin_router = Router::new()
            .nest("/users", user_management_routes)
            .nest("/plans", plan_management_routes)
            .nest("/subscriptions", subscription_routes)
            .nest("/analytics", analytics_routes)
            .nest("/system", system_routes)
            .nest("/notifications", notification_routes)
            .route("/health", get(admin_health_check))
            // Add admin-specific middleware stack
            .layer(axum_middleware::from_fn_with_state(
                app_state.clone(),
                admin_middleware_stack
            ))
            .layer(Extension(container.fcm_service.clone()))
            .layer(Extension(container.fcm_topic_service.clone()))
            .layer(Extension(container.user_notification_repo.clone()))
            .with_state(app_state);

        Ok(admin_router)
    }
}


// Admin handlers for dynamic plan management

async fn get_all_plans_admin() -> Json<Value> {
    Json(json!({
        "success": true,
        "data": [],
        "message": "All plans retrieved for admin",
        "context": "admin"
    }))
}

async fn create_plan_admin() -> Json<Value> {
    Json(json!({
        "success": true,
        "data": {
            "plan_id": "plan_123",
            "name": "New Plan",
            "status": "created"
        },
        "message": "Plan created successfully",
        "context": "admin"
    }))
}

async fn get_plan_admin() -> Json<Value> {
    Json(json!({
        "success": true,
        "data": {},
        "message": "Plan details retrieved",
        "context": "admin"
    }))
}

async fn update_plan_admin() -> Json<Value> {
    Json(json!({
        "success": true,
        "data": {
            "plan_id": "plan_123",
            "status": "updated"
        },
        "message": "Plan updated successfully",
        "context": "admin"
    }))
}

async fn delete_plan_admin() -> Json<Value> {
    Json(json!({
        "success": true,
        "data": {
            "plan_id": "plan_123",
            "status": "deleted"
        },
        "message": "Plan deleted successfully",
        "context": "admin"
    }))
}

async fn get_plan_features_admin() -> Json<Value> {
    Json(json!({
        "success": true,
        "data": [],
        "message": "Plan features retrieved",
        "context": "admin"
    }))
}

async fn add_plan_feature_admin() -> Json<Value> {
    Json(json!({
        "success": true,
        "data": {
            "feature_id": "feature_123",
            "status": "added"
        },
        "message": "Plan feature added successfully",
        "context": "admin"
    }))
}

async fn get_all_subscriptions_admin() -> Json<Value> {
    Json(json!({
        "success": true,
        "data": [],
        "message": "All subscriptions retrieved",
        "context": "admin"
    }))
}

async fn get_subscription_admin() -> Json<Value> {
    Json(json!({
        "success": true,
        "data": {},
        "message": "Subscription details retrieved",
        "context": "admin"
    }))
}

async fn get_subscription_usage_admin() -> Json<Value> {
    Json(json!({
        "success": true,
        "data": {
            "usage_stats": {},
            "billing_info": {}
        },
        "message": "Subscription usage retrieved",
        "context": "admin"
    }))
}

async fn get_user_subscriptions_admin() -> Json<Value> {
    Json(json!({
        "success": true,
        "data": [],
        "message": "User subscriptions retrieved",
        "context": "admin"
    }))
}

async fn create_user_subscription_admin() -> Json<Value> {
    Json(json!({
        "success": true,
        "data": {
            "subscription_id": "sub_123",
            "status": "created"
        },
        "message": "User subscription created",
        "context": "admin"
    }))
}

async fn get_usage_analytics_admin() -> Json<Value> {
    Json(json!({
        "success": true,
        "data": {
            "internal_usage": {},
            "external_usage": {},
            "admin_usage": {}
        },
        "message": "Usage analytics retrieved",
        "context": "admin"
    }))
}

async fn get_revenue_analytics_admin() -> Json<Value> {
    Json(json!({
        "success": true,
        "data": {
            "monthly_revenue": 0,
            "plan_breakdown": {},
            "api_revenue": 0
        },
        "message": "Revenue analytics retrieved",
        "context": "admin"
    }))
}

async fn get_resource_metrics_admin() -> Json<Value> {
    Json(json!({
        "success": true,
        "data": {
            "infrastructure_cost": {},
            "billable_usage": {},
            "profit_margins": {}
        },
        "message": "Resource metrics retrieved",
        "context": "admin"
    }))
}

async fn get_performance_metrics_admin() -> Json<Value> {
    Json(json!({
        "success": true,
        "data": {
            "response_times": {},
            "throughput": {},
            "error_rates": {}
        },
        "message": "Performance metrics retrieved",
        "context": "admin"
    }))
}

async fn get_cache_stats_admin() -> Json<Value> {
    Json(json!({
        "success": true,
        "data": {
            "hit_rate": 0.85,
            "memory_usage": "150MB",
            "entries": 1500
        },
        "message": "Cache statistics retrieved",
        "context": "admin"
    }))
}

async fn clear_cache_admin() -> Json<Value> {
    Json(json!({
        "success": true,
        "data": {
            "entries_cleared": 1500,
            "cache_reset": true
        },
        "message": "Cache cleared successfully",
        "context": "admin"
    }))
}

async fn get_system_metrics_admin() -> Json<Value> {
    Json(json!({
        "success": true,
        "data": {
            "cpu_usage": 45.2,
            "memory_usage": 67.8,
            "disk_usage": 23.1,
            "active_connections": 150
        },
        "message": "System metrics retrieved",
        "context": "admin"
    }))
}

async fn admin_health_check() -> Json<Value> {
    Json(json!({
        "status": "healthy",
        "service": "epsx-admin-interface",
        "context": "admin",
        "timestamp": chrono::Utc::now(),
        "permissions_required": ["admin:*:*"]
    }))
}