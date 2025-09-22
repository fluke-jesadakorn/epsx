use axum::{
  extract::{ Json, State },
  http::StatusCode,
  response::Json as ResponseJson,
  routing::{ get, post },
  Router,
};
use serde::{ Deserialize, Serialize };
use tracing::{ info, error, warn };
use std::sync::Arc;
use sqlx::Row;

use crate::infrastructure::container::AppContainer;
use crate::domain::shared_kernel::value_objects::UserId;
use crate::domain::payment::value_objects::{ PaymentId, TransactionHash };
use crate::application::payment::commands::ActivateSubscriptionCommand;

/// Request to confirm a MetaMask payment and activate subscription
#[derive(Debug, Deserialize)]
pub struct ConfirmPaymentRequest {
  pub plan_id: i32,
  pub transaction_hash: String,
  pub amount: f64,
  pub currency: String,
  pub network: Option<String>,
}

/// Response for payment confirmation
#[derive(Debug, Serialize)]
pub struct ConfirmPaymentResponse {
  pub success: bool,
  pub message: String,
  pub subscription: Option<SubscriptionInfo>,
  pub transaction_hash: String,
  pub payment_id: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SubscriptionInfo {
  pub plan_id: i32,
  pub plan_name: String,
  pub activated_at: String,
  pub expires_at: Option<String>,
}

/// Confirm payment and activate subscription
pub async fn confirm_payment(
  State(_container): State<Arc<AppContainer>>,
  Json(request): Json<ConfirmPaymentRequest>
) -> Result<ResponseJson<ConfirmPaymentResponse>, StatusCode> {
  // TODO: Extract user_id from auth middleware in the future
  // For now, we'll use a test user ID
  let user_id = UserId::new(); // This should come from auth middleware
  info!(
    "Confirming payment for user: {}, plan: {}, tx: {}",
    user_id,
    request.plan_id,
    request.transaction_hash
  );

  // Validate transaction hash format
  if
    request.transaction_hash.len() != 66 ||
    !request.transaction_hash.starts_with("0x")
  {
    warn!("Invalid transaction hash format: {}", request.transaction_hash);
    return Ok(
      ResponseJson(ConfirmPaymentResponse {
        success: false,
        message: "Invalid transaction hash format".to_string(),
        subscription: None,
        transaction_hash: request.transaction_hash,
        payment_id: None,
      })
    );
  }

  // Create transaction hash value object
  let transaction_hash = match
    TransactionHash::new(
      request.transaction_hash.clone(),
      crate::domain::payment::value_objects::Network::Ethereum // Default to Ethereum
    )
  {
    Ok(hash) => hash,
    Err(e) => {
      error!("Failed to create transaction hash: {}", e);
      return Ok(
        ResponseJson(ConfirmPaymentResponse {
          success: false,
          message: format!("Invalid transaction hash: {}", e),
          subscription: None,
          transaction_hash: request.transaction_hash,
          payment_id: None,
        })
      );
    }
  };

  // Generate payment ID for this confirmation
  let payment_id = PaymentId::generate();

  // Create activation command
  let _command = ActivateSubscriptionCommand::new(
    payment_id.clone(),
    user_id.clone(),
    request.plan_id,
    transaction_hash
  );

  // Get activation handler - temporarily disabled
  // let handler = ActivateSubscriptionHandler::new(
  //     container.db_pool().clone(),
  //     cache, // Need proper cache instance
  // );

  // For now, return success without actual activation
  info!("Payment activation temporarily disabled during Web3 migration");
  Ok(
    ResponseJson(ConfirmPaymentResponse {
      success: true,
      message: "Payment activation is temporarily disabled during Web3 migration".to_string(),
      subscription: None,
      transaction_hash: request.transaction_hash,
      payment_id: None,
    })
  )
}

/// Get user's current subscription status
#[derive(Debug, Serialize)]
pub struct SubscriptionStatusResponse {
  pub has_subscription: bool,
  pub plan_name: Option<String>,
  pub plan_type: Option<String>,
  pub expires_at: Option<String>,
  pub permissions: Vec<String>,
}

pub async fn get_subscription_status(State(
  container,
): State<Arc<AppContainer>>) -> Result<
  ResponseJson<SubscriptionStatusResponse>,
  StatusCode
> {
  // TODO: Extract user_id from auth middleware in the future
  let user_id = UserId::new(); // This should come from auth middleware
  info!("Getting subscription status for user: {}", user_id);

  // Get user details from database
  let user_uuid = match uuid::Uuid::parse_str(&user_id.to_string()) {
    Ok(uuid) => uuid,
    Err(_) => {
      error!("Invalid user ID format: {}", user_id);
      return Err(StatusCode::BAD_REQUEST);
    }
  };

  let user_result = sqlx
    ::query("SELECT package_tier FROM users WHERE id = $1")
    .bind(user_uuid)
    .fetch_optional(container.db_pool().as_ref()).await;

  match user_result {
    Ok(Some(user)) => {
      let package_tier: Option<String> = user.try_get("package_tier").ok();
      let has_subscription = package_tier.is_some();

      // Get user permissions
      let permissions_result = sqlx
        ::query(
          "SELECT permission FROM user_permissions WHERE user_id = $1 AND is_active = true"
        )
        .bind(user_uuid)
        .fetch_all(container.db_pool().as_ref()).await;

      let permissions: Vec<String> = permissions_result
        .unwrap_or_default()
        .into_iter()
        .map(|row| row.try_get::<String, _>("permission").unwrap_or_default())
        .collect();

      Ok(
        ResponseJson(SubscriptionStatusResponse {
          has_subscription,
          plan_name: package_tier.clone(),
          plan_type: package_tier,
          expires_at: None, // Would need subscription expiry tracking
          permissions,
        })
      )
    }
    Ok(None) => {
      warn!("User not found: {}", user_id);
      Err(StatusCode::NOT_FOUND)
    }
    Err(e) => {
      error!("Database error getting user subscription: {}", e);
      Err(StatusCode::INTERNAL_SERVER_ERROR)
    }
  }
}

/// Create payments router
pub fn create_payments_router(container: Arc<AppContainer>) -> Router {
  Router::new()
    .route("/confirm", post(confirm_payment))
    .route("/subscription-status", get(get_subscription_status))
    .with_state(container)
}
