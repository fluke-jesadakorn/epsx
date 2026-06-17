//! Static asset serving helpers.

use axum::Router;
use tower_http::services::ServeDir;

pub fn static_assets_router(public_dir: &str) -> Router {
    Router::new().nest_service("/public", ServeDir::new(public_dir))
}
