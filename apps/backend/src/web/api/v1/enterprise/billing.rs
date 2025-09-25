// Enterprise Billing API
// Token-based billing and payment management with comprehensive Web3 integration

use axum::{
    extract::{State, Query},
    routing::{get, post},
    Router, Json,
    http::StatusCode,
    response::Json as ResponseJson,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::collections::HashMap;
use chrono::{DateTime, Utc};

use crate::infrastructure::container::DomainContainer;
use crate::web::middleware::web3_enterprise_auth::Web3AuthenticatedUser;
use crate::core::errors::AppError;
use crate::auth::{
    Web3BillingService, Web3BillingResult, BillingStatus, SubscriptionTier,
    PaymentRecord, ActiveSubscription, Web3PaymentMethod, PaymentAmount,
    BillingType, UsageMetrics, CostSummary, Web3BillingConfig
};

type ApiResult<T> = Result<T, AppError>;

// Enhanced API Request/Response Types

#[derive(Debug, Deserialize)]
pub struct BillingStatusQuery {
    pub include_history: Option<bool>,
    pub include_usage: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct BillingStatusResponse {
    pub billing_result: Web3BillingResult,
    pub tier_compatibility: HashMap<String, bool>,
    pub upgrade_recommendations: Vec<TierUpgradeRecommendation>,
}

#[derive(Debug, Serialize)]
pub struct TierUpgradeRecommendation {
    pub from_tier: SubscriptionTier,
    pub to_tier: SubscriptionTier,
    pub cost_difference: PaymentAmount,
    pub additional_features: Vec<String>,
    pub estimated_savings: Option<f64>,
    pub recommended_reason: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateSubscriptionRequest {
    pub tier: SubscriptionTier,
    pub billing_type: BillingType,
    pub payment_method: Web3PaymentMethod,
    pub auto_renewal: bool,
    pub promo_code: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CreateSubscriptionResponse {
    pub subscription: ActiveSubscription,
    pub initial_payment_required: Option<PaymentAmount>,
    pub payment_instructions: PaymentInstructions,
}

#[derive(Debug, Serialize)]
pub struct PaymentInstructions {
    pub payment_address: String,
    pub exact_amount: PaymentAmount,
    pub payment_deadline: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct ProcessWeb3PaymentRequest {
    pub payment_method: Web3PaymentMethod,
    pub amount: PaymentAmount,
    pub subscription_tier: SubscriptionTier,
    pub transaction_hash: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ProcessWeb3PaymentResponse {
    pub payment_record: PaymentRecord,
    pub payment_status: String,
    pub estimated_confirmation_time: u32,
    pub next_steps: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct PaymentHistoryQuery {
    pub limit: Option<u32>,
    pub offset: Option<u32>,
    pub status_filter: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct PaymentHistoryResponse {
    pub payments: Vec<PaymentRecord>,
    pub total_count: u32,
    pub summary_stats: PaymentSummaryStats,
}

#[derive(Debug, Serialize)]
pub struct PaymentSummaryStats {
    pub total_paid_usd: f64,
    pub payment_count: u32,
    pub average_payment_usd: f64,
    pub success_rate_percentage: f64,
}

pub fn create_billing_routes() -> Router<Arc<DomainContainer>> {
    Router::new()
        // Legacy endpoints (maintained for backward compatibility)
        .route("/usage", get(get_usage_stats))
        .route("/invoices", get(get_invoices))
        .route("/payment-methods", get(get_payment_methods))
        .route("/pay", post(process_payment))
        // New comprehensive Web3 billing endpoints
        .route("/status", get(get_billing_status))
        .route("/subscription", post(create_subscription))
        .route("/subscription/cancel", post(cancel_subscription))
        .route("/payment/process", post(process_web3_payment))
        .route("/payment/history", get(get_payment_history))
        .route("/analytics", get(get_usage_analytics))
        .route("/tiers/compare", get(compare_tiers))
        .route("/tokens/supported", get(get_supported_tokens))
}

pub async fn get_usage_stats(
    State(_container): State<Arc<DomainContainer>>,
    user: Web3AuthenticatedUser,
) -> ApiResult<Json<serde_json::Value>> {
    let response = serde_json::json!({
        "current_period": {
            "requests_made": 8520,
            "requests_limit": user.enterprise_tier.rate_limit_per_minute() * 60 * 24 * 30,
            "data_transfer_gb": 12.5,
            "compute_hours": 45.2
        },
        "billing_cycle": "monthly",
        "next_billing_date": chrono::Utc::now() + chrono::Duration::days(15),
        "cost_breakdown": {
            "base_fee_usd": 0.0, // Token-based payment
            "overage_fees": 0.0,
            "total_tokens_required": 1000 // Example: 1000 USDC
        }
    });

    Ok(Json(response))
}

pub async fn get_invoices(
    State(_container): State<Arc<DomainContainer>>,
    user: Web3AuthenticatedUser,
) -> ApiResult<Json<serde_json::Value>> {
    let response = serde_json::json!({
        "invoices": [
            {
                "id": uuid::Uuid::new_v4(),
                "period": "2024-01",
                "amount_tokens": 1000,
                "token_symbol": "USDC",
                "status": "paid",
                "paid_at": chrono::Utc::now() - chrono::Duration::days(30)
            }
        ]
    });

    Ok(Json(response))
}

pub async fn get_payment_methods(
    State(_container): State<Arc<DomainContainer>>,
    user: Web3AuthenticatedUser,
) -> ApiResult<Json<serde_json::Value>> {
    let response = serde_json::json!({
        "payment_methods": [
            {
                "type": "token_balance",
                "token": "USDC",
                "balance": 5000.0,
                "auto_pay": true
            }
        ],
        "supported_tokens": ["USDC", "USDT", "DAI", "WETH"]
    });

    Ok(Json(response))
}

#[derive(Debug, Deserialize)]
pub struct PaymentRequest {
    pub invoice_id: String,
    pub token_symbol: String,
    pub amount: f64,
}

pub async fn process_payment(
    State(_container): State<Arc<DomainContainer>>,
    user: Web3AuthenticatedUser,
    Json(request): Json<PaymentRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    let response = serde_json::json!({
        "payment_id": uuid::Uuid::new_v4(),
        "status": "pending",
        "transaction_hash": format!("0x{}", hex::encode(&uuid::Uuid::new_v4().as_bytes()[..8])),
        "confirmation_required": true
    });

    Ok(Json(response))
}

// New Comprehensive Web3 Billing Endpoints

/// Get comprehensive billing status for authenticated user
pub async fn get_billing_status(
    State(container): State<Arc<DomainContainer>>,
    user: Web3AuthenticatedUser,
    Query(_query): Query<BillingStatusQuery>,
) -> Result<Json<BillingStatusResponse>, StatusCode> {
    let billing_service = create_billing_service(&container)?;

    let billing_result = billing_service
        .get_billing_status(&user.wallet_address)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let tier_compatibility = HashMap::new();
    let upgrade_recommendations = vec![];

    let response = BillingStatusResponse {
        billing_result,
        tier_compatibility,
        upgrade_recommendations,
    };

    Ok(Json(response))
}

/// Create new subscription
pub async fn create_subscription(
    State(container): State<Arc<DomainContainer>>,
    user: Web3AuthenticatedUser,
    Json(request): Json<CreateSubscriptionRequest>,
) -> Result<Json<CreateSubscriptionResponse>, StatusCode> {
    let billing_service = create_billing_service(&container)?;

    let subscription = billing_service
        .create_subscription(
            &user.wallet_address,
            request.tier.clone(),
            request.billing_type,
            request.payment_method.clone(),
            request.auto_renewal,
        )
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let payment_instructions = PaymentInstructions {
        payment_address: "0x742d35Cc6634C0532925a3b8D3Ac52A2f36fce89".to_string(),
        exact_amount: subscription.billing_cycle.total_amount.clone(),
        payment_deadline: Utc::now() + chrono::Duration::minutes(30),
    };

    let response = CreateSubscriptionResponse {
        initial_payment_required: billing_service.get_initial_payment_amount(&subscription),
        subscription,
        payment_instructions,
    };

    Ok(Json(response))
}

/// Cancel subscription
pub async fn cancel_subscription(
    State(_container): State<Arc<DomainContainer>>,
    user: Web3AuthenticatedUser,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Placeholder implementation
    let response = serde_json::json!({
        "wallet_address": user.wallet_address,
        "status": "cancelled",
        "effective_date": Utc::now(),
        "refund_amount": null
    });

    Ok(Json(response))
}

/// Process Web3 payment
pub async fn process_web3_payment(
    State(container): State<Arc<DomainContainer>>,
    user: Web3AuthenticatedUser,
    Json(request): Json<ProcessWeb3PaymentRequest>,
) -> Result<Json<ProcessWeb3PaymentResponse>, StatusCode> {
    let billing_service = create_billing_service(&container)?;

    let payment_record = billing_service
        .process_payment(
            &user.wallet_address,
            &request.payment_method,
            &request.amount,
            &request.subscription_tier,
        )
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let response = ProcessWeb3PaymentResponse {
        payment_status: format!("{:?}", payment_record.status),
        estimated_confirmation_time: 5,
        next_steps: vec!["Monitor transaction".to_string()],
        payment_record,
    };

    Ok(Json(response))
}

/// Get payment history
pub async fn get_payment_history(
    State(container): State<Arc<DomainContainer>>,
    user: Web3AuthenticatedUser,
    Query(_query): Query<PaymentHistoryQuery>,
) -> Result<Json<PaymentHistoryResponse>, StatusCode> {
    let billing_service = create_billing_service(&container)?;

    let payments = billing_service
        .get_payment_history(&user.wallet_address, 12)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let summary_stats = PaymentSummaryStats {
        total_paid_usd: 0.0,
        payment_count: payments.len() as u32,
        average_payment_usd: 0.0,
        success_rate_percentage: 100.0,
    };

    let response = PaymentHistoryResponse {
        total_count: payments.len() as u32,
        payments,
        summary_stats,
    };

    Ok(Json(response))
}

/// Get usage analytics
pub async fn get_usage_analytics(
    State(container): State<Arc<DomainContainer>>,
    user: Web3AuthenticatedUser,
) -> Result<Json<UsageMetrics>, StatusCode> {
    let billing_service = create_billing_service(&container)?;

    let usage_metrics = billing_service
        .calculate_usage_metrics(&user.wallet_address)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(usage_metrics))
}

/// Compare subscription tiers
pub async fn compare_tiers(
    State(_container): State<Arc<DomainContainer>>,
    user: Web3AuthenticatedUser,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let response = serde_json::json!({
        "current_tier": "Business",
        "available_tiers": [
            {
                "name": "Starter",
                "monthly_cost_usd": 29.0,
                "features": ["Basic Auth", "Community Support"],
                "suitable_for": "Small teams and testing"
            },
            {
                "name": "Business", 
                "monthly_cost_usd": 99.0,
                "features": ["Enhanced Auth", "Compliance", "Priority Support"],
                "suitable_for": "Growing businesses"
            },
            {
                "name": "Enterprise",
                "monthly_cost_usd": 299.0,
                "features": ["Full Auth Suite", "Security Monitoring", "Custom Integrations"],
                "suitable_for": "Large enterprises"
            },
            {
                "name": "Whale",
                "monthly_cost_usd": 999.0,
                "features": ["Premium Features", "Custom Development", "24/7 Support"],
                "suitable_for": "Enterprise-scale operations"
            }
        ],
        "recommendations": [
            {
                "upgrade_to": "Enterprise",
                "reason": "Your usage suggests you would benefit from advanced security features",
                "savings_potential": 15.0
            }
        ]
    });

    Ok(Json(response))
}

/// Get supported payment tokens
pub async fn get_supported_tokens(
    State(_container): State<Arc<DomainContainer>>,
    user: Web3AuthenticatedUser,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let response = serde_json::json!({
        "supported_tokens": [
            {
                "symbol": "USDC",
                "name": "USD Coin",
                "address": "0xA0b86a33E6Aa023cd3e2fE34d8c4c82F1e8f3A7E",
                "decimals": 6,
                "network": "ethereum",
                "is_stable_coin": true,
                "min_payment_amount": "1000000",
                "current_user_balance": "5000000000",
                "gas_estimate": {
                    "gas_limit": 65000,
                    "gas_price_gwei": 20.0,
                    "total_cost_usd": 3.25
                }
            },
            {
                "symbol": "USDT",
                "name": "Tether USD",
                "address": "0xdAC17F958D2ee523a2206206994597C13D831ec7",
                "decimals": 6,
                "network": "ethereum", 
                "is_stable_coin": true,
                "min_payment_amount": "1000000",
                "current_user_balance": "2000000000",
                "gas_estimate": {
                    "gas_limit": 70000,
                    "gas_price_gwei": 20.0,
                    "total_cost_usd": 3.50
                }
            }
        ],
        "recommended_token": "USDC",
        "gas_estimates": {
            "ethereum": {
                "standard": 20.0,
                "fast": 25.0,
                "instant": 35.0
            }
        }
    });

    Ok(Json(response))
}

// Helper Functions

fn create_billing_service(container: &Arc<DomainContainer>) -> Result<Web3BillingService, StatusCode> {
    let config = Web3BillingConfig {
        supported_networks: vec!["ethereum".to_string(), "bsc".to_string()],
        supported_tokens: HashMap::new(),
        pricing_tiers: HashMap::new(),
        gas_price_oracle: "https://ethgasstation.info/api/ethgasAPI.json".to_string(),
        payment_confirmation_blocks: 12,
        grace_period_days: 7,
        auto_renewal_enabled: true,
        discount_strategies: vec![],
    };

    Web3BillingService::new(
        container.database.clone(),
        reqwest::Client::new(),
        config,
        "https://eth-mainnet.alchemyapi.io/v2/your-api-key",
        "https://bsc-dataseed.binance.org",
        "https://polygon-rpc.com",
    ).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}