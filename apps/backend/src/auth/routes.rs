use axum::{
    routing::post,
    Router,
};
use std::sync::Arc;

use super::{AuthService, handlers};

pub fn router(auth_service: Arc<AuthService>) -> Router {
    Router::new()
        .route("/verify", post(handlers::verify_token))
        .with_state(auth_service)
}
