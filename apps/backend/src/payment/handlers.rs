use axum::{extract::{State, Path}, Json};
use std::sync::Arc;
use utoipa::{OpenApi, ToSchema};
use serde::Serialize;
use crate::auth::UserClaims;
use super::service::{PaymentService, PaymentError, CreatePaymentRequest, PaymentResponse};

#[derive(Debug, Serialize, ToSchema)]
pub struct ValidationResponse {
    pub is_valid: bool,
    pub expires_in_days: i64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ErrorResponse {
    pub error: String,
}

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::payment::handlers::create_payment,
        crate::payment::handlers::get_payment,
        crate::payment::handlers::validate_payment,
        crate::payment::handlers::get_qrcode
    ),
    components(
        schemas(CreatePaymentRequest, PaymentResponse, PaymentError, ValidationResponse)
    ),
    tags(
        (name = "Payments", description = "Payment processing endpoints")
    )
)]
#[allow(dead_code)]
struct PaymentApi;

#[utoipa::path(
    post,
    path = "/payment",
    request_body = CreatePaymentRequest,
    security(
        ("bearer" = [])
    ),
    responses(
        (status = 200, description = "Payment created successfully", body = PaymentResponse),
        (status = 400, description = "Invalid request parameters", body = PaymentError),
        (status = 402, description = "Payment failed", body = PaymentError),
        (status = 500, description = "Internal server error", body = PaymentError)
    ),
    tag = "Payments"
)]
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

#[utoipa::path(
    get,
    path = "/payment/validate/{payment_id}",
    responses(
        (status = 200, description = "Payment validated successfully", body = ValidationResponse),
        (status = 400, description = "Invalid payment ID", body = PaymentError),
        (status = 500, description = "Internal server error", body = PaymentError)
    ),
    tag = "Payments"
)]
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

#[utoipa::path(
    get,
    path = "/payment/qrcode/{payment_id}",
    responses(
        (status = 200, description = "QR code URL generated successfully", body = String),
        (status = 400, description = "Invalid payment ID", body = PaymentError),
        (status = 500, description = "Internal server error", body = PaymentError)
    ),
    tag = "Payments"
)]
pub async fn get_qrcode(
    State(_payment_service): State<Arc<PaymentService>>,
    Path(payment_id): Path<String>,
) -> Result<Json<String>, PaymentError> {
    let qr_code = format!("https://api.qrserver.com/v1/create-qr-code/?size=150x150&data={}", payment_id);
    
    Ok(Json(qr_code))
}

#[utoipa::path(
    post,
    path = "/payment",
    request_body = CreatePaymentRequest,
    security(
        ("bearer" = [])
    ),
    responses(
        (status = 200, description = "Payment created successfully", body = PaymentResponse),
        (status = 400, description = "Invalid request parameters", body = PaymentError),
        (status = 402, description = "Payment failed", body = PaymentError),
        (status = 500, description = "Internal server error", body = PaymentError)
    ),
    tag = "Payments"
)]
pub async fn create_payment(
    State(payment_service): State<Arc<PaymentService>>,
    _claims: axum::Extension<UserClaims>,
    Json(request): Json<CreatePaymentRequest>,
) -> Result<Json<PaymentResponse>, PaymentError> {
    let response = payment_service.create_payment(request).await?;
    Ok(Json(response))
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
