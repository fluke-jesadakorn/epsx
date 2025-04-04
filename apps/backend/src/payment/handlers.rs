use axum::{ extract::{ State, Json }, Extension, http::StatusCode };
use serde::{ Deserialize, Serialize };
use uuid::Uuid;
use super::PaymentService;
use crate::auth::AuthUser;

#[derive(Debug, Deserialize, utoipa::ToSchema)]
#[schema(
    example = json!({
    "currency": "USD",
    "amount": "100.00",
    "payment_method": "credit_card",
    "product_name": "Premium Subscription",
    "notify_url": "https://example.com/webhook"
})
)]
pub struct CreatePaymentRequest {
    #[schema(example = "USD")]
    currency: String,
    #[schema(example = "100.00")]
    amount: String,
    #[schema(example = "credit_card")]
    payment_method: String,
    #[schema(example = "Premium Subscription")]
    product_name: String,
    #[schema(example = "https://example.com/webhook")]
    notify_url: Option<String>,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
#[schema(
    example = json!({
    "request_id": "123e4567-e89b-12d3-a456-426614174000",
    "order_no": "ORD12345",
    "currency": "USD",
    "order_amount": "100.00",
    "status": 1,
    "payment_method": "credit_card",
    "receive_address": "0x123...abc",
    "checkout_url": "https://payment.example.com/checkout/123"
})
)]
pub struct CreatePaymentResponse {
    #[schema(example = "123e4567-e89b-12d3-a456-426614174000")]
    request_id: String,
    #[schema(example = "ORD12345")]
    order_no: String,
    #[schema(example = "USD")]
    currency: String,
    #[schema(example = "100.00")]
    order_amount: String,
    #[schema(example = 1)]
    status: i32,
    #[schema(example = "credit_card")]
    payment_method: String,
    #[schema(example = "0x123...abc")]
    receive_address: Option<String>,
    #[schema(example = "https://payment.example.com/checkout/123")]
    checkout_url: Option<String>,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
#[schema(example = json!({
    "code": "PAYMENT_ERROR",
    "message": "Payment processing failed"
}))]
pub struct ErrorResponse {
    #[schema(example = "PAYMENT_ERROR")]
    code: String,
    #[schema(example = "Payment processing failed")]
    message: String,
}

#[utoipa::path(
    post,
    path = "/v1/payment/create",
    request_body = CreatePaymentRequest,
    responses(
        (status = 200, description = "Payment created successfully", body = CreatePaymentResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    tag = "payment"
)]
#[axum::debug_handler]
pub async fn create_payment(
    State(payment_service): State<PaymentService>,
    Extension(auth_user): Extension<AuthUser>,
    Json(request): Json<CreatePaymentRequest>
) -> Result<Json<CreatePaymentResponse>, (StatusCode, Json<ErrorResponse>)> {
    let request_id = Uuid::new_v4().to_string();

    let result = payment_service.musepay_client
        .create_payment(
            &request_id,
            &request.currency,
            &request.amount,
            &request.payment_method,
            &request.product_name,
            auth_user.email.as_deref().unwrap_or_default(),
            request.notify_url.as_deref()
        ).await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    code: "PAYMENT_ERROR".to_string(),
                    message: e.to_string(),
                }),
            )
        })?;

    let data = result.get("data").ok_or_else(|| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                code: "INVALID_RESPONSE".to_string(),
                message: "Invalid response from payment provider".to_string(),
            }),
        )
    })?;

    let response = CreatePaymentResponse {
        request_id: data["request_id"].as_str().unwrap_or_default().to_string(),
        order_no: data["order_no"].as_str().unwrap_or_default().to_string(),
        currency: data["currency"].as_str().unwrap_or_default().to_string(),
        order_amount: data["order_amount"].as_str().unwrap_or_default().to_string(),
        status: data["status"].as_i64().unwrap_or_default() as i32,
        payment_method: data["payment_method"].as_str().unwrap_or_default().to_string(),
        receive_address: data["receive_address"].as_str().map(String::from),
        checkout_url: data["checkout_url"].as_str().map(String::from),
    };

    Ok(Json(response))
}
