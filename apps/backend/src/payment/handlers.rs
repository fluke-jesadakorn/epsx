use axum::{extract::State, Json};
use std::sync::Arc;
use crate::auth::UserClaims;
use utoipa::OpenApi;

use super::service::{PaymentService, PaymentError, CreatePaymentRequest, PaymentResponse};

#[derive(OpenApi)]
#[openapi(
    paths(create_payment),
    components(
        schemas(CreatePaymentRequest, PaymentResponse, PaymentError)
    ),
    tags(
        (name = "Payments", description = "Payment processing endpoints")
    )
)]
#[allow(dead_code)]
struct PaymentApi;

/// Create a new payment
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
    // Here you would typically validate user permissions, etc.
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

#[derive(Debug, serde::Serialize)]
pub struct ErrorResponse {
    pub error: String,
}
