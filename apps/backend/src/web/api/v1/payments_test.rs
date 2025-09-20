/// Testing utilities for payments API
/// This file provides test endpoints and utilities to verify the payment flow

use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::Json as ResponseJson,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::infrastructure::container::AppContainer;

#[derive(Debug, Serialize)]
pub struct PaymentTestResult {
    pub success: bool,
    pub test_name: String,
    pub message: String,
    pub details: Option<serde_json::Value>,
}

/// Test endpoint to verify payment system components
pub async fn test_payment_system(
    State(container): State<Arc<AppContainer>>,
) -> Result<ResponseJson<Vec<PaymentTestResult>>, StatusCode> {
    let mut results = Vec::new();

    // Test 1: Database connection
    let db_test = test_database_connection(container.db_pool()).await;
    results.push(db_test);

    // Test 2: Plans API
    let plans_test = test_plans_api(container.clone()).await;
    results.push(plans_test);

    // Test 3: Subscription activation handler
    let activation_test = test_subscription_activation(container.clone()).await;
    results.push(activation_test);

    Ok(ResponseJson(results))
}

async fn test_database_connection(db_pool: Arc<sqlx::PgPool>) -> PaymentTestResult {
    match sqlx::query!("SELECT 1 as test").fetch_one(db_pool.as_ref()).await {
        Ok(_) => PaymentTestResult {
            success: true,
            test_name: "Database Connection".to_string(),
            message: "Database connection successful".to_string(),
            details: None,
        },
        Err(e) => PaymentTestResult {
            success: false,
            test_name: "Database Connection".to_string(),
            message: format!("Database connection failed: {}", e),
            details: None,
        },
    }
}

async fn test_plans_api(container: Arc<AppContainer>) -> PaymentTestResult {
    // Test if plans can be fetched
    match sqlx::query!("SELECT id, name, plan_type FROM pricing_plans LIMIT 1")
        .fetch_optional(container.db_pool().as_ref())
        .await
    {
        Ok(Some(plan)) => PaymentTestResult {
            success: true,
            test_name: "Plans API".to_string(),
            message: "Plans data available".to_string(),
            details: Some(serde_json::json!({
                "sample_plan": {
                    "id": plan.id,
                    "name": plan.name,
                    "type": plan.plan_type
                }
            })),
        },
        Ok(None) => PaymentTestResult {
            success: false,
            test_name: "Plans API".to_string(),
            message: "No plans found in database".to_string(),
            details: None,
        },
        Err(e) => PaymentTestResult {
            success: false,
            test_name: "Plans API".to_string(),
            message: format!("Plans query failed: {}", e),
            details: None,
        },
    }
}

async fn test_subscription_activation(container: Arc<AppContainer>) -> PaymentTestResult {
    // Test if subscription activation table exists and is accessible
    match sqlx::query!("SELECT COUNT(*) as count FROM user_subscription_activations")
        .fetch_one(container.db_pool().as_ref())
        .await
    {
        Ok(result) => PaymentTestResult {
            success: true,
            test_name: "Subscription Activation".to_string(),
            message: "Subscription activation system ready".to_string(),
            details: Some(serde_json::json!({
                "activations_count": result.count
            })),
        },
        Err(e) => PaymentTestResult {
            success: false,
            test_name: "Subscription Activation".to_string(),
            message: format!("Subscription activation table issue: {}", e),
            details: None,
        },
    }
}

/// Test endpoint for mock payment confirmation
#[derive(Debug, Deserialize)]
pub struct MockPaymentRequest {
    pub plan_id: i32,
    pub user_email: String,
    pub test_mode: bool,
}

pub async fn test_mock_payment(
    State(container): State<Arc<AppContainer>>,
    Json(request): Json<MockPaymentRequest>,
) -> Result<ResponseJson<PaymentTestResult>, StatusCode> {
    if !request.test_mode {
        return Ok(ResponseJson(PaymentTestResult {
            success: false,
            test_name: "Mock Payment".to_string(),
            message: "Test mode must be enabled".to_string(),
            details: None,
        }));
    }

    // Generate mock transaction hash
    let mock_tx_hash = format!("0x{:064x}", rand::random::<u64>());

    // Verify plan exists
    match sqlx::query!("SELECT name FROM pricing_plans WHERE id = $1", request.plan_id)
        .fetch_optional(container.db_pool().as_ref())
        .await
    {
        Ok(Some(plan)) => Ok(ResponseJson(PaymentTestResult {
            success: true,
            test_name: "Mock Payment".to_string(),
            message: "Mock payment test successful".to_string(),
            details: Some(serde_json::json!({
                "plan_id": request.plan_id,
                "plan_name": plan.name,
                "mock_transaction_hash": mock_tx_hash,
                "user_email": request.user_email,
                "note": "This is a test transaction - no real payment processed"
            })),
        })),
        Ok(None) => Ok(ResponseJson(PaymentTestResult {
            success: false,
            test_name: "Mock Payment".to_string(),
            message: format!("Plan {} not found", request.plan_id),
            details: None,
        })),
        Err(e) => Ok(ResponseJson(PaymentTestResult {
            success: false,
            test_name: "Mock Payment".to_string(),
            message: format!("Database error: {}", e),
            details: None,
        })),
    }
}

/// Create test router for payment system testing
pub fn create_payment_test_router(container: Arc<AppContainer>) -> Router {
    Router::new()
        .route("/test-system", get(test_payment_system))
        .route("/test-mock-payment", post(test_mock_payment))
        .with_state(container)
}