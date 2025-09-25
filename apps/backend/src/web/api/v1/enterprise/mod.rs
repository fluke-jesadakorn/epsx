// Enterprise API Gateway Module
// External enterprise customer integration endpoints

pub mod analytics;
pub mod defi;
pub mod governance;
pub mod dao; // Comprehensive DAO integration
pub mod custom;
pub mod auth;
pub mod billing;
pub mod webhooks;
pub mod marketplace; // Self-service enterprise marketplace

use axum::{
    routing::{get, post, put, delete},
    Router,
    middleware,
};
use std::sync::Arc;

use crate::infrastructure::container::DomainContainer;
use crate::web::middleware::web3_enterprise_auth::web3_enterprise_auth_middleware;

/// Create the complete enterprise API router
pub fn create_enterprise_router(container: Arc<DomainContainer>) -> Router {
    Router::new()
        // Authentication endpoints (public)
        .nest("/auth", auth::create_auth_routes())
        
        // Core enterprise features (protected)
        .nest("/analytics", analytics::create_analytics_routes())
        .nest("/defi", defi::create_defi_routes())
        .nest("/governance", governance::create_governance_routes())
        .nest("/dao", dao::create_dao_router())
        .nest("/custom", custom::create_custom_routes())
        .nest("/billing", billing::create_billing_routes())
        .nest("/webhooks", webhooks::create_webhook_routes())
        .nest("/marketplace", marketplace::create_marketplace_routes())
        
        // Health check for enterprise API
        .route("/health", get(enterprise_health))
        .route("/status", get(enterprise_status))
        .route("/tiers", get(get_enterprise_tiers))
        
        // Apply Web3 enterprise authentication middleware to all protected routes
        .layer(middleware::from_fn(web3_enterprise_auth_middleware))
        .with_state(container)
}

/// Enterprise API health check
pub async fn enterprise_health() -> axum::Json<serde_json::Value> {
    axum::Json(serde_json::json!({
        "status": "healthy",
        "api_version": "v1",
        "service": "enterprise-api",
        "timestamp": chrono::Utc::now(),
        "uptime": "operational"
    }))
}

/// Enterprise API status with tier information
pub async fn enterprise_status() -> axum::Json<serde_json::Value> {
    axum::Json(serde_json::json!({
        "api_status": "operational",
        "supported_chains": [
            "ethereum",
            "polygon", 
            "arbitrum",
            "optimism",
            "base",
            "bsc"
        ],
        "supported_tiers": [
            "starter",
            "business", 
            "enterprise",
            "whale"
        ],
        "features": {
            "real_time_data": true,
            "custom_integrations": true,
            "webhook_support": true,
            "white_label": true,
            "multi_chain": true,
            "governance_integration": true,
            "marketplace": true,
            "self_service_billing": true,
            "professional_services": true
        },
        "rate_limits": {
            "starter": "100/minute",
            "business": "1000/minute", 
            "enterprise": "10000/minute",
            "whale": "unlimited"
        },
        "timestamp": chrono::Utc::now()
    }))
}

/// Get available enterprise tiers and their requirements
pub async fn get_enterprise_tiers() -> axum::Json<serde_json::Value> {
    axum::Json(serde_json::json!({
        "tiers": {
            "starter": {
                "name": "Starter",
                "description": "Basic enterprise features for small teams",
                "minimum_token_value_usd": 1000,
                "rate_limit_per_minute": 100,
                "concurrent_requests": 5,
                "features": [
                    "basic_analytics",
                    "market_data", 
                    "portfolio_tracking"
                ],
                "support": "community",
                "data_retention_days": 30
            },
            "business": {
                "name": "Business", 
                "description": "Advanced features for growing businesses",
                "minimum_token_value_usd": 10000,
                "alternative_nft_required": true,
                "rate_limit_per_minute": 1000,
                "concurrent_requests": 25,
                "features": [
                    "real_time_data",
                    "advanced_charts",
                    "custom_alerts",
                    "webhook_endpoints"
                ],
                "support": "standard",
                "data_retention_days": 90
            },
            "enterprise": {
                "name": "Enterprise",
                "description": "Full enterprise features with custom integrations", 
                "minimum_token_value_usd": 100000,
                "dao_membership_required": true,
                "rate_limit_per_minute": 10000,
                "concurrent_requests": 100,
                "features": [
                    "custom_integration",
                    "dedicated_support",
                    "white_label_basic",
                    "priority_processing",
                    "compliance_tools"
                ],
                "support": "priority",
                "data_retention_days": 365
            },
            "whale": {
                "name": "Whale",
                "description": "Unlimited access with custom infrastructure",
                "minimum_token_value_usd": 1000000,
                "rate_limit_per_minute": "unlimited",
                "concurrent_requests": 500,
                "features": [
                    "unlimited_requests",
                    "custom_infrastructure", 
                    "white_label_full",
                    "dedicated_support",
                    "priority_support",
                    "custom_features"
                ],
                "support": "dedicated",
                "data_retention_days": "unlimited"
            }
        },
        "compliance_requirements": {
            "starter": ["region_compliant"],
            "business": ["region_compliant", "kyc_verified"],
            "enterprise": ["region_compliant", "kyc_verified", "accredited_investor", "aml_cleared"],
            "whale": ["region_compliant", "kyc_verified", "accredited_investor", "aml_cleared", "sox_compliant"]
        },
        "supported_tokens": {
            "ethereum": ["USDC", "USDT", "WETH", "DAI"],
            "polygon": ["USDC", "USDT", "WMATIC", "DAI"],
            "arbitrum": ["USDC", "USDT", "WETH", "ARB"],
            "optimism": ["USDC", "USDT", "WETH", "OP"],
            "base": ["USDC", "WETH", "cbETH"],
            "bsc": ["USDC", "USDT", "WBNB", "CAKE"]
        },
        "timestamp": chrono::Utc::now()
    }))
}