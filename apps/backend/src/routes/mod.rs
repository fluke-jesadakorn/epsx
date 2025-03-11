pub mod auth;
pub mod health;

use std::sync::Arc;
use axum::Router;
use tower_http::cors::CorsLayer;
use http::Method;
use http::header::{AUTHORIZATION, CONTENT_TYPE};
use crate::app_state::AppState;

pub fn create_router(state: AppState) -> Router<Arc<AppState>> {
    let state = Arc::new(state);
    
    let cors = CorsLayer::new()
        .allow_origin([state.config.frontend_url.parse().unwrap()])
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE, Method::OPTIONS])
        .allow_headers([AUTHORIZATION, CONTENT_TYPE])
        .allow_credentials(true);

    Router::new()
        .merge(health::router())
        .nest("/auth", auth::auth_routes(state.clone()))
        .layer(cors)
        .with_state(state)
}
