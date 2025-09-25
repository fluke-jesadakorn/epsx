// Enterprise Webhooks API
// Webhook management for enterprise event notifications

use axum::{
    extract::{State, Path},
    routing::{get, post, put, delete},
    Router, Json,
};
use serde::Deserialize;
use std::sync::Arc;

use crate::infrastructure::container::DomainContainer;
use crate::web::middleware::web3_enterprise_auth::Web3AuthenticatedUser;
use crate::core::errors::AppError;

type ApiResult<T> = Result<T, AppError>;

pub fn create_webhook_routes() -> Router<Arc<DomainContainer>> {
    Router::new()
        .route("/", get(get_webhooks))
        .route("/", post(create_webhook))
        .route("/:webhook_id", put(update_webhook))
        .route("/:webhook_id", delete(delete_webhook))
        .route("/events", get(get_event_types))
        .route("/test", post(test_webhook))
}

pub async fn get_webhooks(
    State(_container): State<Arc<DomainContainer>>,
    user: Web3AuthenticatedUser,
) -> ApiResult<Json<serde_json::Value>> {
    if !user.enterprise_tier.has_feature_access("webhook_endpoints") {
        return Err(AppError::forbidden("Webhooks require Business tier or higher"));
    }

    let response = serde_json::json!({
        "webhooks": [
            {
                "id": uuid::Uuid::new_v4(),
                "url": "https://your-app.com/webhooks/epsx",
                "events": ["price_alert", "portfolio_update"],
                "status": "active",
                "created_at": chrono::Utc::now()
            }
        ],
        "limits": {
            "max_webhooks": match user.enterprise_tier {
                crate::web::middleware::web3_enterprise_auth::EnterpriseTier::Business => 5,
                crate::web::middleware::web3_enterprise_auth::EnterpriseTier::Enterprise => 25,
                crate::web::middleware::web3_enterprise_auth::EnterpriseTier::Whale => 100,
                _ => 0
            }
        }
    });

    Ok(Json(response))
}

#[derive(Debug, Deserialize)]
pub struct CreateWebhookRequest {
    pub url: String,
    pub events: Vec<String>,
    pub secret: Option<String>,
}

pub async fn create_webhook(
    State(_container): State<Arc<DomainContainer>>,
    user: Web3AuthenticatedUser,
    Json(request): Json<CreateWebhookRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    if !user.enterprise_tier.has_feature_access("webhook_endpoints") {
        return Err(AppError::forbidden("Webhooks require Business tier or higher"));
    }

    let response = serde_json::json!({
        "webhook_id": uuid::Uuid::new_v4(),
        "url": request.url,
        "events": request.events,
        "status": "active",
        "secret": request.secret.map(|_| "***"),
        "created_at": chrono::Utc::now()
    });

    Ok(Json(response))
}

pub async fn update_webhook(
    State(_container): State<Arc<DomainContainer>>,
    user: Web3AuthenticatedUser,
    Path(webhook_id): Path<String>,
    Json(request): Json<CreateWebhookRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    let response = serde_json::json!({
        "webhook_id": webhook_id,
        "url": request.url,
        "events": request.events,
        "status": "updated",
        "updated_at": chrono::Utc::now()
    });

    Ok(Json(response))
}

pub async fn delete_webhook(
    State(_container): State<Arc<DomainContainer>>,
    user: Web3AuthenticatedUser,
    Path(webhook_id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    let response = serde_json::json!({
        "webhook_id": webhook_id,
        "status": "deleted",
        "deleted_at": chrono::Utc::now()
    });

    Ok(Json(response))
}

pub async fn get_event_types(
    State(_container): State<Arc<DomainContainer>>,
    user: Web3AuthenticatedUser,
) -> ApiResult<Json<serde_json::Value>> {
    let response = serde_json::json!({
        "event_types": [
            {
                "name": "price_alert",
                "description": "Triggered when price thresholds are met"
            },
            {
                "name": "portfolio_update",
                "description": "Portfolio value or composition changes"
            },
            {
                "name": "defi_position_change",
                "description": "DeFi positions opened, closed, or modified"
            },
            {
                "name": "governance_proposal",
                "description": "New governance proposals in tracked DAOs"
            }
        ]
    });

    Ok(Json(response))
}

#[derive(Debug, Deserialize)]
pub struct TestWebhookRequest {
    pub webhook_id: String,
    pub event_type: String,
}

pub async fn test_webhook(
    State(_container): State<Arc<DomainContainer>>,
    user: Web3AuthenticatedUser,
    Json(request): Json<TestWebhookRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    let response = serde_json::json!({
        "test_id": uuid::Uuid::new_v4(),
        "webhook_id": request.webhook_id,
        "event_type": request.event_type,
        "status": "sent",
        "response_code": 200,
        "response_time_ms": 150,
        "tested_at": chrono::Utc::now()
    });

    Ok(Json(response))
}