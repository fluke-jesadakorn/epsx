use axum::{ Router, routing::{ post, get } };
use std::sync::Arc;
use crate::infra::container::AppContainer;

use super::{ handlers, service::PaymentService };

/// Create v1 payment routes
pub fn router_v1(payment_service: Arc<PaymentService>, container: AppContainer) -> Router {
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
        
        .layer(axum::middleware::from_fn_with_state(container.clone(), crate::web::middleware::modern_jwt_auth_middleware))
        .with_state(payment_service)
}

// Legacy payment routes removed - use v1 API structure only
