use axum::{ Router, routing::{ post, get } };
use std::sync::Arc;
use crate::auth::AuthService;

use super::{ handlers, service::PaymentService };

pub fn router(payment_service: Arc<PaymentService>, _auth_service: Arc<AuthService>) -> Router {
    Router::new()
        .route("/create", post(handlers::create_payment))
        .route("/:id", get(handlers::get_payment))
        .route("/:id/validate", get(handlers::validate_payment))
        .route("/:id/qrcode", get(handlers::get_qrcode))
        .layer(axum::middleware::from_fn(crate::auth::middleware::auth_middleware))
        .with_state(payment_service)
}
