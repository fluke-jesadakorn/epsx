use axum::{extract::{State, Path}, Json};
use std::sync::Arc;
use serde::Serialize;
use crate::web::auth::providers::UserClaims;
use super::service::{PaymentService, PaymentError, CreatePaymentRequest, PaymentResponse};

#[derive(Debug, Serialize)]
pub struct ValidationResponse {
    pub is_valid: bool,
    pub expires_in_days: i64,
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
}


pub async fn get_payment(
    State(payment_service): State<Arc<PaymentService>>,
    Path(_payment_id): Path<String>,
) -> Result<Json<PaymentResponse>, PaymentError> {
    let response = payment_service.create_payment(CreatePaymentRequest {
        amount: 1000,
        currency: "USDT".to_string(),
        description: Some("Mock payment".to_string()),
        payment_method: Some("crypto".to_string()),
        metadata: None,
    }).await?;
    
    Ok(Json(response))
}

pub async fn validate_payment(
    State(_payment_service): State<Arc<PaymentService>>,
    Path(_payment_id): Path<String>,
) -> Result<Json<ValidationResponse>, PaymentError> {
    let response = ValidationResponse {
        is_valid: true,
        expires_in_days: 90,
    };
    
    Ok(Json(response))
}

pub async fn get_qrcode(
    State(payment_service): State<Arc<PaymentService>>,
    Path(payment_id): Path<String>,
) -> Result<Json<String>, PaymentError> {
    let config = payment_service.get_config();
    let qr_code = format!(
        "{}?size={}&data={}",
        config.external_services.qr_code.api_base_url,
        config.external_services.qr_code.default_size,
        payment_id
    );
    
    Ok(Json(qr_code))
}

pub async fn create_payment(
    State(payment_service): State<Arc<PaymentService>>,
    _claims: axum::Extension<UserClaims>,
    Json(request): Json<CreatePaymentRequest>,
) -> Result<Json<PaymentResponse>, PaymentError> {
    let response = payment_service.create_payment(request).await?;
    Ok(Json(response))
}

/// Handler for crypto deposit address endpoint
pub async fn get_crypto_deposit_address(
    State(_payment_service): State<Arc<PaymentService>>,
) -> Result<Json<serde_json::Value>, PaymentError> {
    Ok(Json(serde_json::json!({
        "currency": "USDT",
        "network": "TRC20",
        "address": "TYMwiKKBdPWkBwcVXSNhE2MhTXNLTDJJBG",
        "memo": null
    })))
}

/// Handler for MusePay webhook
pub async fn musepay_webhook_handler(
    State(_payment_service): State<Arc<PaymentService>>,
    Json(_payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, PaymentError> {
    Ok(Json(serde_json::json!({
        "status": "received",
        "message": "Webhook processed successfully"
    })))
}

impl axum::response::IntoResponse for PaymentError {
    fn into_response(self) -> axum::response::Response {
        let status = match self {
            PaymentError::InvalidRequest(_) => axum::http::StatusCode::BAD_REQUEST,
            PaymentError::PaymentFailed(_) => axum::http::StatusCode::PAYMENT_REQUIRED,
            PaymentError::InternalError(_) => axum::http::StatusCode::INTERNAL_SERVER_ERROR,
        };

        (status, self.to_string()).into_response()
    }
}
