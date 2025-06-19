use axum::{Router, routing::post};
use std::sync::Arc;
use crate::auth::AuthService;

use super::{handlers, service::PaymentService};

pub fn router(payment_service: Arc<PaymentService>, auth_service: Arc<AuthService>) -> Router {
    Router::new()
        .route("/create", post(handlers::create_payment))
        .layer(axum::middleware::from_fn_with_state(
            auth_service,
            crate::auth::middleware::auth_middleware
        ))
        .with_state(payment_service)
}
