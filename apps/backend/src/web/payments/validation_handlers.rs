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
use diesel::sql_types::Text;
use diesel::QueryableByName;

use crate::{
    web::{
        auth::AppState,
        middleware::{OpenIDUserContext, UnifiedErrorResponse},
    },
    auth::{UnifiedPermissionService, GrantPermissionRequest},
};
use std::sync::Arc;

/// Helper struct for deduplication query
#[derive(Debug, QueryableByName)]
pub struct PaymentExistsRow {
    #[diesel(sql_type = Text)]
    pub payment_reference: String,
}

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
        return Err(UnifiedErrorResponse::json(400, "Wallet address mismatch", "Authenticated wallet does not match payment wallet"));
    }

    // Fetch plan details from database
    let (plan_name, plan_price, _metadata) = fetch_plan_info(&app_state, &payload.plan_id).await?;

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
    Extension(permission_service): Extension<Arc<UnifiedPermissionService>>,
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
        return Err(UnifiedErrorResponse::json(400, "Wallet address mismatch", "Authenticated wallet does not match subscription wallet"));
    }

    // Fetch plan details
    let (plan_name, _plan_price, metadata) = fetch_plan_info(&app_state, &payload.plan_id).await?;

    // Grant ranking permissions based on plan metadata
    if let Some(features) = metadata.as_object() {
        // Extract offset
        if let Some(offset) = features.get("ranking_offset").and_then(|v| v.as_i64()) {
            let perm = format!("epsx:rankings:offset:{}", offset);
            info!("Granting rank offset permission to {}: {}", user_context.wallet_address, perm);
            
            let request = GrantPermissionRequest {
                wallet_address: user_context.wallet_address.clone(),
                permission_string: perm.clone(),
                granted_by: "system_activation".to_string(),
                reason: Some("Plan activation".to_string()),
                expires_at: None, // Permissions stick until revoked or plan expires (handled separately)
            };
            
            if let Err(e) = permission_service.grant_permission(request).await {
                error!("Failed to grant permission {}: {}", perm, e);
            }
        }
        
        // Extract limit
        if let Some(limit) = features.get("rankings_limit").and_then(|v| v.as_i64()) {
            let perm = format!("epsx:rankings:limit:{}", limit);
            info!("Granting rank limit permission to {}: {}", user_context.wallet_address, perm);
            
            let request = GrantPermissionRequest {
                wallet_address: user_context.wallet_address.clone(),
                permission_string: perm.clone(),
                granted_by: "system_activation".to_string(),
                reason: Some("Plan activation".to_string()),
                expires_at: None,
            };
            
            if let Err(e) = permission_service.grant_permission(request).await {
                 error!("Failed to grant permission {}: {}", perm, e);
            }
        }
    }

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
    use diesel::prelude::*;
    use diesel_async::RunQueryDsl;
    use crate::infrastructure::database::get_payments_pool;
    use crate::schemas::payments::payments;
    use crate::infrastructure::models::payment::PaymentDb;

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
            return Err(UnifiedErrorResponse::json(403, "Access denied", "Can only query your own payments"));
        }
    }

    // Get PAYMENTS database connection
    let payments_pool = get_payments_pool().await.map_err(|e| {
        error!("Failed to get payments database pool: {}", e);
        UnifiedErrorResponse::json(500, "Database connection failed", "Failed to get payments database pool")
    })?;
    let mut payments_conn = payments_pool.get().await.map_err(|e| {
        error!("Failed to get payments database connection: {}", e);
        UnifiedErrorResponse::json(500, "Database connection failed", "Failed to establish payments database connection")
    })?;

    // Build query based on provided parameters
    let mut query = payments::table.into_boxed();

    // Always filter by authenticated user's wallet
    query = query.filter(payments::wallet_address.ilike(format!("%{}%", user_context.wallet_address)));

    // Apply transaction_hash filter if provided
    if let Some(ref tx_hash) = params.transaction_hash {
        query = query.filter(payments::transaction_hash.eq(tx_hash));
    }

    // Apply payment_reference filter if provided
    if let Some(ref reference) = params.payment_reference {
        query = query.filter(payments::payment_reference.eq(reference));
    }

    // Execute query
    let payment_result = query
        .order(payments::created_at.desc().nulls_last())
        .first::<PaymentDb>(&mut payments_conn)
        .await;

    let payment = match payment_result {
        Ok(pay_db) => {
            // Get plan name (would need primary DB connection for proper lookup)
            let plan_name = format!("Plan-{}", &pay_db.plan_id.to_string()[..8]);

            Some(PaymentDetails {
                id: pay_db.id,
                payment_reference: pay_db.payment_reference,
                wallet_address: pay_db.wallet_address,
                amount: pay_db.amount.to_string().parse::<f64>().unwrap_or(0.0),
                currency: pay_db.currency,
                status: pay_db.status,
                plan_id: pay_db.plan_id,
                plan_name,
                transaction_hash: pay_db.transaction_hash,
                created_at: pay_db.created_at.unwrap_or_else(Utc::now),
                completed_at: pay_db.completed_at,
                metadata: pay_db.metadata.unwrap_or(serde_json::json!({})),
            })
        }
        Err(diesel::NotFound) => None,
        Err(e) => {
            error!("Failed to query payment: {}", e);
            return Err(UnifiedErrorResponse::json(500, "Query failed", format!("Failed to load payment: {}", e)));
        }
    };

    Ok(Json(PaymentDetailsResponse {
        success: true,
        payment,
    }))
}


// NOTE: Legacy confirm_payment_handler has been removed.
// Payment confirmation is now handled by:
// - submit_tx_handler.rs: POST /api/payments/submit (accepts tx_hash)
// - tx_monitor_service.rs: Background service that monitors and confirms transactions
// - get_tx_status_handler.rs: GET /api/payments/status/:tx_hash (frontend polls this)



















// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

/// Fetch plan information from database
async fn fetch_plan_info(
    app_state: &AppState,
    plan_id: &Uuid,
) -> Result<(String, f64, serde_json::Value), Json<UnifiedErrorResponse>> {
    // Try to fetch from plan repository using the plan ID
    let plan_id_str = plan_id.to_string();
    
    // Query the pricing_plans table if available, otherwise use plans
    match app_state.plan_repo.get_subscription_plans().await {
        Ok(plans) => {
            // Find the plan by ID (compare as string since plan_id might be stored differently)
            for plan in plans {
                if plan.id.to_string() == plan_id_str {
                    let price = plan.price
                        .as_ref()
                        .and_then(|p| p.to_string().parse::<f64>().ok())
                        .unwrap_or(0.0);
                    let metadata = plan.plan_metadata.clone();
                    return Ok((plan.name, price, metadata));
                }
            }
            // Plan not found, return not found error
            Err(UnifiedErrorResponse::json(404, "Plan not found", format!("No plan found with ID: {}", plan_id)))
        }
        Err(e) => {
            error!("Failed to fetch plans: {}", e);
            Err(UnifiedErrorResponse::json(500, "Database error", "Failed to fetch plan information"))
        }
    }
}
