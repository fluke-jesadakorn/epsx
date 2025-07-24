use axum::{ Router, routing::{ post, get } };
use std::sync::Arc;
use crate::auth::AuthService;

use super::{ handlers, service::PaymentService };

/// Create v1 payment routes
pub fn router_v1(payment_service: Arc<PaymentService>, _auth_service: Arc<AuthService>) -> Router {
    Router::new()
        // Crypto payments
        .route("/payments/crypto/deposit-address", get(handlers::get_crypto_deposit_address))
        
        // MusePay payments
        .route("/payments/musepay/create", post(handlers::create_payment))
        .route("/payments/musepay/:id", get(handlers::get_payment))
        .route("/payments/musepay/:id/validate", get(handlers::validate_payment))
        .route("/payments/musepay/:id/qrcode", get(handlers::get_qrcode))
        
        // Webhooks
        .route("/webhooks/payments/musepay", post(handlers::musepay_webhook_handler))
        
        .layer(axum::middleware::from_fn(crate::auth::middleware::auth_middleware))
        .with_state(payment_service)
}

/// Create legacy payment routes (backward compatibility)
pub fn router(payment_service: Arc<PaymentService>, _auth_service: Arc<AuthService>) -> Router {
    Router::new()
        .route("/create", post(handlers::create_payment))
        .route("/:id", get(handlers::get_payment))
        .route("/:id/validate", get(handlers::validate_payment))
        .route("/:id/qrcode", get(handlers::get_qrcode))
        .layer(axum::middleware::from_fn(crate::auth::middleware::auth_middleware))
        .with_state(payment_service)
}
