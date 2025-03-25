use axum::{ routing::post, Router, middleware };
use utoipa::OpenApi;
use super::{ PaymentService, handlers };
use crate::auth::AuthService;

#[derive(OpenApi)]
#[openapi(
    paths(handlers::create_payment),
    components(
        schemas(
            handlers::CreatePaymentRequest,
            handlers::CreatePaymentResponse,
            handlers::ErrorResponse
        )
    ),
    tags((name = "payment", description = "Payment processing endpoints"))
)]
#[allow(dead_code)]
pub struct PaymentApiDoc;

pub fn router(payment_service: PaymentService, auth_service: AuthService) -> Router {
    Router::new()
        .route("/create", post(handlers::create_payment))
        .layer(
            middleware::from_fn_with_state(auth_service, crate::auth::middleware::auth_middleware)
        )
        .with_state(payment_service)
}
