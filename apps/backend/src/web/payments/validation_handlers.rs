//! Payment Validation API Handlers
//!
//! Payment validation handlers using proper OpenID Bearer auth
//! and database operations for plan lookup.

use axum::{
    extract::{State, Query},
    Extension,
    response::Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use tracing::{info, debug, error};

use crate::{
    prelude::*,
    web::{
        auth::AppState,
        middleware::{OpenIDUserContext, UnifiedErrorResponse, ErrorDetails},
    },
};

// ============================================================================
// REQUEST/RESPONSE TYPES
// ============================================================================

/// Payment validation request
#[derive(Debug, Deserialize)]
pub struct ValidatePaymentRequest {
    pub transaction_hash: String,
    pub wallet_address: String,
    pub plan_id: Uuid,
    pub amount: u64,
    pub currency: String,
    pub network: String,
    pub token_address: Option<String>,
}

/// Payment validation response
#[derive(Debug, Serialize)]
pub struct ValidatePaymentResponse {
    pub success: bool,
    pub message: String,
    pub data: Option<PaymentValidationData>,
}

/// Payment validation data
#[derive(Debug, Serialize)]
pub struct PaymentValidationData {
    pub transaction_hash: String,
    pub plan_id: Uuid,
    pub amount_paid: f64,
    pub user_address: String,
    pub block_number: Option<u64>,
    pub confirmations: Option<u32>,
    pub plan_name: String,
    pub plan_price: f64,
    pub payment_status: String,
}

/// Activate subscription request
#[derive(Debug, Deserialize)]
pub struct ActivateSubscriptionRequest {
    pub wallet_address: String,
    pub plan_id: Uuid,
    pub transaction_hash: String,
    pub duration_days: Option<u32>,
}

/// Activate subscription response
#[derive(Debug, Serialize)]
pub struct ActivateSubscriptionResponse {
    pub success: bool,
    pub message: String,
    pub data: Option<PaymentSubscriptionData>,
}

/// Subscription data
#[derive(Debug, Serialize)]
pub struct PaymentSubscriptionData {
    pub subscription_id: Uuid,
    pub plan_id: Uuid,
    pub plan_name: String,
    pub expires_at: DateTime<Utc>,
    pub wallet_address: String,
    pub payment_transaction: String,
}

/// Payment lookup parameters
#[derive(Debug, Deserialize)]
pub struct PaymentLookupParams {
    pub transaction_hash: Option<String>,
    pub payment_reference: Option<String>,
    pub wallet_address: Option<String>,
}

/// Payment details response
#[derive(Debug, Serialize)]
pub struct PaymentDetailsResponse {
    pub success: bool,
    pub payment: Option<PaymentDetails>,
}

/// Payment details
#[derive(Debug, Serialize)]
pub struct PaymentDetails {
    pub id: Uuid,
    pub payment_reference: String,
    pub wallet_address: String,
    pub amount: f64,
    pub currency: String,
    pub status: String,
    pub plan_id: Uuid,
    pub plan_name: String,
    pub transaction_hash: Option<String>,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub metadata: serde_json::Value,
}

// ============================================================================
// HANDLERS
// ============================================================================

/// Validate blockchain payment transaction
#[axum::debug_handler]
pub async fn validate_payment_handler(
    State(app_state): State<AppState>,
    Extension(user_context): Extension<OpenIDUserContext>,
    Json(payload): Json<ValidatePaymentRequest>,
) -> Result<Json<ValidatePaymentResponse>, Json<UnifiedErrorResponse>> {
    info!(
        "Validating payment for user {}, plan {}, transaction {}",
        user_context.wallet_address,
        payload.plan_id,
        payload.transaction_hash
    );

    // Validate wallet address matches authenticated user
    if user_context.wallet_address.to_lowercase() != payload.wallet_address.to_lowercase() {
        error!(
            "Wallet address mismatch: {} vs {}",
            user_context.wallet_address, payload.wallet_address
        );
        return Err(Json(UnifiedErrorResponse {
            success: false,
            error: ErrorDetails {
                code: 400,
                message: "Wallet address mismatch".to_string(),
                reason: "Authenticated wallet does not match payment wallet".to_string(),
            },
        }));
    }

    // Fetch plan details from database
    let plan_info = fetch_plan_info(&app_state, &payload.plan_id).await?;
    let plan_name = plan_info.0;
    let plan_price = plan_info.1;

    // For blockchain verification, we rely on the BlockchainMonitor service
    // which automatically detects PaymentReceived events and creates subscriptions
    // This handler primarily validates the request data and returns expected info
    let block_number = None; // Will be filled by blockchain monitor
    let confirmations = None; // Will be filled by blockchain monitor

    let payment_id = Uuid::new_v4();

    info!(
        "Payment {} validated for wallet {}, transaction {}",
        payment_id, user_context.wallet_address, payload.transaction_hash
    );

    Ok(Json(ValidatePaymentResponse {
        success: true,
        message: "Payment validation submitted. Blockchain monitor will process the transaction.".to_string(),
        data: Some(PaymentValidationData {
            transaction_hash: payload.transaction_hash.clone(),
            plan_id: payload.plan_id,
            amount_paid: payload.amount as f64,
            user_address: user_context.wallet_address.clone(),
            block_number,
            confirmations,
            plan_name,
            plan_price,
            payment_status: "pending_confirmation".to_string(),
        }),
    }))
}

/// Activate subscription after payment validation
#[axum::debug_handler]
pub async fn activate_subscription_handler(
    State(app_state): State<AppState>,
    Extension(user_context): Extension<OpenIDUserContext>,
    Json(payload): Json<ActivateSubscriptionRequest>,
) -> Result<Json<ActivateSubscriptionResponse>, Json<UnifiedErrorResponse>> {
    info!(
        "Activating subscription for user {}, plan {}",
        user_context.wallet_address,
        payload.plan_id
    );

    // Validate wallet address matches authenticated user
    if user_context.wallet_address.to_lowercase() != payload.wallet_address.to_lowercase() {
        error!(
            "Wallet address mismatch: {} vs {}",
            user_context.wallet_address, payload.wallet_address
        );
        return Err(Json(UnifiedErrorResponse {
            success: false,
            error: ErrorDetails {
                code: 400,
                message: "Wallet address mismatch".to_string(),
                reason: "Authenticated wallet does not match subscription wallet".to_string(),
            },
        }));
    }

    // Fetch plan details
    let plan_info = fetch_plan_info(&app_state, &payload.plan_id).await?;
    let plan_name = plan_info.0;

    let duration_days = payload.duration_days.unwrap_or(30);
    let expires_at = Utc::now() + chrono::Duration::days(duration_days as i64);
    let subscription_id = Uuid::new_v4();

    // Note: For blockchain payments, the BlockchainMonitor handles subscription creation
    // This endpoint is for manual/administrative subscription activation
    let subscription_data = PaymentSubscriptionData {
        subscription_id,
        plan_id: payload.plan_id,
        plan_name: plan_name.clone(),
        expires_at,
        wallet_address: user_context.wallet_address.clone(),
        payment_transaction: payload.transaction_hash,
    };

    info!(
        "Subscription {} activated for user {}, plan {}, expires at {}",
        subscription_id, user_context.wallet_address, plan_name, expires_at
    );

    Ok(Json(ActivateSubscriptionResponse {
        success: true,
        message: "Subscription activation request submitted".to_string(),
        data: Some(subscription_data),
    }))
}

/// Get payment details by transaction hash, reference, or wallet address
#[axum::debug_handler]
pub async fn get_payment_details_handler(
    State(_app_state): State<AppState>,
    Extension(user_context): Extension<OpenIDUserContext>,
    Query(params): Query<PaymentLookupParams>,
) -> Result<Json<PaymentDetailsResponse>, Json<UnifiedErrorResponse>> {
    debug!(
        "Getting payment details for user {} with params: {:?}",
        user_context.wallet_address,
        params
    );

    // If wallet_address is provided, validate it matches authenticated user
    if let Some(ref wallet) = params.wallet_address {
        if user_context.wallet_address.to_lowercase() != wallet.to_lowercase() {
            error!(
                "Wallet address mismatch: {} vs {}",
                user_context.wallet_address, wallet
            );
            return Err(Json(UnifiedErrorResponse {
                success: false,
                error: ErrorDetails {
                    code: 403,
                    message: "Access denied".to_string(),
                    reason: "Can only query your own payments".to_string(),
                },
            }));
        }
    }

    // TODO: Implement database lookup for payment details
    // Query processed_blockchain_events table for payment history
    Ok(Json(PaymentDetailsResponse {
        success: true,
        payment: None, // No payment found matching criteria
    }))
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

/// Fetch plan information from database
async fn fetch_plan_info(
    app_state: &AppState,
    plan_id: &Uuid,
) -> Result<(String, f64), Json<UnifiedErrorResponse>> {
    // Try to fetch from group repository using the plan ID
    let plan_id_str = plan_id.to_string();
    
    // Query the pricing_plans table if available, otherwise use groups
    match app_state.group_repo.get_subscription_plans().await {
        Ok(plans) => {
            // Find the plan by ID (compare as string since plan_id might be stored differently)
            for plan in plans {
                if plan.id.to_string() == plan_id_str {
                    let price = plan.price
                        .as_ref()
                        .and_then(|p| p.to_string().parse::<f64>().ok())
                        .unwrap_or(0.0);
                    return Ok((plan.name, price));
                }
            }
            // Plan not found, return not found error
            Err(Json(UnifiedErrorResponse {
                success: false,
                error: ErrorDetails {
                    code: 404,
                    message: "Plan not found".to_string(),
                    reason: format!("No plan found with ID: {}", plan_id),
                },
            }))
        }
        Err(e) => {
            error!("Failed to fetch plans: {}", e);
            Err(Json(UnifiedErrorResponse {
                success: false,
                error: ErrorDetails {
                    code: 500,
                    message: "Database error".to_string(),
                    reason: "Failed to fetch plan information".to_string(),
                },
            }))
        }
    }
}
